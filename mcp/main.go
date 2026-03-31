package main

import (
	"fmt"
	"net/http"
	"os"

	"connectrpc.com/connect"
	mcpserver "github.com/mark3labs/mcp-go/server"
	"github.com/redpanda-data/protoc-gen-go-mcp/pkg/runtime/mark3labs"

	"github.com/centy-io/centy-daemon/mcp/gen/centy/v1/centyv1connect"
	"github.com/centy-io/centy-daemon/mcp/gen/centy/v1/centyv1mcp"
)

func main() {
	addr := os.Getenv("CENTY_DAEMON_ADDR")
	if addr == "" {
		addr = "127.0.0.1:50051"
	}

	client := centyv1connect.NewCentyDaemonClient(
		&http.Client{},
		"http://"+addr,
		connect.WithGRPC(),
	)

	raw, s := mark3labs.NewServer("centy-daemon", "0.9.3")
	centyv1mcp.ForwardToConnectCentyDaemonClient(s, client)

	if err := mcpserver.ServeStdio(raw); err != nil {
		fmt.Fprintf(os.Stderr, "server error: %v\n", err)
		os.Exit(1)
	}
}
