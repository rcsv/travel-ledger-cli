# Caglla.Travel CLI

**旅行の日程・費用・予約・メモ・チェックリストを、ターミナルから管理する local-first CLI。**

[![Rust CI](https://github.com/rcsv/travel-ledger-cli/actions/workflows/rust.yml/badge.svg)](https://github.com/rcsv/travel-ledger-cli/actions/workflows/rust.yml)
[![Latest Release](https://img.shields.io/github/v/release/rcsv/travel-ledger-cli)](https://github.com/rcsv/travel-ledger-cli/releases/latest)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](Cargo.toml)

> Your travel data should belong to you.
> 旅行データを、サービスの中ではなく、あなたの手元に。

データはユーザー管理のローカル SQLite に保存されます。Markdown / JSON として持ち出せます。クラウドアカウントは不要です。

**ナビ:** [出力例](#cli-output-example) · [Quick Start](#quick-start) · [利用目的](#what-you-can-do) · [インストール](#installation) · [ドキュメント](#documentation)

## 名称の対応

| 呼び方 | 意味 |
|---|---|
| **Caglla.Travel CLI** | 製品名（`--about` の表示名） |
| **Travel Ledger** | 公開 Trip データ形式（export schema v8）— [docs/public/](docs/public/) |
| **travel-ledger-cli** | リポジトリ名・Cargo パッケージ名・Release バイナリ名・**実行コマンド名** |
| **caglla.db** / **caglla.toml** | 既定 DB ファイル名とプロジェクト設定ファイル名 |

## CLI output example

`itinerary timeline` で 1 日目の行動が時系列で見えます（以下は実際の CLI 出力）:

```text
沖縄旅行 のタイムライン:

Day 1

09:00 首里城
  所要時間: 90分
  終了予定: 10:30

  ↓ 移動 20分

10:50 国際通り
  所要時間: 60分
  終了予定: 11:50
```

## Why Caglla

旅行計画は予約サイト・地図・メモ・チャットに分散しがちです。Caglla.Travel CLI は、Trip / Day / Itinerary を中心に、日程・費用・予約・メモ・チェックリストを一つのローカル DB にまとめます。

- **Local-first** — データはあなたのマシン上の SQLite。アカウント登録やクラウド同期は不要
- **Portable** — `trip export` / `trip export-md` で JSON・Markdown に持ち出せる
- **Inspectable** — `trip doctor` / `trip advisor` / `trip stats` で計画を点検できる
- **Not a booking site** — 予約サイトや地図アプリの代替ではない。旅行データの台帳（ledger）です

詳細な設計思想: [docs/public/travel-ledger.md](docs/public/travel-ledger.md)

## Quick Start

既存の `caglla.db` には触れません。専用のデモ DB ファイルを `--db` で指定して試せます。
この例は新規 DB を前提に Trip ID `1` を使います。`./okinawa-demo.db` が既にある場合は、別のファイル名を指定するか、既存データを確認してから実行してください。

```bash
travel-ledger-cli --db ./okinawa-demo.db trip add "沖縄旅行" --start 2026-04-26 --end 2026-04-29
travel-ledger-cli --db ./okinawa-demo.db itinerary add 1 --day 1 --time 09:00 --duration 90 --travel 20 "首里城"
travel-ledger-cli --db ./okinawa-demo.db itinerary add 1 --day 1 --time 10:50 --duration 60 --travel 15 "国際通り"
travel-ledger-cli --db ./okinawa-demo.db itinerary timeline 1
```

試し終わったら `./okinawa-demo.db` を削除すれば元の環境に影響はありません。インストール前にソースから試す場合は [Build from source](#build-from-source) を参照してください。

## What you can do

### Plan — 日程・行動・予約・チェックリスト

Trip と Day を軸に Itinerary（行動）を組み立て、Reservation・Note・Checklist を紐づけます。`itinerary timeline` で 1 日の流れを確認できます。

### Track money — Estimate・Expense・Receipt

予定費用（Estimate）と実績（Expense）を分けて記録します。Receipt Inbox から Expense へ昇格するワークフローもあります。

### Own your data — SQLite・JSON・Markdown

ローカル SQLite が正本です。`trip export`（schema v8）と `trip export-md` で持ち出せます。`trip import` / `trip diff` で差分確認もできます。

### Review — stats・doctor・advisor

`trip stats` で集計、`trip doctor` で整合性チェック、`trip advisor` で改善提案を得られます。

### Work with proposals safely

外部 AI やツールからの旅行案は、**dry-run → 確認 → apply** の流れで安全に扱えます。詳細は [docs/ai.md](docs/ai.md) と [docs/public/proposals.md](docs/public/proposals.md) を参照してください。

## Core Concepts / Data Model

```text
Trip（旅行全体）
 └─ Day（日付コンテナ：何日目か）
      └─ Itinerary（行動：旅行中の予定／実績）
           ├─ Expense（支出 — Actual Money）
           ├─ Estimate（予定費用 — Planned Money）
           └─ Note（メモ）
 └─ Receipt（Expense 化待ちの未整理支払い候補。Actual ではない）
```

**Itinerary is not a venue.** — Itinerary は場所（Venue / POI）ではなく、**旅行中の行動を表す最小単位** です。`title` と `--day` があれば登録でき、`location` は任意です。高速道路・給油・チェックイン・帰宅など、固定 POI に紐づかない行も Itinerary として扱います。

詳細: [docs/data-model.md](docs/data-model.md) · [docs/specifications/itinerary-model.md](docs/specifications/itinerary-model.md)

### Local-first database

データはローカル SQLite に保存されます（既定は CWD の `caglla.db`）。`travel-ledger-cli db use` で `./caglla.toml` に既定 DB を記録でき、一時的な上書きには `--db` / `CAGLLA_DB` を使えます。

優先順位: `--db` > `CAGLLA_DB` > `./caglla.toml` > `./caglla.db`

## Installation

### GitHub Release

[GitHub Releases](https://github.com/rcsv/travel-ledger-cli/releases/latest) から OS 向けアーカイブをダウンロードし、バイナリを PATH に置いてください。

| OS | アーカイブ名（例: v4.8.6） | バイナリ |
|---|---|---|
| Linux (x86_64) | `travel-ledger-cli-4.8.6-linux-amd64.tar.gz` | `travel-ledger-cli` |
| macOS (Apple Silicon) | `travel-ledger-cli-4.8.6-macos-arm64.tar.gz` | `travel-ledger-cli` |
| Windows (x86_64) | `travel-ledger-cli-4.8.6-windows-amd64.zip` | `travel-ledger-cli.exe` |

### Build from source

リポジトリをクローンしたあと:

```bash
cargo build --release
```

ビルド後は `target/release/travel-ledger-cli` を直接実行するか、PATH に置いてください。開発中の一回限りの実行には:

```bash
cargo run -- --db ./okinawa-demo.db trip list
```

`cargo run --` は開発用です。インストール済みバイナリと混在させないでください。

## Documentation

| ドキュメント | 内容 |
|---|---|
| [docs/getting-started.md](docs/getting-started.md) | インストール・DB・Quick Start |
| [docs/command-reference.md](docs/command-reference.md) | 全コマンドのオプションと例 |
| [docs/data-model.md](docs/data-model.md) | データモデルと設計原則 |
| [docs/ai.md](docs/ai.md) | AI 連携の概念と責務分担 |
| [docs/export-import.md](docs/export-import.md) | JSON export/import・`--json` 出力 |
| [docs/markdown-export.md](docs/markdown-export.md) | Markdown 旅行しおり出力 |
| [docs/development.md](docs/development.md) | 開発・CI・サンプルデータ |
| [docs/public/](docs/public/) | **Travel Ledger 外向きドキュメント（schema v8）** |
| [docs/releases/](docs/releases/) | リリースノート |

索引: [docs/README.md](docs/README.md)

## Status / Non-goals

| 項目 | 状態 |
|---|---|
| データ保存 | **local-first** — ローカル SQLite。Web 版・クラウド同期は未対応 |
| 役割 | 予約サイト・地図アプリ・SNS ではない |
| JSON 出力（`--json`） | ツール連携向け。**内部仕様扱い**（Travel Ledger schema v8 の公開契約とは別） |
| 精算（Settlement） | 未対応 |
| 類似旅行検索（Similarity） | 将来候補（現 CLI には未実装） |

## Contributing / Security / License

- [CONTRIBUTING.md](CONTRIBUTING.md) — 開発・リリース手順
- [SECURITY.md](SECURITY.md) — セキュリティ報告とローカルデータの扱い
- **License:** MIT — [Cargo.toml](Cargo.toml)

## Latest Release

**[v4.8.9](docs/releases/v4.8.9-notes.md)** — Fragment apply confirm transaction structured errors follow-up

- [GitHub Releases](https://github.com/rcsv/travel-ledger-cli/releases/latest)
- 過去のリリースノート: [docs/releases/](docs/releases/)
