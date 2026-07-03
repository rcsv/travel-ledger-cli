# Current Work

## Current phase

v4.6.34 released — Expense write adapter cleanup（Phase W-5）

## Latest completed

- v4.6.34 Expense write adapter cleanup — **released**
- v4.6.33 Expense write service Phase W-2+W-3 — **released**
- v4.6.32 Expense write service migration plan — **released**
- v4.6.31 Expense write path migration plan — **released**（follow-up `a3a28c8` — DB integrity §15）
- v4.6.30 Expense write path boundary review — **released**

## Repository state

- Cargo version: `4.6.34`
- Latest release: **v4.6.34** — [v4.6.34-notes.md](releases/v4.6.34-notes.md)
- **v4.6.34 spec:** [v4.6.34-expense-write-adapter-cleanup.md](specifications/v4.6.34-expense-write-adapter-cleanup.md)

## Next action

**候補:**

- SQLite migration runner implementation（parallel track）
- Note / Reservation write service 化（Expense パターン踏襲）
- Travel Book — `format_expense_markdown_line` export 接続 or cleanup 判断
- v4.7.x — itinerary aggregate / GUI / schema publication

**Expense write migration（完了）:**

| Phase | 内容 |
|---|---|
| W-0 | v4.6.31 — **完了** |
| W-1 | v4.6.32 — **完了** |
| W-2+W-3 | v4.6.33 — **完了** |
| W-5 | v4.6.34 — **完了** |
| W-H | `RETURNING id` — 低優先 |

**Parallel track:**

- migration runner / FK hardening（v4.6.31 §15 backlog）

## Defer

- repository 層（v4.7.x）
- `import_expense_v3` 共通化（W-7+）
- Tauri / GUI 実装
- itinerary aggregate 実装

Canonical defer list: [long-term-version-strategy.md](long-term-version-strategy.md)
