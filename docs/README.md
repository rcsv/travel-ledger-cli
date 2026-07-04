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
| [public/proposals.md](public/proposals.md) | Proposal / materialize 概要（v4.7.2+ で詳細化） |

関連: [v4.7.0 concept review](specifications/v4.7.0-schema-publication-travel-ledger-public-direction-concept-review.md) / [v4.7.1 public docs outline](specifications/v4.7.1-public-readme-schema-docs-outline.md)

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
