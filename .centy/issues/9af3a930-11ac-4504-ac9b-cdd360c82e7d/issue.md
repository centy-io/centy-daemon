# Add common trait interface for Issues, PRs, and Docs

## Background

Currently, Issues, PRs, and Docs share similar capabilities but lack a unified trait interface:

|Field|Issue|PR|Doc|
|-----|-----|--|---|
|created_at|✓ (via CommonMetadata)|✓ (via CommonMetadata)|✓ (direct)|
|updated_at|✓ (via CommonMetadata)|✓ (via CommonMetadata)|✓ (direct)|
|deleted_at|✓ (direct)|✓ (direct)|✓ (direct)|
|display_number|✓ (via CommonMetadata)|✓ (via CommonMetadata)|✗|
|status|✓ (via CommonMetadata)|✓ (via CommonMetadata)|✗|
|priority|✓ (via CommonMetadata)|✓ (via CommonMetadata)|✗|
|is_org_X|✓|✗|✓|
|org_slug|✓|✗|✓|

## Problem

1. CommonMetadata is shared by Issue/PR but NOT by Doc
1. deleted_at is duplicated in ALL THREE types (not centralized in CommonMetadata)
1. No unified trait interface for common operations
1. OrgSyncable trait exists but only Issues implement it (Docs do manual sync)

## Proposed Solution (Smallest Scope)

Add two minimal traits that capture the true intersection of all item types:

````rust
// 1. Timestamps - ALL items have these
pub trait Timestamped {
    fn created_at(&self) -> &str;
    fn updated_at(&self) -> &str;
    fn touch(&mut self);  // Update updated_at to now
}

// 2. Soft-delete - ALL items have this  
pub trait SoftDeletable: Timestamped {
    fn deleted_at(&self) -> Option<&str>;
    fn is_deleted(&self) -> bool { self.deleted_at().is_some() }
    fn soft_delete(&mut self);
    fn restore(&mut self);
}
````

## Benefits

* 2 traits covering the true intersection of all types
* Zero breaking changes to existing code
* Foundation for unified CRUD and gRPC API later
* Immediate value - enables generic functions

## Implementation Steps

1. Create Timestamped trait in src/common/traits.rs
1. Create SoftDeletable trait extending Timestamped
1. Implement both traits for Issue
1. Implement both traits for PullRequest
1. Implement both traits for Doc
1. Add deleted_at to CommonMetadata (optional consolidation)
1. Update Docs to use OrgSyncable trait instead of manual sync

## Future Extensions

Once the base traits are in place, we can add:

* CentyItem trait for unified CRUD
* Numbered trait for display_number support
* Prioritized trait for priority support
* Unified gRPC API layer
