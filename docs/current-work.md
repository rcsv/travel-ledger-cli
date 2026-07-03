# Current Work

## Current phase

v4.6.20 planning — reservation show or expense show service boundary

## Latest completed

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

- Cargo version: `4.6.19`
- Latest release: **v4.6.19** — [v4.6.19-notes.md](releases/v4.6.19-notes.md)
- **v4.6.19 spec:** [v4.6.19-day-show-service-boundary.md](specifications/v4.6.19-day-show-service-boundary.md)

## Next action

**v4.6.20 — `reservation show` or `expense show` service boundary**（optional implementation）

- show 系 read-only expansion を継続
- `reservation show` / `expense show` は itinerary detail から自然

**Parallel track（v4.6.x、独立）:**

- migration runner / orphan detection（[v4.6.2 review](specifications/v4.6.2-sqlite-migration-strategy-review.md) 順序）

## Defer

- Tauri / GUI 実装
- `main.rs` 一括 `commands/` 移動
- write command の service 化（Tier 3+）
- `trip delete` / `import` / `duplicate` / `receipt assign`

Canonical defer list: [long-term-version-strategy.md](long-term-version-strategy.md)
