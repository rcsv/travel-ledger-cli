# Current Work

## Current phase

v4.8.x Currency ISO validation — **internal registry released**（v4.8.12）

次は **v4.8.13 — CLI create/update write-path hardening**（`IsoStrict`）。

## Latest completed

- v4.8.12 Currency ISO internal registry + validation mode — **released**
- v4.8.11 Currency ISO validation hardening planning — **released** (documentation-only)

## Repository state

- Cargo version: `4.8.12`
- Latest formal release: **v4.8.12** — [v4.8.12-notes.md](releases/v4.8.12-notes.md)
- **Implementation spec:** [v4.8.12-currency-iso-internal-registry-validation-mode.md](specifications/v4.8.12-currency-iso-internal-registry-validation-mode.md)

## v4.8.12 implementation summary

- `CurrencyValidationMode`: `FormatOnly`（default）/ `IsoStrict`
- `validate_currency_code_with_mode()` — existing call sites unchanged
- ISO registry: 262 embedded alpha-3 codes（`iso_currency` 0.5.3 dataset + withdrawn supplement；**dependency 追加なし**）
- Denylist: `XXX`, `XTS`, precious metals, bond units — strict only
- **External CLI behavior unchanged** — write-path strict 化は v4.8.13+

## Next action

**v4.8.13** — CLI create/update write-path hardening（`IsoStrict`）

**Alternatives deferred:** Venue model; Shared Expense v4.9.x

## Defer

- CLI create/update strict wiring（v4.8.13 — next）
- Fragment apply strict integration（v4.8.14）
- validate-export warnings（v4.8.15）
- minor unit ISO lookup（v4.8.16+）
- Venue model
- Shared Expense v4.9.x（currency hardening 後推奨）

Canonical defer list: [long-term-version-strategy.md](long-term-version-strategy.md)
