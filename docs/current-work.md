# Current Work

## Current phase

v4.7.46 P-6o update_estimate planning — **released** (documentation-only)

## Latest completed

- v4.7.46 P-6o update_estimate planning — **released** (documentation-only)
- v4.7.45 Estimate documentation and CLI usage review — **released** (documentation-only)
- v4.7.44 P-6n Planned Money post-release review — **released** (documentation-only)
- v4.7.43 P-6n add_estimate --confirm — **released**
- v4.7.42 P-6n add_estimate dry-run — **released**

## Repository state

- Cargo version: `4.7.46`
- Latest release: **v4.7.46** — [v4.7.46-notes.md](releases/v4.7.46-notes.md)
- **update_estimate planning:** [v4.7.46-p6o-update-estimate-planning.md](specifications/v4.7.46-p6o-update-estimate-planning.md)（正本）
- **add_estimate user docs:** [v4.7.45-estimate-documentation-and-cli-usage-review.md](specifications/v4.7.45-estimate-documentation-and-cli-usage-review.md)
- **Proposal CLI:** `fragment apply --dry-run` / `--confirm` — `add_estimate` まで実装済み；`update_estimate` は **未実装**（v4.7.47 候補）

## v4.7.x Proposal 実装

```text
P-6n add_estimate planning — v4.7.41 完了（docs only）
P-6n add_estimate dry-run — v4.7.42 完了
P-6n add_estimate --confirm — v4.7.43 完了
P-6n Planned Money post-release review — v4.7.44 完了（docs only）
P-6n Estimate user docs / CLI usage review — v4.7.45 完了（docs only）
P-6o update_estimate planning — v4.7.46 完了（docs only）
```

## Next action

**Candidate:** v4.7.47 P-6o update_estimate dry-run（implementation）

## Defer

- P-6i day / sort_order 拡張（reorder 設計後）
- **Venue model 実装** — [venue-model-introduction-policy.md](specifications/venue-model-introduction-policy.md)（planning 済み、v4.8+ 候補）
- delete_estimate Fragment
- idempotency key / structured errors（Fragment apply 横断）
- GUI 実装

Canonical defer list: [long-term-version-strategy.md](long-term-version-strategy.md)
