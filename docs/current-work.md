# Current Work

## Current phase

v4.10.x Desktop vertical slice — **v4.10.0 read-only desktop vertical slice released**

## Latest completed

- v4.10.0 Read-only desktop vertical slice — **released**
- v4.9.2 Desktop readiness service facade — **released**
- v4.9.1 Trip optional metadata DB implementation — **released**

## Repository state

- Cargo version: `4.10.0`
- Latest formal release: **v4.10.0** — [v4.10.0-notes.md](releases/v4.10.0-notes.md)
- Spec: [v4.10.0-read-only-desktop-vertical-slice.md](specifications/v4.10.0-read-only-desktop-vertical-slice.md)

## v4.10.0 shipped

- **desktop/** Tauri + React read-only UI (developer preview / source-only)
- **Commands:** `select_database`, `list_trip_summaries`, `get_trip_detail`, `get_day_timeline`
- **Library:** `travel-ledger-cli` path dependency（root workspace 化なし）
- **Identity:** Travel Ledger Desktop / `com.rcsv.traveledger.desktop`（provisional）
- Itinerary completion state **なし**; Checklist `is_done` は別責務

## Next action

1. write use cases / edit UI（defer until needed）
2. DB path persistence / trip-wide timeline（defer）
3. desktop bundle 配布（deferred — `bundle.active = false`）

## Defer

- Trip / Itinerary 編集 UI
- desktop bundle 配布 / code signing
- root workspace 化（現状不要）
- itinerary done / undone

Canonical defer list: [long-term-version-strategy.md](long-term-version-strategy.md)
