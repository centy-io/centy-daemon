# Roadmap

- We want to let the user configure custom entity types beyond the built-in "issue", "doc", and "pr" types. Users should be able to define new entity types like "feature", "bug", "task", etc. with custom templates and schemas for each type.

- We want to make the `.centy` database directory name generic and configurable. Instead of being hardcoded to `.centy`, users should be able to configure the folder name while maintaining backward compatibility with `.centy`.

- We want to let the user configure custom hooks that run synchronously or asynchronously before and after each database operation (create, list, delete, update, etc.).

- We want to let the user configure editors instead of hardcoding support for VS Code and terminal.

- We want to let the user configure custom templates for each entity type, so they can have different record formats for "doc", "issue", "feature", etc.

- We want to let the user configure custom commands for each entity type, so they can have different operations for "doc", "issue", "feature", etc.
