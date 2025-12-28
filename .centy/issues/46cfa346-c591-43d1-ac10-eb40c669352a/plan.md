# Implementation Plan for Issue #82

**Issue ID**: 46cfa346-c591-43d1-ac10-eb40c669352a
**Title**: Enforce unique project names within organization

---

## Overview

Currently, multiple projects within the same organization can have identical names (derived from their directory names). This creates ambiguity for URL routing (e.g., `/org/myapp/issues`) and could lead to user confusion. This issue implements a uniqueness constraint to ensure project names are unique within each organization.

**Approach**:
- Add validation in `set_project_organization()` to check for duplicate project names
- Introduce a new error variant `ProjectError::DuplicateNameInOrganization`
- Update the RPC handler to return appropriate error responses
- Ensure validation happens atomically within the registry lock
- Handle edge cases like case-sensitivity and organization transfers

**Key Files**:
- `src/registry/organizations.rs` - Add validation logic
- `src/registry/types.rs` - Add new error variant
- `proto/centy.proto` - Update error responses if needed
- `tests/registry_org_test.rs` - Add comprehensive tests

## Tasks

1. **Add new error variant for duplicate project names**
   - Add `DuplicateNameInOrganization` to `ProjectError` enum in `src/registry/types.rs`
   - Include helpful context (project name, organization slug) in error message

2. **Implement uniqueness validation helper**
   - Create `check_project_name_uniqueness()` helper in `src/registry/organizations.rs`
   - Accept `registry`, `organization_slug`, `project_path`, `project_name` parameters
   - Return error if another project in the same org has the same name (case-insensitive comparison)
   - Exclude the current project path from the check (for idempotent calls)

3. **Update `set_project_organization()` to enforce uniqueness**
   - Call `check_project_name_uniqueness()` before assigning organization
   - Perform check within the registry lock to ensure atomicity
   - Return appropriate error if validation fails

4. **Add RPC error handling**
   - Update gRPC handler in `src/server/mod.rs` to convert `ProjectError::DuplicateNameInOrganization` to appropriate gRPC status
   - Ensure error message is user-friendly and actionable

5. **Add comprehensive tests**
   - Test duplicate name rejection when assigning to organization
   - Test that same project path can be re-assigned (idempotent)
   - Test case-insensitive matching (e.g., "MyApp" vs "myapp")
   - Test moving project between organizations with duplicate names
   - Test that projects in different orgs can have same name
   - Test projects without organization can have duplicate names

6. **Documentation updates**
   - Update any relevant error documentation
   - Consider adding to changelog/migration guide if breaking change

## Dependencies

**Prerequisites**:
- None - this is a standalone validation enhancement

**Related Features**:
- Organization management (`src/registry/organizations.rs`)
- Project tracking (`src/registry/tracking.rs`)
- May interact with future URL routing features that rely on unique names

## Edge Cases

1. **Case Sensitivity**
   - Decision: Use case-insensitive comparison to prevent confusion
   - Example: "MyApp" and "myapp" should be considered duplicates

2. **Project Reassignment (Idempotency)**
   - When calling `set_project_organization()` on a project already in the org
   - Should NOT fail validation (exclude current project from check)

3. **Moving Project Between Organizations**
   - Project "myapp" in Org A moving to Org B where "myapp" already exists
   - Should fail with clear error message
   - Successful move if target org doesn't have duplicate

4. **Projects Without Organization**
   - Multiple unorganized projects can have the same name
   - Only enforce uniqueness within organizations

5. **Concurrent Modifications**
   - Registry lock ensures atomic read-modify-write
   - Two simultaneous assignments with same name should fail for one

6. **Directory Name Changes**
   - If user renames project directory, name changes automatically
   - Previous name becomes available for other projects in the org
   - No special handling needed (name is always derived from current path)

7. **Organization Deletion**
   - Projects lose their organization assignment
   - No impact on this feature (handled by existing deletion logic)

8. **Whitespace and Special Characters**
   - Project names are derived from directory names
   - OS already restricts special characters in directory names
   - Trim whitespace during comparison for robustness

## Testing Strategy

**Unit Tests** (in `tests/registry_org_test.rs`):

1. **Basic Duplicate Detection**
   - Create org with project "myapp" at path `/path/to/myapp`
   - Attempt to add project "myapp2" with directory name "myapp" to same org
   - Assert: Returns `ProjectError::DuplicateNameInOrganization`

2. **Case Insensitive Matching**
   - Add project with name "MyApp"
   - Attempt to add project with name "myapp" to same org
   - Assert: Fails validation

3. **Idempotency Check**
   - Assign project to organization
   - Call `set_project_organization()` again with same project and org
   - Assert: Succeeds without error

4. **Different Organizations Allow Same Name**
   - Create two organizations (org-a, org-b)
   - Add project "myapp" to org-a
   - Add different project "myapp" to org-b
   - Assert: Both succeed

5. **Moving Between Organizations**
   - Add "myapp" to org-a
   - Create org-b with different "myapp"
   - Attempt to move org-a's "myapp" to org-b
   - Assert: Fails with duplicate error

6. **Unorganized Projects**
   - Create multiple projects without organization, all named "myapp"
   - Assert: All can exist without organization

7. **Organization to Unorganized**
   - Project in org with duplicate name in another org
   - Remove organization assignment (set to None)
   - Assert: Succeeds

**Integration Tests** (optional):
- Full RPC workflow with gRPC client
- Verify error propagation through gRPC layer
- Test with actual filesystem paths

**Manual Testing**:
- Use `centy` CLI to assign projects to organizations
- Verify error messages are clear and actionable
- Test real-world workflow of organizing existing projects

---

 > 
 > **Note**: After completing this plan, save it using:
 > 
 > ````bash
 > centy add plan 82 --file .centy/issues/46cfa346-c591-43d1-ac10-eb40c669352a/plan.md
 > ````
