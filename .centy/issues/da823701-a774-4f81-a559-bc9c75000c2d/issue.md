# Add pagination support for advanced search

Add pagination support for the AdvancedSearch RPC to handle large result sets efficiently.

## Requirements
- Add offset and limit parameters to AdvancedSearchRequest
- Return pagination info (total count, has_more, next_offset) in response
- Default limit of 50 results
- Maximum limit of 500 results

## Related
- This is a follow-up to #44 Advanced issue searching
