# Current Work

## Current phase

v4.6.38 released — Note write migration **完了**

## Latest completed

- v4.6.38 Note write service Phase N-5 docs-only closeout — **released**
- v4.6.37 Note write service Phase N-2+N-3 — **released**
- v4.6.36 Note write service migration plan — **released**
- v4.6.35 Note write path boundary review — **released**

## Repository state

- Cargo version: `4.6.38`
- Latest release: **v4.6.38** — [v4.6.38-notes.md](releases/v4.6.38-notes.md)
- **v4.6.38 spec:** [v4.6.38-note-write-service-phase-n5-closeout.md](specifications/v4.6.38-note-write-service-phase-n5-closeout.md)

## Note write migration — **完了**

| Phase | Version | 状態 |
|---|---|---|
| N-0 | v4.6.35 | **完了** |
| N-1 | v4.6.36 | **完了** |
| N-2+N-3 | v4.6.37 | **完了** |
| N-5 | v4.6.38 | **完了** |

## Next action

**候補（いずれか）:**

- Reservation write path boundary review（Note/Expense 型紙横展開）
- SQLite migration runner / FK hardening（parallel track）
- v4.7.x — schema-publication / GUI 新章

**Expense write migration（完了）:** W-0〜W-5 — v4.6.31〜v4.6.34

## Defer

- repository 層（v4.7.x）
- Trip Proposal Envelope / schema-publication（v4.7.x）
- `import_export_notes` / `import_expense_v3` 共通化
- N-H / W-H RETURNING id hardening

Canonical defer list: [long-term-version-strategy.md](long-term-version-strategy.md)
