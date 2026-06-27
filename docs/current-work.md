# Current Work

## Current phase

v3.11.0 release verification

## Latest completed

- v3.11.0 DB Use implementation (`db use` / `db use --clear`, `caglla.toml` write).
- v3.10.0 DB Use concept design (documentation-only).
- v3.9.2 legacy migration smoke tests.
- v3.9.0 Config and DB path foundation Phase 1.

## Repository state

- Latest tag: `v3.10.0`
- Latest release: `v3.10.0 — DB Use concept design`
- Cargo version: `3.11.0` (implementation ready; release pending)

## Next action

Pick **one** design or implementation topic (do not parallelize by default):

1. **DB path Phase 3** — parent-directory `caglla.toml` search (design first)
2. **Travel Book v4 concept design** — `trip export-md` as Generator v0
3. **doctor / advisor utilization** — Estimate / Receipt / Pending hints only

Deferred:

- User-global config / profile switching
- Evidence / Attachment / OCR / Settlement

See [v3.8.0-roadmap-realignment-after-receipt-inbox.md](specifications/v3.8.0-roadmap-realignment-after-receipt-inbox.md) §5.

## Do not start yet

Canonical defer list (synced with [long-term-version-strategy.md](long-term-version-strategy.md)):

- Evidence / Attachment（共通レイヤー設計が先）
- image_path（Receipt / Expense 専用の先行実装）
- OCR
- automatic receipt parsing
- Balance / Settlement（精算・振込計算）
- Participant sharing 拡張（Settlement 連動）
- Expense reassign / unassign / trash
- receipt purge
- Travel Journal 実装（v5 — Evidence / Attachment 未設計）
- trip stats / Planned vs Actual への Receipt・Pending 反映
- Potential Actual 表示
- Cloud / Identity / Platform 実装
