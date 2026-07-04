# Current Work

## Current phase

v4.6.42 — Reservation write service Phase R-5 adapter cleanup（リリース準備）

## Latest completed

- v4.6.41 Reservation write service Phase R-2+R-3 — **released**
- Expense write migration W-0〜W-5 — **完了**
- Note write migration N-0〜N-5 — **完了**

## Repository state

- Cargo version: `4.6.42`
- Latest release: **v4.6.41** — [v4.6.41-notes.md](releases/v4.6.41-notes.md)
- **v4.6.42 spec:** [v4.6.42-reservation-write-service-phase-r5-adapter-cleanup.md](specifications/v4.6.42-reservation-write-service-phase-r5-adapter-cleanup.md)

## Reservation write migration — 完了

```text
R-0  v4.6.39  boundary review — 完了
R-1  v4.6.40  migration plan — 完了
R-2+3  v4.6.41  implementation — 完了
R-5  v4.6.42  adapter cleanup — 本書
```

Expense / Note / Reservation の write path 整理ストリームが一通り揃いました。

## Next action

**推奨:** parallel track — migration runner / FK hardening、または v4.7.x repository / schema-publication 計画

## Defer

- repository 層（v4.7.x）
- Trip Proposal Envelope / schema-publication（v4.7.x）
- Release workflow / `release.sh` 整理（別 track）

Canonical defer list: [long-term-version-strategy.md](long-term-version-strategy.md)
