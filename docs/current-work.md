# Current Work

## Current phase

v4.6.36 planning — note write service migration plan

## Latest completed

- v4.6.35 Note write path boundary review — **released**
- v4.6.34 Expense write adapter cleanup — **released**
- v4.6.33 Expense write service Phase W-2+W-3 — **released**
- v4.6.32 Expense write service migration plan — **released**
- v4.6.31 Expense write path migration plan — **released**

## Repository state

- Cargo version: `4.6.35`
- Latest release: **v4.6.35** — [v4.6.35-notes.md](releases/v4.6.35-notes.md)
- **v4.6.35 spec:** [v4.6.35-note-write-path-boundary-review.md](specifications/v4.6.35-note-write-path-boundary-review.md)

## Next action

**推奨:** v4.6.36 — note write service migration plan

- owner validation 契約
- `NoteAddParams` / `NoteUpdateParams` / `NoteDeleteParams`
- result 型（`NoteAddServiceResult` 等）
- N-2+N-3 implementation merge gate

**候補シーケンス:**

```text
v4.6.35 — note write path boundary review — 完了
v4.6.36 — note write service migration plan
v4.6.37 — note write service implementation
v4.6.38 — note write adapter cleanup（要否は N-1 で判定・縮小見込み）
```

**Parallel track:** migration runner / FK hardening（v4.6.31 §15 backlog）

**Expense write migration（完了）:**

| Phase | 内容 |
|---|---|
| W-0〜W-5 | v4.6.31〜v4.6.34 — **完了** |

## Defer

- repository 層（v4.7.x）
- Trip Proposal Envelope / schema-publication（v4.7.x）
- Tauri / GUI 実装

Canonical defer list: [long-term-version-strategy.md](long-term-version-strategy.md)
