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

集計内容: Days、Itineraries、Checklist 進捗、Category Breakdown、Time Summary（所要時間・移動時間・合計）、Estimates（件数と通貨別 **Planned total**）、Expenses（件数と通貨別 **Actual total**）。Estimate と Expense が両方ある場合は通貨別 **Difference**（Actual − Planned）も表示します。

`--json` 出力には `estimate_count` / `estimate_totals`（Planned）、`expense_count` / `expense_totals`（Actual）、および両方がある場合の `difference_totals` が含まれます。

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

2 つの Day の **plan payload** を入れ替えます。Day 番号やカレンダー日付は変更しません。

入れ替え対象: 配下の Itinerary、`days.title` / `days.summary`、Day-level Note。

入れ替えないもの: `days.id`、`day_number`、Trip の開始日・終了日から導出される日付、`created_at`。

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
cargo run -- expense add --itinerary 12 --amount 4000 --currency JPY \
  --paid-by-participant Alice --beneficiary Alice --beneficiary Bob
cargo run -- expense add --itinerary 12 --amount 4000 --currency JPY --shared-with all

cargo run -- expense list --itinerary 12
cargo run -- expense list --trip 1

cargo run -- expense show 1
cargo run -- expense update 1 --amount 2500 --note 後から追記
cargo run -- expense update 1 --clear-paid-by --clear-beneficiaries
cargo run -- expense delete 1
```

| ルール | 内容 |
|---|---|
| 親 | **Itinerary のみ**（`add` は `--itinerary` 必須） |
| `--amount` / `--currency` | **必須** |
| `--title` / `--note` / `--paid-by-name` / `--expense-date` | 任意 |
| `--paid-by-participant` / `--beneficiary` / `--shared-with all` | 任意（Participant 登録 Trip のみ。`add` / `update` とも `--shared-with` と `--beneficiary` は排他） |
| `update` の `--clear-paid-by` / `--clear-beneficiaries` | payer / beneficiary のクリア |
| `expense list` | `--itinerary` または `--trip` のいずれか |
| 金額の保存 | DB は最小通貨単位の **整数**（JPY=円、USD `12.50` → 1250 セント） |

詳細仕様: [specifications/expense-model.md](specifications/expense-model.md) / [shared-expense-entity-design.md](specifications/shared-expense-entity-design.md)

## Estimate（Planned Budget）

Itinerary 配下の **事前見積**（Planned Money）。Expense（実績支出）とは別概念です。

```bash
cargo run -- estimate add --itinerary 12 --amount 14000 --currency JPY
cargo run -- estimate add --itinerary 12 --amount 14000 --currency JPY --title "ホテル朝食"

cargo run -- estimate list --itinerary 12
cargo run -- estimate list --trip 1
cargo run -- estimate list --trip 1 --json

cargo run -- estimate show 3
cargo run -- estimate show 3 --json

cargo run -- estimate update 3 --amount 15000
cargo run -- estimate update 3 --title "ホテル朝食 revised" --note "5人分"
cargo run -- estimate update 3 --clear-title
cargo run -- estimate update 3 --clear-note

