---
"@centy-io/centy-daemon": patch
---

Write the project registry with the `tempfile` crate. Temp files get unique names and are removed on failure, so a failed write no longer leaves a stale `projects.*.json.tmp`.
