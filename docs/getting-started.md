# Getting Started

Caglla.Travel CLI のインストールと初回利用の手順です。

## 必要な環境

### GitHub Release から使う場合

Rust / cargo は不要です。OS 向けのビルド済みバイナリをダウンロードして実行してください。

### ソースからビルドする場合

- [Rust](https://www.rust-lang.org/) と `cargo` が必要です。

## インストール

### GitHub Release からダウンロード

[GitHub Releases](https://github.com/rcsv/travel-ledger-cli/releases/latest) から、OS 向けのアーカイブをダウンロードできます。中身の `travel-ledger-cli`（Windows は `travel-ledger-cli.exe`）を PATH の通った場所に置いて実行してください。

| OS | Asset 名（例: v4.8.6） |
|---|---|
| Linux (x86_64) | `travel-ledger-cli-4.8.6-linux-amd64.tar.gz` |
| macOS (Apple Silicon) | `travel-ledger-cli-4.8.6-macos-arm64.tar.gz` |
| Windows (x86_64) | `travel-ledger-cli-4.8.6-windows-amd64.zip` |

リリース作業の詳細は [../CONTRIBUTING.md](../CONTRIBUTING.md) と [../tools/release/README.md](../tools/release/README.md) を参照してください。

### ソースからビルド

リポジトリをクローンしたあと、プロジェクト直下でビルドします。

```bash
cargo build --release
```

ビルドが成功すれば `target/release/travel-ledger-cli` を実行できます。

開発中の一回限りの実行には `cargo run --` を使えます（インストール済みバイナリと混在させないでください）:

```bash
cargo run -- --db ./okinawa-demo.db trip list
```

## データベース

- 既定の DB ファイル名: `caglla.db`（CWD に作成されます）
- 初回起動時にテーブルが自動作成されます
- 既存の DB がある場合は、不足している列を自動で追加します（マイグレーション）
- 一時的に別 DB を使う: `--db ./path/to.db` または環境変数 `CAGLLA_DB`
- プロジェクトごとの既定 DB: `travel-ledger-cli db use ./data/my-trip.db`（`caglla.toml` に記録）

### DB 初期化（開発用）

**開発・動作確認用** のコマンドです。本番運用や一般ユーザー向け Quick Start では使わないでください。

```bash
travel-ledger-cli db reset
```

- `checklist_items` / `itinerary_items` / `trips` のデータを全削除
- テーブル定義は残す
- ID の採番（AUTOINCREMENT）をリセット

## Quick Start

既存の `caglla.db` には触れません。専用のデモ DB を `--db` で指定して試します。
この例は新規 DB を前提に Trip ID `1` を使います。`./okinawa-demo.db` が既にある場合は、別のファイル名を指定するか、既存データを確認してから実行してください。

```bash
travel-ledger-cli --db ./okinawa-demo.db trip add "沖縄旅行" --start 2026-04-26 --end 2026-04-29
travel-ledger-cli --db ./okinawa-demo.db itinerary add 1 --day 1 --time 09:00 --duration 90 --travel 20 "首里城"
travel-ledger-cli --db ./okinawa-demo.db itinerary add 1 --day 1 --time 10:50 --duration 60 --travel 15 "国際通り"
travel-ledger-cli --db ./okinawa-demo.db itinerary timeline 1
```

ソースからビルドした直後で Release バイナリをまだ PATH に置いていない場合:

```bash
cargo run -- --db ./okinawa-demo.db trip add "沖縄旅行" --start 2026-04-26 --end 2026-04-29
# 以降も cargo run -- --db ./okinawa-demo.db ... と同様
```

途中で登録内容を確認したい場合:

```bash
travel-ledger-cli --db ./okinawa-demo.db trip list
travel-ledger-cli --db ./okinawa-demo.db itinerary list 1
```

試し終わったら `./okinawa-demo.db` を削除すれば元の環境に影響はありません。

## コマンド一覧（概要）

| カテゴリ | 主なコマンド |
|---|---|
| Trip | `trip add`, `trip list`, `trip show`, `trip update`, `trip delete`, `trip duplicate` |
| Day | `day list`, `day show`, `day swap` |
| Note | `note add`, `note list`, `note show`, `note update`, `note delete` |
| Expense | `expense add`, `expense list`, `expense show`, `expense update`, `expense delete` |
| Itinerary | `itinerary add`, `itinerary list`, `itinerary show`, `itinerary update`, `itinerary delete` |
| Checklist | `checklist add`, `checklist list`, `checklist show`, `checklist update`, `checklist check`, `checklist uncheck`, `checklist delete` |
| Timeline | `itinerary timeline` |
| Stats | `trip stats` |
| Doctor / Advisor | `trip doctor`, `trip advisor` |
| Export / Import / Diff | `trip export`, `trip import`, `trip validate-export`, `trip diff` |
| Markdown | `trip export-md` |
| その他 | `trip checklist-generate`, `db path`, `db status`, `db use` |

詳細は [command-reference.md](command-reference.md) を参照してください。
