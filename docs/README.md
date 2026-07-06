# Documentation

Caglla CLI のドキュメント索引です。GitHub トップの [README.md](../README.md) は概要とクイックスタート向けの短い入口です。

## User Guide

| ドキュメント | 内容 |
|---|---|
| [getting-started.md](getting-started.md) | インストール・DB 設定・Quick Start |
| [command-reference.md](command-reference.md) | 全コマンドのオプション・例・出力 |
| [data-model.md](data-model.md) | Trip / Day / Itinerary 階層と設計原則 |
| [export-import.md](export-import.md) | JSON export/import・validate-export・diff・`--json` |
| [markdown-export.md](markdown-export.md) | `trip export-md` による旅行しおり出力 |
| [development.md](development.md) | `make check`・CI・プロジェクト構成・サンプル |
| [github-workflow.md](github-workflow.md) | Issue / Milestone / Project による開発運用 |

## Planning

| ドキュメント | 内容 |
|---|---|
| [long-term-version-strategy.md](long-term-version-strategy.md) | Caglla.Travel メジャーバージョン戦略（v1 Planning Foundation 〜 v9 Platform）— 設計判断用参考 |
| [future-roadmap-planning-memo.md](future-roadmap-planning-memo.md) | v4.6.x 完了後の将来方向性（Travel Data Ledger / Proposal / Calendar / 避けたい方向）— **現行 v4.6.x 作業とは別軸** |

## Public documentation

Travel Ledger の **外向き入口**（schema v8 / 公開方向 / Proposal 概要）:

| ドキュメント | 内容 |
|---|---|
| [public/README.md](public/README.md) | 公開ドキュメントの入口・読み順・責務 |
| [public/travel-ledger.md](public/travel-ledger.md) | Travel Ledger / CLI / future GUI |
| [public/schema.md](public/schema.md) | schema v8（canonical）と schema v3+（歴史） |
| [public/proposals.md](public/proposals.md) | Envelope / Fragment / adoption gate（[v4.7.2](specifications/v4.7.2-trip-proposal-envelope-concept-spec.md) / [v4.7.3](specifications/v4.7.3-proposal-fragment-concept-spec.md) / [v4.7.4](specifications/v4.7.4-materialize-gate-concept-validation-rules.md)） |
| [public/examples.md](public/examples.md) | 最小例・gate 前後・validate-export |
| [public/examples/](public/examples/) | schema v8 Trip JSON files |
| [public/examples-non-normative/](public/examples-non-normative/) | Proposal / Fragment 概念例（non-normative） |
| [public/ai-json-generation-guide.md](public/ai-json-generation-guide.md) | 生成 AI 向け JSON 作法・プロンプト例 |

関連: [v4.7.0 concept review](specifications/v4.7.0-schema-publication-travel-ledger-public-direction-concept-review.md) / [v4.7.1 public docs outline](specifications/v4.7.1-public-readme-schema-docs-outline.md) / [v4.7.2 Trip Proposal Envelope spec](specifications/v4.7.2-trip-proposal-envelope-concept-spec.md) / [v4.7.3 Proposal Fragment spec](specifications/v4.7.3-proposal-fragment-concept-spec.md) / [v4.7.4 Materialize gate spec](specifications/v4.7.4-materialize-gate-concept-validation-rules.md) / [v4.7.5 Public examples / AI guide spec](specifications/v4.7.5-public-examples-ai-json-generation-guide.md) / [v4.7.6 Public JSON examples spec](specifications/v4.7.6-public-json-examples-concept-stream-post-review.md) / [v4.7.7 Public schema post-review spec](specifications/v4.7.7-public-schema-post-review.md) / [v4.7.8 Proposal implementation planning spec](specifications/v4.7.8-proposal-implementation-planning.md) / [v4.7.9 Proposal Envelope file validation spec](specifications/v4.7.9-proposal-envelope-file-validation.md) / [v4.7.10 Proposal Envelope show / inspect spec](specifications/v4.7.10-proposal-envelope-show-inspect.md) / [v4.7.11 Proposal Fragment file validation spec](specifications/v4.7.11-proposal-fragment-file-validation.md) / [v4.7.12 Public examples validation guard spec](specifications/v4.7.12-public-examples-validation-guard.md) / [v4.7.13 Proposal storage strategy planning spec](specifications/v4.7.13-proposal-storage-strategy-planning.md) / [v4.7.14 Public examples guard CI isolation hotfix spec](specifications/v4.7.14-public-examples-guard-ci-isolation-hotfix.md) / [v4.7.15 Materialize / apply planning spec](specifications/v4.7.15-materialize-apply-planning-spec.md) / [v4.7.16 Proposal materialize dry-run spec](specifications/v4.7.16-proposal-materialize-dry-run.md) / [v4.7.17 Proposal materialize --confirm spec](specifications/v4.7.17-proposal-materialize-confirm.md) / [v4.7.18 Fragment apply dry-run spec](specifications/v4.7.18-fragment-apply-dry-run.md) / [v4.7.19 Fragment apply --confirm spec](specifications/v4.7.19-fragment-apply-confirm.md) / [v4.7.20 P-6 post-implementation review](specifications/v4.7.20-p6-post-implementation-review.md) / [v4.7.21 Fragment apply add_itinerary field expansion](specifications/v4.7.21-fragment-apply-add-itinerary-field-expansion.md) / [v4.7.22 Fragment apply add_note dry-run](specifications/v4.7.22-fragment-apply-add-note-dry-run.md) / [v4.7.23 Fragment apply add_note --confirm](specifications/v4.7.23-fragment-apply-add-note-confirm.md) / [v4.7.24 Fragment apply add_expense dry-run](specifications/v4.7.24-fragment-apply-add-expense-dry-run.md)

## Specifications

内部モデル・設計仕様は [specifications/](specifications/) にあります。

| ドキュメント | 内容 |
|---|---|
| [specifications/README.md](specifications/README.md) | 仕様ドキュメントの索引 |
| [specifications/day-model.md](specifications/day-model.md) | Day モデル |
| [specifications/itinerary-model.md](specifications/itinerary-model.md) | Itinerary モデル（not a venue） |
| [specifications/export-schema.md](specifications/export-schema.md) | Export JSON スキーマ |
| [specifications/note-model.md](specifications/note-model.md) | Note モデル |
| [specifications/expense-model.md](specifications/expense-model.md) | Expense モデル |

## Releases

[releases/](releases/) — バージョンごとのリリースノート

## Samples

| サンプル | 内容 |
|---|---|
| [samples/okinawa_sesoko_2026/](../samples/okinawa_sesoko_2026/README.md) | 行動台帳 canonical sample |
| [samples/markdown_sample_commands.sh](../samples/markdown_sample_commands.sh) | Markdown Export 確認用データ |
| [samples/trip_doctor/](../samples/trip_doctor/) | trip doctor 検証用出力 |
| [samples/advisor/](../samples/advisor/) | trip advisor 検証用出力 |
