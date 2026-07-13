# Current Work

## Current phase

v4.8.x Currency ISO validation — **currency hardening follow-up review released**（v4.8.17）

次は v4.8.18+ の候補（minor unit / trip import strict）を整理する。

## Latest completed

- v4.8.17 Currency hardening follow-up review — **released** (documentation-only)
- v4.8.16 Receipt / inbox CLI strict currency integration — **released**
- v4.8.15 validate-export currency warnings — **released**
- v4.8.14 Proposal Fragment apply strict currency integration — **released**

## Repository state

- Cargo version: `4.8.17`
- Latest formal release: **v4.8.17** — [v4.8.17-notes.md](releases/v4.8.17-notes.md)
- **Review:** [v4.8.17-currency-hardening-follow-up-review.md](specifications/v4.8.17-currency-hardening-follow-up-review.md)

## v4.8.16 release summary

- **IsoStrict:** `receipt add` / `receipt update`（currency 明示）/ `receipt assign`（CLI currency 明示）
- **FormatOnly 維持:** trip import, validate-export, receipt domain internal paths, read/export
- **Legacy currency:** update memo-only / assign with stored currency 非破壊

## Next action

**v4.8.18+** — minor unit ISO lookup / trip import strict reject（optional）

## Defer

- minor unit ISO lookup（v4.8.18+）
- trip import strict reject
- Venue model

Canonical defer list: [long-term-version-strategy.md](long-term-version-strategy.md)
