# Agent Instructions

## General working policy

- Do not expand the task scope unless explicitly requested.
- Do not refactor unrelated code.
- Do not change behavior while doing documentation, release, or versioning work.
- Before editing, inspect the current state and explain the minimal change plan.
- Prefer small, reviewable diffs.
- Keep the working tree clean after release work.

## Scope discipline

When the user asks for a local implementation fix:

- Touch only the files needed for that fix.
- Do not update docs, fixtures, generated output, or golden files unless required by the change.
- If a related change seems necessary, stop and explain why before editing it.

When the user asks for release/version work:

- Treat it as a repository-wide consistency pass.
- Search first, edit second.
- Look for old version references across the whole repository.
- Update all required release/version references, not only README.md.
- Do not implement new features.
- Do not refactor.
- Do not change runtime behavior.

## Release/version checklist

For every release bump, check at least:

- Cargo.toml
- Cargo.lock
- README.md
- docs/current-work.md
- docs/release/index.md
- docs/specification/index.md
- release notes
- markdown golden files
- generated fixture outputs
- any colophon/version footer
- GitHub workflow or packaging files if they embed the version

Before editing, run or equivalent:

```sh
git status --short
rg -n "v[0-9]+\.[0-9]+\.[0-9]+|Version [0-9]+\.[0-9]+\.[0-9]+|version = \"[0-9]+\.[0-9]+\.[0-9]+\""
```

Then classify findings as:

- Must update
- Should remain historical
- Unsure / ask before editing