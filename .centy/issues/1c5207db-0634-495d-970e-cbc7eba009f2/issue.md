# Increase test coverage to 50%

## Objective

Increase test coverage from current 37.62% to 50% (target: ~830 more lines covered).

## Current State

* **Current coverage:** 37.62% (2,524/6,709 lines)
* **Target coverage:** 50% (~3,355 lines)
* **Lines to cover:** ~830 additional lines

## Modules with Zero Tests (High Impact)

|Module|Lines|Tests|Priority|
|------|-----|-----|--------|
|`src/config/mod.rs`|204|0|High|
|`src/reconciliation/managed_files.rs`|317|0|High|
|`src/reconciliation/plan.rs`|176|0|High|
|`src/reconciliation/execute.rs`|132|0|High|
|`src/migration/executor.rs`|173|0|Medium|
|`src/migration/types.rs`|74|0|Medium|

## Implementation Plan

### 1. Add tests for `src/config/mod.rs`

* [ ] Test `default_priority_levels()` returns 3
* [ ] Test `default_allowed_states()` returns correct states
* [ ] Test `default_state()` returns “open”
* [ ] Test `CentyConfig::default()` initialization
* [ ] Test `CentyConfig::effective_version()` with and without version
* [ ] Test `CentyConfig` serialization/deserialization
* [ ] Test `LlmConfig::default()` initialization
* [ ] Test `CustomFieldDefinition` serialization
* [ ] Test `ProjectMetadata` serialization

### 2. Add tests for `src/reconciliation/managed_files.rs`

* [ ] Test `get_managed_files()` returns expected file templates
* [ ] Test `ManagedFileTemplate` contains correct file types
* [ ] Test all managed directories are included
* [ ] Test all managed files have content

### 3. Add tests for `src/reconciliation/plan.rs`

* [ ] Test `ReconciliationPlan::needs_decisions()` returns true when restore/reset not empty
* [ ] Test `ReconciliationPlan::needs_decisions()` returns false when empty
* [ ] Test `FileInfo` struct initialization
* [ ] Test `build_reconciliation_plan()` with empty directory
* [ ] Test `build_reconciliation_plan()` with existing files

### 4. Add tests for `src/reconciliation/execute.rs`

* [ ] Test `ReconciliationDecisions::default()` initialization
* [ ] Test `ReconciliationResult::default()` initialization

### 5. Add tests for `src/migration/types.rs`

* [ ] Test `MigrationError` variants
* [ ] Test `MigrationResult` struct
* [ ] Test `MigrationDirection` enum values

### 6. Add tests for `src/migration/executor.rs`

* [ ] Test `MigrationExecutor::new()` initialization

### 7. Expand tests in `src/features/crud.rs`

* [ ] Test `build_compacted_refs()` with various inputs
* [ ] Test `generate_migration_frontmatter()`

## Success Criteria

* [ ] Coverage reaches 50% or higher
* [ ] All new tests pass
* [ ] CI pipeline passes
* [ ] No regressions in existing tests
