# Enforce function size limit in Clippy lints

Currently `too_many_lines` is set to "allow" in Cargo.toml (line 92), which means there's no limit on function size. This can lead to overly long functions that are hard to read and maintain.

Proposed change:
- Remove or change `too_many_lines = "allow"` to enforce a reasonable limit
- Consider also enabling `cognitive_complexity` lint to catch complex functions
- Default Clippy limit is 100 lines per function, which is a reasonable starting point

Location: Cargo.toml lines 79-107 ([lints.clippy] section)
