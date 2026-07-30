---
"@centy-io/centy-daemon": patch
---

Fixed `centy link` ignoring the `type:` prefix on source/target arguments (e.g. `plan:1`, `issue:<uuid>`), which broke cross-type linking and could raise a false self-link error when two items of different types shared the same display number. (#285)
