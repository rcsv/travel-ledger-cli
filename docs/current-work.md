# Current Work

## Current phase

v4.8.x Fragment apply cross-cutting — **JSON structured_errors exposure released**（v4.8.6）

P-6p `delete_estimate` 系列は **complete**（v4.8.3 released）。

Planned Money Fragment CRUD（add / update / delete）は P-6n / P-6o / P-6p で **完結**。

## Latest completed

- v4.8.6 Fragment apply JSON structured_errors[] exposure — **released**
- v4.8.5 Fragment apply internal structured error model + code registry — **released**
- v4.8.4 Fragment apply structured errors / API readiness planning — **released** (documentation-only)
- v4.8.3 P-6p delete_estimate post-release review — **released** (documentation-only)
- v4.8.2 P-6p delete_estimate Proposal Fragment --confirm — **released**

## Repository state

- Cargo version: `4.8.6`
- Latest formal release: **v4.8.6** — [v4.8.6-notes.md](releases/v4.8.6-notes.md)
- **v4.8.6 JSON exposure:** [v4.8.6-fragment-apply-json-structured-errors-exposure.md](specifications/v4.8.6-fragment-apply-json-structured-errors-exposure.md)
- **v4.8.5 internal model:** [v4.8.5-fragment-apply-internal-structured-error-model.md](specifications/v4.8.5-fragment-apply-internal-structured-error-model.md)
- **Structured errors planning:** [v4.8.4-fragment-apply-structured-errors-api-readiness-planning.md](specifications/v4.8.4-fragment-apply-structured-errors-api-readiness-planning.md)

## v4.7.x / v4.8.x Proposal 実装

```text
P-6n add_estimate — v4.7.41–43 + v4.7.44 review 完了
P-6o update_estimate — v4.7.46–48 + v4.7.49 review 完了
P-6p delete_estimate — v4.8.0–8.3 完了
P-6j–l itinerary structural — 完了済み
```

## Next action

**Candidate:** v4.8.7 — structured errors public contract review / hardening

**Alternatives:** Currency ISO validation (Issue #66); Venue model implementation (defer)

## Defer

- **Venue model 実装** — [venue-model-introduction-policy.md](specifications/venue-model-introduction-policy.md)
- retry token / ETag / strict idempotency 実装
- GUI 実装
- public Proposal Fragment schema version bump
- confirm transaction structured exposure（preview mismatch, TOCTOU, scoped write）

Canonical defer list: [long-term-version-strategy.md](long-term-version-strategy.md)
