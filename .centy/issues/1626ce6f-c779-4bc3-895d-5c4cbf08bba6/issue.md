# Add strict Clippy lints to deny panic/unwrap usage

Add strict Clippy restriction lints to the `Cargo.toml` to enforce safer error handling patterns and eliminate runtime panics.

## Proposed Changes

Add the following lints to `Cargo.toml`:

```toml
[lints.clippy]
panic = "deny"
unwrap_used = "deny"
expect_used = "deny"
panic_in_result_fn = "deny"
unwrap_in_result = "deny"
```

## Scope of Work

**174 occurrences** across **37 files** will need to be refactored:

| File | Count |
|------|-------|
| `search/parser.rs` | 21 |
| `item/entities/issue/reconcile.rs` | 19 |
| `item/entities/pr/reconcile.rs` | 19 |
| `llm/work.rs` | 11 |
| `manifest/mod.rs` | 8 |
| `item/entities/issue/metadata.rs` | 8 |
| `common/metadata.rs` | 7 |
| `item/core/metadata.rs` | 7 |
| `llm/config.rs` | 7 |
| `sync/worktree.rs` | 6 |
| ... and 27 more files |

## Motivation

1. **Reliability**: Panics crash the daemon process, causing service disruption
2. **Predictability**: Forces explicit error handling at all call sites
3. **API Contract**: Functions that return `Result` should never panic
4. **Debuggability**: Proper error propagation provides better error messages than panic backtraces
5. **Production Readiness**: Critical for a service that runs continuously

## Suggested Approach

1. Add lints with `"warn"` level first to identify all violations
2. Refactor code incrementally by module:
   - Replace `unwrap()`/`expect()` with `?` operator where possible
   - Use `ok_or()`/`ok_or_else()` for `Option` → `Result` conversions
   - Replace `panic!()` with proper error returns
   - Consider `anyhow::bail!()` for complex error scenarios
3. Once all warnings are resolved, change lint level to `"deny"`
