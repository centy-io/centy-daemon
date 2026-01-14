# Support creating workspaces without an issue (standalone workspaces)

Add support for creating temporary workspaces that are not tied to a specific issue.

## Motivation

Currently, workspaces can only be created in the context of an issue. This limits use cases like:

* Ad-hoc exploration or prototyping
* General AI-assisted development work
* Quick experiments not worth creating an issue for

## Implementation

1. Add new proto messages:
   
   * `OpenStandaloneWorkspaceRequest` (project_path, optional name/description, ttl_hours, agent_name)
   * `OpenStandaloneWorkspaceResponse` (similar to OpenInTempVscodeResponse but without issue fields)
1. Add new RPC endpoint:
   
   * `OpenStandaloneWorkspace` - creates workspace without issue context
1. Modify workspace management:
   
   * Update workspace metadata schema to make issue_id optional
   * Update `ListTempWorkspaces` to return a flag indicating standalone vs issue-based
   * Update workspace naming to use custom name or generate one for standalone workspaces
1. Agent prompt handling:
   
   * Create a generic prompt template for standalone workspaces (no issue context)
   * Allow user to provide custom instructions/goals
