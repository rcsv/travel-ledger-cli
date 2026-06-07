# Day モデル

Caglla CLI における Trip / Day / Itinerary の関係と、Day 自動生成のルールをまとめます。

## エンティティ関係

```text
Trip
 └─ Day (day_number, title, description)
      └─ Itinerary (itinerary_items.day = day_number)
```

| エンティティ | 役割 |
|---|---|
| **Trip** | 旅行全体。`start_date` / `end_date` を必須で持つ |
| **Day** | 旅行内の「何日目か」。`title` / `description` を将来の章立て・GUI 用に保持 |
| **Itinerary** | 1 日の中の予定。現時点では `itinerary_items.day`（整数）で Day に紐づく |

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

## CLI の操作感

ユーザー向け API は変更しません。

```bash
caglla itinerary add 1 --day 2 "首里城"
```

- `--day N` は「Trip 内の N 日目」を意味する
- 内部では Trip 期間から生成済みの Day 行に対応
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
```

`itinerary_items` は引き続き `day INTEGER NOT NULL` を保持しています。

## 次フェーズ（未実装）

| 項目 | 状態 |
|---|---|
| `itinerary_items.day_id` への移行 | 予定 |
| `itinerary_items.day` 列の削除 | 予定 |
| Day コマンド（`day list` / `show` / `update` / `swap`） | 未実装 |
| export schema v2（`days[]`） | 未実装 |
| Markdown への Day title / description 反映 | 未実装 |

## Import / Export

- **import** には `trip.start_date` / `trip.end_date` が必須（日付なし legacy export は import 不可）
- **export** JSON には現時点で `days[]` は含めない（schema v1 のまま）
- import 成功時に Day 1..N を Trip 期間から自動生成する

## 参照

- ユーザー向け説明: [README.md](../../README.md) の Trip / Itinerary 節
- 実装: `src/day.rs`, `src/trip.rs`, `src/db.rs`
