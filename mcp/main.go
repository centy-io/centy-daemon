package main

import (
	"context"
	"crypto/tls"
	"fmt"
	"net"
	"net/http"
	"os"
	"os/exec"
	"path/filepath"
	"runtime"
	"strconv"
	"strings"
	"time"

	"connectrpc.com/connect"
	"github.com/mark3labs/mcp-go/mcp"
	mcpserver "github.com/mark3labs/mcp-go/server"
	"github.com/redpanda-data/protoc-gen-go-mcp/pkg/runtime/mark3labs"
	"golang.org/x/net/http2"

	centyv1 "github.com/centy-io/centy-daemon/mcp/gen/centy/v1"
	"github.com/centy-io/centy-daemon/mcp/gen/centy/v1/centyv1connect"
	"github.com/centy-io/centy-daemon/mcp/gen/centy/v1/centyv1mcp"
)

var version = "dev"

func isDaemonRunning(addr string) bool {
	conn, err := net.DialTimeout("tcp", addr, 500*time.Millisecond)
	if err != nil {
		return false
	}
	conn.Close()
	return true
}

// semverMajorMinor extracts the major and minor integers from a semver string
// like "1.2.3". Returns (-1, -1, false) if the string is not parseable.
func semverMajorMinor(v string) (int, int, bool) {
	v = strings.TrimPrefix(v, "v")
	parts := strings.SplitN(v, ".", 3)
	if len(parts) < 2 {
		return -1, -1, false
	}
	major, err1 := strconv.Atoi(parts[0])
	minor, err2 := strconv.Atoi(parts[1])
	if err1 != nil || err2 != nil {
		return -1, -1, false
	}
	return major, minor, true
}

// checkDaemonCompatibility calls GetDaemonInfo and exits with a clear error if
// the daemon version is incompatible with this MCP server version.
// Compatibility policy: major and minor versions must match.
// The check is skipped when either version is "dev" or when the daemon is unreachable.
func checkDaemonCompatibility(client centyv1connect.CentyDaemonClient, mcpVersion string) {
	if mcpVersion == "dev" {
		return
	}

	ctx, cancel := context.WithTimeout(context.Background(), 2*time.Second)
	defer cancel()

	resp, err := client.GetDaemonInfo(ctx, connect.NewRequest(&centyv1.GetDaemonInfoRequest{}))
	if err != nil {
		// Daemon unreachable — skip version check, tools will fail gracefully at call time.
		return
	}

	daemonVersion := resp.Msg.GetVersion()
	if daemonVersion == "dev" {
		return
	}

	mcpMaj, mcpMin, mcpOK := semverMajorMinor(mcpVersion)
	daeMaj, daeMin, daeOK := semverMajorMinor(daemonVersion)

	if !mcpOK || !daeOK {
		return
	}

	if mcpMaj != daeMaj || mcpMin != daeMin {
		fmt.Fprintf(os.Stderr,
			"centy-mcp v%s is incompatible with centy-daemon v%s. Please update centy-mcp.\n",
			mcpVersion, daemonVersion)
		os.Exit(1)
	}
}

func findDaemonBinary() (string, error) {
	if p := os.Getenv("CENTY_DAEMON_PATH"); p != "" {
		return p, nil
	}
	home, err := os.UserHomeDir()
	if err == nil {
		candidate := filepath.Join(home, ".centy", "bin", "centy-daemon")
		if runtime.GOOS == "windows" {
			candidate += ".exe"
		}
		if _, err := os.Stat(candidate); err == nil {
			return candidate, nil
		}
	}
	return exec.LookPath("centy-daemon")
}

func main() {
	addr := os.Getenv("CENTY_DAEMON_ADDR")
	if addr == "" {
		addr = "127.0.0.1:50051"
	}

	h2cClient := &http.Client{
		Transport: &http2.Transport{
			AllowHTTP: true,
			DialTLSContext: func(_ context.Context, network, addr string, _ *tls.Config) (net.Conn, error) {
				return net.Dial(network, addr)
			},
		},
	}

	client := centyv1connect.NewCentyDaemonClient(
		h2cClient,
		"http://"+addr,
		connect.WithGRPC(),
	)

	checkDaemonCompatibility(client, version)

	raw, s := mark3labs.NewServer("centy-daemon", version)
	centyv1mcp.ForwardToConnectCentyDaemonClient(s, client)

	raw.AddTool(
		mcp.NewTool("IsRunning",
			mcp.WithDescription("Check whether the centy daemon is currently running"),
		),
		func(_ context.Context, _ mcp.CallToolRequest) (*mcp.CallToolResult, error) {
			if isDaemonRunning(addr) {
				return mcp.NewToolResultText(fmt.Sprintf("Daemon is running at %s", addr)), nil
			}
			return mcp.NewToolResultText(fmt.Sprintf("Daemon is not running (checked %s)", addr)), nil
		},
	)

	raw.AddTool(
		mcp.NewTool("StartDaemon",
			mcp.WithDescription("Start the centy daemon if it is not already running"),
		),
		func(_ context.Context, _ mcp.CallToolRequest) (*mcp.CallToolResult, error) {
			if isDaemonRunning(addr) {
				return mcp.NewToolResultText(fmt.Sprintf("Daemon is already running at %s", addr)), nil
			}
			binaryPath, err := findDaemonBinary()
			if err != nil {
				return nil, fmt.Errorf("centy-daemon binary not found: %w", err)
			}
			cmd := exec.Command(binaryPath)
			setDetachedAttrs(cmd)
			if err := cmd.Start(); err != nil {
				return nil, fmt.Errorf("failed to start daemon: %w", err)
			}
			cmd.Process.Release() //nolint:errcheck

			deadline := time.Now().Add(5 * time.Second)
			for time.Now().Before(deadline) {
				if isDaemonRunning(addr) {
					return mcp.NewToolResultText(fmt.Sprintf("Daemon started successfully at %s", addr)), nil
				}
				time.Sleep(200 * time.Millisecond)
			}
			return nil, fmt.Errorf("daemon did not become ready within 5s (addr: %s)", addr)
		},
	)

	if err := mcpserver.ServeStdio(raw); err != nil {
		fmt.Fprintf(os.Stderr, "server error: %v\n", err)
		os.Exit(1)
	}
}
