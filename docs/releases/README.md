# Release notes index

GitHub Release 用ノート一覧（新しい順）。

| Version | Title | File |
|---|---|---|
| v1.8.1 | Travel activity terminology consistency | [v1.8.1-notes.md](v1.8.1-notes.md) |
| v1.8.0 | Itinerary model (not a venue) | [v1.8.0-notes.md](v1.8.0-notes.md) |
| v1.7.0 | Canonical sample dataset and expense verification | [v1.7.0-notes.md](v1.7.0-notes.md) |
| v1.6.0 | Export Schema v3 with Expense backup | [v1.6.0-notes.md](v1.6.0-notes.md) |
| v1.5.0 | Expense CRUD support | [v1.5.0-notes.md](v1.5.0-notes.md) |
| v1.4.1 | trip diff Note support | [v1.4.1-notes.md](v1.4.1-notes.md) |
| v1.4.0 | Export Schema v2 (Notes) | [v1.4.0-notes.md](v1.4.0-notes.md) |
| v1.3.1 | Transaction-safe cascade deletion | [v1.3.1-notes.md](v1.3.1-notes.md) |
| v1.3.0 | Note commands | [v1.3.0-notes.md](v1.3.0-notes.md) |
| v1.2.0 | Day commands | [v1.2.0-notes.md](v1.2.0-notes.md) |
| v1.1.0 | Itinerary day_id model | [v1.1.0-notes.md](v1.1.0-notes.md) |
| v1.0.9 | Day model and required trip dates | [v1.0.9-notes.md](v1.0.9-notes.md) |
| v1.0.8 | Export metadata | [v1.0.8-notes.md](v1.0.8-notes.md) |
| v1.0.7 | Backup validation and import summary | [v1.0.7-notes.md](v1.0.7-notes.md) |
| v1.0.6 | Structured doctor/advisor JSON | [v1.0.6-notes.md](v1.0.6-notes.md) |
| v1.0.5 | Data completeness and backup reliability | [v1.0.5-notes.md](v1.0.5-notes.md) |
| v1.0.4 | Checklist export/import and release binaries | [v1.0.4-notes.md](v1.0.4-notes.md) |
| v1.0.3 | Release workflow validation (prerelease) | [v1.0.3-notes.md](v1.0.3-notes.md) |
| v1.0.2 | Markdown export and documentation polish | [v1.0.2-notes.md](v1.0.2-notes.md) |
| v1.0.1 | Not found handling polish | [v1.0.1-notes.md](v1.0.1-notes.md) |
| v1.0.0 | First stable CLI baseline | [v1.0.0-notes.md](v1.0.0-notes.md) |
| v0.9.5 | CI and release verification polish | [v0.9.5-notes.md](v0.9.5-notes.md) |
| v0.9.4 | Command reference polish | [v0.9.4-notes.md](v0.9.4-notes.md) |
| v0.9.3 | Doctor JSON output | [v0.9.3-notes.md](v0.9.3-notes.md) |
| v0.9.2 | Checklist JSON output | [v0.9.2-notes.md](v0.9.2-notes.md) |
| v0.9.1 | JSON output polish | [v0.9.1-notes.md](v0.9.1-notes.md) |
| v0.9.0 | Structured DoctorIssue Targets | [v0.9.0-notes.md](v0.9.0-notes.md) |
| v0.8.1 | Advisor command hints | [v0.8.1-notes.md](v0.8.1-notes.md) |
| v0.8.0 | Trip Advisor | [v0.8.0-notes.md](v0.8.0-notes.md) |
| v0.7.0 | checklist-generate combination rules | [v0.7.0-notes.md](v0.7.0-notes.md) |
| v0.6.1 | Improved trip doctor output | [v0.6.1-notes.md](v0.6.1-notes.md) |
| v0.6.0 | trip doctor | [v0.6.0-notes.md](v0.6.0-notes.md) |

Legacy: [v0.6.0.md](v0.6.0.md)

## Release verification

Before creating a release, run:

- `cargo fmt`
- `cargo clippy -- -D warnings`
- `cargo test`
- `make check`

Confirm GitHub Actions `Rust CI` succeeds on the release commit (`master` push).

After creating a release, verify:

- Git tag exists
- GitHub Release exists
- Release notes are linked from `docs/releases/README.md`
- Working tree is clean
