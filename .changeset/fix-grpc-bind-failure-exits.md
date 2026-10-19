---
"@centy-io/centy-daemon": patch
---

Fixed the daemon staying up after its gRPC port was already in use. It now exits non-zero with an error that names the address.
