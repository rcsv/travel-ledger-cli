# Data Model

Caglla CLI では、旅行計画を次の階層で表現します。

```text
Trip（旅行全体）
 ├─ Participant（同行者）← v2 予定
 └─ Day（日付コンテナ：何日目か）
      └─ Itinerary（行動：旅行中の予定／実績）
           ├─ Expense（支出）
           ├─ Note（メモ）
           └─ Reservation（予約）← v1.18+
```

## 設計原則

### Local-first SQLite

データはローカルの SQLite データベース（`caglla.db`）に保存されます。Web 版やクラウド同期は未対応です。

### Trip / Day / Itinerary 階層

- **Trip** は必ず開始日・終了日を持ちます。作成時に **Day 1..N**（N = 期間の日数）が内部的に自動生成されます。
- **Day** のカレンダー上の日付は DB には保存せず、`Trip.start_date + (day_number - 1)` で導出します（例: 開始日 2026-12-01 なら Day 1 = 2026-12-01、Day 2 = 2026-12-02）。
- Day は Trip 作成時および `trip update` 時に **自動生成** されます。`day add` / `day delete` はありません。

### Itinerary is not a venue

Itinerary は場所（Venue / POI）ではなく、**旅行中の行動を表す最小単位** です。計画時には予定として、旅行後には実績として扱えます。`title` と `--day` があれば登録でき、`location`（場所文字列）は **任意** です。高速道路・給油・チェックイン・部屋食・帰宅など、固定 POI に紐づかない行も Itinerary として扱います。費用は Itinerary 自身ではなく、配下の **Expense** に記録します。

### Note と itinerary --note の違い

Trip / Day / Itinerary に付けられる **自由記述メモ**（`note` コマンド）は、既存の `itinerary add ... --note`（1 予定 1 フィールド）とは別エンティティです。

### Checklist

チェックリストは **Trip ID** に紐づきます。`trip checklist-generate` により、カテゴリ定義・組み合わせルールから自動生成できます。

## 仕様ドキュメント

| トピック | ドキュメント |
|---|---|
| Day モデル | [specifications/day-model.md](specifications/day-model.md) |
| Itinerary モデル | [specifications/itinerary-model.md](specifications/itinerary-model.md) |
| Ordering | [specifications/ordering-model.md](specifications/ordering-model.md) |
| Note モデル | [specifications/note-model.md](specifications/note-model.md) |
| Expense モデル | [specifications/expense-model.md](specifications/expense-model.md) |
| Participant モデル | [specifications/participant-model.md](specifications/participant-model.md)（v2 Responsibilities Review） |
| Participant Entity Design | [specifications/participant-entity-design.md](specifications/participant-entity-design.md)（v2 Entity Design） |
| Export JSON スキーマ | [specifications/export-schema.md](specifications/export-schema.md) |
| 全仕様の索引 | [specifications/README.md](specifications/README.md) |

## 検証データ

実旅行由来の **行動台帳** canonical sample（沖縄・瀬底 2026）は [`samples/okinawa_sesoko_2026/`](../samples/okinawa_sesoko_2026/README.md) を参照してください。
