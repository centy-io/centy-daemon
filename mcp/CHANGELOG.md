# Changelog

All notable changes to the `centy-mcp` package will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Version is now injected via `-ldflags` at build time and defaults to `"dev"`; `mcp/npm/package.json` version is reset to `0.0.0` and set from the git tag at publish time

### Fixed
- Use `h2c` (HTTP/2 cleartext) transport so the MCP client can connect to the tonic gRPC daemon, which requires HTTP/2

### Changed
- Move `Makefile` and `buf.gen.yaml` from repo root into `mcp/` — run `make generate` from `mcp/` to regenerate stubs
- CI workflows updated to run `make -C mcp generate`

## [0.9.3] — 2026-04-01

### Changed
- Align package version with centy-daemon 0.9.3

## [0.9.2] — 2026-04-01

### Added
- README for the npm package with installation and Claude Desktop config instructions

### Changed
- Fix `package.json` repository field

## [0.9.1] — 2026-04-01

### Added
- Initial release of the `centy-mcp` npm package
- MCP server that exposes all `CentyDaemon` gRPC RPCs as MCP tools
- Pre-built binary download via `postinstall` script for macOS (aarch64/x86_64) and Linux (x86_64)
- `npx centy-mcp` usage — no global install required
- `CENTY_DAEMON_ADDR` environment variable to override the default `127.0.0.1:50051` address
