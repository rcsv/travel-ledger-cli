# Agent Instructions

## General working policy

- Do not expand the task scope unless explicitly requested.
- Do not refactor unrelated code.
- Do not change behavior while doing documentation, release, or versioning work.
- Before editing, inspect the current state and explain the minimal change plan.
- Prefer small, reviewable diffs.
- Do not leave generated artifacts or temporary files behind after release work.

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
