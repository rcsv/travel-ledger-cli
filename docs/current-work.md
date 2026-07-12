# Current Work

## Current phase

v4.8.x Fragment apply cross-cutting — **confirm transaction structured errors follow-up released**（v4.8.9）

P-6p `delete_estimate` 系列は **complete**（v4.8.3 released）。

Planned Money Fragment CRUD（add / update / delete）は P-6n / P-6o / P-6p で **完結**。

## Latest completed

- v4.8.9 Fragment apply confirm transaction structured errors follow-up — **released**
- v4.8.8 Fragment apply structured errors limited wiring expansion — **released**
- v4.8.7 Fragment apply structured errors public contract review / hardening — **released**

## Repository state

- Cargo version: `4.8.9`
- Latest formal release: **v4.8.9** — [v4.8.9-notes.md](releases/v4.8.9-notes.md)
- **v4.8.9 spec:** [v4.8.9-fragment-apply-confirm-transaction-structured-errors-follow-up.md](specifications/v4.8.9-fragment-apply-confirm-transaction-structured-errors-follow-up.md)

## v4.8.9 release summary

- **`delete_estimate` confirm baseline mismatch** → `APPLY_BASELINE_MISMATCH`（`confirm_transaction`）
- **scoped DELETE 0 rows** → `APPLY_SCOPED_WRITE_ZERO_ROWS`
- **scoped DELETE multiple rows** → `APPLY_SCOPED_WRITE_MULTIPLE_ROWS`（classifier test）
- **`schema_version: 2`** 維持 / legacy `errors[]` 維持

## Next action

**Candidate:** v4.8.10 — confirm transaction expansion（optional）

**Alternatives:** Currency ISO validation (Issue #66); Venue model (defer)

## Defer

- confirm transaction 全面 wiring（v4.8.10+）
- fragment validation / export validation wiring
- retry token / ETag / strict idempotency
- GUI 実装

Canonical defer list: [long-term-version-strategy.md](long-term-version-strategy.md)