cargo run -- estimate delete 3
```

| ルール | 内容 |
|---|---|
| 親 | **Itinerary のみ**（`add` は `--itinerary` 必須） |
| `--amount` / `--currency` | `add` で必須。`update` で `--currency` 変更時は `--amount` も必須 |
| `--title` / `--note` / `--sort-order` | 任意 |
| `update` の `--clear-title` / `--clear-note` | nullable フィールドのクリア |
| `estimate list` | `--itinerary` または `--trip` のいずれか（排他） |
| 金額の保存 | DB は最小通貨単位の **整数**（JPY=円、USD `12.50` → 1250 セント） |
| **export / import** | `trip export` / `trip import` の schema v6 で `days[].itineraries[].estimates[]` に含まれる（id / timestamps は出力しない） |
| **trip diff** | schema v6+ 同士で added / removed / field changed を比較 |
| **trip stats** | Trip 配下 Estimate の件数・通貨別 **Planned total**（`estimate_count` / `estimate_totals`）。Estimate と Expense 両方がある場合は `difference_totals`（Actual − Planned） |
| **export-md** | Travel Book 章立て（v4.2.0+）— Cover / Trip overview / Daily schedule / Reservations / Checklist / **Planned cost**（Estimate）/ Notes / Colophon。**Expense・Difference は含めない**（会計は `trip stats`） |
| **itinerary replicate** | source Itinerary 配下 Estimate を target にコピー（デフォルト） |

**未実装:** `--without-estimates`（将来需要が明確になった場合に検討）

責務整理: [specifications/estimate-model.md](specifications/estimate-model.md)  
Entity Design: [specifications/estimate-entity-design.md](specifications/estimate-entity-design.md)  
Implementation Plan: [specifications/estimate-implementation-plan.md](specifications/estimate-implementation-plan.md)

### Proposal Fragment `add_estimate`（Planned Money）

Itinerary target のみ。`fragment apply --dry-run` で preview、`--confirm` で Estimate 1 件追加。通常 `estimate add` とは別入口です。

利用者向け契約・CLI 例の正本: [v4.7.45 Estimate documentation and CLI usage review](specifications/v4.7.45-estimate-documentation-and-cli-usage-review.md)

`update_estimate` Fragment planning（未実装）: [v4.7.46 P-6o update_estimate planning](specifications/v4.7.46-p6o-update-estimate-planning.md)

## Receipt Inbox（metadata-only）

Trip 直下の **Expense 化待ちの未整理支払い候補**（Receipt）。Expense（確定 Actual）ではありません。`image_path` / OCR / Attachment は **非対象**（将来の証憑画像は Receipt / Expense 共通の Evidence / Attachment レイヤーで検討）。

```bash
cargo run -- receipt add --trip 1 --day 1 --amount 1700 --currency JPY --memo "これなんだっけ？"
cargo run -- receipt list --trip 1
cargo run -- receipt list --trip 1 --unreviewed
cargo run -- receipt list --trip 1 --trashed
cargo run -- receipt list --trip 1 --all
cargo run -- receipt list --trip 1 --trashed --status ignored
cargo run -- receipt list --trip 1 --json

cargo run -- receipt show 3
cargo run -- receipt show 3 --json

cargo run -- receipt update 3 --memo "おかんのお土産っぽい"
cargo run -- receipt update 3 --amount 1700 --currency JPY
cargo run -- receipt update 3 --occurred-date 2026-04-26

