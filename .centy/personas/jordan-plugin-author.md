---
createdAt: 2026-02-26T00:41:55.132528+00:00
updatedAt: 2026-02-26T00:41:55.132528+00:00
customFields:
  pain-points: The hooks system lets me run a script on events, but there is no install contract — users must manually edit hook config, keep the binary path correct, and remember to re-enable it after daemon updates. There is no way to declare what events my plugin listens to, what version it is, or whether it is healthy. If my plugin crashes silently, nobody knows. I cannot distribute my plugin via npm/cargo and have it just work.
  role: Open-Source Developer / Tool Builder
  goals: Distribute a centy plugin (e.g. GitHub sync) that hooks into daemon lifecycle events. Have the daemon install and track the plugin — version, enabled state, health — so it runs reliably without the user wiring hooks by hand. Surface the plugin in centy's own UI so it feels like a first-class integration, not a hidden shell script.
---

# Jordan – Plugin Author

Independent developer or OSS contributor who builds tools that extend centy. Has already written a working GitHub sync tool — it listens to item events and mirrors issues to/from GitHub Issues via the hooks system. Wants to ship it as an installable plugin so any centy user can add it with one command, and the daemon manages its lifecycle.
