.PHONY: generate generate-mcp build-mcp

VERSION := $(shell grep '^version' ../Cargo.toml | head -1 | cut -d'"' -f2)

generate: generate-mcp

generate-mcp:
	buf generate --template buf.gen.yaml ../proto

build-mcp: generate-mcp
	go build -ldflags "-X main.version=$(VERSION)" -o centy-mcp .
