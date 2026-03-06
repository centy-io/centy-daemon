---
createdAt: 2026-03-06T15:34:49.729296+00:00
updatedAt: 2026-03-06T15:34:49.729296+00:00
---

# Replace Custom Logic with Libraries

# Replace Custom Logic with Libraries

Scan the codebase for hand-rolled implementations that duplicate functionality already available in well-maintained open-source libraries. For each finding, create a centy issue describing the custom code, the suggested library replacement, and the benefit of switching.

## What to Look For
- Custom HTTP clients or request wrappers (replace with axios, ky, got, undici)
- Hand-rolled date/time formatting or manipulation (replace with date-fns, dayjs, luxon)
- Custom deep clone, merge, or diff utilities (replace with lodash, structuredClone, deepmerge)
- DIY schema validation (replace with zod, yup, ajv, valibot)
- Custom retry/backoff logic (replace with p-retry, async-retry)
- Hand-written CSV/YAML/TOML/INI parsers (replace with papaparse, js-yaml, smol-toml)
- Custom glob or file matching (replace with globby, picomatch, micromatch)
- Custom slug/string utilities (replace with slugify, change-case)
- Custom UUID/ID generation (replace with uuid, nanoid, cuid2)
- Custom color/ANSI handling (replace with chalk, picocolors)
- Custom argument parsing (replace with commander, yargs, citty)
- Custom semver comparison (replace with semver)
- Custom event emitter implementations (replace with eventemitter3, mitt)
- Custom queue or rate-limiting logic (replace with p-queue, p-limit, bottleneck)
- Custom caching layers (replace with lru-cache, keyv)

## Process
1. Walk through the source tree file by file
2. Identify custom utility code that reimplements library functionality
3. For each finding, create a centy issue with:
   - Title: short description of what to replace
   - Body: file path, lines of custom code, suggested library, and migration notes
4. Mark this routine as completed when the full scan is done

<!-- centy-routine-version: 1 -->
