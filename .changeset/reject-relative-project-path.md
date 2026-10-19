---
"@centy-io/centy-daemon": patch
---

Reject a relative `project_path` with an `INVALID_PATH` error. `assert_initialized` and `init` now require an absolute path, so the daemon no longer resolves it against its own working directory.
