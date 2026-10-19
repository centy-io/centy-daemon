---
"centy-daemon": patch
---

Make the built-in MCP endpoint stateless: no session ids or session store, and plain JSON responses, so any client can POST to `/mcp` without an `Mcp-Session-Id` header.
