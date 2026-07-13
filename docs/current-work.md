# Current Work

## Current phase

v4.8.x Currency ISO validation — **validate-export Receipt currency warnings released**（v4.8.18）

次は v4.8.19+ の候補（minor unit / trip import strict）を整理する。

## Latest completed

- v4.8.18 validate-export Receipt currency warnings — **released**
- v4.8.17 Currency hardening follow-up review — **released** (documentation-only)
- v4.8.16 Receipt / inbox CLI strict currency integration — **released**
- v4.8.15 validate-export currency warnings — **released**

## Repository state

- Cargo version: `4.8.18`
- Latest formal release: **v4.8.18** — [v4.8.18-notes.md](releases/v4.8.18-notes.md)
- **Implementation:** [v4.8.18-validate-export-receipt-currency-warnings.md](specifications/v4.8.18-validate-export-receipt-currency-warnings.md)

## v4.8.18 release summary

- **Warning-only:** `trip validate-export` — `receipts[{i}].currency`（schema ≥ v7）
- **Unknown / denylist:** warning（`valid: true` 維持）
- **Format invalid:** error（既存契約維持）
- **FormatOnly 維持:** trip import, read/export, domain layer

## Next action

**v4.8.19+** — minor unit ISO lookup / trip import strict reject（optional）

## Defer

- minor unit ISO lookup（v4.8.19+）
- trip import strict reject
- Venue model

Canonical defer list: [long-term-version-strategy.md](long-term-version-strategy.md)
