# Current Work

## Current phase

v4.6.38 planning — Note write service Phase N-5 docs-only closeout

## Latest completed

- v4.6.37 Note write service Phase N-2+N-3 — **released**
- v4.6.36 Note write service migration plan — **released**
- v4.6.35 Note write path boundary review — **released**
- v4.6.34 Expense write adapter cleanup — **released**

## Repository state

- Cargo version: `4.6.37`
- Latest release: **v4.6.37** — [v4.6.37-notes.md](releases/v4.6.37-notes.md)
- **v4.6.37 spec:** [v4.6.37-note-write-service-phase-n2-n3.md](specifications/v4.6.37-note-write-service-phase-n2-n3.md)

## Next action

**推奨:** v4.6.38 — Note write service Phase N-5 docs-only closeout

- adapter 削除対象なしの確認と明文化
- Note write migration ストリーム完了宣言
- documentation-only — `make check` はコード/schema/golden 変更時のみ

**Note write migration:**

```text
N-0  v4.6.35  boundary review — 完了
N-1  v4.6.36  migration plan — 完了
N-2+3  v4.6.37  implementation — 完了
N-5  v4.6.38  docs-only closeout（推奨次）
```

**Parallel track:** migration runner / FK hardening、Reservation write service 化

## Defer

- repository 層（v4.7.x）
- Trip Proposal Envelope / schema-publication（v4.7.x）
- Tauri / GUI 実装

Canonical defer list: [long-term-version-strategy.md](long-term-version-strategy.md)
