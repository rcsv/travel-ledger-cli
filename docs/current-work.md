# Current Work

## Current phase

v4.6.23 planning — read-only helper context review（推奨候補）

## Latest completed

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

- Cargo version: `4.6.22`
- Latest release: **v4.6.22** — [v4.6.22-notes.md](releases/v4.6.22-notes.md)
- **v4.6.22 spec:** [v4.6.22-read-only-service-boundary-completion-review.md](specifications/v4.6.22-read-only-service-boundary-completion-review.md)

## Next action

**v4.6.23 — read-only helper context review**（推奨、documentation-first）

- `expense_to_json` 等、CLI/helper 残存 DB context の整理方針
- itinerary show Reservations aggregate 境界も同 review で扱える
- write command / Tauri 実装の前に判断

**代替候補:**

- v4.6.23 — itinerary show aggregate boundary review
- v4.6.23 — SQLite migration runner implementation（parallel track）
- v4.7.0 — Trip Proposal Envelope / Travel Ledger schema publication planning

**Parallel track（v4.6.x、独立）:**

- migration runner / orphan detection / FK hardening（[v4.6.1](specifications/v4.6.1-sqlite-fk-orphan-data-hardening-review.md) / [v4.6.2](specifications/v4.6.2-sqlite-migration-strategy-review.md) review 済み、実装未着手）

## Defer

- Tauri / GUI 実装
- `main.rs` 一括 `commands/` 移動
- write command の service 化（Tier 3+）
- `trip delete` / `import` / `duplicate` / `receipt assign`

Canonical defer list: [long-term-version-strategy.md](long-term-version-strategy.md)
