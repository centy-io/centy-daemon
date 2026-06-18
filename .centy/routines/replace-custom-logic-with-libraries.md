---
createdAt: 2026-03-06T19:05:58.770021+00:00
updatedAt: 2026-03-06T19:05:58.770021+00:00
---

# Replace Custom Logic with Libraries

# Replace Custom Logic with Libraries

Scan the codebase for hand-rolled implementations that duplicate functionality already available in well-maintained, existing libraries. For each finding, create a centy issue describing the custom code, the suggested replacement, and the benefit of switching.

## What to Look For
- Any custom utility, helper, or module that reimplements functionality available in an established library
- Wrapper code that adds little value over using a library directly
- Complex logic that a battle-tested library handles more reliably (parsing, validation, networking, crypto, etc.)

## Process
1. Walk through the source tree file by file
2. For each piece of custom logic, consider whether a well-known library already solves the same problem
3. For each finding, create a centy issue with:
   - Title: short description of what to replace
   - Body: file path, what the custom code does, suggested library, and migration notes
4. Mark this routine as completed when the full scan is done

<!-- centy-routine-version: 2 -->
