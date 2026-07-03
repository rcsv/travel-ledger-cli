# Current Work

## Current phase

v4.6.41 planning — reservation write service Phase R-2+R-3 implementation

## Latest completed

- v4.6.40 Reservation write service migration plan — **released**
- v4.6.39 Reservation write path boundary review — **released**
- Expense write migration W-0〜W-5 — **完了**
- Note write migration N-0〜N-5 — **完了**

## Repository state

- Cargo version: `4.6.40`
- Latest release: **v4.6.40** — [v4.6.40-notes.md](releases/v4.6.40-notes.md)
- **v4.6.40 spec:** [v4.6.40-reservation-write-service-migration-plan.md](specifications/v4.6.40-reservation-write-service-migration-plan.md)

## Next action

**推奨:** v4.6.41 — reservation write service Phase R-2+R-3 implementation

**Reservation write migration:**

```text
R-0  v4.6.39  boundary review — 完了
R-1  v4.6.40  migration plan — 完了
R-2+3  v4.6.41  implementation（推奨次）
R-5  v4.6.42  adapter cleanup（print_reservation_detail 削除）
```

## Defer

- repository 層（v4.7.x）
- Trip Proposal Envelope / schema-publication（v4.7.x）
- migration runner / FK hardening

Canonical defer list: [long-term-version-strategy.md](long-term-version-strategy.md)
