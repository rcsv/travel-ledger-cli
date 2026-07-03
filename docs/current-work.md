# Current Work

## Current phase

v4.6.39 — Reservation write path boundary review（documentation-only、リリース準備）

## Latest completed

- v4.6.38 Note write service Phase N-5 docs-only closeout — **released**
- v4.6.37 Note write service Phase N-2+N-3 — **released**
- Expense write migration W-0〜W-5 — v4.6.31〜v4.6.34 **完了**
- Note write migration N-0〜N-5 — v4.6.35〜v4.6.38 **完了**

## Repository state

- Cargo version: `4.6.39`
- Latest release: **v4.6.38** — [v4.6.38-notes.md](releases/v4.6.38-notes.md)
- **v4.6.39 spec:** [v4.6.39-reservation-write-path-boundary-review.md](specifications/v4.6.39-reservation-write-path-boundary-review.md)

## Next action

**推奨:** v4.6.40 — reservation write service migration plan

**候補シーケンス:**

```text
R-0  v4.6.39  boundary review — 本書
R-1  v4.6.40  migration plan
R-2+3  v4.6.41  implementation
R-5  v4.6.42  adapter cleanup
```

**Parallel track:** migration runner / FK hardening

## Defer

- repository 層（v4.7.x）
- Trip Proposal Envelope / schema-publication（v4.7.x）
- Tauri / GUI 実装

Canonical defer list: [long-term-version-strategy.md](long-term-version-strategy.md)
