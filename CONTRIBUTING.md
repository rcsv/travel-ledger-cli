# Contributing

Thank you for contributing to this project.

This document defines the basic development rules for this repository. It is intended for both human contributors and AI coding assistants.

## Project principles

This project values small, reviewable changes.

Keep changes focused, intentional, and easy to verify. Avoid mixing unrelated refactoring, formatting churn, or opportunistic fixes into feature work.

A good change should make it clear:

* what was changed
* why it was changed
* how it was verified
* whether any behavior, output, or documentation changed

When in doubt, prefer a smaller change.

## Scope discipline

Follow the requested scope strictly.

Do not make unrelated changes just because they appear nearby or seem easy to fix. If you notice an issue outside the current scope, document it separately instead of fixing it immediately.

Examples of out-of-scope changes include:

* unrelated refactoring
* renaming files, modules, functions, or commands without explicit instruction
* changing output formats unexpectedly
* updating golden files without an intentional output change
* changing Cargo versions during documentation-only planning (without an explicit release instruction)
* modifying release notes or roadmap files for unrelated versions

If the requested work is unclear, make the smallest reasonable interpretation and state any assumptions in the completion report.

## Development commands

Before considering a change complete, run the appropriate checks.

The default final verification command is:

```bash
make check
```

Depending on the scope of the change, the following commands may also be useful:

```bash
cargo fmt
cargo clippy
cargo test
cargo build
```

If a check cannot be run, explain why in the completion report.

Do not claim that a check passed unless it was actually run.

## Documentation rules

Specification documents go under:

```text
docs/specifications/
```

Release notes go under:

```text
docs/releases/
```

The current active planning state is tracked in:

```text
docs/current-work.md
```

When behavior changes, update the relevant documentation.

When the change is documentation-only, do not modify source code or golden files unless explicitly instructed.

Documentation-only **planning** work should not bump `Cargo.toml` / `Cargo.lock`. An **explicit release** may bump the package version and Okinawa colophon per the repository release checklist.

Documentation-only changes should be limited to the requested documents and related indexes.

## Golden file rules

Golden files must only be updated when an output change is intentional.

Do not update golden files just to make tests pass without understanding the reason for the output difference.

When golden files are changed, the completion report must explain:

* which golden files changed
* what output changed
* why the change is intentional

If the output change is unexpected, stop and report it instead of accepting the new golden output.

## Release rules

Do not perform release actions unless explicitly instructed.

Release actions include:

* changing the package version
* updating `Cargo.toml`
* updating `Cargo.lock`
* creating a git tag
* creating or editing a GitHub Release
* pushing release commits
* running release workflows

Preparing release notes is not the same as performing a release.

Version references must be kept consistent when a release is explicitly requested. Common files to check include:

```text
Cargo.toml
Cargo.lock
README.md
docs/current-work.md
docs/releases/README.md
docs/specifications/README.md
samples/okinawa_sesoko_2026/expected-export-md.md  # colophon Version only
```

Release procedure details: [tools/release/README.md](tools/release/README.md) and the latest release-workflow follow-up spec under `docs/specifications/`.

After a formal release, a separate follow-up commit may update `docs/current-work.md` to mark the version as released and note the next planning phase.

Do not assume release permission from planning, review, or documentation work.

## AI assistant rules

AI coding assistants must follow these rules:

* Read `docs/current-work.md` before starting work.
* Follow the requested scope strictly.
* Prefer documenting concerns over making unsolicited fixes.
* Do not perform unrelated refactoring.
* Do not update golden files unless the output change is intentional.
* Do not modify Cargo versions during documentation-only planning without an explicit release instruction.
* Do not create tags, releases, or release commits unless explicitly instructed.
* Report changed files, checks run, and any skipped checks.
* State assumptions clearly.
* Never assume release permission.

AI assistants should optimize for predictable, reviewable changes rather than broad cleanup.

## Completion report format

At the end of the work, provide a concise completion report with the following sections:

```text
## Summary

- What was changed
- Why it was changed

## Changed files

- path/to/file
- path/to/another-file

## Verification

- command run: result
- command run: result

## Notes

- Any assumptions
- Any skipped checks
- Any follow-up items
```

If nothing was changed, say so clearly.

If any check failed, include the failure and do not describe the work as complete unless the failure is expected and explained.

## General rule

A contribution is complete only when the change is focused, documented when necessary, and verified.

Small, boring, predictable changes are preferred.
