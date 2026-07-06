# Current Work

## Current phase

v4.7.21 released — P-6e add_itinerary field expansion

## Latest completed

- v4.7.21 Fragment apply add_itinerary field expansion (P-6e) — **released**
- v4.7.20 P-6 post-implementation review — **released**
- v4.7.19 Fragment apply --confirm (P-6d) — **released**
- v4.7.18 Fragment apply dry-run (P-6c) — **released**
- v4.7.17 Proposal materialize --confirm (P-6b) — **released**
- v4.7.16 Proposal materialize dry-run (P-6a) — **released**

## Repository state

- Cargo version: `4.7.21`
- Latest release: **v4.7.21** — [v4.7.21-notes.md](releases/v4.7.21-notes.md)
- **Proposal CLI:** `proposal validate` · `show` · `inspect` · `materialize --dry-run` · `materialize --confirm` · `fragment validate` · `fragment apply --dry-run` · `fragment apply --confirm`（P-6e フィールド拡張）
- **P-6 minimal route:** P-6a〜P-6e **完了** — [v4.7.21 spec](specifications/v4.7.21-fragment-apply-add-itinerary-field-expansion.md)

## v4.7.x Proposal 実装

```text
P-6a Envelope materialize --dry-run — v4.7.16 完了
P-6b Envelope materialize --confirm — v4.7.17 完了
P-6c Fragment apply --dry-run — v4.7.18 完了
P-6d Fragment apply --confirm — v4.7.19 完了
P-6  post-implementation review — v4.7.20 完了
P-6e add_itinerary field expansion — v4.7.21 完了
```

## Next action

**P-6f+ candidate** — add_note / add_expense / update / delete / reorder — 相談

## Defer

- P-6f+ add_note / add_expense / update / delete / reorder
- doctor / advisor finding schema / AI Fragment generation
- DB proposal storage / import / list
- fragment show / inspect
- GUI 実装

Canonical defer list: [long-term-version-strategy.md](long-term-version-strategy.md)
