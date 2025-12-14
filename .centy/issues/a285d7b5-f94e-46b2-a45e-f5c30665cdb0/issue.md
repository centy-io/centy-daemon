# Add organization filtering for advanced search

Add organization filtering to the AdvancedSearch RPC to allow searching within a specific organization.

## Requirements
- Add optional organization_slug parameter to AdvancedSearchRequest
- When multi_project=true and organization_slug is set, only search projects in that organization
- Allow combining with project_path for single project search within an org

## Related
- This is a follow-up to #44 Advanced issue searching
