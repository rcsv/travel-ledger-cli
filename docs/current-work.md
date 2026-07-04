# Current Work

## Current phase

v4.6.x write path 整理ストリーム — **完了**

## Latest completed

- v4.6.42 Reservation write service Phase R-5 adapter cleanup — **released**
- v4.6.41 Reservation write service Phase R-2+R-3 — **released**
- Expense write migration W-0〜W-5 — **完了**
- Note write migration N-0〜N-5 — **完了**
- Reservation write migration R-0〜R-5 — **完了**

## Repository state

- Cargo version: `4.6.42`
- Latest release: **v4.6.42** — [v4.6.42-notes.md](releases/v4.6.42-notes.md)
- **v4.6.42 spec:** [v4.6.42-reservation-write-service-phase-r5-adapter-cleanup.md](specifications/v4.6.42-reservation-write-service-phase-r5-adapter-cleanup.md)

## Write path migration 完了サマリ

```text
Expense     W-0〜W-5   v4.6.31〜v4.6.34  完了
Note        N-0〜N-5   v4.6.35〜v4.6.38  完了
Reservation R-0〜R-5   v4.6.39〜v4.6.42  完了
```

## Next action

**推奨候補（いずれか）:**

- migration runner / FK hardening track
- Release workflow / `release.sh` asset upload follow-up
- v4.7.x — schema-publication / repository 層 / GUI 新章

## Defer

- repository 層（v4.7.x）
- Trip Proposal Envelope / schema-publication（v4.7.x）

Canonical defer list: [long-term-version-strategy.md](long-term-version-strategy.md)
