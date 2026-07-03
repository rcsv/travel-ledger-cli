# Current Work

## Current phase

v4.6.33 planning — expense write service Phase W-2+W-3 implementation（推奨候補）

## Latest completed

- v4.6.32 Expense write service migration plan — **released**
- v4.6.31 Expense write path migration plan — **released**（follow-up `a3a28c8` — DB integrity §15）
- v4.6.30 Expense write path boundary review — **released**
- v4.6.29 Itinerary show aggregate migration plan — **released**
- v4.6.28 Itinerary show aggregate boundary review — **released**
- v4.6.27 Expense output DTO migration follow-up review — **released**
- v4.6.26 Expense output DTO migration Phase 2+3 — **released**
- v4.6.25 Expense output DTO migration plan — **released**
- v4.6.24 Expense DTO context ownership review — **released**
- v4.6.23 Read-only helper context review — **released**
- v4.6.22 Read-only service boundary completion review — **released**
- v4.6.21 Expense show service boundary — **released**
- v4.6.20 Reservation show service boundary — **released**
- v4.6.19 Day show service boundary — **released**
- v4.6.18 Note show service boundary — **released**
- v4.6.17 Checklist show service boundary — **released**
- v4.6.16 Read-only service boundary follow-up review — **released**
- v4.6.15 Checklist list service boundary — **released**
- v4.6.14 Expense list service boundary — **released**
- v4.6.13 Reservation list service boundary — **released**
- v4.6.12 Note list service boundary — **released**
- v4.6.11 Read-only service boundary review — **released**
- v4.6.10 Itinerary show service boundary — **released**
- v4.6.9 Itinerary timeline service boundary — **released**
- v4.6.8 Itinerary list service boundary — **released**
- v4.6.7 Day list service boundary — **released**
- v4.6.6 Trip show service boundary — **released**
- v4.6.5 Read-only service boundary expansion — **released**
- v4.6.4 Read-only service boundary pilot — **released**

## Repository state

- Cargo version: `4.6.32`
- Latest release: **v4.6.32** — [v4.6.32-notes.md](releases/v4.6.32-notes.md)
- **v4.6.32 plan:** [v4.6.32-expense-write-service-migration-plan.md](specifications/v4.6.32-expense-write-service-migration-plan.md)
- **v4.6.31 plan:** [v4.6.31-expense-write-path-migration-plan.md](specifications/v4.6.31-expense-write-path-migration-plan.md)

## Next action

**v4.6.33 — expense write service Phase W-2+W-3 implementation**（推奨）

実装仕様: [v4.6.32-expense-write-service-migration-plan.md](specifications/v4.6.32-expense-write-service-migration-plan.md)

| 項目 | 内容 |
|---|---|
| 新規 | `expense_add` / `expense_update` / `expense_delete` services |
| handler | `print_expense_detail_from_enriched` 接続 |
| result | add/update → `ExpenseEnrichedPart`、delete → snapshot |
| gate | `expense_cli` tests 出力不変 |

**Expense write migration Phase:**

| Phase | 内容 |
|---|---|
| W-0 | v4.6.31 overview — **完了** |
| W-1 | v4.6.32 detailed plan — **完了** |
| W-2+W-3 | implementation — **v4.6.33 候補** |
| W-H | `RETURNING id` hardening — 任意・低優先 |
| W-5 | adapter cleanup — 任意 |

**代替候補:**

- v4.6.33 — SQLite migration runner implementation（parallel track）
- v4.7.x — itinerary aggregate / GUI / schema publication

**Parallel track（v4.6.x、独立）:**

- migration runner / orphan detection / FK hardening（[v4.6.1](specifications/v4.6.1-sqlite-fk-orphan-data-hardening-review.md) / [v4.6.2](specifications/v4.6.2-sqlite-migration-strategy-review.md)）
  - `itinerary_id` DB FK なし — v4.6.31 §15 backlog（write service 化とは別ストリーム）

## Defer

- repository 層（v4.7.x）
- `import_expense_v3` 共通化（W-7+）
- `last_insert_rowid` → `RETURNING`（W-H、W-3 後）
- Note / Reservation write service 化
- itinerary aggregate / Tauri / GUI

Canonical defer list: [long-term-version-strategy.md](long-term-version-strategy.md)
