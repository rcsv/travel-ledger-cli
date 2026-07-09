# Current Work

## Current phase

v4.7.38 implement move_itinerary dry-run — P-6l cross-day move (dry-run only)

## Latest completed

- v4.7.37 P-6l cross-day move planning — **released** (documentation-only)
- v4.7.36 P-6k reorder_itinerary --confirm（same-day）— **released**
- v4.7.35 P-6k reorder_itinerary dry-run（same-day）— **released**
- v4.7.34 P-6k reorder_itinerary planning — **released** (documentation-only)
- v4.7.33 P-6j safety / UX hardening for delete_itinerary — **released**
- v4.7.32 Fragment apply delete_itinerary --confirm (P-6j) — **released**
- v4.7.31 Fragment apply delete_itinerary dry-run (P-6j) — **released**
- v4.7.30 P-6j destructive / structural apply operations policy — **released** (documentation-only)
- v4.7.29 Fragment apply update_itinerary --confirm (P-6i) — **released**
- v4.7.28 Fragment apply update_itinerary dry-run (P-6i) — **released**
- v4.7.27 Fragment apply add_reservation --confirm (P-6h) — **released**
- v4.7.26 Fragment apply add_reservation dry-run (P-6h) — **released**
- v4.7.25 Fragment apply add_expense --confirm (P-6g) — **released**
- v4.7.24 Fragment apply add_expense dry-run (P-6g) — **released**
- v4.7.23 Fragment apply add_note --confirm (P-6f) — **released**
- v4.7.22 Fragment apply add_note dry-run (P-6f) — **released**
- v4.7.21 Fragment apply add_itinerary field expansion (P-6e) — **released**

## Repository state

- Cargo version: `4.7.37`
- Latest release: **v4.7.37** — [v4.7.37-notes.md](releases/v4.7.37-notes.md)
- **Proposal CLI:** `fragment apply --dry-run` — `add` / `add_note` / `add_expense` / `add_reservation` / `update_itinerary` / `delete_itinerary`（itinerary）/ `reorder_itinerary`（day）/ `move_itinerary`（itinerary）；`fragment apply --confirm` — `add_itinerary` / `add_note` / `add_expense` / `add_reservation` / `update_itinerary`（itinerary）/ `delete_itinerary`（itinerary）/ `reorder_itinerary`（day）
- **P-6j policy:** [v4.7.30-p6j-destructive-structural-apply-policy.md](specifications/v4.7.30-p6j-destructive-structural-apply-policy.md) — delete / reorder 方針正本
- **P-6j delete dry-run:** [v4.7.31-p6j-delete-itinerary-dry-run.md](specifications/v4.7.31-p6j-delete-itinerary-dry-run.md) — **released**（Venue / Place は delete blocker 外）

## v4.7.x Proposal 実装

```text
P-6g add_expense dry-run — v4.7.24 完了
P-6g add_expense --confirm — v4.7.25 完了
P-6h add_reservation dry-run — v4.7.26 完了
P-6h add_reservation --confirm — v4.7.27 完了
P-6i update_itinerary dry-run — v4.7.28 完了
P-6i update_itinerary --confirm — v4.7.29 完了
P-6j destructive / structural policy — v4.7.30 完了（docs only）
P-6j delete_itinerary dry-run — v4.7.31 完了
P-6j delete_itinerary --confirm — v4.7.32 完了
P-6j safety / UX hardening — v4.7.33 完了
P-6k reorder planning — v4.7.34 完了（docs only）
P-6l move_itinerary planning — v4.7.37 完了（docs only）
```

## Next action

**Candidate:** v4.7.38 — move_itinerary dry-run（P-6l）を実装して commit（release は別タスク）

## Defer

- P-6k reorder_itinerary cross-day move（別 intent、v4.7.37+ planning 候補）
- P-6i day / sort_order 拡張（reorder 設計後）
- **Venue model 実装** — [venue-model-introduction-policy.md](specifications/venue-model-introduction-policy.md)（planning 済み、v4.8+ 候補）
- safety / UX hardening（blocking 時 structured `delete_preview`、Estimate blocking integration test、inline `itinerary.note` 非 blocking 専用 test、ambiguous / not-found delete 専用 test、`delete_itinerary --confirm` unsupported 専用 test）
- doctor / advisor finding schema / AI Fragment generation
- DB proposal storage / import / list
- fragment show / inspect
- GUI 実装

Canonical defer list: [long-term-version-strategy.md](long-term-version-strategy.md)
