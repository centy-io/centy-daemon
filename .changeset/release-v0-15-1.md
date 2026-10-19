---
"centy-daemon": patch
---

Release v0.15.1: the built-in MCP endpoint is stateless and its tools/call no longer panics, so clients consume Centy MCP only through http://127.0.0.1:50052/mcp. The Claude plugin now points at that URL.
