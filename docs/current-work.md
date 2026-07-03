# Current Work

## Current phase

v4.6.29 planning — itinerary show aggregate migration plan（推奨候補）

## Latest completed

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

- Cargo version: `4.6.28`
- Latest release: **v4.6.28** — [v4.6.28-notes.md](releases/v4.6.28-notes.md)
- **v4.6.28 spec:** [v4.6.28-itinerary-show-aggregate-boundary-review.md](specifications/v4.6.28-itinerary-show-aggregate-boundary-review.md)

## Next action

**v4.6.29 — itinerary show aggregate migration plan**（推奨、documentation-first）

- Option B（Reservations を service result に）を第一候補
- 既存 `itinerary show --json` は `ItineraryItem` のみ維持
- Notes / Expenses の全面 inclusion は慎重に defer

**代替候補:**

- v4.6.29 — expense write path boundary review
- v4.6.29 — SQLite migration runner implementation（parallel track）
- v4.7.0 — Trip Proposal Envelope / Travel Ledger schema publication planning

**Itinerary show aggregate（v4.6.28 結論）:**

| 項目 | 状態 |
|---|---|
| itinerary 本体 | service 化済み |
| Reservations human | handler 追加取得 — **許容** |
| JSON | `ItineraryItem` のみ — **不変** |

**Parallel track（v4.6.x、独立）:**

- migration runner / orphan detection / FK hardening（[v4.6.1](specifications/v4.6.1-sqlite-fk-orphan-data-hardening-review.md) / [v4.6.2](specifications/v4.6.2-sqlite-migration-strategy-review.md) review 済み、実装未着手）

## Defer

- itinerary aggregate 実装（migration plan 後）
- expense adapter cleanup / write path（v4.6.27 defer 継続）
- Tauri / GUI 実装
- write command の service 化（Tier 3+）

Canonical defer list: [long-term-version-strategy.md](long-term-version-strategy.md)
