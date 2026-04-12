.PHONY: build build-daemon build-cli build-mcp

build: build-daemon build-cli build-mcp

build-daemon:
	cargo build --release

build-cli:
	$(MAKE) -C cli build-cli

build-mcp:
	$(MAKE) -C mcp build-mcp
