# Current Work

## Current phase

v4.8.x Fragment apply cross-cutting — **structured errors public contract review released**（v4.8.7）

P-6p `delete_estimate` 系列は **complete**（v4.8.3 released）。

Planned Money Fragment CRUD（add / update / delete）は P-6n / P-6o / P-6p で **完結**。

## Latest completed

- v4.8.7 Fragment apply structured errors public contract review / hardening — **released** (documentation-only)
- v4.8.6 Fragment apply JSON structured_errors[] exposure — **released**
- v4.8.5 Fragment apply internal structured error model + code registry — **released**
- v4.8.4 Fragment apply structured errors / API readiness planning — **released** (documentation-only)
- v4.8.3 P-6p delete_estimate post-release review — **released** (documentation-only)

## Repository state

- Cargo version: `4.8.7`
- Latest formal release: **v4.8.7** — [v4.8.7-notes.md](releases/v4.8.7-notes.md)
- **Public contract review:** [v4.8.7-fragment-apply-structured-errors-public-contract-review.md](specifications/v4.8.7-fragment-apply-structured-errors-public-contract-review.md)
- **JSON exposure:** [v4.8.6-fragment-apply-json-structured-errors-exposure.md](specifications/v4.8.6-fragment-apply-json-structured-errors-exposure.md)
- **Internal model:** [v4.8.5-fragment-apply-internal-structured-error-model.md](specifications/v4.8.5-fragment-apply-internal-structured-error-model.md)

## v4.8.7 review conclusion（要約）

- **P0 / P1 なし** — v4.8.4→8.6 契約は実装・tests と整合
- **`schema_version: 2` 維持** — additive field のみ
- **Public contract:** client は `code` で分岐、`message` は unstable
- **Limited exposure:** 未 wiring path は `errors[]` のみが正常
- **Provisional:** `APPLY_REQUIRED_DECISION` は現状 `kind=blocking`（`decision_required` は deferred）

## Next action

**Candidate:** v4.8.8 — structured errors wiring expansion（optional）

**Alternatives:** Currency ISO validation (Issue #66); Venue model (defer)

## Defer

- confirm transaction structured exposure（preview mismatch, TOCTOU, scoped write）
- retry token / ETag / strict idempotency
- GUI 実装
- public Proposal Fragment schema version bump

Canonical defer list: [long-term-version-strategy.md](long-term-version-strategy.md)
