---
"@centy-io/centy-daemon": patch
---

Fixed the Centy plugin skills referencing MCP tool names that do not exist (e.g. `CreateWorkspace`, `GetLinks`, `UnregisterProject`, `InitializeProject`). They now use the real `centy_v1_CentyDaemon_<Method>` tools and protobuf JSON argument names.
