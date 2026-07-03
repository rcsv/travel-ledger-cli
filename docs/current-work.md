# Current Work

## Current phase

v4.6.27 planning — expense output DTO migration follow-up review

## Latest completed

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

- Cargo version: `4.6.26`
- Latest release: **v4.6.26** — [v4.6.26-notes.md](releases/v4.6.26-notes.md)
- **v4.6.26 spec:** [v4.6.26-expense-output-dto-migration-phase-2-3.md](specifications/v4.6.26-expense-output-dto-migration-phase-2-3.md)
- Implementation: `bed654a` — enriched expense output context for read-only services

## Next action

**v4.6.27 — expense output DTO migration follow-up review**（推奨、documentation-first）

- adapter 残置（`expense_to_json` / `print_expense_*`）の整理方針
- write path と read-only path の責務確認
- Phase 4（adapter 縮小）着手前の確認

**代替候補:**

- v4.6.27 — itinerary show aggregate boundary review
- v4.6.27 — SQLite migration runner implementation（parallel track）
- v4.7.0 — Trip Proposal Envelope / Travel Ledger schema publication planning

**Migration status（v4.6.25 plan）:**

| Phase | 内容 | 状態 |
|---|---|---|
| 1 | migration plan | v4.6.25 ✓ |
| 2 | service enriched parts | **v4.6.26 ✓** |
| 3 | CLI ExpenseJson mapper | **v4.6.26 ✓** |
| 4 | `expense_to_json` adapter 化 | v4.6.27+ 候補 |
| 5 | GUI / Tauri | v4.7.x 以降 |

**Parallel track（v4.6.x、独立）:**

- migration runner / orphan detection / FK hardening（[v4.6.1](specifications/v4.6.1-sqlite-fk-orphan-data-hardening-review.md) / [v4.6.2](specifications/v4.6.2-sqlite-migration-strategy-review.md) review 済み、実装未着手）

## Defer

- Tauri / GUI 実装（Phase 5 まで defer）
- write command の service 化（Tier 3+）
- `trip delete` / `import` / `duplicate` / `receipt assign`

Canonical defer list: [long-term-version-strategy.md](long-term-version-strategy.md)
