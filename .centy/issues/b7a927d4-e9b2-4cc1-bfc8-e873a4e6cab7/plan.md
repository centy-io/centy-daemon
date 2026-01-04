# Implementation Plan for Issue #100

**Issue ID**: b7a927d4-e9b2-4cc1-bfc8-e873a4e6cab7
**Title**: Docker E2E Testing
**Status**: Implemented

---

## Overview

Implemented Docker-based end-to-end testing infrastructure for the centy-daemon. The solution provides:

1. **Multi-stage Docker build** that compiles the Rust daemon and sets up Node.js testing environment
2. **CLI wrapper** that simulates CLI commands via gRPC calls
3. **Filesystem snapshot utility** that captures project state after each command
4. **Docker Compose orchestration** for easy test execution
5. **Sample test suite** demonstrating CLI workflow with snapshot assertions

## Implementation

### Files Created

All Docker e2e testing files are encapsulated in the `e2e/` folder:

| File | Purpose |
|------|---------|
| `e2e/Dockerfile` | Multi-stage Docker build for e2e testing |
| `e2e/docker-compose.yml` | Docker Compose orchestration |
| `e2e/scripts/run-docker.sh` | Shell script to run Docker e2e tests |
| `e2e/fixtures/cli-wrapper.ts` | CLI simulation via gRPC |
| `e2e/fixtures/snapshot.ts` | Filesystem snapshot utilities |
| `e2e/cli-snapshot.e2e.spec.ts` | Sample e2e tests with CLI and snapshots |

### Architecture

```
+-------------------------------------------------------------+
|                    Docker Container                          |
+-------------------------------------------------------------+
|  +-----------------+    +-------------------------------+   |
|  |  centy-daemon   |<---|  Vitest E2E Tests             |   |
|  |  (Rust binary)  |    |  +---------------------------+|   |
|  |                 |    |  |  CLI Wrapper (gRPC client)||   |
|  |  Port: 50051    |    |  +---------------------------+|   |
|  |                 |    |  +---------------------------+|   |
|  +-----------------+    |  |  Snapshot Manager         ||   |
|                         |  +---------------------------+|   |
|                         +-------------------------------+   |
|                                                             |
|  /tmp/centy-test-*     --- Test Project Directories        |
|  /app/test-output/     --- Snapshot Output                  |
+-------------------------------------------------------------+
```

### Usage

From the `e2e/` folder:

```bash
# Run all e2e tests in Docker
pnpm docker

# Run tests in watch mode (interactive)
pnpm docker:watch

# Build Docker image only
pnpm docker:build

# Clean up Docker resources
pnpm docker:clean
```

Or using the shell script directly:

```bash
cd e2e
./scripts/run-docker.sh
./scripts/run-docker.sh --watch
./scripts/run-docker.sh --build-only
./scripts/run-docker.sh --clean
```

### CLI Wrapper

The CLI wrapper (`e2e/fixtures/cli-wrapper.ts`) provides a programmatic interface that simulates CLI behavior:

```typescript
const cli = createCLI({ cwd: projectPath });

// Initialize project
const result = await cli.init({ force: true });
expect(result.exitCode).toBe(0);

// Create issues
await cli.issueCreate('Fix bug', { priority: 1 });

// List issues
const list = await cli.issueList({ status: 'open' });
```

### Snapshot Utility

The snapshot utility (`e2e/fixtures/snapshot.ts`) captures filesystem state:

```typescript
const snapshots = new SnapshotManager({ rootPath: projectPath });

// Take snapshots at different points
await snapshots.take('before-init');
await cli.init({ force: true });
await snapshots.take('after-init');

// Compare snapshots
const diff = snapshots.compare('before-init', 'after-init');
expect(diff.added.length).toBeGreaterThan(0);

// Assert file existence
expect(snapshots.assertFileExists('after-init', '.centy/config.json')).toBe(true);

// Get file content from snapshot
const content = snapshots.getFileContent('after-init', '.centy/README.md');
```

### Test Pattern

Each test follows this pattern:

1. Create isolated temp project directory
2. Initialize CLI wrapper and snapshot manager
3. Take baseline snapshot
4. Execute CLI commands
5. Take snapshot after each command
6. Compare snapshots to assert changes
7. Clean up

## Tasks Completed

1. [x] Create Dockerfile.e2e for e2e testing container
2. [x] Create docker-compose.e2e.yml for orchestration
3. [x] Create CLI wrapper for testing (simulates CLI via gRPC)
4. [x] Implement filesystem snapshot utility
5. [x] Create Docker e2e test runner script
6. [x] Update e2e package.json with Docker test scripts
7. [x] Create sample Docker-based e2e test with CLI and snapshots

## Dependencies

- Docker and Docker Compose
- Rust 1.83+ (for building daemon)
- Node.js 22+ (for running tests)
- pnpm 10.24.0 (for package management)

## Edge Cases Handled

- Empty project initialization
- Multiple concurrent snapshots
- Text file content extraction
- Binary file handling (hash-only)
- Directory vs file differentiation
- Error propagation from gRPC to CLI

## Testing Strategy

The sample test suite (`cli-snapshot.e2e.spec.ts`) covers:

- Project initialization workflow
- Issue CRUD operations with snapshot tracking
- Document creation and listing
- Snapshot utilities and diff formatting
- Error handling scenarios
- Daemon info retrieval

All tests run in isolation using temporary directories and clean up after completion.
