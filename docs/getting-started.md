# Getting Started

Caglla CLI のインストールと初回利用の手順です。

## 必要な環境

- [Rust](https://www.rust-lang.org/)（`cargo` が使えること）

## インストール

### GitHub Release からダウンロード

[GitHub Releases](https://github.com/rcsv/travel-ledger-cli/releases) から、OS 向けのアーカイブ（Linux / macOS: `.tar.gz`、Windows: `.zip`）をダウンロードできます。中身の `travel-ledger-cli`（Windows は `travel-ledger-cli.exe`）を PATH の通った場所に置いて実行してください。

| OS | Asset 名（例: v1.0.5） |
|---|---|
| Linux (x86_64) | `travel-ledger-cli-1.0.5-linux-amd64.tar.gz` |
| macOS (Apple Silicon) | `travel-ledger-cli-1.0.5-macos-arm64.tar.gz` |
| Windows (x86_64) | `travel-ledger-cli-1.0.5-windows-amd64.zip` |

`v*` タグを push すると、GitHub Actions が release build を作成し、上記アセットを Release に添付します。

### ソースからビルド

リポジトリをクローンしたあと、プロジェクト直下でビルドします。

```bash
cargo build
```

ビルドが成功すれば、`cargo run --` の後ろにコマンドを付けて実行できます。以降の例も同形式です（インストール済みの `caglla` バイナリに読み替え可能）。

## データベース

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

## Quick Start

沖縄旅行の 1 日目を登録し、タイムラインで確認する例です。まず DB を空にしてから、順番に実行してください。

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
| その他 | `trip checklist-generate`, `db reset` |

詳細は [command-reference.md](command-reference.md) を参照してください。
