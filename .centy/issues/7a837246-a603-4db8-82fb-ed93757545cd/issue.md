# Agent process should attach to IDE terminal when spawned from workspace

## Problem

When spawning an agent via `centy issue <id> --action plan` from a temporary workspace:

1. The agent process runs but is not visibly attached to the IDE terminal
2. There's no indication to the user that the agent will update the `plan.md` file
3. The user cannot see the agent's progress or output

## Expected Behavior

- The agent process should be attached to the VS Code terminal that runs the task
- The agent's output should be visible in real-time
- Clear indication that the agent is working on filling in `plan.md`

## Current Behavior

- Agent spawns with `Stdio::inherit()` but may not properly attach to the IDE's terminal
- No visual feedback about what the agent is doing
- User doesn't know if/when the plan.md will be updated

## Technical Context

In `src/llm/agent.rs`, the agent is spawned with:
```rust
cmd.stdin(Stdio::null())
    .stdout(Stdio::inherit())
    .stderr(Stdio::inherit());
```

This inherits stdout/stderr but the process may not be properly connected when spawned from a VS Code task.

## Possible Solutions

1. Spawn agent as a foreground process instead of fire-and-forget
2. Use `exec` to replace the centy process with the agent process
3. Pipe agent output through centy and display it
4. Add progress indicators or status updates
