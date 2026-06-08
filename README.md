# Caglla.Travel CLI

Caglla.Travel のコマンドライン版です。旅行の計画を、ターミナルから管理できます。

現時点では **ローカルの SQLite データベース**（`caglla.db`）にデータを保存する CLI アプリです。Web 版やクラウド同期には未対応です。

## できること

- **Trip（旅行）** の登録・一覧・詳細・更新・削除
- **Day（日）** の一覧・詳細・Itinerary 入れ替え（`day swap`）
- **Note（メモ）** の Trip / Day / Itinerary 向け CRUD（`note add/list/show/update/delete`）
- **Itinerary（日程）** の登録・一覧・詳細・更新・削除
- 各予定への **開始時刻・所要時間・移動時間・場所・カテゴリ** の設定
- **Timeline（タイムライン）** による旅行の流れの表示
- **Checklist（持ち物・準備リスト）** の管理
- **checklist-generate** によるカテゴリ定義・組み合わせルールからのチェックリスト自動生成
- **JSON エクスポート / インポート**（`trip export` / `trip import`）
- **trip diff** による 2 つの旅行 JSON の比較
- **Markdown エクスポート**（`trip export-md`）による旅行しおり出力
- **trip stats** による旅行統計（日数・件数・カテゴリ内訳・時間集計・チェックリスト進捗）
- **trip doctor** による旅行計画の簡易点検（予定過多・食事不足・移動時間など）
- **trip advisor** による旅行計画の改善提案（doctor が検出した問題への具体的アドバイス）
- **db reset** による開発用 DB 初期化

## 制約・未対応機能

v1.0.0 時点で README に記載している CLI の範囲外、または将来候補の機能です。

