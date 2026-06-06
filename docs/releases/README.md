# Release notes index

GitHub Release 用ノート一覧（新しい順）。

| Version | Title | File |
|---|---|---|
| v1.0.0 (draft) | First stable CLI baseline | [v1.0.0-draft.md](v1.0.0-draft.md) |
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
