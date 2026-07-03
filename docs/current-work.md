# Current Work

## Current phase

v4.6.25 planning — expense output DTO migration plan（推奨候補）

## Latest completed

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

- Cargo version: `4.6.24`
- Latest release: **v4.6.24** — [v4.6.24-notes.md](releases/v4.6.24-notes.md)
- **v4.6.24 spec:** [v4.6.24-expense-dto-context-ownership-review.md](specifications/v4.6.24-expense-dto-context-ownership-review.md)

## Next action

**v4.6.25 — expense output DTO migration plan**（推奨、documentation-first）

- Option C: service enriched parts + CLI mapper → 既存 `ExpenseJson`（shape 不変）
- v4.6.24 ownership 方針の Phase 1 設計書

**代替候補:**

- v4.6.25 — itinerary show aggregate boundary review
- v4.6.25 — SQLite migration runner implementation（parallel track）
- v4.7.0 — Trip Proposal Envelope / Travel Ledger schema publication planning

**Ownership 方針（v4.6.24 結論）:**

- 短期: Option A 現状維持（`expense_to_json` / `ExpenseJson` 不変）
- migration 推奨: Option C（service が context parts、CLI が `ExpenseJson` 組み立て）

**Parallel track（v4.6.x、独立）:**

- migration runner / orphan detection / FK hardening（[v4.6.1](specifications/v4.6.1-sqlite-fk-orphan-data-hardening-review.md) / [v4.6.2](specifications/v4.6.2-sqlite-migration-strategy-review.md) review 済み、実装未着手）

## Defer

- Tauri / GUI 実装
- `main.rs` 一括 `commands/` 移動
- write command の service 化（Tier 3+）
- `trip delete` / `import` / `duplicate` / `receipt assign`

Canonical defer list: [long-term-version-strategy.md](long-term-version-strategy.md)
