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
