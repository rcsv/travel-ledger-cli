# Development

Caglla CLI のローカル開発・CI・サンプルデータの手順です。

## 品質チェック（make check）

```bash
make check
```

内部では `cargo fmt --check` → `cargo clippy -- -D warnings` → `cargo test` → `cargo build` を順に実行します。ローカル開発ではこのコマンドを推奨します。

| コマンド | 内容 |
|---|---|
| `make check` | fmt + clippy + test + build |
| `make test` | テストのみ実行 |
| `make run` | `cargo run` を実行 |
| `make clean` | ビルド成果物を削除 |

## GitHub Actions（CI）

`master` への push と pull request で [`.github/workflows/rust.yml`](../.github/workflows/rust.yml) が実行され、以下を確認します。

| チェック | 内容 |
|---|---|
| formatting | `cargo fmt -- --check` |
| clippy | `cargo clippy -- -D warnings` |
| tests | `cargo test` |
| build | `cargo build` |

リリース前後の確認手順は [`docs/releases/README.md`](releases/README.md#release-verification) を参照してください。

## GitHub 開発運用

Issue / PR / Milestone / Project を使った設計・実装・リリースの流れは [`docs/github-workflow.md`](github-workflow.md) を参照してください。

## プロジェクト構成

```
travel-ledger-cli/
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
│   ├── advisor/                     # trip advisor 検証用サンプル
│   └── okinawa_sesoko_2026/         # 行動台帳 canonical sample
├── docs/
│   ├── getting-started.md
│   ├── command-reference.md
│   ├── data-model.md
│   ├── export-import.md
│   ├── markdown-export.md
│   ├── development.md
│   ├── releases/                    # GitHub Release 用ノート
│   └── specifications/              # 仕様メモ
├── Cargo.toml
├── Makefile
├── caglla.db         # ローカル DB（実行時に自動作成、git 管理外）
└── README.md
```

## サンプルデータ

### Markdown Export 確認用

[markdown-export.md](markdown-export.md#確認用サンプル) を参照してください。

### trip doctor

検証用の実出力サンプルは [`samples/trip_doctor/`](../samples/trip_doctor/) を参照してください。再生成:

```bash
bash samples/trip_doctor/generate_outputs.sh
```

### trip advisor

検証用の実出力サンプルは [`samples/advisor/`](../samples/advisor/) を参照してください。再生成:

```bash
bash samples/advisor/generate_outputs.sh
bash samples/advisor/generate_outputs_with_commands.sh
```

### 行動台帳 canonical sample

実旅行由来の **行動台帳** canonical sample（沖縄・瀬底 2026：高速道路・給油・チェックイン・買い出しなども Itinerary として表現、58 Itinerary / 49 Expense）は [`samples/okinawa_sesoko_2026/`](../samples/okinawa_sesoko_2026/README.md) を参照してください。
