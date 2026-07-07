# Data Model

Caglla CLI では、旅行計画を次の階層で表現します。

```text
Trip（旅行全体）
 ├─ Participant（参加関係）← v2.0.0 実装済み。Trip 内の参加行（人そのものの正本ではない）
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

将来の Venue 導入方針（primary venue ref のみ・複数 role は初期スコープ外）: [specifications/venue-model-introduction-policy.md](specifications/venue-model-introduction-policy.md)。

### Note と itinerary --note の違い

Trip / Day / Itinerary に付けられる **自由記述メモ**（`note` コマンド）は、既存の `itinerary add ... --note`（1 予定 1 フィールド）とは別エンティティです。

### Checklist

チェックリストは **Trip ID** に紐づきます。`trip checklist-generate` により、カテゴリ定義・組み合わせルールから自動生成できます。

### Participant count semantics（v2）

- **Participant** = その Trip の旅行参加者 **全員**（**自分を含む**）
- **Companion** = **自分以外** の同行者
- **`participant_count` / `traveler_count`** = 自分を含む参加人数
- **`companion_count`** = 自分以外の人数 — `is_self = true` が 1 件あるときのみ `count(participants) - 1` で算出
- participants **未登録** の一人旅は **0 人ではなく unknown / not recorded**
- v2.0.0 の `participants` テーブルに **`is_self`** 列を含める（詳細は [participant-model.md](specifications/participant-model.md#participant-count-semantics)）

### Participant と Person / Traveler Profile（v2）

v2 の **Participant** は Trip 配下の **参加行**（`participants` テーブル）であり、その Trip に誰が参加しているかを識別・表示するためのデータです。**人そのものの正本ではありません。**

将来、Root スコープに **Person / Traveler Profile**（パスポート・生年月日・マイレージ・連絡先等）を導入し、参加行が `person_id` で参照する構造が拡張候補です。**v2.0.0 では Person は実装しません。** 詳細は [participant-model.md](specifications/participant-model.md#conceptual-model-person-vs-trip-participation)。

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
| Participant Implementation Plan | [specifications/participant-implementation-plan.md](specifications/participant-implementation-plan.md) |
| Planning Foundation 総括 | [specifications/planning-foundation-completion-review.md](specifications/planning-foundation-completion-review.md)（v1 クローズ） |
| Export JSON スキーマ | [specifications/export-schema.md](specifications/export-schema.md) |
| 全仕様の索引 | [specifications/README.md](specifications/README.md) |

## 検証データ

実旅行由来の **行動台帳** canonical sample（沖縄・瀬底 2026）は [`samples/okinawa_sesoko_2026/`](../samples/okinawa_sesoko_2026/README.md) を参照してください。
