# Caglla.Travel CLI

A local-first travel planning CLI for managing trips, itineraries, checklists, expenses, and Markdown/JSON exports.

Caglla.Travel のコマンドライン版です。旅行の計画を、ターミナルから管理できます。データはローカルの SQLite データベースに保存されます（既定は CWD の `caglla.db`）。`caglla db use` で `./caglla.toml` に既定 DB を記録でき、一時的な上書きには `--db` / `CAGLLA_DB` を使えます。Web 版やクラウド同期は未対応です。

## Features

- **Trip（旅行）** の登録・一覧・詳細・更新・削除・複製
- **Day（日）** の一覧・詳細・Itinerary 入れ替え（`day swap`）
- **Itinerary（行動・予定）** の登録・一覧・詳細・更新・削除・タイムライン表示
- **Note（メモ）** / **Expense（支出）** / **Estimate（予定費用 / Planned Budget）** / **Receipt Inbox（Expense 化待ちの未整理支払い候補）** / **Participant（参加者）** の CRUD
- **Checklist（持ち物・準備リスト）** の管理と自動生成（`checklist-generate`）
- **JSON エクスポート / インポート**（`trip export` / `trip import`、現行 **schema v8**）と `trip diff`
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
 └─ Receipt（Expense 化待ちの未整理支払い候補。Actual ではない）
