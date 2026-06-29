# Current Work

## Current phase

v4.2.1 post-release review（documentation-only）

## Latest completed

- v4.2.0 Travel Book export-md chapter layout — **released**
- v4.2.1 Travel Book export-md post-release review（本書・分類表）

## Repository state

- Cargo version: `4.2.0`（v4.2.1 はドキュメントのみ — バージョン bump は release 時）
- Review doc: [v4.2.1-travel-book-export-md-post-release-review.md](specifications/v4.2.1-travel-book-export-md-post-release-review.md)

## Next action

**v4.2.2 — Travel Book Markdown polish**（review の Defer 項目。小規模実装）:

1. Trip overview — omit all-zero Stay / Travel / Total time lines
2. Okinawa seed — user-facing remark / estimate note wording（fixture 注記は README 正本へ）
3. Notes export order — Trip → Day → Itinerary
4. Reservations — reduce Provider duplication

その後:

- **v4.3.0** — Reservation / Summary display refinement（ロードマップ）

## Do not start yet

- Travel Book への Expense / Receipt 追加
- Highlights 自動抽出
- PDF export
- fixture 文字列の export 時自動除去

Canonical defer list: [long-term-version-strategy.md](long-term-version-strategy.md)
