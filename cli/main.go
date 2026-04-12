package main

import (
	"fmt"
	"os"

	"github.com/NathanBaulch/protoc-gen-cobra/client"

	centyv1 "github.com/centy-io/centy-daemon/cli/gen/centy/v1"
)

var version = "dev"

func main() {
	addr := os.Getenv("CENTY_DAEMON_ADDR")
	if addr == "" {
		addr = "127.0.0.1:50051"
	}

	root := centyv1.CentyDaemonClientCommand(client.WithServerAddr(addr))
	root.Use = "centy"
	root.Short = "CLI client for the centy daemon"
	root.Long = ""
	root.Version = version

	if err := root.Execute(); err != nil {
		fmt.Fprintln(os.Stderr, err)
		os.Exit(1)
	}
}
