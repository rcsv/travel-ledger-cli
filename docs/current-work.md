# Current Work

## Current phase

v4.8.x Planned Money delete — **P-6p planning complete**（v4.8.0 released）

v4.7.x Proposal Fragment mutation 本線は **series complete**（v4.7.49）。

## Latest completed

- v4.8.0 P-6p delete_estimate Proposal Fragment planning — **released** (documentation-only)
- v4.7.49 P-6o update_estimate post-release review — **released** (documentation-only)
- v4.7.48 P-6o update_estimate --confirm — **released**
- v4.7.47 P-6o update_estimate dry-run — **released**
- v4.7.46 P-6o update_estimate planning — **released** (documentation-only)

## Repository state

- Cargo version: `4.8.0`
- Latest formal release: **v4.8.0** — [v4.8.0-notes.md](releases/v4.8.0-notes.md)
- **P-6p planning:** [v4.8.0-p6p-delete-estimate-planning.md](specifications/v4.8.0-p6p-delete-estimate-planning.md)
- **P-6o review:** [v4.7.49-p6o-update-estimate-post-release-review.md](specifications/v4.7.49-p6o-update-estimate-post-release-review.md)
- **update_estimate confirm:** [v4.7.48-p6o-update-estimate-confirm.md](specifications/v4.7.48-p6o-update-estimate-confirm.md)
- **update_estimate dry-run:** [v4.7.47-p6o-update-estimate-dry-run.md](specifications/v4.7.47-p6o-update-estimate-dry-run.md)

## v4.7.x Proposal 実装

```text
P-6n add_estimate — v4.7.41–43 + v4.7.44 review 完了
P-6o update_estimate — v4.7.46–48 + v4.7.49 review 完了
P-6j–l itinerary structural — 完了済み
```

## Next action

**Candidate:** v4.8.1 — P-6p `delete_estimate` dry-run（implementation）

**Alternatives:** Fragment apply structured errors; Currency ISO validation (Issue #66)

## Defer

- **Venue model 実装** — [venue-model-introduction-policy.md](specifications/venue-model-introduction-policy.md)（v4.8+ 候補）
- idempotency key / structured errors 実装（Fragment apply 横断 — v4.8+ planning）
- public Proposal Fragment schema versioning / version・ETag 相当
- successful destructive confirm 後の undo / apply journal
- GUI 実装

Canonical defer list: [long-term-version-strategy.md](long-term-version-strategy.md)
