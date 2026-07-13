# Current Work

## Current phase

v4.9.x Desktop transition — **Trip optional metadata DB implementation released**（v4.9.1）

次は **v4.9.2 — Desktop readiness service facade**。

## Latest completed

- v4.9.1 Trip optional metadata DB implementation — **released**
- v4.9.0 Desktop transition and Trip metadata foundation — **released** (documentation-only)
- v4.8.18 validate-export Receipt currency warnings — **released**

## Repository state

- Cargo version: `4.9.1`
- Latest formal release: **v4.9.1** — [v4.9.1-notes.md](releases/v4.9.1-notes.md)
- **Implementation:** [v4.9.1-trip-optional-metadata-db-implementation.md](specifications/v4.9.1-trip-optional-metadata-db-implementation.md)

## v4.9.1 release summary

- **DB:** nullable `main_destination` / `main_destination_country_code` / `default_currency`
- **CLI:** add / update / show / JSON（set + clear flags）
- **Country validation:** ISO 3166-1 alpha-2 strict registry（2026-07 snapshot）
- **Export/import:** unchanged（`TripCliJson` / `#[serde(skip)]` 分離）

## Next action

**v4.9.2** — Desktop readiness service facade

- `TripSummary` / `TripDetail` / `DayDetail` / `ItineraryDetail`
- structured errors / DB path handling
- CLI 表示構造から独立した read path

その後: Desktop vertical slice（Tauri read-only Trip list / detail / Day timeline）

## Defer

- Trip metadata の追加拡張（v4.9.1 で一区切り）
- country-to-currency suggestion
- Expense default currency auto-apply
- Venue model
- Tauri scaffold（v4.9.2 後）

Canonical defer list: [long-term-version-strategy.md](long-term-version-strategy.md)
