.PHONY: generate generate-cli build-cli

VERSION := $(shell grep '^version' ../Cargo.toml | head -1 | cut -d'"' -f2)

generate: generate-cli

generate-cli:
	buf generate --template buf.gen.yaml ../proto

build-cli: generate-cli
	go build -ldflags "-X main.version=$(VERSION)" -o centy-cli .
