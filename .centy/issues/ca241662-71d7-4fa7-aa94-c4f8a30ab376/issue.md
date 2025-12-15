# Allow users to pin/declare project version in config.json

Users should be able to manually set and edit the version field in config.json to pin their project to a specific daemon version.

## Current State
- `version` field exists in config.json but is only set by migrations
- Users cannot directly edit/set the version through CLI or UI
- No validation that a declared version is supported by the daemon

## Expected Behavior
1. **centy init --version X.X.X**: Initialize project at specific version
2. **centy config set version X.X.X**: Set version in existing project
3. **Validation**: Warn if declared version is not in `available_versions`
4. **UI**: Add version dropdown in settings to pin version
5. **Semver support**: Accept semver ranges like `^0.1.0` or exact `0.1.0`

## Use Cases
- Team wants all developers on same daemon version
- User wants to stay on older version until ready to migrate
- Testing compatibility with specific version

## Files
- `centy-daemon/src/config/mod.rs` - CentyConfig struct
- `centy-cli/src/commands/init.ts` - Init command
- `centy-cli/src/commands/config.ts` - Config command
- `centy-app/components/settings/ProjectConfig.tsx` - UI settings