cargo run -- receipt assign 3 --itinerary 1
cargo run -- receipt trash 3
cargo run -- receipt restore 3
cargo run -- receipt ignore 3 --memo "旅行費用ではない" # deprecated alias (trash)
cargo run -- receipt delete 3
```

| ルール | 内容 |
|---|---|
| 親 | **Trip**（`receipts[]` は export で Trip-level。Itinerary 配下にはネストしない） |
| `--amount` / `--currency` | **ペア必須**（片方だけはエラー）。どちらも省略可（その場合は `--memo` 必須） |
| `add` の default status | `unreviewed` |
| `receipt trash` / `restore` | Trash 移動 / 復元（`trashed_at` を更新）。**物理削除ではない** |
| `receipt ignore` | **deprecated alias**。`trash` 相当（内部的に `trashed_at` を設定） |
| `receipt assign` | Receipt → Expense（transaction 必須）。Expense 作成後、Receipt は **削除**される |
| pending sum | `receipt list` の先頭に **Pending Receipts** サマリを表示（**Actual ではない**） |
| status 値 | **`unreviewed` / `ignored` のみ**（user-facing） |
| **export / import** | schema **v8** で `receipts[]`（Trip-level、`day_ref` optional、`trashed_at` optional）。v6 / v7 import は互換 |
| **trip stats / export-md** | Receipt は **含めない**。`trip stats` の Planned / Actual / Difference は Estimate + Expense。`export-md` は Travel Book 向けのため Expense / Difference も **含めない** |
| **未実装** | `receipt purge` / `receipt summary` standalone、Evidence / Attachment 画像証憑 |

設計: [specifications/v3.5.0-receipt-inbox-concept-design.md](specifications/v3.5.0-receipt-inbox-concept-design.md)  
Implementation Plan: [specifications/v3.6.0-receipt-inbox-metadata-only-implementation-plan.md](specifications/v3.6.0-receipt-inbox-metadata-only-implementation-plan.md)

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
cargo run -- itinerary add 1 --day 1 "Wi-Fiを借りる" --after 3
cargo run -- itinerary add 1 --day 1 "保安検査へ向かう" --before 7
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
| `--order` | 並び順を明示指定（任意。`--after` / `--before` と同時指定不可） |
| `--after` | 指定 Itinerary ID の直後へ挿入（`--before` と同時指定不可） |
| `--before` | 指定 Itinerary ID の直前へ挿入（`--after` と同時指定不可） |

**並び順の挙動:**

- `--order` 未指定時は、対象 Day の末尾へ自動追加される（`sort_order` は 1000 刻みで採番）
- `--after` / `--before` 指定時は、対象 Itinerary の直後 / 直前へ挿入される
- 参照先は同じ Trip / Day 内である必要がある
- 前後の `sort_order` に隙間がない場合は、対象 Day を自動で正規化してから挿入する

### 一覧・詳細・更新・削除

```bash
cargo run -- itinerary list 1
cargo run -- itinerary show 1
cargo run -- itinerary update 1 --time 09:30 --duration 120
cargo run -- itinerary update 1 --title "首里城公園" --travel 25
cargo run -- itinerary delete 1
```

一覧は **日目 → 並び順（`sort_order`）→ `id`** の順で表示されます。通常表示では **順序** 列に `sort_order` が表示されます。

### sort_order の正規化

```bash
cargo run -- itinerary normalize 1 --day 1
```

対象 Day の Itinerary の表示順を保ったまま、`sort_order` を `1000, 2000, 3000...` に再採番します。

### Itinerary の移動

```bash
cargo run -- itinerary move 5 --after 3
cargo run -- itinerary move 5 --before 7
```

既存 Itinerary を、対象 Itinerary の直後 / 直前へ移動します。`--after` と `--before` は同時指定できず、自分自身を基準位置に指定することもできません。

### Itinerary の複製

```bash
cargo run -- itinerary replicate --items 12,13,18,21 --to-days 3-5
cargo run -- itinerary replicate --items 12 --to-days 3,4,5
cargo run -- itinerary replicate --items 12,13 --to-days 2,4-6 --without-notes
cargo run -- itinerary replicate --items 12,13 --to-days 3-5 --dry-run
```

| オプション | 説明 |
|---|---|
| `--items` | 複製元 Itinerary ID（カンマ区切り）。同一 Trip・同一 Day に属する必要がある |
| `--to-days` | コピー先 Day（`3`, `3,4,5`, `3-5`, `2,4-6` など） |
| `--without-notes` | Itinerary-level notes（Note エンティティ）をコピーしない。Itinerary 本体の `note` はコピーする |
| `--dry-run` | DB を更新せず、作成予定のみ表示 |

**挙動:**

- Google Calendar の recurring event ではなく、複製後の各 Itinerary は **独立** して編集できる
- コピー対象: `title`, `note`, `start_time`, `sort_order`, `duration_minutes`, `travel_minutes`, `location`, `category`, Itinerary-level notes（デフォルト）、**Estimate（予定費用）**
- コピーしない: Expense（実績支出）, Reservation（予約実体）, `id`, `created_at`, `updated_at`
- `--to-days` に source Day を含めるとエラー
- 複数 item × 複数 Day の複製は **1 トランザクション** で実行される

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

## Database

Caglla CLI は実行時の **作業ディレクトリ（CWD）** 直下の `caglla.db` を参照します。別ディレクトリで実行すると別ファイルを指すため、サンプル DB・本番 DB・テスト DB の混同に注意してください。

### DB パス表示

```bash
cargo run -- db path
```

| 項目 | 説明 |
|---|---|
| 出力 | 解決済み DB パス（**絶対パス**、1 行） |
| 副作用 | **なし** — DB を open しない。ファイル未存在でも **作成しない** |

### DB 状態確認

```bash
cargo run -- db status
cargo run -- db status --json
```

| 項目 | 説明 |
|---|---|
| Path | 解決済み DB パス（絶対パス） |
| Exists | ファイルの有無 |
| File size | **存在時のみ** — バイト数 |
| Trip export schema version | 現行 CLI の trip export JSON schema（`TRIP_EXPORT_SCHEMA_VERSION`、現行 **6**）。**SQLite migration version ではない** |
| Table counts | **存在時のみ** — 主要テーブル行数（migration 適用後） |

DB ファイルが **存在しない** 場合: open せず、ファイルも作成しません。`file_size` / `table_counts` は表示しません。

`--json` 出力は envelope `schema_version: 1`（CLI JSON フォーマット版）。`trip_export_schema_version` は trip export schema です。`exists: false` のとき `file_size_bytes` / `table_counts` は **省略** されます。

### DB リセット（開発用）

```bash
cargo run -- db reset
```

全 Trip / Itinerary / Checklist 等を削除し、AUTOINCREMENT をリセットします。**本番運用では使わない** でください。

JSON 出力の詳細は [export-import.md](export-import.md#json-出力について) を参照してください。
