# Current Work

## Current phase

v4.2.0 release verification

## Latest completed

- v4.2.0 Travel Book export-md chapter layout (`src/io/markdown.rs`).
- Okinawa `expected-export-md.md` golden + integration test.
- v4.1.2 Okinawa Travel Book sample enrichment (seed + golden).

## Repository state

- Cargo version: `4.2.0`
- Next tag candidate: `v4.2.0`

## Next action

After v4.2.0 release:

- **v4.3.0** — Reservation / Summary display refinement (per roadmap)
- or export-md follow-ups (Highlights, output profiles)

Deferred:

- DB path Phase 3
- PDF export
- Highlights auto-extraction
- Expense / Receipt in Travel Book output

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
