# Markdown Export

旅行計画を Markdown 形式の「旅行しおり」として出力します。

```bash
cargo run -- trip export-md 1
```

## 出力例

```md
# 沖縄旅行

2026-04-26 〜 2026-04-29

## Overview

- Days: 4
- Itineraries: 15
- Checklist: 4 / 10 completed
- Stay Time: 22h15m
- Travel Time: 6h50m
- Total Time: 29h05m

## Day 1

### 09:00 那覇空港

- Category: transport
- 場所: 那覇空港
- 所要時間: 60分
- 移動時間: 30分
- メモ: レンタカー受け取り

### 12:30 昼食

- Category: restaurant
- 場所: 国際通り
- 所要時間: 60分

Expenses:
- 入館料: 2,500 JPY
- 駐車場: 500 JPY
```

Expense がある Itinerary のみ、Itinerary ブロックの下に `Expenses:` 一覧を出力します（データ確認用。表組みや PDF 向けの整形はしません）。`location` がある場合のみ `- 場所:` 行が付きます。

Itinerary は **日目 → 並び順（sort_order）** の順で出力されます。日程が登録されていない日目は表示されません。冒頭の **Overview** セクションには `trip stats` と同様の集計サマリー（日数・件数・チェックリスト進捗・時間集計）が含まれます。Category Breakdown は含みません。各 Day 見出し・予定ブロック・Checklist セクションの前後には空行が入り、読みやすさを優先しています。

チェックリストが登録されている場合、末尾に以下の形式で出力されます。

```md
## Checklist

- [ ] パスポート
- [x] 充電器
```

チェックリストがない場合は `## Checklist` セクション自体を出力しません。

## 出力先

### 標準出力（デフォルト）

`--output` を省略すると、Markdown 本体のみ stdout に出力されます。

```bash
cargo run -- trip export-md 1
```

シェルのリダイレクトでも保存できます。

```bash
cargo run -- trip export-md 1 > trip.md
```

### ファイル出力（`--output`）

`--output` を指定すると、指定ファイルへ保存します（既存ファイルは確認なしで上書き）。

```bash
cargo run -- trip export-md 1 --output trip.md
```

成功時の表示例:

```text
Markdown exported: trip.md
```

## 確認用サンプル

`trip export-md` / `trip stats` の見た目確認用に、4日間・Itinerary 15件・チェックリスト10件のサンプルデータを一括投入できます。

```bash
bash samples/markdown_sample_commands.sh
```

投入内容の概要:

| 項目 | 内容 |
|---|---|
| 旅行 | Okinawa Sample Trip（2026-04-26 〜 2026-04-29） |
| Itinerary | 15件（flight / hotel / restaurant / activity / transport / beach / shopping + uncategorized 1件） |
| チェックリスト | 10件（うち4件を完了済みに設定） |

確認コマンド:

```bash
cargo run -- trip stats 1
cargo run -- trip export-md 1
cargo run -- trip export-md 1 --output sample-trip.md
```

スクリプト本体は [`samples/markdown_sample_commands.sh`](../samples/markdown_sample_commands.sh) です。
