# Current Work

## Current phase

v4.8.x Fragment apply cross-cutting — **structured errors limited wiring expansion released**（v4.8.8）

P-6p `delete_estimate` 系列は **complete**（v4.8.3 released）。

Planned Money Fragment CRUD（add / update / delete）は P-6n / P-6o / P-6p で **完結**。

## Latest completed

- v4.8.8 Fragment apply structured errors limited wiring expansion — **released**
- v4.8.7 Fragment apply structured errors public contract review / hardening — **released**
- v4.8.6 Fragment apply JSON structured_errors[] exposure — **released**
- v4.8.5 Fragment apply internal structured error model + code registry — **released**

## Repository state

- Cargo version: `4.8.8`
- Latest formal release: **v4.8.8** — [v4.8.8-notes.md](releases/v4.8.8-notes.md)
- **v4.8.8 spec:** [v4.8.8-fragment-apply-structured-errors-limited-wiring-expansion.md](specifications/v4.8.8-fragment-apply-structured-errors-limited-wiring-expansion.md)
- **Public contract review:** [v4.8.7-fragment-apply-structured-errors-public-contract-review.md](specifications/v4.8.7-fragment-apply-structured-errors-public-contract-review.md)

## v4.8.8 release summary

- **`APPLY_REQUIRED_DECISION`** → `kind=decision_required`（gate / confirm_scope）
- **`add_estimate` unsupported target** → `APPLY_UNSUPPORTED_TARGET`（simulate）
- **`delete_estimate` confirm** — preview mismatch + TOCTOU revalidate wiring（代表 2 path）
- **`schema_version: 2`** 維持 / legacy `errors[]` 維持

## Next action

**Candidate:** v4.8.9 — confirm transaction structured errors expansion（optional）

**Alternatives:** Currency ISO validation (Issue #66); Venue model (defer)

## Defer

- confirm transaction 全面 wiring（v4.8.9+）
- `APPLY_SCOPED_WRITE_*` wiring
- retry token / ETag / strict idempotency
- GUI 実装
- public Proposal Fragment schema version bump

Canonical defer list: [long-term-version-strategy.md](long-term-version-strategy.md)
