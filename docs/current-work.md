# Current Work

## Current phase

v4.7.28 released — P-6i `update_itinerary` dry-run

## Latest completed

- v4.7.28 Fragment apply update_itinerary dry-run (P-6i) — **released**
- v4.7.27 Fragment apply add_reservation --confirm (P-6h) — **released**
- v4.7.26 Fragment apply add_reservation dry-run (P-6h) — **released**
- v4.7.25 Fragment apply add_expense --confirm (P-6g) — **released**
- v4.7.24 Fragment apply add_expense dry-run (P-6g) — **released**
- v4.7.23 Fragment apply add_note --confirm (P-6f) — **released**
- v4.7.22 Fragment apply add_note dry-run (P-6f) — **released**
- v4.7.21 Fragment apply add_itinerary field expansion (P-6e) — **released**

## Repository state

- Cargo version: `4.7.28`
- Latest release: **v4.7.28** — [v4.7.28-notes.md](releases/v4.7.28-notes.md)
- **Proposal CLI:** `fragment apply --dry-run` — `add` / `add_note` / `add_expense` / `add_reservation` / `update_itinerary`（itinerary）；`fragment apply --confirm` — `add_itinerary` / `add_note` / `add_expense` / `add_reservation`（itinerary）

## v4.7.x Proposal 実装

```text
P-6g add_expense dry-run — v4.7.24 完了
P-6g add_expense --confirm — v4.7.25 完了
P-6h add_reservation dry-run — v4.7.26 完了
P-6h add_reservation --confirm — v4.7.27 完了
P-6i update_itinerary dry-run — v4.7.28 完了
```

## Next action

**Candidate:** v4.7.29 — P-6i `update_itinerary --confirm`

## Defer

- P-6j delete / reorder
- P-6i day / sort_order 拡張（reorder 設計後）
- safety test hardening（day target / ambiguous / invalid time / negative numeric / estimate 不変の専用 test）
- doctor / advisor finding schema / AI Fragment generation
- DB proposal storage / import / list
- fragment show / inspect
- GUI 実装

Canonical defer list: [long-term-version-strategy.md](long-term-version-strategy.md)
