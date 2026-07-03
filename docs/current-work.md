# Current Work

## Current phase

v4.6.31 planning — expense write path migration plan（推奨候補）

## Latest completed

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

- Cargo version: `4.6.30`
- Latest release: **v4.6.30** — [v4.6.30-notes.md](releases/v4.6.30-notes.md)
- **v4.6.30 review:** [v4.6.30-expense-write-path-boundary-review.md](specifications/v4.6.30-expense-write-path-boundary-review.md)

## Next action

**v4.6.31 — expense write path migration plan**（推奨）

- Option W-B（thin write services）を中心に Phase 定義
- human output / JSON golden 不変 gate
- adapter 縮小は plan 後

**Expense write path（v4.6.30 結論）:**

| 項目 | 状態 |
|---|---|
| write service 化 | **未着手**（Tier 3+ defer 継続） |
| 責務 | handler 薄配線 + `expense.rs` mutation |
| add/update 後表示 | `print_expense_detail` adapter — **許容** |
| itinerary 親子 | **一貫**（`--itinerary` / `itinerary_id` FK） |

**Itinerary aggregate（v4.6.29）:**

- Phase 1 維持（Option A）— GUI 着手まで defer

**代替候補:**

- v4.6.31 — SQLite migration runner implementation（parallel track）
- v4.7.x — itinerary aggregate Phase 2+ / schema publication

**Parallel track（v4.6.x、独立）:**

- migration runner / orphan detection / FK hardening（[v4.6.1](specifications/v4.6.1-sqlite-fk-orphan-data-hardening-review.md) / [v4.6.2](specifications/v4.6.2-sqlite-migration-strategy-review.md) review 済み、実装未着手）

## Defer

- expense write 実装（migration plan 後）
- itinerary aggregate 実装（GUI タイムライン連動）
- Tauri / GUI 実装
- repository 層抽出（v4.7.x 候補）

Canonical defer list: [long-term-version-strategy.md](long-term-version-strategy.md)
