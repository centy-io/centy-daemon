---
"@centy-io/centy-daemon": patch
---

`SyncUsers` now fills in `git_usernames` for existing users that have none, using the contributor name from git history. The result tracks `updated` and `would_update` internally. They are not in the gRPC response yet because the proto has no field for them.
