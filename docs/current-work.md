# Current Work

## Current phase

v4.6.28 planning — itinerary show aggregate boundary review（推奨候補）

## Latest completed

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

- Cargo version: `4.6.27`
- Latest release: **v4.6.27** — [v4.6.27-notes.md](releases/v4.6.27-notes.md)
- **v4.6.27 spec:** [v4.6.27-expense-output-dto-migration-follow-up-review.md](specifications/v4.6.27-expense-output-dto-migration-follow-up-review.md)

## Next action

**v4.6.28 — itinerary show aggregate boundary review**（推奨、documentation-first）

- Reservations handler-side fetch の aggregate 方針整理
- expense migration track は v4.6.27 で一区切り

**代替候補:**

- v4.6.28 — expense write path boundary review
- v4.6.28 — SQLite migration runner implementation（parallel track）
- v4.7.0 — Trip Proposal Envelope / Travel Ledger schema publication planning

**Expense migration 状態（v4.6.27 結論）:**

| 項目 | 状態 |
|---|---|
| read-only list/show | service enriched parts ✓ |
| adapter 残置 | write path（add/update）+ tests — **妥当** |
| Phase 4 | **部分達成** — cleanup は write 連動まで defer |

**Parallel track（v4.6.x、独立）:**

- migration runner / orphan detection / FK hardening（[v4.6.1](specifications/v4.6.1-sqlite-fk-orphan-data-hardening-review.md) / [v4.6.2](specifications/v4.6.2-sqlite-migration-strategy-review.md) review 済み、実装未着手）

## Defer

- adapter 削除 / Phase 4 cleanup（write path まで defer）
- Tauri / GUI 実装
- write command の service 化（Tier 3+）
- `trip delete` / `import` / `duplicate` / `receipt assign`

Canonical defer list: [long-term-version-strategy.md](long-term-version-strategy.md)
