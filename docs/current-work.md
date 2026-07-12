# Current Work

## Current phase

v4.8.x Currency ISO validation — **Receipt / inbox CLI strict integration released**（v4.8.16）

次は **v4.8.17+** 候補（minor unit ISO lookup / trip import strict reject）。

## Latest completed

- v4.8.16 Receipt / inbox CLI strict currency integration — **released**
- v4.8.15 validate-export currency warnings — **released**
- v4.8.14 Proposal Fragment apply strict currency integration — **released**

## Repository state

- Cargo version: `4.8.16`
- Latest formal release: **v4.8.16** — [v4.8.16-notes.md](releases/v4.8.16-notes.md)
- **Implementation spec:** [v4.8.16-receipt-inbox-cli-strict-currency-integration.md](specifications/v4.8.16-receipt-inbox-cli-strict-currency-integration.md)

## v4.8.16 release summary

- **IsoStrict:** `receipt add` / `receipt update`（currency 明示）/ `receipt assign`（CLI currency 明示）
- **FormatOnly 維持:** trip import, validate-export, receipt domain internal paths, read/export
- **Legacy currency:** update memo-only / assign with stored currency 非破壊

## Next action

**v4.8.17+** — minor unit ISO lookup / trip import strict reject（optional）

## Defer

- minor unit ISO lookup（v4.8.17+）
- trip import strict reject
- Venue model

Canonical defer list: [long-term-version-strategy.md](long-term-version-strategy.md)
