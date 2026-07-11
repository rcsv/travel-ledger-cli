# Documentation

Caglla.Travel CLI のドキュメント索引です。GitHub トップの [README.md](../README.md) は製品概要・出力例・安全な Quick Start・インストール手順の入口です。

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
| [specifications/v4.8.7-fragment-apply-structured-errors-public-contract-review.md](specifications/v4.8.7-fragment-apply-structured-errors-public-contract-review.md) | Fragment apply structured errors public contract review（v4.8.7 unreleased） |
| [specifications/v4.8.6-fragment-apply-json-structured-errors-exposure.md](specifications/v4.8.6-fragment-apply-json-structured-errors-exposure.md) | Fragment apply JSON structured_errors exposure（v4.8.6 released） |
| [specifications/v4.8.5-fragment-apply-internal-structured-error-model.md](specifications/v4.8.5-fragment-apply-internal-structured-error-model.md) | Fragment apply internal structured error model（v4.8.5 released） |
| [specifications/v4.8.4-fragment-apply-structured-errors-api-readiness-planning.md](specifications/v4.8.4-fragment-apply-structured-errors-api-readiness-planning.md) | Fragment apply structured errors planning（v4.8.4 released） |
| [specifications/v4.8.3-p6p-delete-estimate-post-release-review.md](specifications/v4.8.3-p6p-delete-estimate-post-release-review.md) | P-6p delete_estimate post-release review（v4.8.3 released） |
| [specifications/v4.8.2-p6p-delete-estimate-confirm.md](specifications/v4.8.2-p6p-delete-estimate-confirm.md) | P-6p delete_estimate --confirm（v4.8.2 released） |
| [specifications/v4.8.1-p6p-delete-estimate-dry-run.md](specifications/v4.8.1-p6p-delete-estimate-dry-run.md) | P-6p delete_estimate dry-run（v4.8.1 released） |
| [specifications/v4.8.0-p6p-delete-estimate-planning.md](specifications/v4.8.0-p6p-delete-estimate-planning.md) | P-6p delete_estimate Proposal Fragment planning（v4.8.0 released） |
| [specifications/v4.7.31-p6j-delete-itinerary-dry-run.md](specifications/v4.7.31-p6j-delete-itinerary-dry-run.md) | P-6j delete_itinerary dry-run planning（Venue 前提補足、v4.7.31–32 実装） |
| [specifications/v4.7.43-fragment-apply-add-estimate-confirm.md](specifications/v4.7.43-fragment-apply-add-estimate-confirm.md) | P-6n add_estimate --confirm |
| [specifications/v4.7.42-fragment-apply-add-estimate-dry-run.md](specifications/v4.7.42-fragment-apply-add-estimate-dry-run.md) | P-6n add_estimate dry-run |
| [specifications/v4.7.41-p6n-add-estimate-planning.md](specifications/v4.7.41-p6n-add-estimate-planning.md) | P-6n add_estimate planning（documentation-only） |
| [specifications/v4.7.40-p6m-itinerary-ordering-move-post-release-review.md](specifications/v4.7.40-p6m-itinerary-ordering-move-post-release-review.md) | P-6m reorder / move post-release review（documentation-only） |
| [specifications/v4.7.34-p6k-reorder-itinerary-planning.md](specifications/v4.7.34-p6k-reorder-itinerary-planning.md) | P-6k reorder_itinerary planning（documentation-only） |
| [specifications/v4.7.37-p6l-cross-day-move-planning.md](specifications/v4.7.37-p6l-cross-day-move-planning.md) | P-6l cross-day itinerary move planning（documentation-only） |
| [specifications/v4.7.30-p6j-destructive-structural-apply-policy.md](specifications/v4.7.30-p6j-destructive-structural-apply-policy.md) | P-6j delete / reorder policy（destructive / structural apply） |

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

