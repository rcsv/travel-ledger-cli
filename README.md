# Caglla.Travel CLI

Caglla.Travel のコマンドライン版です。旅行の計画を、ターミナルから管理できます。

現時点では **ローカルの SQLite データベース**（`caglla.db`）にデータを保存する CLI アプリです。Web 版やクラウド同期には未対応です。

## できること

- **Trip（旅行）** の登録・一覧・詳細・更新・削除
- **Itinerary（日程）** の登録・一覧・詳細・更新・削除
- 各予定への **開始時刻・所要時間・移動時間** の設定
- **Timeline（タイムライン）** による旅行の流れの表示
- **db reset** による開発用 DB 初期化

## 必要な環境

- [Rust](https://www.rust-lang.org/)（`cargo` が使えること）

## インストール

リポジトリをクローンしたあと、プロジェクト直下でビルドします。

```bash
cargo build
```

ビルドが成功すれば、`cargo run --` の後ろにコマンドを付けて実行できます。

## 品質チェック（make check）

コードの整形・静的解析・テスト・ビルドをまとめて確認できます。

```bash
make check
```

内部では次のコマンドを順番に実行します。

1. `cargo fmt --check` — コード整形の確認
2. `cargo clippy -- -D warnings` — 警告なしの静的解析
3. `cargo test` — ユニットテスト
4. `cargo build` — ビルド

その他の Make ターゲット:

| コマンド | 内容 |
|---|---|
| `make test` | テストのみ実行 |
| `make run` | `cargo run` を実行 |
| `make clean` | ビルド成果物を削除 |

## データベースについて

- DB ファイル名: `caglla.db`（プロジェクト直下に作成されます）
- 初回起動時に `trips` / `itinerary_items` テーブルが自動作成されます
- 既存の DB がある場合は、不足している列を自動で追加します（マイグレーション）

### DB 初期化（開発用）

**開発・動作確認用** のコマンドです。本番運用では使わないでください。

```bash
cargo run -- db reset
```

- `itinerary_items` のデータを全削除
- `trips` のデータを全削除
- テーブル定義は残す
- ID の採番（AUTOINCREMENT）をリセット

## Trip（旅行）の使い方

### 旅行を追加

```bash
cargo run -- trip add "沖縄旅行"
cargo run -- trip add "京都旅行" --start 2026-05-01 --end 2026-05-03
```

| オプション | 説明 |
|---|---|
| `name` | 旅行名（必須） |
| `--start` | 開始日（YYYY-MM-DD、任意） |
| `--end` | 終了日（YYYY-MM-DD、任意） |

### 一覧・詳細

```bash
cargo run -- trip list
cargo run -- trip show 1
```

### 更新・削除

```bash
cargo run -- trip update 1 --name "沖縄・石垣旅行"
cargo run -- trip update 1 --start 2026-04-26 --end 2026-04-30
cargo run -- trip delete 1
```

更新時は `--name` / `--start` / `--end` のうち、変更したい項目だけ指定します。

### JSON エクスポート

旅行 1 件と、紐づく日程を JSON で出力します。将来の Web 版や Firebase / Firestore への移行を想定した形式です。

```bash
# 標準出力に表示
cargo run -- trip export 1

# ファイルに保存
cargo run -- trip export 1 --output trip-1.json
```

出力例（構造）:

```json
{
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
  ]
}
```

`itinerary_items` は一覧表示と同じく、日目・時刻・並び順でソートされた状態で出力されます。

### Markdown エクスポート（旅行しおり）

旅行計画を Markdown 形式の「旅行しおり」として標準出力できます。

```bash
cargo run -- trip export-md 1
```

出力例:

```md
# 沖縄旅行

2026-04-26 〜 2026-04-29

## Day 1

### 09:00 那覇空港
- 場所: 那覇空港
- 所要時間: 60分
- 移動時間: 30分
- メモ: レンタカー受け取り
```

日程は **日目 → 並び順（sort_order）** の順で出力されます。日程が登録されていない日目は表示されません。

チェックリストが登録されている場合、末尾に以下の形式で出力されます。

```md
## Checklist

- [ ] パスポート
- [x] 充電器
```

チェックリストがない場合は `## Checklist` セクション自体を出力しません。

### JSON インポート

`export` で出力した JSON を読み込み、**新しい Trip として**登録します。

```bash
cargo run -- trip import trip-1.json
```

| 動作 | 説明 |
|---|---|
| ID の扱い | JSON 内の `id` / `trip_id` は無視し、DB の AUTOINCREMENT で新規採番 |
| trip_id の変換 | 日程の `trip_id` は、新しく作成された Trip の ID に置き換わる |
| 日時 | `created_at` / `updated_at` はインポート時に新しく設定される |

完了時の表示例:

```
旅行をインポートしました (ID: 2)
  名前: 沖縄旅行
  日程: 3 件
```

エクスポートとインポートの流れ:

```bash
cargo run -- trip export 1 --output trip-1.json
cargo run -- trip import trip-1.json
```

## Itinerary（日程）の使い方

日程は **Trip ID** に紐づきます。先に `trip add` で旅行を作成してください。

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

日程に設定されたカテゴリの `CategoryDefinition.default_checklist` から、チェックリスト項目を自動追加します。

```bash
cargo run -- trip checklist-generate 1
```

| ルール | 説明 |
|---|---|
| 対象 | カテゴリが設定されている itinerary items |
| 重複防止 | 同じ trip 内に同じ title が既にある場合は追加しない |
| 並び順 | 既存の最大 `sort_order` の次から採番 |
| 0件追加 | エラーにせず成功として扱う |

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

## Checklist（持ち物・準備リスト）の使い方

チェックリストは **Trip ID** に紐づきます。

### 項目の追加・一覧

```bash
cargo run -- checklist add 1 "パスポート"
cargo run -- checklist add 1 "充電器"
cargo run -- checklist list 1
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
cargo run -- checklist update 1 --title "旅券" --sort-order 5
cargo run -- checklist check 2
cargo run -- checklist uncheck 2
cargo run -- checklist delete 1
```

## Timeline（タイムライン）の使い方

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

## 開発用サンプルシナリオ

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

## プロジェクト構成（現時点）

```
caglla-cli/
├── src/
│   ├── main.rs       # CLI の入口
│   ├── models.rs     # Trip / ItineraryItem / ItineraryCategory / CategoryDefinition など
│   ├── db.rs         # DB 接続・マイグレーション
│   ├── trip.rs       # Trip CRUD・JSON export/import
│   ├── itinerary.rs  # Itinerary CRUD・タイムライン
│   ├── checklist.rs  # Checklist CRUD
│   ├── markdown.rs   # trip export-md
│   └── diff.rs       # trip diff
├── Cargo.toml
├── Makefile
├── caglla.db         # ローカル DB（実行時に自動作成、git 管理外）
└── README.md
```
