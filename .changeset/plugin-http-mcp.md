---
"centy-daemon": patch
---

Point the Centy plugin and install docs at the daemon's HTTP MCP endpoint (`http://127.0.0.1:50052/mcp`). Clients no longer spawn `centy-mcp` through `npx`. Bump the plugin to 1.1.0 so existing installs refresh their cached config.
