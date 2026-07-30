---
"@centy-io/centy-daemon": patch
---

Fixed the orphan-link cleanup sweep wrongly deleting valid cross-type `relates-to` links when a custom item type's folder name didn't naively pluralize to match (e.g. a `story` type stored in a `stories` folder). Endpoint existence is now resolved through the item type registry, consistent with link creation. (#296)
