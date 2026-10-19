---
"centy-daemon": minor
---

Release v0.16.0.

Behaviour change: a relative `project_path` is now rejected with `INVALID_PATH` (#231). Clients must send absolute paths; centy-cli 0.8.6 already does.

New:
- `CheckForUpdates` RPC, which checks GitHub releases with a 10 s timeout (#225)
- `ListItemsAcrossProjects` RPC (#226)
- Built-in `mentions` / `mentioned-by` link type (#332)
- Warning on duplicate project slugs within an organization (#229)
- Missing `git_usernames` filled for existing users during sync (#232)
- Startup failures show the log file location (#230)
- Hook execution history and logging (#103)

Fixes:
- The daemon shuts down gracefully on SIGTERM and SIGINT instead of being killed (#334)
- Item types are returned in a stable order (#333)
- `projects.json` is written atomically with the tempfile crate (#8)
