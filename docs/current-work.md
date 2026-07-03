# Current Work

## Current phase

v4.6.42 planning — Phase R-5 adapter cleanup（`print_reservation_detail(&conn, ...)` 削除）

## Latest completed

- v4.6.41 Reservation write service Phase R-2+R-3 — **released**
- v4.6.40 Reservation write service migration plan — **released**
- Expense / Note write migration — **完了**

## Repository state

- Cargo version: `4.6.41`
- Latest release: **v4.6.41** — [v4.6.41-notes.md](releases/v4.6.41-notes.md)
- **v4.6.41 spec:** [v4.6.41-reservation-write-service-phase-r2-r3.md](specifications/v4.6.41-reservation-write-service-phase-r2-r3.md)

## Next action

**推奨:** v4.6.42 — Phase R-5 adapter cleanup

- `print_reservation_detail(&conn, ...)` 削除可否確認
- production 参照ゼロ gate
- `reservation_cli` + `make check` PASS

**Reservation write migration:**

```text
R-0  v4.6.39  boundary review — 完了
R-1  v4.6.40  migration plan — 完了
R-2+3  v4.6.41  implementation — 完了
R-5  v4.6.42  adapter cleanup — 推奨次
```

## Defer

- repository 層（v4.7.x）
- Trip Proposal Envelope / schema-publication（v4.7.x）
- migration runner / FK hardening

Canonical defer list: [long-term-version-strategy.md](long-term-version-strategy.md)
