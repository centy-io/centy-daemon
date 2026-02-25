---
displayNumber: 188
status: in-progress
priority: 2
createdAt: 2026-02-15T18:09:48.637151+00:00
updatedAt: 2026-02-24T14:40:45.668640+00:00
---

# Create an assert service to enforce preconditions before each command

Currently, many setup and validation steps only happen during `centy init`. If a user runs other commands (e.g., item commands) without having run `init` first, or if managed files get deleted/corrupted after init, things can break silently.

## Goal

Create an assert service that runs before each command to ensure preconditions are met. This moves validation out of `init`-only logic and into a shared layer that guards every command execution in a repo context.

## Requirements

* The assert service should run automatically before any command that operates within a centy-managed repo
* It should check that required files exist (e.g., `config.yaml` for item commands, `.centy-manifest.json`, managed files)
* If a precondition is not met, it should either:
  * Auto-fix it (recreate the missing file from the template, like a lightweight reconciliation)
  * Or return a clear error telling the user to run `centy init`
* The assert checks should be composable - different commands may require different subsets of assertions
* The service should be fast and not add noticeable latency to every command
* `init` itself should be exempt from most assertions (since it’s the command that establishes the preconditions)

## Implementation Notes

* Consider a trait or middleware pattern where each command declares its required assertions
* Extract the relevant file-existence checks and template creation logic out of the init command into the assert service
* The assert service could also be the place where future invariants are enforced (e.g., manifest schema version compatibility, minimum centy version checks)
