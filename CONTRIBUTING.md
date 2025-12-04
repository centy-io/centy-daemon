# Contributing to centy-daemon

Thank you for your interest in contributing to centy-daemon! This document provides guidelines and information for contributors.

## Getting Started

### Prerequisites

- Rust 1.70+ (2021 edition)
- Git

### Setup

```bash
git clone https://github.com/centy-io/centy-daemon.git
cd centy-daemon
cargo build
```

### Verify Your Setup

```bash
cargo check        # Type checking
cargo test         # Run tests
cargo run          # Start the daemon
```

## Development Workflow

### Branch Naming

- `feat/description` - New features
- `fix/description` - Bug fixes
- `docs/description` - Documentation changes
- `refactor/description` - Code refactoring
- `test/description` - Test additions or changes

### Making Changes

1. Create a new branch from `main`
2. Make your changes
3. Run tests: `cargo test --all-targets`
4. Run checks: `cargo check --all-targets`
5. Build release: `cargo build --release`
6. Commit with a descriptive message
7. Open a pull request

### Commit Messages

Follow the [Conventional Commits](https://www.conventionalcommits.org/) specification:

```
<type>(<scope>): <description>

[optional body]

[optional footer]
```

Types:
- `feat` - New feature
- `fix` - Bug fix
- `docs` - Documentation only
- `refactor` - Code change that neither fixes a bug nor adds a feature
- `test` - Adding or updating tests
- `chore` - Maintenance tasks
- `ci` - CI/CD changes

Examples:
```
feat(issue): add support for issue labels
fix(manifest): correct hash calculation for empty files
docs(readme): add grpcui testing instructions
```

## Project Structure

```
src/
├── main.rs              # Entry point, server setup
├── lib.rs               # Public API exports
├── server/              # gRPC service implementation
├── manifest/            # Manifest handling
├── issue/               # Issue CRUD operations
├── reconciliation/      # .centy folder reconciliation
├── config/              # Configuration management
└── utils/               # Shared utilities
```

### Adding a New Feature

1. **Update proto** - Add messages/RPCs in `proto/centy.proto`
2. **Rebuild** - Run `cargo build` to regenerate proto code
3. **Add domain logic** - Create/update modules in `src/`
4. **Implement RPC** - Add handler in `src/server/mod.rs`
5. **Write tests** - Add integration tests in `tests/`
6. **Update docs** - Update README if needed

## Code Style

### Rust Conventions

- Use `rustfmt` for formatting (default settings)
- Follow [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Use `thiserror` for custom error types
- Prefer `async/await` for I/O operations

### JSON Serialization

All JSON uses camelCase:

```rust
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MyStruct {
    pub field_name: String,  // Serializes as "fieldName"
}
```

### Error Handling

Use custom error enums with `thiserror`:

```rust
#[derive(Error, Debug)]
pub enum MyError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Custom error message")]
    CustomError,
}
```

## Testing

### Running Tests

```bash
cargo test                    # All tests
cargo test --all-targets      # Including integration tests
cargo test issue              # Tests matching "issue"
cargo test -- --nocapture     # Show println output
```

### Writing Tests

- Place unit tests in the same file using `#[cfg(test)]` module
- Place integration tests in `tests/` directory
- Use `tempfile` crate for isolated test directories
- Use the test utilities in `tests/common/mod.rs`

Example integration test:

```rust
use centy_daemon::issue::{create_issue, CreateIssueOptions};
use common::{create_test_dir, init_centy_project};

#[tokio::test]
async fn test_my_feature() {
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();

    init_centy_project(project_path).await;

    // Your test logic here
}
```

## Testing the gRPC API

### Using grpcui

```bash
# Install grpcui
go install github.com/fullstorydev/grpcui/cmd/grpcui@latest

# Start the daemon
cargo run --release

# In another terminal, launch grpcui
grpcui -plaintext 127.0.0.1:50051
```

### Using grpcurl

```bash
# List services
grpcurl -plaintext 127.0.0.1:50051 list

# Call an RPC
grpcurl -plaintext -d '{"project_path": "/path/to/project"}' \
  127.0.0.1:50051 centy.CentyDaemon/IsInitialized
```

## Pull Request Process

1. Ensure all tests pass: `cargo test --all-targets`
2. Ensure code compiles: `cargo build --release`
3. Update documentation if needed
4. Fill out the PR template with:
   - Description of changes
   - Related issue (if any)
   - Testing performed
5. Request review from maintainers

## Reporting Issues

When reporting issues, please include:

- Rust version (`rustc --version`)
- Operating system
- Steps to reproduce
- Expected behavior
- Actual behavior
- Error messages or logs

## Questions?

- Open an issue for questions about the codebase
- Check existing issues before creating new ones

## License

By contributing, you agree that your contributions will be licensed under the MIT License.
