# Agent Instructions

## General working policy

- Do not expand the task scope unless explicitly requested.
- Do not refactor unrelated code.
- Do not change behavior while doing documentation, release, or versioning work.
- Before editing, inspect the current state and explain the minimal change plan.
- Prefer small, reviewable diffs.
- Do not leave generated artifacts or temporary files behind after release work.

## Source of truth and required reading

Before making decisions or changing files, inspect the repository state and read the documents relevant to the task.

At minimum, read:

* `docs/current-work.md`
* this `AGENTS.md`
* `CONTRIBUTING.md`
* the relevant specification, release note, source code, and tests

For Desktop GUI work, also read:

* `docs/specifications/desktop-gui-north-star.md`

Use the following source priority when information conflicts:

1. Current repository code and tests
2. `docs/current-work.md`
3. Relevant current specifications and release documents
4. `AGENTS.md` and `CONTRIBUTING.md`
5. Conversation context and previous summaries

Do not rely on remembered project state when the repository can be inspected.

## Work modes

Classify the requested work before editing.

### Discussion / design consultation

When the user asks to discuss, explore, review, compare, or design:

* Inspect the relevant repository context.
* Discuss options, tradeoffs, risks, and recommended direction.
* Do not edit files.
* Do not create implementation plans unless requested.
* Do not commit, push, bump versions, create tags, or perform release actions.
* Wait for an explicit request before converting the discussion into repository changes.

A request to discuss a feature is not permission to implement it.

### Documentation-only work

When the user explicitly requests documentation changes:

* Change only the requested documents and directly related indexes.
* Do not change runtime code, tests, fixtures, golden files, schemas, or Cargo versions unless explicitly requested.
* Do not turn a design document into an implementation without a separate instruction.
* Treat planning, implementation, and release as separate work units.

### Implementation work

When the user explicitly requests implementation:

* Implement the smallest safe and independently useful slice.
* Keep GUI, CLI, application services, data model, and export/import responsibilities separate.
* Do not mix unrelated refactoring into feature work.
* Do not change schemas, migrations, public formats, or versions unless explicitly authorized.
* Run the checks appropriate to the actual change.

### Review work

When the user asks for a review:

* Inspect the requested code, diff, specification, or repository state.
* Report findings before making changes.
* Do not fix findings unless the user explicitly requests fixes.
* Separate blocking issues, non-blocking concerns, and future considerations.

### Release work

Release work requires an explicit release instruction.

Planning a release, writing release notes, reviewing readiness, or committing an implementation does not grant permission to:

* bump versions
* create release commits
* push
* create tags
* create GitHub Releases
* run release workflows

## Authority boundaries

Permissions apply only to the current requested task. Do not carry permissions forward from an earlier task.

| Action                                  | Default                        |
| --------------------------------------- | ------------------------------ |
| Read/search repository files            | Allowed                        |
| Run non-destructive inspection commands | Allowed                        |
| Run relevant tests and checks           | Allowed                        |
| Edit files                              | Only when explicitly requested |
| Create or modify documentation          | Only when explicitly requested |
| Change runtime behavior                 | Only when explicitly requested |
| Change DB schema or migrations          | Requires explicit permission   |
| Change export/import or public schema   | Requires explicit permission   |
| Change Cargo version                    | Requires explicit permission   |
| Commit                                  | Requires explicit permission   |
| Push                                    | Requires explicit permission   |
| Create or update a PR                   | Requires explicit permission   |
| Create tag or GitHub Release            | Requires explicit permission   |
| Perform formal release                  | Requires explicit permission   |

Permission to commit does not imply permission to push.

Permission to implement does not imply permission to commit.

Permission to prepare a release does not imply permission to perform a release.

If the user explicitly grants one of these permissions, perform only that action and the minimum prerequisites required for it.

## Repository state safety

Before editing, inspect:

```sh
git status --short --branch
git diff
```

