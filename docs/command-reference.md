# Command Reference

Caglla CLI の全コマンドリファレンスです。データモデルの概要は [data-model.md](data-model.md) を参照してください。

## Trip

### 旅行を追加

```bash
cargo run -- trip add "沖縄旅行" --start 2026-04-26 --end 2026-04-29
cargo run -- trip add "京都旅行" --start 2026-05-01 --end 2026-05-03
```

| オプション | 説明 |
|---|---|
| `name` | 旅行名（必須） |
| `--start` | 開始日（YYYY-MM-DD、**必須**） |
| `--end` | 終了日（YYYY-MM-DD、**必須**、`start` 以降） |

Trip は必ず開始日・終了日を持ちます。作成時に **Day 1..N** が内部的に自動生成されます。詳細は [specifications/day-model.md](specifications/day-model.md) を参照してください。

### 一覧・詳細

```bash
cargo run -- trip list
cargo run -- trip show 1
cargo run -- trip list --json
cargo run -- trip show 1 --json
```

### 更新・削除

```bash
cargo run -- trip update 1 --name "沖縄・石垣旅行"
cargo run -- trip update 1 --start 2026-04-26 --end 2026-04-30
cargo run -- trip delete 1
```

更新時は `--name` / `--start` / `--end` のうち、変更したい項目だけ指定します。期間変更時は Day 行数が自動調整されます。

| 変更 | Day の扱い |
|---|---|
| `start_date` 変更 | Day 行は維持。表示上の日付のみ導出結果が変わる |
| `end_date` 延長 | 不足する Day を追加 |
| `end_date` 短縮 | 削除対象 Day に itinerary / title / description がある場合はエラー。空の Day のみ削除 |

### 複製（trip duplicate）

```bash
cargo run -- trip duplicate 1
cargo run -- trip duplicate 1 --name "Okinawa Copy"
```

| オプション | 説明 |
|---|---|
| `id` | 複製元の旅行 ID |
| `--name` | 複製後の旅行名（省略時は `元の名前 (Copy)`） |

### Stats

```bash
cargo run -- trip stats 1
cargo run -- trip stats 1 --json
```

集計内容: Days、Itineraries、Checklist 進捗、Category Breakdown、Time Summary（所要時間・移動時間・合計）、Expenses（件数と通貨別合計）。

## Day

Day は Trip 作成時および `trip update` 時に **自動生成** されます。`day add` / `day delete` はありません。

**ID の指定について:** `day list` / `day show` / `day swap` では旅行 ID と **日目（day_number）** を指定します。

### Day 一覧

```bash
cargo run -- day list 1
cargo run -- day list 1 --json
```

### Day 詳細

```bash
cargo run -- day show 1 2
cargo run -- day show 1 2 --json
```

対象 Day に属する Itinerary を、timeline と同じ **日目 → 並び順（`sort_order`）** で表示します。

### Day Swap

2 つの Day 配下の Itinerary を **入れ替え** ます。`day_number` と Trip の開始日・終了日は変更しません。

```bash
cargo run -- day swap 1 2 3
```

詳細仕様: [specifications/day-model.md](specifications/day-model.md)

## Note

Trip / Day / Itinerary に付けられる **自由記述メモ** です。

```bash
cargo run -- note add --trip 1 --title "全体メモ" --body "..."
cargo run -- note add --trip 1 --day 2 --body "2日目メモ"
cargo run -- note add --itinerary 12 --body "駐車場メモ"

cargo run -- note list --trip 1
cargo run -- note list --trip 1 --day 2
cargo run -- note list --itinerary 12

cargo run -- note show 1
cargo run -- note update 1 --body "更新後"
cargo run -- note delete 1
```

| ルール | 内容 |
|---|---|
| `--body` | 必須（空文字不可） |
| `--title` | 任意 |
| owner 指定 | `--trip` / `--trip + --day` / `--itinerary` のいずれか（排他） |
| `note list --trip 1` | Trip 直下の Note のみ（Day / Itinerary 配下は含まない） |

詳細仕様: [specifications/note-model.md](specifications/note-model.md)

## Expense

Itinerary 配下の **支出記録** です。

```bash
cargo run -- expense add --itinerary 12 --amount 2200 --currency JPY
cargo run -- expense add --itinerary 12 --amount 12.50 --currency USD --title "Coffee"

cargo run -- expense list --itinerary 12
cargo run -- expense list --trip 1

cargo run -- expense show 1
cargo run -- expense update 1 --amount 2500 --note 後から追記
cargo run -- expense delete 1
```

| ルール | 内容 |
|---|---|
| 親 | **Itinerary のみ**（`add` は `--itinerary` 必須） |
| `--amount` / `--currency` | **必須** |
| `--title` / `--note` / `--paid-by-name` / `--expense-date` | 任意 |
| `expense list` | `--itinerary` または `--trip` のいずれか |
| 金額の保存 | DB は最小通貨単位の **整数**（JPY=円、USD `12.50` → 1250 セント） |

詳細仕様: [specifications/expense-model.md](specifications/expense-model.md)

## Participant

Trip 配下の **参加行**（自分を含む旅行者全員）です。

```bash
cargo run -- participant add --trip 1 --name "ともさん" --self
cargo run -- participant add --trip 1 --name "妻" --sort-order 1

cargo run -- participant list --trip 1
cargo run -- participant list --trip 1 --json

cargo run -- participant show 1
cargo run -- participant update 2 --name "パートナー"
cargo run -- participant update 2 --self
cargo run -- participant update 1 --not-self
cargo run -- participant delete 2
```

