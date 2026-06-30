# Markdown Export

旅行計画を Markdown 形式の **Travel Book**（旅行しおり）として出力します。v4.2.0 以降は [v4.1.0 章立て設計](specifications/v4.1.0-travel-book-chapter-structure-design.md) に沿った構成です。

```bash
cargo run -- trip export-md 1
```

## 章構成（v4.2.0+）

```text
Cover
  → Trip overview
  → Daily schedule
  → Reservations（あれば）
  → Checklist（あれば）
  → Planned cost（Estimate あれば）
  → Notes（Note entity あれば）
  → Colophon（常に末尾）
```

### Cover（表紙）

- Trip 名（`#` 見出し）
- 旅行期間
- 同行者数（Participant 登録時）

Trip `summary` は Cover ではなく **Trip overview** に載せます。

### Trip overview（旅行概要）

- Trip `summary`（要約）
- Participants 表（登録時）
- 日数・Itinerary 件数・チェックリスト進捗・滞在/移動時間などの運用サマリー
- Stay / Travel / Total の3つがすべて 0 のときは時間行を出さない（v4.2.2+）
- **Days overview**（v4.3.0+）— Day `summary` がある日だけ、メトリクスの直後に一覧表示

```md
### Days overview

- **Day 1 — 2026-04-26**: その日の Day summary
- **Day 2 — 2026-04-27**: その日の Day summary
```

Day summary が 1 件もない場合は `### Days overview` ごと省略します。Daily schedule 各日冒頭の Day summary は **維持**（overview は俯瞰、日別章は詳細用）。

会計向けの Planned / Actual 合計や Difference は **含めません**（`trip stats` を使用）。

### Daily schedule（日別旅程）

```md
## Daily schedule

### Day 1 — 2026-04-26

その日の Day summary（あれば）

#### 09:00 那覇空港

- Category: transport
- 場所: 那覇空港
- 所要時間: 60分
- 移動時間: 30分
- メモ: レンタカー受け取り
```

- Itinerary は **日目 → sort_order** の順
- Trip 期間内の全日を走査（Itinerary が無い日は `_No itineraries scheduled._`）
- Itinerary 配下の **Estimate / Expense は日別章に出さない**

### Reservations / Checklist / Planned cost / Notes

- **Reservations** — 予約情報（0 件なら章ごと省略）。**旅程順フラット**（種別 `###` 見出しなし）。見出しは `**Day N / {itinerary}**`、provider は非冗長時のみ末尾に付与。`確認番号:` / `期間:` / `備考:` ラベル（v4.3.0+）
- **Checklist** — 持ち物（0 件なら省略）
- **Planned cost** — Estimate の通貨別合計と Itinerary 別表（Estimate = 予定費用）
- **Notes** — Trip → Day → Itinerary の順で **Note entity**（Itinerary の単行 `note` フィールドは Daily schedule 内のまま）

### Colophon（奥付）

Generator 名・CLI バージョン・生成日時・Trip 名・期間を末尾に常時出力します。

## Travel Book に載せないもの

- **Expense**（実績費用）— 旅行前しおりの主役にしない
- **Receipt** / Pending Receipt
- **Planned vs Actual Difference**

実績・会計確認は `trip stats` および JSON export を使用してください。

## 出力先

### 標準出力（デフォルト）

`--output` を省略すると、Markdown 本体のみ stdout に出力されます。

```bash
cargo run -- trip export-md 1
```

### ファイル出力（`--output`）

```bash
cargo run -- trip export-md 1 --output trip.md
```

成功時:

```text
Markdown exported: trip.md
```

## 確認用サンプル

### Okinawa Sesoko 2026（canonical + Travel Book fixture）

```bash
bash samples/okinawa_sesoko_2026/seed.sh
cargo run -- trip export-md 1
```

[v4.1.2 で拡充](specifications/v4.1.2-okinawa-travel-book-sample-enrichment-implementation-plan.md)した Summary / Note / Reservation / Estimate が章ごとに表示されます。台帳不変条件（Itinerary 58 / Expense 49 / ¥561,780）は `trip stats` で確認します。

Golden: `samples/okinawa_sesoko_2026/expected-export-md.md`

Post-release review（v4.2.1）: [v4.2.1-travel-book-export-md-post-release-review.md](specifications/v4.2.1-travel-book-export-md-post-release-review.md)

Polish implementation（v4.2.2）: [v4.2.2-travel-book-markdown-polish-implementation-plan.md](specifications/v4.2.2-travel-book-markdown-polish-implementation-plan.md)

Reservation / Summary refinement（v4.3.0）: [v4.3.0-reservation-summary-display-refinement-design.md](specifications/v4.3.0-reservation-summary-display-refinement-design.md)

Post-release review（v4.3.1）: [v4.3.1-reservation-summary-display-post-release-review.md](specifications/v4.3.1-reservation-summary-display-post-release-review.md)

Planned cost polish（v4.3.2）: [v4.3.2-travel-book-planned-cost-polish-implementation-plan.md](specifications/v4.3.2-travel-book-planned-cost-polish-implementation-plan.md)

Presentation model review（v4.4.0）: [v4.4.0-travel-book-presentation-model-review.md](specifications/v4.4.0-travel-book-presentation-model-review.md)

### checklist_generate サンプル

4日間・Itinerary 15件・チェックリスト10件の小規模サンプル:

```bash
bash samples/checklist_generate/seed.sh
cargo run -- trip export-md 1
```

## 関連ドキュメント

- [v4.0.0 Travel Book concept](specifications/v4.0.0-travel-book-concept-design.md)
- [v4.1.0 chapter structure](specifications/v4.1.0-travel-book-chapter-structure-design.md)
- [command-reference.md](command-reference.md) — `trip export-md` / `trip stats` の責務分担