| 項目 | 状態 |
|---|---|
| データ保存 | ローカル SQLite（`caglla.db`）のみ。Web 版・クラウド同期は未対応 |
| JSON 出力（`--json`） | ツール連携・自動化向け。**内部仕様扱い**（構造は将来変更の可能性あり）。詳細は [JSON 出力について](#json-出力について) |
| 費用管理・通貨換算 | 未対応 |
| 類似旅行検索（Similarity） | 将来候補（現 CLI には未実装） |

## 必要な環境

- [Rust](https://www.rust-lang.org/)（`cargo` が使えること）

## インストール

### GitHub Release からダウンロード

[GitHub Releases](https://github.com/rcsv/caglla-cli/releases) から、OS 向けのアーカイブ（Linux / macOS: `.tar.gz`、Windows: `.zip`）をダウンロードできます。中身の `caglla-cli`（Windows は `caglla-cli.exe`）を PATH の通った場所に置いて実行してください。

| OS | Asset 名（例: v1.0.5） |
|---|---|
| Linux (x86_64) | `caglla-cli-1.0.5-linux-amd64.tar.gz` |
| macOS (Apple Silicon) | `caglla-cli-1.0.5-macos-arm64.tar.gz` |
| Windows (x86_64) | `caglla-cli-1.0.5-windows-amd64.zip` |

`v*` タグを push すると、GitHub Actions が release build を作成し、上記アセットを Release に添付します。

### ソースからビルド

リポジトリをクローンしたあと、プロジェクト直下でビルドします。

```bash
cargo build
```

ビルドが成功すれば、`cargo run --` の後ろにコマンドを付けて実行できます。以降の例も同形式です（インストール済みの `caglla` バイナリに読み替え可能）。

## 使い方

| カテゴリ | 主なコマンド |
|---|---|
| Trip | `trip add`, `trip list`, `trip show`, `trip update`, `trip delete`, `trip duplicate` |
| Day | `day list`, `day show`, `day swap` |
| Note | `note add`, `note list`, `note show`, `note update`, `note delete` |
| Itinerary | `itinerary add`, `itinerary list`, `itinerary show`, `itinerary update`, `itinerary delete` |
| Checklist | `checklist add`, `checklist list`, `checklist show`, `checklist update`, `checklist check`, `checklist uncheck`, `checklist delete` |
| Timeline | `itinerary timeline` |
| Stats | `trip stats` |
| Doctor / Advisor | `trip doctor`, `trip advisor` |
| Export / Import / Diff | `trip export`, `trip import`, `trip validate-export`, `trip diff` |
| Markdown | `trip export-md` |
| その他 | `trip checklist-generate`, `db reset` |

### DB

- DB ファイル名: `caglla.db`（プロジェクト直下に作成されます）
- 初回起動時に `trips` / `itinerary_items` / `checklist_items` テーブルが自動作成されます
- 既存の DB がある場合は、不足している列を自動で追加します（マイグレーション）

### DB 初期化（開発用）

**開発・動作確認用** のコマンドです。本番運用では使わないでください。

```bash
cargo run -- db reset
```

- `checklist_items` / `itinerary_items` / `trips` のデータを全削除
- テーブル定義は残す
- ID の採番（AUTOINCREMENT）をリセット

### Trip

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

Trip は必ず開始日・終了日を持ちます。作成時に **Day 1..N**（N = 期間の日数）が内部的に自動生成されます。Day のカレンダー上の日付は DB には保存せず、`Trip.start_date + (day_number - 1)` で導出します（例: 開始日 2026-12-01 なら Day 1 = 2026-12-01、Day 2 = 2026-12-02）。

詳細は [`docs/specifications/day-model.md`](docs/specifications/day-model.md) を参照してください。

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

更新時は `--name` / `--start` / `--end` のうち、変更したい項目だけ指定します。更新後も Trip は開始日・終了日の両方を持つ必要があります（`end_date >= start_date`）。

期間変更時は Day 行数が自動調整されます。

| 変更 | Day の扱い |
|---|---|
| `start_date` 変更 | Day 行は維持。表示上の日付のみ導出結果が変わる |
| `end_date` 延長 | 不足する Day を追加 |
| `end_date` 短縮 | 削除対象 Day に itinerary / title / description がある場合はエラー。空の Day のみ削除 |

### 複製（trip duplicate）

既存の Trip と、紐づく Itinerary / Checklist を新しい ID で複製します。

```bash
cargo run -- trip duplicate 1
cargo run -- trip duplicate 1 --name "Okinawa Copy"
```

| オプション | 説明 |
|---|---|
| `id` | 複製元の旅行 ID |
| `--name` | 複製後の旅行名（省略時は `元の名前 (Copy)`） |

出力例:

```text
Created trip 12 from trip 1
```

### Stats

旅行の概要統計を表示します。

```bash
cargo run -- trip stats 1
cargo run -- trip stats 1 --json
```

出力例:

```
Trip Stats
==========

Trip: Okinawa Sample Trip

Days: 4

Itineraries: 15

Checklist
---------
Completed: 4 / 10

Category Breakdown
------------------
flight       2
hotel        2
restaurant   3
...

Time Summary
------------
Stay Time:   22h15m
Travel Time: 6h50m
Total Time:  29h05m
```

集計内容:

| 項目 | 説明 |
|---|---|
| Days | 日程が登録されている最大日目 |
| Itineraries | 日程の件数 |
| Checklist | 完了数 / 総数 |
| Category Breakdown | カテゴリ別件数（未設定は `uncategorized`） |
| Time Summary | 所要時間・移動時間・合計（`3h20m` 形式） |

### Export / Import / Diff

#### JSON エクスポート（trip export）

旅行 1 件と、紐づく日程・チェックリストを JSON で出力します。将来の Web 版や Firebase / Firestore への移行を想定した形式です。

**export / import の対象:** **Trip**、**Itinerary（`itinerary_items`）**、**Checklist（`checklist_items`）**、**Note（`notes`、v1.4.0+ / schema v2）** です。`trip export` → `db reset` → `trip import` で、これらのデータをバックアップ／リストアできます。

Export JSON には **`schema_version`**（現在は `2`）、**`generator`**（`caglla-cli`）、**`generator_version`**（export 実行時の CLI バージョン）、**`exported_at`**（export 実行時刻、RFC3339）が含まれます。Import は **`schema_version` 未指定 / `1`（v1 形式）** および **`schema_version: 2`（v2 形式）** の両方に対応します。`generator` / `generator_version` / `exported_at` が無い旧形式とも後方互換です。詳細は [export-schema.md](docs/specifications/export-schema.md) を参照してください。

```bash
# 標準出力に表示
cargo run -- trip export 1

# ファイルに保存
cargo run -- trip export 1 --output backup.json
```

出力例（構造）:

```json
{
  "schema_version": 2,
  "generator": "caglla-cli",
  "generator_version": "1.4.0",
  "exported_at": "2026-06-07T00:00:00Z",
  "trip": {
    "id": 1,
    "name": "沖縄旅行",
    "start_date": "2026-04-26",
    "end_date": "2026-04-29",
    "created_at": "...",
    "updated_at": "..."
  },
  "itinerary_items": [
    {
      "id": 1,
      "trip_id": 1,
      "day": 1,
      "title": "首里城",
      "start_time": "09:00",
      "duration_minutes": 90,
      "travel_minutes": 20,
      "location": "沖縄県那覇市首里金城町1-2"
    }
  ],
  "checklist_items": [
    {
      "id": 1,
      "trip_id": 1,
      "title": "パスポート",
      "is_done": false,
      "sort_order": 0
    }
  ],
  "notes": [
    {
      "owner_type": "trip",
      "title": "全体メモ",
      "body": "..."
    },
    {
      "owner_type": "day",
      "day_number": 2,
      "title": "2日目メモ",
      "body": "..."
    },
    {
      "owner_type": "itinerary",
      "itinerary_key": {
        "day_number": 2,
        "sort_order": 3,
        "start_time": "09:00",
        "title": "美ら海水族館"
      },
      "title": "水族館メモ",
      "body": "..."
    }
  ]
}
```

`itinerary_items` は一覧表示と同じく、日目・時刻・並び順でソートされた状態で出力されます。`checklist_items` は一覧表示と同じく、未完了 → 完了済み、同状態内では `sort_order` → `id` の順で出力されます。

**旧フォーマットとの互換:** Import は次の旧形式も読み込めます（**ただし `trip.start_date` / `trip.end_date` は必須**）。

| 旧形式 | 扱い |
|---|---|
| `trip.start_date` / `trip.end_date` なし | **import 不可** |
| `schema_version` 未指定 / `1` | v1 形式として import（`notes` なし） |
| `schema_version: 2` | v2 形式として import（`notes` あり） |
| `schema_version` / `generator` / `generator_version` / `exported_at` なし | メタデータなしとして import（問題なし） |
| `generator: "unknown"` や旧 `generator_version` | import 可能（warning なし） |
| `checklist_items` なし（v1.0.2 以前） | チェックリストは空として import |

#### JSON インポート（trip import）

`export` で出力した JSON を読み込み、**新しい Trip として**登録します。

```bash
cargo run -- trip import backup.json
```

| 動作 | 説明 |
|---|---|
| ID の扱い | JSON 内の `id` / `trip_id` は無視し、DB の AUTOINCREMENT で新規採番 |
| trip_id の変換 | 日程・チェックリストの `trip_id` は、新しく作成された Trip の ID に置き換わる |
| 日時 | `created_at` / `updated_at` はインポート時に新しく設定される |
| 旅行期間 | `trip.start_date` / `trip.end_date` は必須。import 時に Day 1..N を自動生成 |
| Checklist | `checklist_items` があれば復元する。省略時は空配列として扱う |
| Note | `notes` があれば復元する（schema v2）。省略時は空配列として扱う。Itinerary Note は `itinerary_key` で紐づけ（DB の内部 id は export しない） |
| メタデータ | `schema_version` / `exported_at` は import 時に無視される。`generator` / `generator_version` は import サマリー表示用（DB には保存しない） |
| 旧フォーマット | `checklist_items` や export メタデータが無い JSON も import 可能（日付なし export は不可） |

**import 後の Trip ID について:** export JSON 内の `trip.id` は、import 後の DB 上の ID を保証しません（他の Trip が既にある場合、採番は 2, 3, … になることもあります）。import 完了サマリーに表示される ID（例: `沖縄旅行 (ID: 2)`）を使ってください。確認は `trip list` や `trip show <new_id>`、`itinerary list <new_id>`、`checklist list <new_id>` で行います。export 元の Trip ID をそのまま指定しないでください。

完了時の表示例（現行 export）:

```text
旅行をインポートしました

Trip:
  沖縄家族旅行 (ID: 1)

Created:
  日程           : 3 件
  チェックリスト : 2 件
  Note           : 1 件

Schema:
  version 2

Export:
  generator : caglla-cli
  version   : 1.0.8
```

旧形式（`schema_version` キーなし）の import 完了例:

```text
旅行をインポートしました

Trip:
  Legacy Trip (ID: 1)

Created:
  日程           : 0 件
  チェックリスト : 0 件

Schema:
  未指定（旧形式）

Export:
  generator : 不明
  version   : 不明
```

エクスポートとインポートの流れ:

```bash
cargo run -- trip export 1 --output backup.json
cargo run -- trip validate-export backup.json
cargo run -- trip import backup.json
```

#### export ファイルの検証（trip validate-export）

export ファイルが **import 可能か** を import 前に確認します。DB は使わず、ファイルのみを読み込みます。

```bash
cargo run -- trip validate-export backup.json
cargo run -- trip validate-export backup.json --json
```

| 用語 | 意味 |
|---|---|
| `valid` | import 可能か（`errors` が空） |
| `warnings` | 推奨形式との差異や注意点（import 可能でも表示される） |
| `checks` | export 形式としての確認結果（✓/✗） |

**legacy について:** v1 形式（`schema_version` 未指定 / `1`）の export は import 可能であれば `valid` になります。`checklist_items` キーがない場合などは `warnings` が表示され、対応する `checks` が ✗ になることがあります。`schema_version` 未指定の旧形式では `schema_version` チェックは ✓（effective v1）です。

テキスト出力例（現行 export）:

```text
Export file: backup.json

Checks:
  ✓ JSON形式
  ✓ schema_version
  ✓ trip
  ✓ itinerary_items
  ✓ checklist_items
  ✓ notes

Summary:
  Trip         : 沖縄家族旅行
  Itineraries  : 3 件
  Checklists   : 2 件
  Notes        : 1 件

Metadata:
  Generator : caglla-cli
  Version   : 1.0.8
  Exported  : 2026-06-07T12:34:56Z

Warnings:
  なし

Result:
  有効な export ファイル
```

legacy export の例:

```text
Checks:
  ✓ JSON形式
  ✗ schema_version
  ✓ trip
  ✓ itinerary_items
  ✗ checklist_items

Summary:
  Trip         : Legacy Trip
  Itineraries  : 0 件
  Checklists   : 0 件

Metadata:
  Generator : 不明
  Version   : 不明
  Exported  : 不明

Warnings:
  - schema_version がありません（旧形式）
  - checklist_items がありません（旧形式）

Result:
  有効な export ファイル
```

**metadata の warning 方針:** `generator` / `generator_version` の欠落や `unknown` 値、旧バージョン表記は warning にしません。`exported_at` が RFC3339 として不正な場合のみ warning になります。

終了コード: `valid: true` → exit 0 / `valid: false` またはファイル読込エラー → exit 1。

`--json` 出力は `ExportValidationReport`（`schema_version` = レポート形式、`export_schema_version` = ファイル内の値）です。`generator` / `generator_version` / `exported_at` はファイル内の値を flat に含みます（キーが無い場合は `null`）。`errors` は常に配列で出力されます（空の場合は `[]`）。

```json
{
  "schema_version": 1,
  "file": "backup.json",
  "valid": true,
  "trip_name": "Okinawa Family Trip",
  "export_schema_version": 1,
  "itinerary_count": 15,
  "checklist_count": 10,
  "generator": "caglla-cli",
  "generator_version": "1.0.8",
  "exported_at": "2026-06-07T12:34:56Z",
  "checks": [
    { "id": "json_format", "passed": true },
    { "id": "schema_version", "passed": true }
  ],
  "warnings": [],
  "errors": []
}
```

legacy ファイルの `--json` では `"generator": null`, `"generator_version": null`, `"exported_at": null` が常に出力されます。

#### 旅行 JSON の比較（trip diff）

2 つの `trip export` JSON を比較し、Trip 名・日程の追加・削除・フィールド変更を表示します。

```bash
cargo run -- trip diff trip-old.json trip-new.json
```

### Markdown Export

旅行計画を Markdown 形式の「旅行しおり」として出力します。

```bash
cargo run -- trip export-md 1
```

出力例:

```md
# 沖縄旅行

2026-04-26 〜 2026-04-29

## Overview

- Days: 4
- Itineraries: 15
- Checklist: 4 / 10 completed
- Stay Time: 22h15m
- Travel Time: 6h50m
- Total Time: 29h05m

## Day 1

### 09:00 那覇空港

- Category: transport
- 場所: 那覇空港
- 所要時間: 60分
- 移動時間: 30分
- メモ: レンタカー受け取り

### 12:30 昼食

- Category: restaurant
- 場所: 国際通り
- 所要時間: 60分
```

日程は **日目 → 並び順（sort_order）** の順で出力されます。日程が登録されていない日目は表示されません。冒頭の **Overview** セクションには `trip stats` と同様の集計サマリー（日数・件数・チェックリスト進捗・時間集計）が含まれます。Category Breakdown は含みません。各 Day 見出し・予定ブロック・Checklist セクションの前後には空行が入り、読みやすさを優先しています。

チェックリストが登録されている場合、末尾に以下の形式で出力されます。

```md
## Checklist

- [ ] パスポート
- [x] 充電器
```

チェックリストがない場合は `## Checklist` セクション自体を出力しません。

#### 標準出力（デフォルト）

`--output` を省略すると、Markdown 本体のみ stdout に出力されます。

```bash
cargo run -- trip export-md 1
```

シェルのリダイレクトでも保存できます。

```bash
cargo run -- trip export-md 1 > trip.md
```

#### ファイル出力（`--output`）

`--output` を指定すると、指定ファイルへ保存します（既存ファイルは確認なしで上書き）。

```bash
cargo run -- trip export-md 1 --output trip.md
```

成功時の表示例:

```text
Markdown exported: trip.md
```

手動確認用のサンプルデータ投入は [Markdown Export 確認用サンプル](#markdown-export-確認用サンプル) を参照してください。

### Doctor

旅行計画を点検し、予定の詰め込みすぎ、食事予定の不足、移動時間の長さなどを確認します。

```bash
cargo run -- trip doctor 1
cargo run -- trip doctor 1 --json
```

出力例:

```
Trip Doctor
===========

Trip: Okinawa Sample Trip

Warnings
--------
- Day 2 has many itineraries (8)
- Day 3 has no restaurant
- Day 4 has high travel time (3h20m)

Suggestions
-----------
- Consider adding a lunch or dinner plan to Day 3
- Consider reducing travel time on Day 4
```

問題がない場合:

```
Trip Doctor
===========

Trip: Okinawa Sample Trip

No major issues found.
```

itinerary が0件の場合:

```
Trip Doctor
===========

Trip: Empty Trip

Info
----
- No itinerary found.
```

点検内容:

| チェック | 目安 |
|---|---|
| 1日の予定数 | 7件以上で warning |
| 食事予定 | その日に `restaurant` カテゴリがなければ warning / suggestion |
| 移動時間 | 1日合計 180分以上で warning / suggestion |
| 所要時間 | 未設定の itinerary がある場合に warning（件数付き） |

検証用の実出力サンプルは [`samples/trip_doctor/`](samples/trip_doctor/) を参照してください。再生成:

```bash
bash samples/trip_doctor/generate_outputs.sh
```

### Advisor

`trip doctor` が検出した問題に対し、ルールベースで具体的な改善提案を表示します。

| コマンド | 役割 |
|---|---|
| `trip doctor` | 問題の検出（Warnings / Suggestions / Info） |
| `trip advisor` | 問題ごとの改善提案（Warning + Advice） |

```bash
cargo run -- trip advisor 1
cargo run -- trip advisor 1 --with-commands
cargo run -- trip advisor 1 --json
cargo run -- trip advisor 1 --json --with-commands
```

`--with-commands` を指定すると、改善提案に加えて次に試せる CLI コマンド例を表示します。

Note: `trip advisor` は Trip 系の診断コマンドですが、予定の追加・一覧・タイムライン確認は `itinerary ...` コマンドを使います。カテゴリ設定は `itinerary add` ではなく `itinerary update --category` で行います。

出力例:

```
Trip Advisor
============

Trip: High Travel Trip

Warning
-------
- Day 1 has high travel time (3h25m)

Advice
------
- Consider reducing travel time.
- Group nearby attractions together.

Warning
-------
- Day 2 has no restaurant

Advice
------
- Consider adding a lunch or dinner plan.
```

問題がない場合は `No major issues found.` を表示します。itinerary 0件の場合は `Info` と改善提案を表示します。

`--with-commands` 指定時は各 issue の Advice の後に `Try` セクションでコマンド例を表示します（問題がない clean trip では `Try` は出ません）。`trip advisor --with-commands` は、DoctorIssue の構造化された対象情報を使って、より具体的なコマンド例を表示します。

検証用の実出力サンプルは [`samples/advisor/`](samples/advisor/) を参照してください。再生成:

```bash
bash samples/advisor/generate_outputs.sh
bash samples/advisor/generate_outputs_with_commands.sh
```

### Day

Day は Trip 作成時および `trip update` 時に **自動生成** されます。`day add` / `day delete` はありません。

日付は Day テーブルに保存せず、`trip.start_date + (day_number - 1)` から導出します。

**ID の指定について:** `day list` / `day show` / `day swap` では旅行 ID と **日目（day_number）** を指定します。Day の内部 ID（`days.id`）を CLI で指定する必要はありません。

#### Day 一覧

```bash
cargo run -- day list 1
cargo run -- day list 1 --json
```

出力例:

```text
Trip: Okinawa Family Trip

Day 1  2026-04-26
Day 2  2026-04-27
Day 3  2026-04-28
Day 4  2026-04-29
```

#### Day 詳細

```bash
cargo run -- day show 1 2
cargo run -- day show 1 2 --json
```

対象 Day に属する Itinerary を、timeline と同じ **日目 → 時刻 → 並び順** で表示します。

出力例:

```text
Trip: Okinawa Family Trip

Day 2
Date: 2026-04-27

Itineraries:
- 09:00 美ら海水族館
- 13:00 海邦丸
- 16:00 古宇利島
```

Itinerary が 0 件の Day でも `Itineraries:` 見出しまで表示します。

#### Day Swap

2 つの Day 配下の Itinerary を **入れ替え** ます。`day_number` と Trip の開始日・終了日は変更しません。

```bash
cargo run -- day swap 1 2 3
```

| 交換対象 | 説明 |
|---|---|
| Itinerary | Day 2 配下 ↔ Day 3 配下の予定が完全に入れ替わる |
| 変更しないもの | `day_number`、`trip.start_date`、`trip.end_date` |

同一 Day を指定した場合はエラーになります。処理は DB トランザクション内で実行されます。

詳細仕様: [`docs/specifications/day-model.md`](docs/specifications/day-model.md)

### Note

Trip / Day / Itinerary に付けられる **自由記述メモ** です。既存の `itinerary add ... --note`（1 予定 1 フィールド）とは別エンティティです。

Export schema v2 / Markdown Export への Note 出力は **未対応**（v1.3.0）。

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

cargo run -- note list --trip 1 --json
cargo run -- note show 1 --json
```

| ルール | 内容 |
|---|---|
| `--body` | 必須（空文字不可） |
| `--title` | 任意 |
| owner 指定 | `--trip` / `--trip + --day` / `--itinerary` のいずれか（排他） |
| `note list --trip 1` | Trip 直下の Note のみ（Day / Itinerary 配下は含まない） |

Trip / Day / Itinerary 削除時に関連 Note は自動削除されます（v1.3.1 以降、削除処理は DB トランザクション内で実行）。`day swap` では Day Note は Day 番号側に残ります。

詳細仕様: [`docs/specifications/note-model.md`](docs/specifications/note-model.md)

### Itinerary

日程は **Trip ID** に紐づきます。先に `trip add --start ... --end ...` で旅行を作成してください。

CLI では従来どおり **`--day N`（何日目か）** で指定します。内部では `days.id`（`day_id`）に解決して保存し、読み取り時は `day_number` を JOIN して返します。export JSON の `day` フィールド（schema v1）は従来どおりです。

**ID の指定について:** `list` 系コマンドでは親リソース（旅行）の ID を指定します。`show` / `update` / `delete` 系では、対象の日程 ID を指定します。ID が `1`, `2`, `3` のように小さい整数でも、コマンドごとに意味が異なる点に注意してください。

- `itinerary list 2` … 旅行 ID 2 に属する日程一覧を表示
- `itinerary show 11` … 日程 ID 11 の詳細を表示
- 日程 ID は `itinerary list <trip_id>` の一覧（先頭の `ID` 列）で確認できます

### 日程を追加

```bash
cargo run -- itinerary add 1 --day 1 --time 09:00 --duration 90 --travel 20 "首里城"
cargo run -- itinerary add 1 --day 1 --time 12:30 "昼食" --note "沖縄そば"
cargo run -- itinerary add 1 --day 1 "ホテルチェックイン" --order 99
```

| オプション | 説明 |
|---|---|
| `trip_id` | 旅行 ID（必須） |
| `--day` | 何日目か（必須） |
| `title` | 予定名（必須） |
| `--time` | 開始時刻（HH:MM、任意） |
| `--duration` | 所要時間（分、任意） |
| `--travel` | 次の予定までの移動時間（分、任意） |
| `--note` | メモ（任意） |
| `--order` | 並び順（任意、小さいほど先。時刻未定のときに便利） |

### 一覧・詳細

```bash
cargo run -- itinerary list 1
cargo run -- itinerary show 1
cargo run -- itinerary list 1 --json
cargo run -- itinerary show 1 --json
```

一覧は **日目 → 時刻 → 並び順** の順で表示されます。時刻がある予定が先、時刻未定が後です。

### 更新・削除

```bash
cargo run -- itinerary update 1 --time 09:30 --duration 120
cargo run -- itinerary update 1 --title "首里城公園" --travel 25
cargo run -- itinerary delete 1
```

### カテゴリ

日程にカテゴリを付与できます（定義済みの8種類のみ）。

```bash
cargo run -- itinerary update 1 --category hotel
cargo run -- itinerary update 1 --category none   # 解除
```

| 保存値 | 表示名 | 標準チェックリスト候補（将来の自動生成用） |
|---|---|---|
| `flight` | フライト | 航空券確認、身分証明書確認、空港到着時刻確認 |
| `hotel` | ホテル | 宿泊予約確認、チェックイン時間確認、住所確認 |
| `restaurant` | 食事 | 予約確認、営業時間確認 |
| `activity` | アクティビティ | 予約確認、所要時間確認、服装確認 |
| `transport` | 移動 | 移動手段確認、所要時間確認 |
| `shopping` | 買い物 | 営業時間確認、支払い方法確認 |
| `beach` | ビーチ | 水着、タオル、日焼け止め |
| `museum` | 博物館・展示 | 営業時間確認、チケット確認 |

Rust 側では `CategoryDefinition` 構造体として `display_name` と `default_checklist` を保持しています。DB には従来どおり lowercase 文字列（例: `hotel`）で保存され、将来の checklist-generate 機能でこの定義を参照する想定です。

```rust
// 例: ItineraryCategory::Hotel.definition()
//   display_name: "ホテル"
//   default_checklist: ["宿泊予約確認", "チェックイン時間確認", "住所確認"]
```

### チェックリスト自動生成

日程に設定されたカテゴリの `CategoryDefinition.default_checklist` に加え、旅行全体のカテゴリ構成に応じた組み合わせルールからチェックリスト項目を自動追加します。

```bash
cargo run -- trip checklist-generate 1
cargo run -- trip checklist-generate 1 --dry-run
```

| オプション | 説明 |
|---|---|
| `id` | 旅行 ID |
| `--dry-run` | DB を更新せず、追加・スキップ候補のみ表示 |

`--dry-run` では DB を更新しません。追加候補とスキップ候補、それぞれの件数を表示します。

| ルール | 説明 |
|---|---|
| カテゴリ単体 | 各 itinerary の category に対応する `default_checklist` を展開 |
| カテゴリ組み合わせ | 旅行内に必要な category が揃っている場合に追加（例: `flight + hotel`） |
| 重複防止 | 同じ trip 内に同じ title が既にある場合は追加しない |
| 並び順 | 既存の最大 `sort_order` の次から採番 |
| 0件追加 | エラーにせず成功として扱う |

組み合わせルール例:

| 条件 | 追加候補 |
|---|---|
| flight + hotel | 宿泊予約確認, 身分証明書, 充電器 |
| flight + transport | ETCカード, 運転免許証, レンタカー予約確認 |
| beach | 水着, タオル, 日焼け止め, サンダル |
| beach + activity | 着替え, 防水バッグ, 酔い止め |
| shopping | エコバッグ, 現金（小銭） |
| museum + activity | 事前予約確認, 入場チケット |

`default_checklist` と重複する項目はスキップされます。

出力例:

```
チェックリストを自動生成しました
追加: 5 件
スキップ: 2 件

追加された項目:
- 宿泊予約確認
- チェックイン時間確認
- 水着
- タオル
- 日焼け止め

スキップされた項目:
- 住所確認
- 営業時間確認
```

`--dry-run` 出力例:

```text
Would add: 3
- 宿泊予約確認
- チェックイン時間確認
- 水着

Would skip: 2
- 住所確認
- 営業時間確認
```

### Checklist

チェックリストは **Trip ID** に紐づきます。

**ID の指定について:** Itinerary と同様に、`list` 系では旅行 ID、`show` / `update` / `check` / `uncheck` / `delete` 系ではチェックリスト項目 ID を指定します。

- `checklist list 2` … 旅行 ID 2 に属するチェックリスト一覧を表示
- `checklist show 5` … チェックリスト項目 ID 5 の詳細を表示
- チェックリスト項目 ID は `checklist list <trip_id>` の一覧（各行先頭の番号）で確認できます

### 項目の追加・一覧

```bash
cargo run -- checklist add 1 "パスポート"
cargo run -- checklist add 1 "充電器"
cargo run -- checklist list 1
cargo run -- checklist list 1 --json
```

一覧の表示例:

```
[ ] 1. パスポート
[x] 2. 充電器
[ ] 3. ETCカード
```

並び順は **未完了 → 完了済み**、同じ状態内では **sort_order → id** の順です。

### 詳細・更新・完了切り替え・削除

```bash
cargo run -- checklist show 1
cargo run -- checklist show 1 --json
cargo run -- checklist update 1 --title "旅券" --sort-order 5
cargo run -- checklist check 2
cargo run -- checklist uncheck 2
cargo run -- checklist delete 1
```

### Timeline

旅行の 1 日の流れを、時系列で見やすく表示します。

```bash
cargo run -- itinerary timeline 1
```

表示例（イメージ）:

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

- 時刻が設定されている予定: 開始時刻・所要時間・終了予定を表示
- 時刻が未設定の予定: `時刻: 未定` と表示（終了予定は表示しません）
- 移動時間がある場合: 次の予定の前に `↓ 移動 N分` を表示

## JSON 出力について

一部の read 系コマンドは `--json` に対応しています。ツール連携・自動化向けです。`trip doctor --json` と `trip advisor --json` は **v1.0.6 以降の構造化フォーマット**（`schema_version: 1`）を使います。

`--json` 指定時は人間向けの見出しや説明文を出さず、pretty JSON のみ stdout に出力します（`trip::print_json()`）。

| コマンド | 例 |
|---|---|
| `trip list` | `cargo run -- trip list --json` |
| `trip show` | `cargo run -- trip show 1 --json` |
| `trip stats` | `cargo run -- trip stats 1 --json` |
| `trip doctor` | `cargo run -- trip doctor 1 --json` |
| `trip advisor` | `cargo run -- trip advisor 1 --json` |
| `trip validate-export` | `cargo run -- trip validate-export backup.json --json` |
| `day list` | `cargo run -- day list 1 --json` |
| `day show` | `cargo run -- day show 1 2 --json` |
| `note list` | `cargo run -- note list --trip 1 --json` |
| `note show` | `cargo run -- note show 1 --json` |
| `itinerary list` | `cargo run -- itinerary list 1 --json` |
| `itinerary show` | `cargo run -- itinerary show 1 --json` |
| `checklist list` | `cargo run -- checklist list 1 --json` |
| `checklist show` | `cargo run -- checklist show 1 --json` |

### trip doctor / advisor JSON（v1.0.6+）

**破壊的変更:** v0.9.3 以前の `trip doctor --json` は issue 配列を root に出力していました。v1.0.6 以降は envelope オブジェクトです（`kind` → `code`、`target` 形状変更、`details` 追加）。

#### Doctor envelope

```json
{
  "schema_version": 1,
  "trip_id": 1,
  "issues": [
    {
      "code": "no_restaurant",
      "severity": "warning",
      "message": "Day 3 has no restaurant",
      "target": { "type": "day", "id": 3 },
      "details": { "day": 3 }
    }
  ]
}
```

#### Advisor envelope

Doctor と同じ issue フィールドに加え、各 issue に `advice`（必須）と `commands`（`--with-commands` 時のみ、空なら省略）があります。root に `with_commands` があります。

```json
{
  "schema_version": 1,
  "trip_id": 1,
  "with_commands": true,
  "issues": [
    {
      "code": "no_restaurant",
      "severity": "warning",
      "message": "Day 3 has no restaurant",
      "target": { "type": "day", "id": 3 },
      "details": { "day": 3 },
      "advice": ["Consider adding a lunch or dinner plan."],
      "commands": [
        "cargo run -- itinerary add 1 --day 3 --time 12:00 --duration 60 \"Lunch\"",
        "cargo run -- itinerary update <itinerary_id> --category restaurant"
      ]
    }
  ]
}
```

#### Issue フィールド

| フィールド | 説明 |
|---|---|
| `code` | 安定 ID（snake_case）: `empty_itinerary`, `overloaded_day`, `no_restaurant`, `high_travel_time`, `missing_duration` |
| `severity` | `info` または `warning` |
| `message` | 人間向けテキスト（表示用。解析に依存しないよう `details` を優先） |
| `target` | 問題の対象（下表参照） |
| `details` | code ごとの構造化メタデータ（空なら省略） |

#### `target.id` の意味（`target.type` 依存）

| `target.type` | `target.id` の意味 |
|---|---|
| `trip` | 旅行 ID（`trips.id` / trip_id） |
| `day` | 旅行内の日数（1-based。**DB の day 行 ID ではない**） |
| `itinerary` | 日程 ID（`itinerary_items.id`） |

#### `details` フィールド（code 別）

| `code` | `details` に含まれるフィールド |
|---|---|
| `empty_itinerary` | （省略） |
| `overloaded_day` | `day`, `itinerary_count` |
| `no_restaurant` | `day` |
| `high_travel_time` | `day`, `travel_minutes` |
| `missing_duration` | `itinerary_id` |

人間向けの doctor/advisor テキスト出力（Warnings / Advice / Try 等）は v1.0.6 でも変更していません。

## 開発用コマンド

### 品質チェック（make check）

```bash
make check
```

内部では `cargo fmt --check` → `cargo clippy -- -D warnings` → `cargo test` → `cargo build` を順に実行します。ローカル開発ではこのコマンドを推奨します。

### GitHub Actions（CI）

`master` への push と pull request で [`.github/workflows/rust.yml`](.github/workflows/rust.yml) が実行され、以下を確認します。

| チェック | 内容 |
|---|---|
| formatting | `cargo fmt -- --check` |
| clippy | `cargo clippy -- -D warnings` |
| tests | `cargo test` |
| build | `cargo build` |

リリース前後の確認手順は [`docs/releases/README.md`](docs/releases/README.md#release-verification) を参照してください。

| コマンド | 内容 |
|---|---|
| `make test` | テストのみ実行 |
| `make run` | `cargo run` を実行 |
| `make clean` | ビルド成果物を削除 |

### Markdown Export 確認用サンプル

`trip export-md` / `trip stats` の見た目確認用に、4日間・日程15件・チェックリスト10件のサンプルデータを一括投入できます。

```bash
bash samples/markdown_sample_commands.sh
```

投入内容の概要:

| 項目 | 内容 |
|---|---|
| 旅行 | Okinawa Sample Trip（2026-04-26 〜 2026-04-29） |
| 日程 | 15件（flight / hotel / restaurant / activity / transport / beach / shopping + uncategorized 1件） |
| チェックリスト | 10件（うち4件を完了済みに設定） |

確認コマンド:

```bash
cargo run -- trip stats 1
cargo run -- trip export-md 1
cargo run -- trip export-md 1 --output sample-trip.md
```

スクリプト本体は [`samples/markdown_sample_commands.sh`](samples/markdown_sample_commands.sh) です。カテゴリは `itinerary update --category` で設定しています。

### 開発用サンプルシナリオ

沖縄旅行の 1 日目を登録し、タイムラインで確認する例です。  
まず DB を空にしてから、順番に実行してください。

```bash
cargo run -- db reset
cargo run -- trip add "沖縄旅行" --start 2026-04-26 --end 2026-04-29
cargo run -- itinerary add 1 --day 1 --time 09:00 --duration 90 --travel 20 "首里城"
cargo run -- itinerary add 1 --day 1 --time 10:50 --duration 60 --travel 15 "国際通り"
cargo run -- itinerary add 1 --day 1 --time 13:00 --duration 120 "ホテルチェックイン"
cargo run -- itinerary timeline 1
```

途中で登録内容を確認したい場合:

```bash
cargo run -- trip list
cargo run -- itinerary list 1
```

## リリース履歴

GitHub Release 用ノートは [`docs/releases/`](docs/releases/) にあります。

| バージョン | 概要 |
|---|---|
| [v1.3.1](docs/releases/v1.3.1-notes.md) | Transaction-safe cascade deletion |
| [v1.3.0](docs/releases/v1.3.0-notes.md) | Note commands |
| [v1.2.0](docs/releases/v1.2.0-notes.md) | Day commands |
| [v1.1.0](docs/releases/v1.1.0-notes.md) | Itinerary day_id model |
| [v1.0.9](docs/releases/v1.0.9-notes.md) | Day model and required trip dates |
| [v1.0.8](docs/releases/v1.0.8-notes.md) | Export metadata |
| [v1.0.7](docs/releases/v1.0.7-notes.md) | Backup validation and import summary |
| [v1.0.6](docs/releases/v1.0.6-notes.md) | Structured doctor/advisor JSON |
| [v1.0.5](docs/releases/v1.0.5-notes.md) | Data completeness and backup reliability |
| [v1.0.4](docs/releases/v1.0.4-notes.md) | Checklist export/import and release binaries |
| [v1.0.3](docs/releases/v1.0.3-notes.md) | Release workflow validation (prerelease) |
| [v1.0.2](docs/releases/v1.0.2-notes.md) | Markdown export and documentation polish |
| [v1.0.1](docs/releases/v1.0.1-notes.md) | Not found handling polish |
| [v1.0.0](docs/releases/v1.0.0-notes.md) | First stable CLI baseline |
| [v0.9.5](docs/releases/v0.9.5-notes.md) | CI and release verification polish |
| [v0.9.4](docs/releases/v0.9.4-notes.md) | Command reference polish |
| [v0.9.3](docs/releases/v0.9.3-notes.md) | Doctor JSON output |
| [v0.9.2](docs/releases/v0.9.2-notes.md) | Checklist JSON output |
| [v0.9.1](docs/releases/v0.9.1-notes.md) | JSON output polish（Trip / Itinerary / Stats） |
| [v0.9.0](docs/releases/v0.9.0-notes.md) | Structured DoctorIssue Targets |
| [v0.8.1](docs/releases/v0.8.1-notes.md) | Advisor command hints |
| [v0.8.0](docs/releases/v0.8.0-notes.md) | Trip Advisor |
| [v0.7.0](docs/releases/v0.7.0-notes.md) | checklist-generate 強化 |
| [v0.6.1](docs/releases/v0.6.1-notes.md) | trip doctor 出力改善 |
| [v0.6.0](docs/releases/v0.6.0-notes.md) | trip doctor |

## プロジェクト構成（現時点）

```
caglla-cli/
├── src/
│   ├── main.rs       # CLI の入口
│   ├── models.rs     # Trip / Day / ItineraryItem / ItineraryCategory など
│   ├── db.rs         # DB 接続・マイグレーション
│   ├── day.rs        # Day CRUD・期間同期
│   ├── trip.rs       # Trip CRUD・JSON export/import/validate
│   ├── itinerary.rs  # Itinerary CRUD・タイムライン
│   ├── checklist.rs  # Checklist CRUD
│   ├── markdown.rs   # trip export-md
│   ├── stats.rs      # trip stats
│   ├── doctor.rs     # trip doctor
│   ├── advisor.rs    # trip advisor
│   └── diff.rs       # trip diff
├── samples/
│   ├── markdown_sample_commands.sh  # Markdown Export 確認用データ投入
│   ├── trip_doctor/                 # trip doctor 検証用サンプル・実出力
│   ├── checklist_generate/          # checklist-generate 検証用サンプル
│   └── advisor/                     # trip advisor 検証用サンプル
├── docs/
│   ├── releases/                    # GitHub Release 用ノート
│   └── specifications/              # 仕様メモ（Day モデルなど）
├── Cargo.toml
├── Makefile
├── caglla.db         # ローカル DB（実行時に自動作成、git 管理外）
└── README.md
```