| ルール | 内容 |
|---|---|
| 親 | **Trip のみ**（`add` / `list` は `--trip` 必須） |
| `--name` | `add` で必須 |
| `--self` / `--not-self` | Trip 内で `is_self=true` は最大 1 件。`add --self` は既存 self があるとエラー。`update --self` は付け替え |
| 人数統計 | `participant_count` = 自分含む。`companion_count` は self が 1 件のときのみ算出、それ以外は unknown |

詳細仕様: [specifications/participant-model.md](specifications/participant-model.md)

## Itinerary

**Itinerary は Day 内の行動単位** です。`title` と `--day` が必須で、`--location` は任意です。

**ID の指定について:**

- `itinerary list 2` … 旅行 ID 2 に属する Itinerary 一覧を表示
- `itinerary show 11` … Itinerary ID 11 の詳細を表示

詳細仕様: [specifications/itinerary-model.md](specifications/itinerary-model.md)

### Itinerary を追加

```bash
cargo run -- itinerary add 1 --day 1 --time 06:00 --order 1 "出発"
cargo run -- itinerary add 1 --day 1 --time 09:00 --duration 90 --travel 20 "首里城"
cargo run -- itinerary add 1 --day 1 --time 12:30 "昼食" --note "沖縄そば"
```

| オプション | 説明 |
|---|---|
| `trip_id` | 旅行 ID（必須） |
| `--day` | 何日目か（必須） |
| `title` | 行動の要約（必須） |
| `--time` | 開始時刻（HH:MM、任意） |
| `--duration` | 所要時間（分、任意） |
| `--travel` | 次の Itinerary までの移動時間（分、任意） |
| `--location` | 場所の自由記述（任意） |
| `--note` | 短い補足メモ（任意） |
| `--order` | 並び順（任意、小さいほど先） |

### 一覧・詳細・更新・削除

```bash
cargo run -- itinerary list 1
cargo run -- itinerary show 1
cargo run -- itinerary update 1 --time 09:30 --duration 120
cargo run -- itinerary update 1 --title "首里城公園" --travel 25
cargo run -- itinerary delete 1
```

一覧は **日目 → 並び順（`sort_order`）→ `id`** の順で表示されます。

### カテゴリ

```bash
cargo run -- itinerary update 1 --category hotel
cargo run -- itinerary update 1 --category none   # 解除
```

| 保存値 | 表示名 |
|---|---|
| `flight` | フライト |
| `hotel` | ホテル |
| `restaurant` | 食事 |
| `activity` | アクティビティ |
| `transport` | 移動 |
| `shopping` | 買い物 |
| `beach` | ビーチ |
| `museum` | 博物館・展示 |

### チェックリスト自動生成

```bash
cargo run -- trip checklist-generate 1
cargo run -- trip checklist-generate 1 --dry-run
```

| オプション | 説明 |
|---|---|
| `id` | 旅行 ID |
| `--dry-run` | DB を更新せず、追加・スキップ候補のみ表示 |

カテゴリ単体の `default_checklist` と、旅行内のカテゴリ組み合わせルールからチェックリスト項目を自動追加します。同じ trip 内に同じ title が既にある場合は追加しません。

## Checklist

チェックリストは **Trip ID** に紐づきます。

```bash
cargo run -- checklist add 1 "パスポート"
cargo run -- checklist list 1
cargo run -- checklist show 1
cargo run -- checklist update 1 --title "旅券" --sort-order 5
cargo run -- checklist check 2
cargo run -- checklist uncheck 2
cargo run -- checklist delete 1
```

並び順は **未完了 → 完了済み**、同じ状態内では **sort_order → id** の順です。

## Timeline

```bash
cargo run -- itinerary timeline 1
```

旅行の 1 日の流れを、**並び順（`sort_order`）どおり** に見やすく表示します。

```
Day 1

09:00 首里城
  所要時間: 90分
  終了予定: 10:30

  ↓ 移動 20分

10:50 国際通り
  所要時間: 60分
  終了予定: 11:50
```

## Doctor

旅行計画を点検し、予定の詰め込みすぎ、食事予定の不足、移動時間の長さなどを確認します。

```bash
cargo run -- trip doctor 1
cargo run -- trip doctor 1 --json
```

| チェック | 目安 |
|---|---|
| 1日の予定数 | 7件以上で warning |
| 食事予定 | その日に `restaurant` カテゴリがなければ warning |
| 移動時間 | 1日合計 180分以上で warning |
| 所要時間 | 未設定の itinerary がある場合に warning |

検証用サンプル: [`samples/trip_doctor/`](../samples/trip_doctor/)

## Advisor

`trip doctor` が検出した問題に対し、ルールベースで具体的な改善提案を表示します。

```bash
cargo run -- trip advisor 1
cargo run -- trip advisor 1 --with-commands
cargo run -- trip advisor 1 --json
```

| コマンド | 役割 |
|---|---|
| `trip doctor` | 問題の検出（Warnings / Suggestions / Info） |
| `trip advisor` | 問題ごとの改善提案（Warning + Advice） |

`--with-commands` を指定すると、改善提案に加えて次に試せる CLI コマンド例を表示します。

検証用サンプル: [`samples/advisor/`](../samples/advisor/)

JSON 出力の詳細は [export-import.md](export-import.md#json-出力について) を参照してください。
