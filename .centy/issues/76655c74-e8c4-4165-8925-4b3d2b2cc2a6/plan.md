# Implementation Plan for Issue #117

**Issue ID**: 76655c74-e8c4-4165-8925-4b3d2b2cc2a6
**Title**: Add 100 lines limit for Rust files

---

## Overview

Add a CI-enforced check that limits Rust source files to 100 lines maximum. This follows the existing pattern in `.github/workflows/build.yml` where the project already enforces `too_many_lines` lint suppression detection via shell script.

**Current State**: The codebase has files significantly exceeding 100 lines (e.g., `src/server/mod.rs` at 4,611 lines). A staged rollout approach is required.

**Approach**: Implement a bash-based CI check (consistent with existing lint enforcement patterns) with an allowlist mechanism for existing large files that will be refactored incrementally.

## Tasks

1. **Create file line count check script**
   - Add `scripts/check-file-lines.sh` that scans `*.rs` files in `src/`
   - Count lines for each file and compare against 100-line threshold
   - Support reading an allowlist file for exceptions
   - Output clear error messages listing violating files

2. **Create allowlist for existing files**
   - Add `.centy/allowlist-large-files.txt` containing paths of files exceeding 100 lines
   - Reference the corresponding refactoring issue for each file
   - This serves as technical debt tracking

3. **Create separate issues for refactoring large files**
   - Issue #118: Refactor `src/server/mod.rs` (4,611 lines) - Split into submodules
   - Issue #119: Refactor `src/item/entities/doc/crud.rs` (1,343 lines)
   - Issue #120: Refactor `src/item/entities/issue/crud.rs` (1,277 lines)
   - Issue #121: Refactor `src/item/entities/pr/crud.rs` (692 lines)
   - Issue #122: Refactor `src/item/entities/issue/assets.rs` (639 lines)
   - Issue #123: Refactor `src/workspace/metadata.rs` (543 lines)
   - Issue #124: Refactor `src/registry/organizations.rs` (525 lines)
   - Issue #125: Refactor `src/llm/agent.rs` (523 lines)
   - Issue #126: Refactor `src/config/mod.rs` (502 lines)
   - Issue #127: Refactor `src/reconciliation/managed_files.rs` (499 lines)
   - Additional issues for remaining files >100 lines (to be enumerated during implementation)

4. **Integrate check into CI workflow**
   - Add new step in `.github/workflows/build.yml` after the existing forbidden lint suppression check
   - Run the check script as a required gate
   - Fail the build if any non-allowlisted file exceeds 100 lines

5. **Add pre-commit hook integration**
   - Extend `cargo-husky` hooks to run the check locally
   - Developers get immediate feedback before pushing

6. **Document the policy**
   - Update `CONTRIBUTING.md` (if exists) or add section to README
   - Explain the 100-line limit rationale and how to request exceptions

## Dependencies

- No external dependencies required
- Uses existing CI infrastructure (GitHub Actions)
- Uses existing pre-commit hook system (cargo-husky)

## Edge Cases

| Case | Handling |
|------|----------|
| Generated code files | Add to allowlist with `# generated` comment |
| Test files with many test cases | Consider if tests should also be split; may need separate threshold |
| Files at exactly 100 lines | Pass (limit is > 100, not >= 100) |
| Empty files | Pass (0 lines < 100) |
| Files with only comments/whitespace | Count all lines (including blank lines) |
| Symlinks | Skip or resolve to avoid double-counting |
| Hidden files (`.*.rs`) | Skip - not standard Rust source files |

## Testing Strategy

1. **Unit test the script**
   - Create test files with various line counts (99, 100, 101 lines)
   - Verify script correctly identifies violations
   - Verify allowlist mechanism works

2. **Integration test in CI**
   - Create a test branch with a deliberately large file
   - Verify CI fails as expected
   - Verify allowlisted files don't cause failures

3. **Local developer testing**
   - Test pre-commit hook triggers on new large files
   - Test that existing allowlisted files don't block commits

## Implementation Details

### Script Structure (`scripts/check-file-lines.sh`)
```bash
#!/bin/bash
MAX_LINES=100
ALLOWLIST=".centy/allowlist-large-files.txt"

# Find all .rs files, count lines, check against limit
# Skip files in allowlist
# Exit 1 if violations found
```

### CI Integration (`.github/workflows/build.yml`)
```yaml
- name: Check Rust file line limits
  run: |
    chmod +x scripts/check-file-lines.sh
    ./scripts/check-file-lines.sh
```

### Allowlist Format (`.centy/allowlist-large-files.txt`)
```
# Files exceeding 100 lines - linked to refactoring issues
# Format: path/to/file.rs # Issue #NNN

src/server/mod.rs # Issue #118
src/item/entities/doc/crud.rs # Issue #119
src/item/entities/issue/crud.rs # Issue #120
src/item/entities/pr/crud.rs # Issue #121
src/item/entities/issue/assets.rs # Issue #122
src/workspace/metadata.rs # Issue #123
src/registry/organizations.rs # Issue #124
src/llm/agent.rs # Issue #125
src/config/mod.rs # Issue #126
src/reconciliation/managed_files.rs # Issue #127
# ... additional files with their corresponding issues
```

---

> **Note**: After completing this plan, save it using:
> ```bash
> centy add plan 117 --file .centy/issues/76655c74-e8c4-4165-8925-4b3d2b2cc2a6/plan.md
> ```
