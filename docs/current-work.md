# Current Work

## Current phase

v4.4.1 Category display name in Travel Book（planning）

## Latest completed

- v4.4.0 Travel Book presentation model review — **released**
- v4.3.2 Travel Book planned cost polish — **released**

## Repository state

- Cargo version: `4.4.0`
- **v4.4.0 review:** [v4.4.0-travel-book-presentation-model-review.md](specifications/v4.4.0-travel-book-presentation-model-review.md)
- Release notes: [v4.4.0-notes.md](releases/v4.4.0-notes.md)

## Next action

**v4.4.1 — Category display name in Travel Book**

- Daily schedule で `ItineraryCategory::definition().display_name` を使う
- Markdown 専用の日本語化ハックではなく、presentation model へ移せる形で実装
- 設計根拠: [v4.4.0-travel-book-presentation-model-review.md](specifications/v4.4.0-travel-book-presentation-model-review.md) §4.4 / §5.2

## Do not start yet

- 大規模 view model 一括導入
- Daily schedule の Markdown-only polish 追加（カテゴリ表示名以外）
- GUI / native app コード
- Venue model / map provider / 移動時間自動算出
- Travel Book への Expense / Receipt 追加

Canonical defer list: [long-term-version-strategy.md](long-term-version-strategy.md)
