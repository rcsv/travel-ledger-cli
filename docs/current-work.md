# Current Work

## Current phase

v4.8.x Currency ISO validation — **CLI write-path hardening released**（v4.8.13）

次は **v4.8.14 — Proposal Fragment apply strict integration**。

## Latest completed

- v4.8.13 Currency CLI write-path hardening — **released**
- v4.8.12 Currency ISO internal registry + validation mode — **released**
- v4.8.11 Currency ISO validation hardening planning — **released** (documentation-only)

## Repository state

- Cargo version: `4.8.13`
- Latest formal release: **v4.8.13** — [v4.8.13-notes.md](releases/v4.8.13-notes.md)
- **Implementation spec:** [v4.8.13-currency-cli-write-path-hardening.md](specifications/v4.8.13-currency-cli-write-path-hardening.md)

## v4.8.13 release summary

- **IsoStrict:** `expense add/update`, `estimate add/update`（CLI entry — `main.rs`）
- **FormatOnly 維持:** import, validate-export, Fragment apply, domain layer, receipt/inbox
- **Legacy unknown currency:** read/update without `--currency` 非破壊

## Next action

**v4.8.14** — Proposal Fragment apply strict integration + `APPLY_FIELD_INVALID`

**Alternatives deferred:** Receipt/inbox CLI strict; validate-export warnings（v4.8.15）

## Defer

- Fragment apply strict + structured wiring（v4.8.14 — next）
- Receipt/inbox CLI strict（v4.8.14+）
- validate-export warnings（v4.8.15）
- minor unit ISO lookup（v4.8.16+）
- Venue model

Canonical defer list: [long-term-version-strategy.md](long-term-version-strategy.md)
