---
"@centy-io/centy-daemon": patch
---

Fixed `centy init` creating a nested `.centy/.centy` folder when run from inside a directory that is already a `.centy` repo (e.g. an org's `.centy` repo). The existing directory is now reused instead of being duplicated. (#284)
