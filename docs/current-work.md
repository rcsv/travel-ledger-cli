# Current Work

## Current phase

v4.6.34 planning — Phase W-5 adapter cleanup（任意）または parallel track

## Latest completed

- v4.6.33 Expense write service Phase W-2+W-3 — **released**
- v4.6.32 Expense write service migration plan — **released**
- v4.6.31 Expense write path migration plan — **released**（follow-up `a3a28c8` — DB integrity §15）
- v4.6.30 Expense write path boundary review — **released**
- v4.6.29 Itinerary show aggregate migration plan — **released**
- v4.6.28 Itinerary show aggregate boundary review — **released**
- v4.6.27 Expense output DTO migration follow-up review — **released**
- v4.6.26 Expense output DTO migration Phase 2+3 — **released**
- v4.6.25 Expense output DTO migration plan — **released**

## Repository state

- Cargo version: `4.6.33`
- Latest release: **v4.6.33** — [v4.6.33-notes.md](releases/v4.6.33-notes.md)
- **v4.6.33 spec:** [v4.6.33-expense-write-service-phase-w2-w3.md](specifications/v4.6.33-expense-write-service-phase-w2-w3.md)

## Next action

**任意:** v4.6.34 — Phase W-5 `print_expense_detail` adapter cleanup

**代替候補:**

- SQLite migration runner implementation（parallel track）
- Note / Reservation write service 化（Expense パターン踏襲）
- v4.7.x — itinerary aggregate / GUI / schema publication

**Expense write migration（完了）:**

| Phase | 内容 |
|---|---|
| W-0 | v4.6.31 — **完了** |
| W-1 | v4.6.32 — **完了** |
| W-2+W-3 | v4.6.33 — **完了** |
| W-5 | adapter cleanup — 任意 |
| W-H | `RETURNING id` — 低優先 |

**Parallel track:**

- migration runner / FK hardening（v4.6.31 §15 backlog）

## Defer

- repository 層（v4.7.x）
- `import_expense_v3` 共通化（W-7+）
- Tauri / GUI 実装
- itinerary aggregate 実装

Canonical defer list: [long-term-version-strategy.md](long-term-version-strategy.md)
