---
"centy-daemon": patch
---

Make the built-in MCP endpoint stateless: no session ids or session store, and plain JSON responses, so any client can POST to `/mcp` without an `Mcp-Session-Id` header. Also fix `tools/call` hanging and panicking by waiting for the gRPC channel to be ready before each unary call.