* Preserve unrelated user changes.
* Do not discard, reset, stash, clean, or overwrite changes that are outside the requested scope.
* Do not include unrelated changes in a commit.
* If the working tree contains unexpected changes that make the requested work unsafe, explain the conflict before modifying those files.
* Do not leave temporary files, generated artifacts, debug output, or local-only configuration behind.

## Product and Desktop GUI direction

For Desktop GUI design and implementation, treat the following as the North Star:

```text
docs/specifications/desktop-gui-north-star.md
```

Key principles include:

* The primary experience is planning a trip, not managing database entities.
* The central hierarchy remains `Trip → Day → Itinerary`.
* The Trip Workspace is organized around user goals such as `Overview`, `Plan`, `Checklist`, `Travelers`, and `Money`.
* Proposal Fragment / Suggestions is a supporting intake and adoption mechanism, not the primary product experience.
* Prefer the smallest independently useful Desktop slice.
* Do not postpone current usability solely for speculative future architecture.
* Do not introduce schema changes merely to simplify a GUI implementation.
* Keep GUI, CLI, data model, application services, and export/import responsibilities distinct.

When a proposed GUI feature conflicts with the North Star, identify the conflict before implementing it.

## Completion report

Keep completion reports concise. Do not repeat the full task instructions or provide a long chronological work log.

Use this format:

```text
## Result
- Complete / incomplete
- Outcome summary

## Changed files
- `path`
  - Change summary
- `None` if no files changed

## Validation
- `command`: PASS / FAIL
- Skipped checks and reason

## Git
- Commit: `<hash>` or `not performed`
- Status: concise `git status --short --branch` summary

## Permissions
- push: performed / not performed
- tag: performed / not performed
- version bump: performed / not performed
- formal release: performed / not performed
- PR: performed / not performed

## Decisions needed
- Items requiring user judgment
- `None` if no decision is needed

## Next candidate
- One smallest useful next step
```

Do not propose or begin the next candidate automatically.

## README editorial policy

The root README is a product entrance, not a changelog, spec dump, or internal planning note.

Keep it focused on:

- What the CLI does
- Real CLI output
- Safe Quick Start using `--db`
- User-facing capabilities
- Installation
- Links to deeper docs

Do not reintroduce:

- Destructive Quick Start examples such as `db reset`
- Long release history tables
- Unverified commands or output
- Internal AI/spec/planning material
- Confusion between Caglla.Travel CLI, `travel-ledger-cli`, Travel Ledger, and `caglla.db`

When adding detail, link out instead of expanding README:

| Topic | Put it here |
|---|---|
| Product overview, Quick Start, install | `README.md` |
| AI coding agent rules | `AGENTS.md` |
| AI integration concepts | `docs/ai.md` |
| Proposal / Fragment public contract | `docs/public/proposals.md` |
| AI JSON generation rules | `docs/public/ai-json-generation-guide.md` |
| Command options and examples | `docs/command-reference.md` |
| Internal specs and planning | `docs/specifications/` |
| Release history | `docs/releases/` |
| Contributor / release workflow | `CONTRIBUTING.md`, `tools/release/README.md` |

## Naming conventions

Use names consistently. Do not guess or unify without checking the binary, Cargo package, and release workflow.

| Name | Meaning |
|---|---|
| **Caglla.Travel CLI** | Product name (`--about` output) |
| **Travel Ledger** | Public Trip data format (export schema v8) |
| **travel-ledger-cli** | Repository, Cargo package, release binary, and **command name** |
| **caglla.db** / **caglla.toml** | Default DB filename and project config filename |

In user-facing docs and Quick Start examples, prefer `travel-ledger-cli`. Do not mix `cargo run --` with installed-binary examples in the same Quick Start block.

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
- README.md (Latest Release one-liner only — not a full history table)
- docs/current-work.md
- docs/release/README.md
- docs/specification/README.md
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
