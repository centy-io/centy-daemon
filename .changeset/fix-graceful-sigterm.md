---
"centy-daemon": patch
---

Shut the daemon down gracefully on SIGTERM and SIGINT. Previously SIGTERM killed the process at once, so launchd stops and upgrades could cut off in-progress writes.