```

**Itinerary is not a venue.** — Itinerary は場所（Venue / POI）ではなく、**旅行中の行動を表す最小単位** です。`title` と `--day` があれば登録でき、`location` は任意です。高速道路・給油・チェックイン・帰宅など、固定 POI に紐づかない行も Itinerary として扱います。

詳細: [docs/data-model.md](docs/data-model.md) · [docs/specifications/itinerary-model.md](docs/specifications/itinerary-model.md)

## Installation

### GitHub Release

[GitHub Releases](https://github.com/rcsv/travel-ledger-cli/releases) から OS 向けアーカイブをダウンロードし、`travel-ledger-cli` を PATH に置いてください。

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
| Note / Expense / Estimate / Receipt / Reservation | `note add/list/...`, `expense add/list/...`, `estimate add/list/...`, `receipt add/list/...`, `reservation add/list/...` |
| Checklist | `checklist add/list/check/...`, `trip checklist-generate` |
| Export | `trip export`, `trip import`, `trip validate-export`, `trip diff`, `trip export-md` |
| Diagnostics | `trip doctor`, `trip advisor` |
| Dev | `db path`, `db status`, `db reset`, `db use`（`--db` / `CAGLLA_DB` / `caglla.toml` で DB パス指定可） |

### Database path（`caglla.toml`）

プロジェクトごとに既定の DB を記録するには:

```bash
caglla db use ./data/okinawa.db   # CWD の caglla.toml に [database].path を保存
caglla db path                  # 解決後の DB パスを表示
caglla db use --clear           # config の path を削除（既定 ./caglla.db に戻る）
```

優先順位: `--db` > `CAGLLA_DB` > `./caglla.toml` > `./caglla.db`。`db use` は config 更新のみで、当該コマンド実行中の DB を即時切り替えしません。

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
| [docs/public/](docs/public/) | **Travel Ledger 外向きドキュメント（schema v8 / 公開方向）** |
| [docs/releases/](docs/releases/) | リリースノート |

索引: [docs/README.md](docs/README.md)

## Status / Limitations

| 項目 | 状態 |
|---|---|
| データ保存 | ローカル SQLite（既定 `caglla.db`）。`db use` で `caglla.toml` に既定パスを保存。`--db` / `CAGLLA_DB` で一時上書き可。Web 版・クラウド同期は未対応 |
| JSON 出力（`--json`） | ツール連携向け。**内部仕様扱い**（構造は将来変更の可能性あり） |
| 費用管理・通貨換算 | Expense（実績）・Estimate（予定）CRUD は対応。Trip / Itinerary 単位の Planned vs Actual 差分表示は対応。精算（Settlement）は未対応 |
| 類似旅行検索（Similarity） | 将来候補（現 CLI には未実装） |

## Releases

Related documents for contributors and AI assistants:

- [CONTRIBUTING.md](CONTRIBUTING.md) — development and release rules
- [docs/current-work.md](docs/current-work.md) — active planning state
- [tools/release/README.md](tools/release/README.md) — release procedure

GitHub Release 用ノートは [docs/releases/](docs/releases/) にあります。

**最新:** [v4.7.23](docs/releases/v4.7.23-notes.md) — P-6f add_note --confirm。

**直近のリリース履歴:**

| Version | 種別 | 概要 |
|---|---|---|
| [v4.7.23](docs/releases/v4.7.23-notes.md) | minor | Fragment apply add_note --confirm (P-6f) |
| [v4.7.22](docs/releases/v4.7.22-notes.md) | minor | Fragment apply add_note dry-run (P-6f) |
| [v4.7.21](docs/releases/v4.7.21-notes.md) | minor | Fragment apply add_itinerary field expansion (P-6e) |
| [v4.7.20](docs/releases/v4.7.20-notes.md) | docs | P-6 post-implementation review |
| [v4.7.19](docs/releases/v4.7.19-notes.md) | minor | Fragment apply --confirm (P-6d) |
| [v4.7.18](docs/releases/v4.7.18-notes.md) | minor | Fragment apply dry-run (P-6c) |
| [v4.7.17](docs/releases/v4.7.17-notes.md) | minor | Proposal materialize --confirm (P-6b) |
| [v4.7.16](docs/releases/v4.7.16-notes.md) | minor | Proposal materialize dry-run (P-6a) |
| [v4.7.15](docs/releases/v4.7.15-notes.md) | docs | Materialize / apply planning spec |
| [v4.7.14](docs/releases/v4.7.14-notes.md) | hotfix | Public examples guard CI isolation |
| [v4.7.13](docs/releases/v4.7.13-notes.md) | docs | Proposal storage strategy planning |
| [v4.7.12](docs/releases/v4.7.12-notes.md) | minor | Public examples validation guard |
| [v4.7.11](docs/releases/v4.7.11-notes.md) | minor | Proposal Fragment file validation |
| [v4.7.10](docs/releases/v4.7.10-notes.md) | minor | Proposal Envelope show / inspect |
| [v4.7.9](docs/releases/v4.7.9-notes.md) | minor | Proposal Envelope file validation |
| [v4.7.8](docs/releases/v4.7.8-notes.md) | docs | Proposal implementation planning |
| [v4.7.7](docs/releases/v4.7.7-notes.md) | docs | Public schema post-review |
| [v4.7.6](docs/releases/v4.7.6-notes.md) | docs | Public JSON examples / concept stream post-review |
| [v4.7.5](docs/releases/v4.7.5-notes.md) | docs | Public examples / AI JSON generation guide |
| [v4.7.4](docs/releases/v4.7.4-notes.md) | docs | Materialize gate / validation rules |
| [v4.7.3](docs/releases/v4.7.3-notes.md) | docs | Proposal Fragment concept specification |
| [v4.7.2](docs/releases/v4.7.2-notes.md) | docs | Trip Proposal Envelope concept specification |
| [v4.7.1](docs/releases/v4.7.1-notes.md) | docs | Public README / schema docs outline |
| [v4.7.0](docs/releases/v4.7.0-notes.md) | docs | Travel Ledger public direction concept review |
| [v4.6.43](docs/releases/v4.6.43-notes.md) | docs | Release workflow asset upload follow-up |
| [v4.6.42](docs/releases/v4.6.42-notes.md) | minor | Reservation write service Phase R-5 adapter cleanup |
| [v4.6.41](docs/releases/v4.6.41-notes.md) | minor | Reservation write service Phase R-2+R-3 |
| [v4.6.40](docs/releases/v4.6.40-notes.md) | docs | Reservation write service migration plan |
| [v4.6.39](docs/releases/v4.6.39-notes.md) | docs | Reservation write path boundary review |
| [v4.6.38](docs/releases/v4.6.38-notes.md) | docs | Note write service Phase N-5 closeout |
| [v4.6.37](docs/releases/v4.6.37-notes.md) | minor | Note write service Phase N-2+N-3 |
| [v4.6.36](docs/releases/v4.6.36-notes.md) | docs | Note write service migration plan |
| [v4.6.35](docs/releases/v4.6.35-notes.md) | docs | Note write path boundary review |
| [v4.6.34](docs/releases/v4.6.34-notes.md) | minor | Expense write adapter cleanup |
| [v4.6.33](docs/releases/v4.6.33-notes.md) | minor | Expense write service Phase W-2+W-3 |
| [v4.6.32](docs/releases/v4.6.32-notes.md) | docs | Expense write service migration plan |
| [v4.6.31](docs/releases/v4.6.31-notes.md) | docs | Expense write path migration plan |
| [v4.6.30](docs/releases/v4.6.30-notes.md) | docs | Expense write path boundary review |
| [v4.6.29](docs/releases/v4.6.29-notes.md) | docs | Itinerary show aggregate migration plan |
| [v4.6.28](docs/releases/v4.6.28-notes.md) | docs | Itinerary show aggregate boundary review |
| [v4.6.27](docs/releases/v4.6.27-notes.md) | docs | Expense output DTO migration follow-up review |
| [v4.6.26](docs/releases/v4.6.26-notes.md) | minor | Expense output DTO migration Phase 2+3 |
| [v4.6.25](docs/releases/v4.6.25-notes.md) | docs | Expense output DTO migration plan |
| [v4.6.24](docs/releases/v4.6.24-notes.md) | docs | Expense DTO context ownership review |
| [v4.6.23](docs/releases/v4.6.23-notes.md) | docs | Read-only helper context review |
| [v4.6.22](docs/releases/v4.6.22-notes.md) | docs | Read-only service boundary completion review |
| [v4.6.21](docs/releases/v4.6.21-notes.md) | minor | Expense show service boundary (`expense show`) |
| [v4.6.20](docs/releases/v4.6.20-notes.md) | minor | Reservation show service boundary (`reservation show`) |
| [v4.6.19](docs/releases/v4.6.19-notes.md) | minor | Day show service boundary (`day show`) |
| [v4.6.18](docs/releases/v4.6.18-notes.md) | minor | Note show service boundary (`note show`) |
| [v4.6.17](docs/releases/v4.6.17-notes.md) | minor | Checklist show service boundary (`checklist show`) |
| [v4.6.16](docs/releases/v4.6.16-notes.md) | docs | Read-only service boundary follow-up review |
| [v4.6.15](docs/releases/v4.6.15-notes.md) | minor | Checklist list service boundary (`checklist list`) |
| [v4.6.14](docs/releases/v4.6.14-notes.md) | minor | Expense list service boundary (`expense list`) |
| [v4.6.13](docs/releases/v4.6.13-notes.md) | minor | Reservation list service boundary (`reservation list`) |
| [v4.6.12](docs/releases/v4.6.12-notes.md) | minor | Note list service boundary (`note list`) |
| [v4.6.11](docs/releases/v4.6.11-notes.md) | docs | Read-only service boundary review |
| [v4.6.10](docs/releases/v4.6.10-notes.md) | minor | Itinerary show service boundary (`itinerary show`) |
| [v4.6.9](docs/releases/v4.6.9-notes.md) | minor | Itinerary timeline service boundary (`itinerary timeline`) |
| [v4.6.8](docs/releases/v4.6.8-notes.md) | minor | Itinerary list service boundary (`itinerary list`) |
| [v4.6.7](docs/releases/v4.6.7-notes.md) | minor | Day list service boundary (`day list`) |
| [v4.6.6](docs/releases/v4.6.6-notes.md) | minor | Trip show service boundary (`trip show`) |
| [v4.6.5](docs/releases/v4.6.5-notes.md) | minor | Read-only service boundary expansion (`trip list`) |
| [v4.6.4](docs/releases/v4.6.4-notes.md) | minor | Read-only service boundary pilot (`trip stats`) |
| [v4.6.3](docs/releases/v4.6.3-notes.md) | docs | Command handler split Phase 1 |
| [v4.6.2](docs/releases/v4.6.2-notes.md) | docs | SQLite migration strategy review |
| [v4.6.1](docs/releases/v4.6.1-notes.md) | docs | SQLite FK / orphan data hardening review |
| [v4.6.0](docs/releases/v4.6.0-notes.md) | minor | TripStats.days semantics fix |
| [v4.5.1](docs/releases/v4.5.1-notes.md) | minor | doctor / advisor Receipt utilization |
| [v4.5.0](docs/releases/v4.5.0-notes.md) | docs | Receipt Inbox responsibilities review |
| [v4.4.8](docs/releases/v4.4.8-notes.md) | patch | Travel Book presentation helper cleanup |
| [v4.4.7](docs/releases/v4.4.7-notes.md) | docs | Travel Book presentation helpers final review |
| [v4.4.6](docs/releases/v4.4.6-notes.md) | patch | Travel Book presentation helpers extraction Phase 3 |
| [v4.1.2](docs/releases/v4.1.2-notes.md) | minor | Okinawa Travel Book sample enrichment |
| [v4.1.1](docs/releases/v4.1.1-notes.md) | docs | Okinawa sample enrichment plan |
| [v4.1.0](docs/releases/v4.1.0-notes.md) | docs | Travel Book chapter structure design |
| [v4.0.0](docs/releases/v4.0.0-notes.md) | docs | Travel Book concept design |
| [v3.11.0](docs/releases/v3.11.0-notes.md) | minor | DB Use implementation |
| [v3.10.0](docs/releases/v3.10.0-notes.md) | docs | DB Use concept design |
| [v3.9.2](docs/releases/v3.9.2-notes.md) | test | Legacy migration test hardening |
| [v3.9.1](docs/releases/v3.9.1-notes.md) | patch | Legacy days summary migration fix |
| [v3.9.0](docs/releases/v3.9.0-notes.md) | minor | Config and DB path foundation |
| [v3.8.0](docs/releases/v3.8.0-notes.md) | docs | Roadmap realignment after Receipt Inbox |
| [v3.7.1](docs/releases/v3.7.1-notes.md) | patch | Okinawa Receipt Inbox sample + trashed Receipt export fix |
| [v3.7.0](docs/releases/v3.7.0-notes.md) | minor | Receipt assignment and trash workflow — `receipt assign` / trash / restore, pending sum, export schema v8 |

## Security

- [Security Policy](SECURITY.md)

## License

MIT — see [Cargo.toml](Cargo.toml)
