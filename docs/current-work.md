# Current Work

## Current phase

v4.5.0 Receipt Inbox responsibilities review（planning / review）

## Latest completed

- v4.4.8 Travel Book presentation helper cleanup — **released**
- v4.4.7 Travel Book presentation helpers final review — **released**
- v4.4.6 Travel Book presentation helpers extraction Phase 3 — **released**

## Repository state

- Cargo version: `4.4.8`
- **v4.5.0 review:** [v4.5.0-receipt-inbox-responsibilities-review.md](specifications/v4.5.0-receipt-inbox-responsibilities-review.md)
- Receipt Inbox: v3.6–v3.7 実装済み（metadata-only + assign/trash workflow）

## v4.4.x arc status

Travel Book presentation helper extraction **complete**（Phase 1–3 + v4.4.8 cleanup）。

## Next action（v4.5.0 レビュー結論）

**A 推奨:**

- **v4.5.1** — doctor / advisor Receipt utilization（ledger 側の未整理 receipt 可視化）
- trip stats / Travel Book / Actual 定義は変更しない

**Defer:**

- `TravelBookDocument` prototype（UI / Venue 要件まで）
- Evidence / Attachment / Travel Journal 実装
- trip stats への Receipt 反映、Potential Actual 表示

## Do not start yet

- Receipt 専用 `image_path` 先行実装
- trip stats / Planned vs Actual への Receipt・Pending 反映
- Balance / Settlement
- `TravelBookDocument` full abstraction（UI/Venue requirements）

Canonical defer list: [long-term-version-strategy.md](long-term-version-strategy.md)
