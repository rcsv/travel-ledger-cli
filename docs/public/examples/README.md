# Public JSON examples — schema v8 Trip

Travel Ledger **schema v8 Trip** の public JSON example files です。読み方の narrative は [examples.md](../examples.md)。Proposal / Fragment の概念例（non-normative）は [examples-non-normative/](../examples-non-normative/) を参照してください。

---

## これらは何か

| ファイル | 内容 |
|---|---|
| [schema-v8-minimal-trip.json](schema-v8-minimal-trip.json) | 最小構成 — 読みやすさ重視 |
| [schema-v8-okinawa-sesoko-trip.json](schema-v8-okinawa-sesoko-trip.json) | 沖縄瀬底 — canonical sample からの短縮版 |
| [schema-v8-with-reservations-expenses-notes.json](schema-v8-with-reservations-expenses-notes.json) | reservations / expenses / notes / participants を含む例 |

```text
採用済み Trip の例
候補案ではない
日付未定の Proposal ではない
trip validate-export の対象
```

---

## validate-export

schema v8 Trip examples は **`trip validate-export` の対象** です。CI では `tests/public_examples_validation_guard.rs` が 3 本すべて PASS することを確認します（v4.7.12+）。

```bash
trip validate-export docs/public/examples/schema-v8-minimal-trip.json
```

| JSON 種別 | validate-export |
|---|---|
| **schema v8 Trip**（本ディレクトリ） | **対象** |
| Trip Proposal Envelope | 対象外 — [examples-non-normative/](../examples-non-normative/) |
| Proposal Fragment | 対象外 — [examples-non-normative/](../examples-non-normative/) |
| materialize / apply 後の Trip | schema v8 として **対象** |

Proposal / Fragment を validate-export に通さない — 型が異なる。

---

## フィールドについて

- 正本: [export-schema.md](../../specifications/export-schema.md)
- `trip.id` / `created_at` / `updated_at` — validate-export 互換のため example に含む。第三者が新規 Trip JSON を書く場合も、import / validate 互換のためこれらが必要
- `generator` / `exported_at` — 任意（live export では付与される）
- nested `days[].itineraries[]` — schema v3+ の中核構造

---

## Okinawa 短縮版について

[schema-v8-okinawa-sesoko-trip.json](schema-v8-okinawa-sesoko-trip.json) は [samples/okinawa_sesoko_2026/](../../../samples/okinawa_sesoko_2026/) の canonical export を **短縮** した public example です。

- 全 itinerary は含まない（Day 1 は先頭 3 件、他 Day は 1 件）
- receipts / checklist は省略（構造説明用）
- 完全な golden / regression 用データは canonical sample を参照

## public example と canonical sample の境界

| | `docs/public/examples/` | `samples/okinawa_sesoko_2026/` |
|---|---|---|
| 目的 | 外向き・読みやすさ | CI golden / regression |
| 匿名化 | **example 用の架空名・番号** | v4.7.6 以降同方針（Alex / Jordan 等） |
| validate-export | 3 本 PASS | golden 比較用（trip.id 省略可） |
| Proposal | **含めない** | **含めない** |

旅行サンプル内の個人名・自宅名・実予約番号らしき値は public example に載せない。`Cargo.toml` の `authors` 等、プロジェクト著作者表記は別扱い。

---

## 関連

- [examples.md](../examples.md) — narrative と概念図
- [ai-json-generation-guide.md](../ai-json-generation-guide.md) — 生成 AI 向け作法
- [proposals.md](../proposals.md) — Envelope / Fragment / gate
- [v4.7.8 spec](../../specifications/v4.7.8-proposal-implementation-planning.md)
- [v4.7.7 spec](../../specifications/v4.7.7-public-schema-post-review.md)
- [v4.7.6 spec](../../specifications/v4.7.6-public-json-examples-concept-stream-post-review.md)
