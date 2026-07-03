# Current Work

## Current phase

v4.6.30 planning — expense write path boundary review（推奨候補）

## Latest completed

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

- Cargo version: `4.6.29`
- Latest release: **v4.6.29** — [v4.6.29-notes.md](releases/v4.6.29-notes.md)
- **v4.6.29 plan:** [v4.6.29-itinerary-show-aggregate-migration-plan.md](specifications/v4.6.29-itinerary-show-aggregate-migration-plan.md)
- **v4.6.28 review:** [v4.6.28-itinerary-show-aggregate-boundary-review.md](specifications/v4.6.28-itinerary-show-aggregate-boundary-review.md)

## Next action

**v4.6.30 — expense write path boundary review**（推奨）

**Itinerary aggregate migration（v4.6.29 結論）:**

| Phase | 内容 |
|---|---|
| 0 | v4.6.29 plan — **完了** |
| 1 | CLI 現状維持（Option A） |
| 2 | GUI 初号機は個別 service compose |
| 3 | Option C / D — UX 確定後 |
| 4 | CLI human 内部寄せ（任意） |

| 方針 | 内容 |
|---|---|
| Option A | 短期維持 |
| Option B | **慎重** — reservations だけ show result に混ぜない |
| Option C / D | 中期有力候補 |
| JSON | `ItineraryItem` のみ — **不変** |

**代替候補:**

- v4.6.30 — SQLite migration runner implementation（parallel track）
- v4.7.0 — Trip Proposal Envelope / Travel Ledger schema publication planning
- v4.7.x — itinerary aggregate Phase 2〜3（GUI 着手後）

**Parallel track（v4.6.x、独立）:**

- migration runner / orphan detection / FK hardening（[v4.6.1](specifications/v4.6.1-sqlite-fk-orphan-data-hardening-review.md) / [v4.6.2](specifications/v4.6.2-sqlite-migration-strategy-review.md) review 済み、実装未着手）

## Defer

- itinerary aggregate 実装（Phase 2+、GUI タイムライン連動）
- expense adapter cleanup / write path（v4.6.30 候補）
- Tauri / GUI 実装
- write command の service 化（Tier 3+）

Canonical defer list: [long-term-version-strategy.md](long-term-version-strategy.md)
