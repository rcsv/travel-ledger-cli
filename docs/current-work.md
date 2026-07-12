# Current Work

## Current phase

v4.8.x Currency ISO validation — **planning released**（v4.8.11）

次は **v4.8.12 — internal ISO registry + CurrencyValidationMode implementation**。

v4.8.x Fragment apply structured errors 系列は **complete**（v4.8.10 released）。

P-6p `delete_estimate` 系列は **complete**（v4.8.3 released）。

## Latest completed

- v4.8.11 Currency ISO validation hardening planning — **released** (documentation-only)
- v4.8.10 Fragment apply structured errors post-release review — **released** (documentation-only)
- v4.8.9 Fragment apply confirm transaction structured errors follow-up — **released**

## Repository state

- Cargo version: `4.8.11`
- Latest formal release: **v4.8.11** — [v4.8.11-notes.md](releases/v4.8.11-notes.md)
- **Planning spec:** [v4.8.11-currency-iso-validation-hardening-planning.md](specifications/v4.8.11-currency-iso-validation-hardening-planning.md)

## v4.8.11 planning conclusion（要約）

- **P0 / P1 なし** — compatibility policy で段階導入は non-breaking
- **DB schema 変更なし** / read·export は legacy unknown currency 非破壊
- **create/update/apply** — ISO registry + denylist strict（v4.8.12+ 実装）
- **import** — format-only 維持；**validate-export** — unknown/denylist は warning
- **minor units** — 今回 scope 外（v4.8.16+ optional）
- **Fragment apply** — `APPLY_FIELD_INVALID` 再利用；`schema_version: 2` 影響なし

## Next action

**v4.8.12** — internal ISO registry + `validate_currency_code(mode)` implementation

**Alternatives deferred:** Venue model; Shared Expense（v4.9.x — currency hardening 後推奨）

## Defer

- ISO registry / validator implementation（v4.8.12 — next）
- CLI create/update hardening（v4.8.13）
- Fragment apply currency structured wiring（v4.8.14）
- validate-export currency warnings（v4.8.15）
- minor unit ISO-backed lookup（v4.8.16+）
- confirm transaction — 他 intent structured expansion
- retry token / ETag / strict idempotency
- GUI 実装

Canonical defer list: [long-term-version-strategy.md](long-term-version-strategy.md)
