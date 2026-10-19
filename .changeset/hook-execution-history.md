---
"@centy-io/centy-daemon": patch
---

Record every hook run (output, exit code, duration, timeout, blocked operation) in `.centy/hook_executions.jsonl`. Add the `ListHookExecutions` and `GetHookExecution` gRPC endpoints to read that history.
