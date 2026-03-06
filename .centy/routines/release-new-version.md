---
createdAt: 2026-03-06T15:55:28.749802+00:00
updatedAt: 2026-03-06T15:55:28.749802+00:00
---

# Release New Version

# Release New Version

Prepare and release a new version of the project. Walk through the full release checklist, ensuring every step is completed before publishing.

**Important: This routine must be executed fully autonomously. Do not ask the user for the version number or any other input. Determine everything from the git history and project context.**

## Step 0 — Understand the Ecosystem

Before doing anything else, identify the project's ecosystem and current secure release practices:

1. Detect the project type by inspecting manifest files (package.json, pyproject.toml, Cargo.toml, go.mod, *.gemspec, etc.) and the existing CI/CD configuration
2. Search the web for the **current** most secure, recommended way to publish packages for that ecosystem — focus on:
   - Trusted publishing / OIDC-based authentication (e.g. npm trusted publishers, PyPI trusted publishing) instead of long-lived secret tokens
   - Provenance attestations and supply-chain signing (e.g. npm provenance, Sigstore)
   - Required CLI / runtime versions for these features
   - Any recently deprecated flows (e.g. legacy tokens, older auth methods) that the project might still be using
3. Review the project's existing CI/CD publish workflow and flag anything that is outdated or insecure compared to current best practices
4. Use the findings from steps 1-3 to inform every subsequent step — the release must follow the most up-to-date, secure process available for the detected ecosystem

## Process

1. Determine the next version number by analyzing git log since the last tag — use commit messages and change scope to decide the semver bump (patch, minor, or major). Do not ask the user.
2. Update the version in all relevant files (package.json, lock files, etc.)
3. Update the changelog (or create one if missing) with a summary of changes since the last release
4. Ensure all tests pass and the project builds cleanly
5. If Step 0 revealed that the CI/CD workflow is outdated or insecure, update it to follow current best practices before proceeding
6. Create a git commit with the version bump and changelog update (and CI/CD changes, if any)
7. Create a git tag for the new version
8. Push the commit and tag to the remote
9. Create a GitHub release with the changelog entry as the body — this should trigger the CI/CD publish workflow
10. Wait for the CI/CD workflow triggered by the release to complete, then verify it passed successfully. If it failed, inspect the logs and attempt to fix the issue.

## Notes

- Do not prompt the user for any input — infer everything from git history, tags, and project files
- If any step fails, stop and create a centy issue describing the failure
- Do not skip the changelog — every release should be documented
- Mark this routine as completed once the release is published and CI/CD has passed


<!-- centy-routine-version: 2 -->
