# Current Work

## Current phase

v4.6.26 planning — expense output DTO migration Phase 2+3（推奨候補）

## Latest completed

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

- Cargo version: `4.6.25`
- Latest release: **v4.6.25** — [v4.6.25-notes.md](releases/v4.6.25-notes.md)
- **v4.6.25 spec:** [v4.6.25-expense-output-dto-migration-plan.md](specifications/v4.6.25-expense-output-dto-migration-plan.md)

## Next action

**v4.6.26 — expense output DTO migration Phase 2+3**（推奨）

- service enriched parts（`ExpenseEnrichedPart` 仮称）追加
- CLI mapper → 既存 `ExpenseJson` / `ExpenseListJson`（shape 不変）
- JSON golden / human output 不変 gate

**代替候補:**

- v4.6.26 — itinerary show aggregate boundary review
- v4.6.26 — SQLite migration runner implementation（parallel track）
- v4.7.0 — Trip Proposal Envelope / Travel Ledger schema publication planning

**Migration plan（v4.6.25）:**

| Phase | 内容 | 状態 |
|---|---|---|
| 1 | migration plan | **v4.6.25 完了** |
| 2 | service enriched parts | v4.6.26 候補 |
| 3 | CLI ExpenseJson mapper | v4.6.26 候補（2 と同時推奨） |
| 4 | `expense_to_json` adapter 化 | v4.6.27 任意 |
| 5 | GUI / Tauri | v4.7.x 以降 |

**Parallel track（v4.6.x、独立）:**

- migration runner / orphan detection / FK hardening（[v4.6.1](specifications/v4.6.1-sqlite-fk-orphan-data-hardening-review.md) / [v4.6.2](specifications/v4.6.2-sqlite-migration-strategy-review.md) review 済み、実装未着手）

## Defer

- Tauri / GUI 実装（Phase 5 まで defer）
- `main.rs` 一括 `commands/` 移動
- write command の service 化（Tier 3+）
- `trip delete` / `import` / `duplicate` / `receipt assign`

Canonical defer list: [long-term-version-strategy.md](long-term-version-strategy.md)
