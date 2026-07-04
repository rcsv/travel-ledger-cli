# Current Work

## Current phase

v4.6.x — **完了**（write path 整理 + release workflow housekeeping）

## Latest completed

- v4.6.43 Release workflow asset upload follow-up — **released**
- v4.6.42 Reservation write migration R-5 — **released**
- v4.6.x Expense / Note / Reservation write path 整理 — **完了**
- CONTRIBUTING.md（生成 AI 向け）— **released**（`4378a0d` + v4.6.43 微修正）

## Repository state

- Cargo version: `4.6.43`
- Latest release: **v4.6.43** — [v4.6.43-notes.md](releases/v4.6.43-notes.md)
- **v4.6.43 spec:** [v4.6.43-release-workflow-asset-upload-follow-up.md](specifications/v4.6.43-release-workflow-asset-upload-follow-up.md)

## v4.6.x 完了サマリ

```text
Expense write migration     W-0〜W-5   v4.6.31〜v4.6.34
Note write migration        N-0〜N-5   v4.6.35〜v4.6.38
Reservation write migration R-0〜R-5   v4.6.39〜v4.6.42
Release workflow housekeeping          v4.6.43
```

## Next action

**推奨:** v4.7.0 — schema-publication / Travel Ledger public direction concept review

**Parallel track:** migration runner / FK hardening

## Defer

- repository 層実装（v4.7.x 以降で検討）
- Trip Proposal Envelope 詳細設計

Canonical defer list: [long-term-version-strategy.md](long-term-version-strategy.md)
