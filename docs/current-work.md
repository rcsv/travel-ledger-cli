# Current Work

## Current phase

v4.6.36 — Note write service migration plan（documentation-only、merge gate 済み）

## Latest completed

- v4.6.35 Note write path boundary review — **released**
- v4.6.34 Expense write adapter cleanup — **released**
- v4.6.33 Expense write service Phase W-2+W-3 — **released**
- v4.6.32 Expense write service migration plan — **released**

## Repository state

- Cargo version: `4.6.36`
- Latest release: **v4.6.35** — [v4.6.35-notes.md](releases/v4.6.35-notes.md)
- **v4.6.36 spec:** [v4.6.36-note-write-service-migration-plan.md](specifications/v4.6.36-note-write-service-migration-plan.md)

## Next action

**推奨:** v4.6.37 — note write service Phase N-2+N-3 implementation

- `note_add` / `note_update` / `note_delete` services
- handler 差し替え（`NoteAddParams` 等）
- merge gate: `note_cli` + JSON/human 不変

**Note write migration シーケンス:**

```text
N-0  v4.6.35  boundary review — 完了
N-1  v4.6.36  migration plan — 本書
N-2+3  v4.6.37  implementation（推奨次）
N-5  v4.6.38  docs-only closeout（adapter なし）
```

**Parallel track:** migration runner / FK hardening（v4.6.31 §15 backlog）

**Expense write migration（完了）:** W-0〜W-5 — v4.6.31〜v4.6.34

## Defer

- repository 層（v4.7.x）
- Trip Proposal Envelope / schema-publication（v4.7.x）
- Reservation write service 化（Note 完了後）
- Tauri / GUI 実装

Canonical defer list: [long-term-version-strategy.md](long-term-version-strategy.md)
