# Add migration dry-run/preview option

Users should be able to preview what migrations will run before committing to the update.

## Current State
- `centy update` prompts for confirmation but doesn't show what will change
- No way to see migration descriptions/impact before running
- Migration executor runs all migrations without preview

## Expected Behavior

### CLI
```bash
$ centy update --dry-run
Would migrate from 0.0.0 to 0.1.0

Migrations to apply:
  1. 0.0.0 → 0.1.0: Establish version tracking system (upgrade)

No changes will be made. Run without --dry-run to apply.
```

### API Addition
Add `GetMigrationPlan` RPC:
```protobuf
message GetMigrationPlanRequest {
  string project_path = 1;
  string target_version = 2;
}

message MigrationPlan {
  string current_version = 1;
  string target_version = 2;
  string direction = 3;  // "upgrade" or "downgrade"
  repeated MigrationStep steps = 4;
}

message MigrationStep {
  string from_version = 1;
  string to_version = 2;
  string description = 3;
}
```

## Files
- `centy-cli/src/commands/update.ts` - Add --dry-run flag
- `centy-daemon/proto/centy.proto` - Add GetMigrationPlan RPC
- `centy-daemon/src/migration/registry.rs` - Expose migration plan info
- `centy-daemon/src/server/mod.rs` - Implement RPC handler
