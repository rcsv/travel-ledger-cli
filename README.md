# Caglla.Travel CLI

A local-first travel planning CLI for managing trips, itineraries, checklists, expenses, and Markdown/JSON exports.

Caglla.Travel のコマンドライン版です。旅行の計画を、ターミナルから管理できます。データはローカルの SQLite データベース（`caglla.db`）に保存されます。Web 版やクラウド同期は未対応です。

## Features

- **Trip（旅行）** の登録・一覧・詳細・更新・削除・複製
- **Day（日）** の一覧・詳細・Itinerary 入れ替え（`day swap`）
- **Itinerary（行動・予定）** の登録・一覧・詳細・更新・削除・タイムライン表示
- **Note（メモ）** / **Expense（支出）** / **Estimate（予定費用 / Planned Budget）** / **Participant（参加者）** の CRUD
- **Checklist（持ち物・準備リスト）** の管理と自動生成（`checklist-generate`）
- **JSON エクスポート / インポート**（`trip export` / `trip import`、現行 **schema v6**）と `trip diff`
- **Markdown エクスポート**（`trip export-md`）による旅行しおり出力
- **trip stats** による旅行統計
- **trip doctor / advisor** による旅行計画の点検と改善提案
- **Trip / Day Summary**（`--summary` — 旅行・日ごとの短い概要）

## Data Model

```text
Trip（旅行全体）
 └─ Day（日付コンテナ：何日目か）
      └─ Itinerary（行動：旅行中の予定／実績）
           ├─ Expense（支出 — Actual Money）
           ├─ Estimate（予定費用 — Planned Money）
           └─ Note（メモ）
```

**Itinerary is not a venue.** — Itinerary は場所（Venue / POI）ではなく、**旅行中の行動を表す最小単位** です。`title` と `--day` があれば登録でき、`location` は任意です。高速道路・給油・チェックイン・帰宅など、固定 POI に紐づかない行も Itinerary として扱います。

詳細: [docs/data-model.md](docs/data-model.md) · [docs/specifications/itinerary-model.md](docs/specifications/itinerary-model.md)

## Installation

### GitHub Release

[GitHub Releases](https://github.com/rcsv/travel-ledger-cli/releases) から OS 向けアーカイブをダウンロードし、`caglla-cli` を PATH に置いてください。

### Build from source

```bash
cargo build
```

詳細: [docs/getting-started.md](docs/getting-started.md)

## Quick Start

```bash
cargo run -- db reset
cargo run -- trip add "沖縄旅行" --start 2026-04-26 --end 2026-04-29
cargo run -- itinerary add 1 --day 1 --time 09:00 --duration 90 --travel 20 "首里城"
cargo run -- itinerary add 1 --day 1 --time 10:50 --duration 60 --travel 15 "国際通り"
cargo run -- itinerary timeline 1
```

## Main Commands

| カテゴリ | 主なコマンド |
|---|---|
| Trip | `trip add`, `trip list`, `trip show`, `trip update`, `trip delete`, `trip duplicate`, `trip stats` |
| Day | `day list`, `day show`, `day update`, `day swap` |
| Itinerary | `itinerary add`, `itinerary list`, `itinerary show`, `itinerary update`, `itinerary delete`, `itinerary timeline` |
| Note / Expense / Estimate / Reservation | `note add/list/...`, `expense add/list/...`, `estimate add/list/...`, `reservation add/list/...` |
| Checklist | `checklist add/list/check/...`, `trip checklist-generate` |
| Export | `trip export`, `trip import`, `trip validate-export`, `trip diff`, `trip export-md` |
| Diagnostics | `trip doctor`, `trip advisor` |
| Dev | `db path`, `db status`, `db reset` |

コマンド詳細: [docs/command-reference.md](docs/command-reference.md)

## Documentation

| ドキュメント | 内容 |
|---|---|
| [docs/getting-started.md](docs/getting-started.md) | インストール・DB・Quick Start |
| [docs/command-reference.md](docs/command-reference.md) | 全コマンドのオプションと例 |
| [docs/data-model.md](docs/data-model.md) | データモデルと設計原則 |
| [docs/export-import.md](docs/export-import.md) | JSON export/import・`--json` 出力 |
| [docs/markdown-export.md](docs/markdown-export.md) | Markdown 旅行しおり出力 |
| [docs/development.md](docs/development.md) | 開発・CI・サンプルデータ |
| [docs/github-workflow.md](docs/github-workflow.md) | Issue / Milestone / Project による開発運用 |
| [docs/specifications/](docs/specifications/) | 内部モデル・設計仕様 |
| [docs/releases/](docs/releases/) | リリースノート |

索引: [docs/README.md](docs/README.md)

## Status / Limitations

| 項目 | 状態 |
|---|---|
| データ保存 | ローカル SQLite（`caglla.db`）のみ。Web 版・クラウド同期は未対応 |
| JSON 出力（`--json`） | ツール連携向け。**内部仕様扱い**（構造は将来変更の可能性あり） |
| 費用管理・通貨換算 | Expense（実績）・Estimate（予定）CRUD は対応。Trip / Itinerary 単位の Planned vs Actual 差分表示は対応。精算（Settlement）は未対応 |
| 類似旅行検索（Similarity） | 将来候補（現 CLI には未実装） |

## Releases

GitHub Release 用ノートは [docs/releases/](docs/releases/) にあります。最新: [v3.4.0](docs/releases/v3.4.0-notes.md)

## Security

- [Security Policy](SECURITY.md)

## License

MIT — see [Cargo.toml](Cargo.toml)
