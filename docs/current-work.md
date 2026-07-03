# Current Work

## Current phase

v4.6.32 planning — expense write path Phase W-2+W-3 implementation（推奨候補）

## Latest completed

- v4.6.31 Expense write path migration plan — **released**
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

- Cargo version: `4.6.31`
- Latest release: **v4.6.31** — [v4.6.31-notes.md](releases/v4.6.31-notes.md)
- **v4.6.31 plan:** [v4.6.31-expense-write-path-migration-plan.md](specifications/v4.6.31-expense-write-path-migration-plan.md)

## Next action

**v4.6.32 — expense write path Phase W-2+W-3 implementation**（推奨）

- `expense_add` / `expense_update` / `expense_delete` thin services
- handler 接続 + `print_expense_detail_from_enriched`
- `expense_cli` tests 出力不変 gate

**Expense write migration（v4.6.31 結論）:**

| Phase | 内容 |
|---|---|
| W-0 | v4.6.31 plan — **完了** |
| W-1 | CLI 現状維持 |
| W-2+W-3 | thin services + handler — **v4.6.32 候補** |
| W-5 | adapter cleanup — 任意 |
| W-D | repository — v4.7.x defer |

**代替候補:**

- v4.6.32 — SQLite migration runner implementation（parallel track）
- v4.7.x — itinerary aggregate / GUI / schema publication

**Parallel track（v4.6.x、独立）:**

- migration runner / orphan detection / FK hardening（[v4.6.1](specifications/v4.6.1-sqlite-fk-orphan-data-hardening-review.md) / [v4.6.2](specifications/v4.6.2-sqlite-migration-strategy-review.md) review 済み、実装未着手）
  - **NEW backlog（v4.6.31 §15）:** `expenses` / `reservations` / `estimates` の `itinerary_id` は **DB FK なし・app validation 依存**。FK 導入は orphan detection → `user_version` → table rebuild の順で本 track へ課題化。

## Defer

- repository 層抽出（v4.7.x）
- Note / Reservation write service 化（Expense 先行後）
- itinerary aggregate 実装（GUI タイムライン連動）
- Tauri / GUI 実装
- `import_expense_v3` 共通化（W-6 以降）

Canonical defer list: [long-term-version-strategy.md](long-term-version-strategy.md)
