# Current Work

## Current phase

v4.8.x Currency ISO validation — **validate-export currency warnings released**（v4.8.15）

次は **v4.8.16+** 候補（Receipt/inbox CLI strict / minor unit ISO lookup）。

## Latest completed

- v4.8.15 validate-export currency warnings — **released**
- v4.8.14 Proposal Fragment apply strict currency integration — **released**
- v4.8.13 Currency CLI write-path hardening — **released**

## Repository state

- Cargo version: `4.8.15`
- Latest formal release: **v4.8.15** — [v4.8.15-notes.md](releases/v4.8.15-notes.md)
- **Implementation spec:** [v4.8.15-validate-export-currency-warnings.md](specifications/v4.8.15-validate-export-currency-warnings.md)

## v4.8.15 release summary

- **validate-export warnings:** unknown ISO / denylist currency（format invalid は error 維持）
- **import 互換:** unknown currency import 非破壊
- **FormatOnly 維持:** import, domain layer, read/export, Receipt/inbox

## Next action

**v4.8.16+** — Receipt/inbox CLI strict / minor unit ISO lookup（optional）

## Defer

- Receipt/inbox CLI strict（v4.8.16+）
- minor unit ISO lookup（v4.8.16+）
- trip import strict reject
- Venue model

Canonical defer list: [long-term-version-strategy.md](long-term-version-strategy.md)
