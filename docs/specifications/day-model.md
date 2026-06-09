# Day モデル

Caglla CLI における Trip / Day / Itinerary の関係と、Day 自動生成のルールをまとめます。

関連: [Itinerary モデル](itinerary-model.md) / [Expense モデル](expense-model.md)

## エンティティ関係

```text
Trip
 └─ Day (day_number, title, description)
      └─ Itinerary (itinerary_items.day_id → days.id)
```

| エンティティ | 役割 |
|---|---|
| **Trip** | 旅行全体。`start_date` / `end_date` を必須で持つ |
| **Day** | 旅行内の「何日目か」（日付コンテナ）。`title` / `description` を将来の章立て・GUI 用に保持 |
| **Itinerary** | 1 日の中の **行動**（予定／実績）。DB 上は `day_id` で Day に紐づく。CLI / export では `day`（= day_number）を使用。詳細は [Itinerary モデル](itinerary-model.md) |

## Day に date カラムを持たせない理由

Day のカレンダー日付は Trip から導出します。

```text
day_date = trip.start_date + (day_number - 1)
```

例: `start_date = 2026-12-01` のとき、Day 1 = 2026-12-01、Day 2 = 2026-12-02。

- Trip の開始日と Day ごとの日付の二重管理を避ける
- `start_date` 変更時に Day 行を作り直さず、表示日付だけが変わる
- Day は「日数（day_number）」の概念として扱う

## Day 自動生成のルール

### Trip 作成時（`trip add`）

- `--start` / `--end` は必須
- `end_date >= start_date` を検証
- 期間日数 N（開始・終了を含む）を求め、**Day 1..N** を自動作成
- `title` / `description` は空

### Trip 更新時（`trip update`）

更新後も Trip は開始日・終了日の両方を持つ必要があります。

| 操作 | Day の扱い |
|---|---|
| `start_date` 変更 | Day 行は維持。日付は導出のみ変わる |
| `end_date` 延長 | 不足する `day_number` の Day を追加 |
| `end_date` 短縮 | 削除対象 Day に itinerary がある → **エラー** |
| `end_date` 短縮 | 削除対象 Day に title / description がある → **エラー**（現時点） |
| `end_date` 短縮 | 上記が無い空 Day のみ → 削除 |

Itinerary が 0 件になっても Day は削除しません（Day の title / description が残る可能性があるため）。

## Day コマンド（v1.2.0+）

Day の作成・削除コマンドは **実装しない**。Day は Trip 作成 / 更新時に自動管理される。

| コマンド | 状態 | 説明 |
|---|---|---|
| `day list <trip_id>` | 実装済 | Day 一覧と導出日付を表示。`--json` 対応 |
| `day show <trip_id> <day_number>` | 実装済 | Day 詳細と配下 Itinerary を表示。`--json` 対応 |
| `day swap <trip_id> <day_a> <day_b>` | 実装済 | 2 Day 配下の Itinerary を入れ替え |
| `day add` | **非対象** | Trip 自動生成に委ねる |
| `day delete` | **非対象** | Trip 更新時の期間短縮ロジックに委ねる |
| `day update` | 未実装 | title / description 編集（将来） |

### Day Swap の設計

```text
Day番号は固定
中身（Itinerary）だけ交換
```

- `day_number` / `trip.start_date` / `trip.end_date` は変更しない
- 交換対象は `itinerary_items` の `day_id`（および export 互換用 `day` 列）
- `UPDATE ... CASE` を **トランザクション内** で実行。失敗時は rollback

## Itinerary CLI の操作感

ユーザー向け Itinerary API は変更しません。

```bash
caglla itinerary add 1 --day 2 "首里城"
```

- `--day N` は「Trip 内の N 日目」を意味する
- 内部では `day_id` に解決して保存（`find_day_id_by_trip_and_day_number`）
- 読み取り時は `days.day_number` を JOIN して `ItineraryItem.day` に反映
- `day_id` を CLI で指定する必要はない

## 現行 DB スキーマ（抜粋）

```sql
CREATE TABLE days (
  id          INTEGER PRIMARY KEY AUTOINCREMENT,
  trip_id     INTEGER NOT NULL,
  day_number  INTEGER NOT NULL,
  title       TEXT NOT NULL DEFAULT '',
  description TEXT,
  ...
  UNIQUE(trip_id, day_number)
);

-- itinerary_items（抜粋）
day_id INTEGER REFERENCES days(id)  -- 正規の親 FK
day    INTEGER NOT NULL             -- export / 互換用に day_number を同期保持
```

`migrate_itinerary_day_id` が既存行の `day_id` を `days` テーブルから backfill します。

## 次フェーズ（未実装）

| 項目 | 状態 |
|---|---|
| `itinerary_items.day` 列の削除 | 予定（export は引き続き day_number） |
| `day update`（title / description） | 未実装 |
| export schema v2（`days[]`） | 未実装 |
| Markdown への Day title / description 反映 | 未実装 |

## Import / Export

- **import** には `trip.start_date` / `trip.end_date` が必須（日付なし legacy export は import 不可）
- **export** JSON には現時点で `days[]` は含めない（schema v1 のまま）
- import 成功時に Day 1..N を Trip 期間から自動生成する
- `day swap` は DB 内の `day_id` / `day` 列のみ更新。export JSON の構造は変わらない

## 参照

- ユーザー向け説明: [command-reference.md](../command-reference.md) の Day / Itinerary 節
- 実装: `src/day.rs`, `src/trip.rs`, `src/itinerary.rs`, `src/db.rs`
