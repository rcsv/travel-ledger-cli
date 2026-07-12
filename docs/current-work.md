# Current Work

## Current phase

v4.8.x Currency ISO validation — **Proposal Fragment apply strict integration released**（v4.8.14）

次は **v4.8.15** 候補（validate-export warnings / Receipt inbox CLI strict）。

## Latest completed

- v4.8.14 Proposal Fragment apply strict currency integration — **released**
- v4.8.13 Currency CLI write-path hardening — **released**
- v4.8.12 Currency ISO internal registry + validation mode — **released**
- v4.8.11 Currency ISO validation hardening planning — **released** (documentation-only)

## Repository state

- Cargo version: `4.8.14`
- Latest formal release: **v4.8.14** — [v4.8.14-notes.md](releases/v4.8.14-notes.md)
- **Implementation spec:** [v4.8.14-currency-fragment-apply-strict-integration.md](specifications/v4.8.14-currency-fragment-apply-strict-integration.md)

## v4.8.14 release summary

- **IsoStrict:** Fragment apply `add_expense` / `add_estimate` / `update_estimate`（currency 明示時）
- **Structured errors:** `APPLY_FIELD_INVALID` + `candidate_content.currency`（新規 code なし）
- **FormatOnly 維持:** import, validate-export, domain layer, receipt/inbox, read/export
- **Legacy unknown currency:** `update_estimate` without explicit currency 非破壊

## Next action

**v4.8.15** — validate-export warnings / Receipt inbox CLI strict（optional）

## Defer

- Receipt/inbox CLI strict（v4.8.15）
- validate-export warnings（v4.8.15）
- minor unit ISO lookup（v4.8.16+）
- Venue model

Canonical defer list: [long-term-version-strategy.md](long-term-version-strategy.md)
