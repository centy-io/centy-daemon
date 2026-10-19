---
"@centy-io/centy-daemon": patch
---

Fixed the Centy plugin skills referencing MCP tool names that do not exist (e.g. `CreateWorkspace`, `GetLinks`, `UnregisterProject`, `InitializeProject`). They now use the real `centy_v1_CentyDaemon_<Method>` tools and protobuf JSON argument names, and describe intent instead of literal calls. Bumps the `centy` plugin version from 1.1.0 to 1.2.0 so installed plugins refresh.
