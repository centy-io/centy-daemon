# Changelog

All notable changes to the `centy-cli` package will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.12.4] — 2026-04-13

No changes to the CLI binary.

## [0.12.3] — 2026-04-13

### Added
- Initial release: auto-generated gRPC CLI via `protoc-gen-cobra`
- All 80+ `CentyDaemon` RPCs exposed as kebab-case subcommands directly under `centy <rpc>`
- Flags auto-mapped from proto request message fields
- JSON/XML request input and multiple prettified response output formats
- Default server address `127.0.0.1:50051` with `CENTY_DAEMON_ADDR` env-var override
- TLS flags (`--tls`, `--tls-ca-cert-file`, etc.) for secure connections
- `buf generate --template buf.gen.yaml ../proto` regenerates all CLI code as proto evolves
