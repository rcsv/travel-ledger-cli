# Current Work

## Current phase

v4.9.x Desktop transition — **Trip metadata foundation released**（v4.9.0, documentation-only）

次は **v4.9.1 — Trip optional metadata DB implementation**（実装リリース）。

## Latest completed

- v4.9.0 Desktop transition and Trip metadata foundation — **released** (documentation-only)
- v4.8.18 validate-export Receipt currency warnings — **released**
- v4.8.17 Currency hardening follow-up review — **released** (documentation-only)
- v4.8.16 Receipt / inbox CLI strict currency integration — **released**

## Repository state

- Cargo version: `4.9.0`
- Latest formal release: **v4.9.0** — [v4.9.0-notes.md](releases/v4.9.0-notes.md)
- **Foundation:** [v4.9.0-desktop-transition-and-trip-metadata-foundation.md](specifications/v4.9.0-desktop-transition-and-trip-metadata-foundation.md)

## v4.9.0 release summary

- **GUI transition:** export/import 深掘りを下げ、Trip / Day / Itinerary 日常操作と service facade を優先
- **Trip metadata (optional):** `main_destination` / `main_destination_country_code` / `default_currency`
- **Boundaries:** Main Destination ≠ Venue、Default Currency ≠ Expense currency 正本
- **Country code validation:** format vs ISO registry strict は v4.9.1 で決定

## Next action

**v4.9.1** — Trip optional metadata DB implementation

- `trips.main_destination` / `main_destination_country_code` / `default_currency`
- migration（NULL 許容）
- `trip add` / `trip update` / `trip show` / JSON output
- country code validation policy（registry / reserved / historical）

## Defer

- Desktop readiness service facade（v4.9.2+）
- Desktop vertical slice / Tauri（v4.9.x 後半〜）
- minor unit ISO lookup
- trip import strict reject
- Venue model
- cloud / login / sync

Canonical defer list: [long-term-version-strategy.md](long-term-version-strategy.md)
