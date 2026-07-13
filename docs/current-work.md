# Current Work

## Current phase

v4.9.x Desktop transition — **v4.9.2 Desktop readiness service facade released**

次は **Desktop vertical slice**（Tauri read-only Trip list / detail / Day timeline）。

## Latest completed

- v4.9.2 Desktop readiness service facade — **released**
- v4.9.1 Trip optional metadata DB implementation — **released**
- v4.9.0 Desktop transition and Trip metadata foundation — **released** (documentation-only)

## Repository state

- Cargo version: `4.9.2`
- Latest formal release: **v4.9.2** — [v4.9.2-notes.md](releases/v4.9.2-notes.md)
- **Implementation:** [v4.9.2-desktop-read-service-facade.md](specifications/v4.9.2-desktop-read-service-facade.md)

## v4.9.2 release summary

- **Library:** `travel_ledger_cli` — `open_db` + read facade（Tauri import 可能）
- **Facade:** `list_trip_summaries` / `get_trip_detail` / `get_day_timeline`
- **DTOs + structured errors**
- **CLI:** trip/day read paths migrated; legacy thin services removed
- **Deferred:** `itinerary timeline` trip-wide facade

## Next action

**Desktop vertical slice** — Tauri scaffold + read-only UI using `travel_ledger_cli`

## Defer

- Trip metadata の追加拡張
- `itinerary timeline` facade 移行
- workspace / Tauri 依存（vertical slice で着手）

Canonical defer list: [long-term-version-strategy.md](long-term-version-strategy.md)
