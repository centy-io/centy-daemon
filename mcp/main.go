package main

import (
	"context"
	"crypto/tls"
	"fmt"
	"net"
	"net/http"
	"os"

	"connectrpc.com/connect"
	mcpserver "github.com/mark3labs/mcp-go/server"
	"github.com/redpanda-data/protoc-gen-go-mcp/pkg/runtime/mark3labs"
	"golang.org/x/net/http2"

	"github.com/centy-io/centy-daemon/mcp/gen/centy/v1/centyv1connect"
	"github.com/centy-io/centy-daemon/mcp/gen/centy/v1/centyv1mcp"
)

var version = "dev"

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

	raw, s := mark3labs.NewServer("centy-daemon", version)
	centyv1mcp.ForwardToConnectCentyDaemonClient(s, client)

	if err := mcpserver.ServeStdio(raw); err != nil {
		fmt.Fprintf(os.Stderr, "server error: %v\n", err)
		os.Exit(1)
	}
}