関連: [v4.7.0 concept review](specifications/v4.7.0-schema-publication-travel-ledger-public-direction-concept-review.md) / [v4.7.1 public docs outline](specifications/v4.7.1-public-readme-schema-docs-outline.md) / [v4.7.2 Trip Proposal Envelope spec](specifications/v4.7.2-trip-proposal-envelope-concept-spec.md) / [v4.7.3 Proposal Fragment spec](specifications/v4.7.3-proposal-fragment-concept-spec.md) / [v4.7.4 Materialize gate spec](specifications/v4.7.4-materialize-gate-concept-validation-rules.md) / [v4.7.5 Public examples / AI guide spec](specifications/v4.7.5-public-examples-ai-json-generation-guide.md) / [v4.7.6 Public JSON examples spec](specifications/v4.7.6-public-json-examples-concept-stream-post-review.md) / [v4.7.7 Public schema post-review spec](specifications/v4.7.7-public-schema-post-review.md) / [v4.7.8 Proposal implementation planning spec](specifications/v4.7.8-proposal-implementation-planning.md) / [v4.7.9 Proposal Envelope file validation spec](specifications/v4.7.9-proposal-envelope-file-validation.md) / [v4.7.10 Proposal Envelope show / inspect spec](specifications/v4.7.10-proposal-envelope-show-inspect.md) / [v4.7.11 Proposal Fragment file validation spec](specifications/v4.7.11-proposal-fragment-file-validation.md) / [v4.7.12 Public examples validation guard spec](specifications/v4.7.12-public-examples-validation-guard.md) / [v4.7.13 Proposal storage strategy planning spec](specifications/v4.7.13-proposal-storage-strategy-planning.md) / [v4.7.14 Public examples guard CI isolation hotfix spec](specifications/v4.7.14-public-examples-guard-ci-isolation-hotfix.md) / [v4.7.15 Materialize / apply planning spec](specifications/v4.7.15-materialize-apply-planning-spec.md) / [v4.7.16 Proposal materialize dry-run spec](specifications/v4.7.16-proposal-materialize-dry-run.md) / [v4.7.17 Proposal materialize --confirm spec](specifications/v4.7.17-proposal-materialize-confirm.md) / [v4.7.18 Fragment apply dry-run spec](specifications/v4.7.18-fragment-apply-dry-run.md) / [v4.7.19 Fragment apply --confirm spec](specifications/v4.7.19-fragment-apply-confirm.md) / [v4.7.20 P-6 post-implementation review](specifications/v4.7.20-p6-post-implementation-review.md) / [v4.7.21 Fragment apply add_itinerary field expansion](specifications/v4.7.21-fragment-apply-add-itinerary-field-expansion.md) / [v4.7.22 Fragment apply add_note dry-run](specifications/v4.7.22-fragment-apply-add-note-dry-run.md) / [v4.7.23 Fragment apply add_note --confirm](specifications/v4.7.23-fragment-apply-add-note-confirm.md) / [v4.7.24 Fragment apply add_expense dry-run](specifications/v4.7.24-fragment-apply-add-expense-dry-run.md) / [v4.7.25 Fragment apply add_expense --confirm](specifications/v4.7.25-fragment-apply-add-expense-confirm.md) / [v4.7.26 Fragment apply add_reservation dry-run](specifications/v4.7.26-fragment-apply-add-reservation-dry-run.md)

Latest P-6p planning: [v4.8.0 delete_estimate planning](specifications/v4.8.0-p6p-delete-estimate-planning.md)（released）

Latest structured errors public contract: [v4.8.7 public contract review](specifications/v4.8.7-fragment-apply-structured-errors-public-contract-review.md)（unreleased）

Latest structured errors JSON exposure: [v4.8.6 structured_errors[] exposure](specifications/v4.8.6-fragment-apply-json-structured-errors-exposure.md)（released）

Latest structured errors implementation: [v4.8.5 internal structured error model](specifications/v4.8.5-fragment-apply-internal-structured-error-model.md)（released）

Latest structured errors planning: [v4.8.4 Fragment apply structured errors / API readiness](specifications/v4.8.4-fragment-apply-structured-errors-api-readiness-planning.md)（released）

Latest P-6p review: [v4.8.3 delete_estimate post-release review](specifications/v4.8.3-p6p-delete-estimate-post-release-review.md)（released — P-6p series complete）

Latest P-6p confirm: [v4.8.2 delete_estimate --confirm](specifications/v4.8.2-p6p-delete-estimate-confirm.md)（released）

Latest implemented Estimate Fragment: [v4.7.48 update_estimate --confirm](specifications/v4.7.48-p6o-update-estimate-confirm.md)

## Specifications

内部モデル・設計仕様は [specifications/](specifications/) にあります。

| ドキュメント | 内容 |
|---|---|
| [specifications/README.md](specifications/README.md) | 仕様ドキュメントの索引 |
| [specifications/day-model.md](specifications/day-model.md) | Day モデル |
| [specifications/itinerary-model.md](specifications/itinerary-model.md) | Itinerary モデル（not a venue） |
| [specifications/venue-model-introduction-policy.md](specifications/venue-model-introduction-policy.md) | Venue 導入方針（planning — primary venue ref only） |
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
