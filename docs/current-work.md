# Current Work

## Current phase

v4.8.x Fragment apply cross-cutting — **structured errors post-release series review released**（v4.8.10）

P-6p `delete_estimate` 系列は **complete**（v4.8.3 released）。

Planned Money Fragment CRUD（add / update / delete）は P-6n / P-6o / P-6p で **完結**。

## Latest completed

- v4.8.10 Fragment apply structured errors post-release review — **released** (documentation-only)
- v4.8.9 Fragment apply confirm transaction structured errors follow-up — **released**
- v4.8.8 Fragment apply structured errors limited wiring expansion — **released**

## Repository state

- Cargo version: `4.8.10`
- Latest formal release: **v4.8.10** — [v4.8.10-notes.md](releases/v4.8.10-notes.md)
- **Series review:** [v4.8.10-fragment-apply-structured-errors-post-release-review.md](specifications/v4.8.10-fragment-apply-structured-errors-post-release-review.md)

## v4.8.10 review conclusion（要約）

- **P0 / P1 なし** — v4.8.4→8.9 系列は実装・tests と整合
- **系列完了:** planning / internal model / JSON exposure / public contract / limited wiring / confirm follow-up
- **`schema_version: 2` 維持** / legacy `errors[]` 維持 / `code` 正本

## Next action

**Candidate:** v4.8.11 — confirm transaction expansion or cross-cutting follow-up（optional）

**Alternatives:** Currency ISO validation (Issue #66); Venue model (defer)

## Defer

- confirm transaction — 他 intent（v4.8.11+）
- fragment / export validation wiring
- retry token / ETag / strict idempotency
- GUI 実装

Canonical defer list: [long-term-version-strategy.md](long-term-version-strategy.md)
