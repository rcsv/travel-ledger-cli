# 沖縄・瀬底 2026 canonical sample (v0)

実旅行データ（`EstimateTrip_20260426.pdf`）由来の canonical sample です。

**このサンプルは観光地一覧ではありません。** 高速道路・駐車場・空港内買い物・レンタカー・給油・買い出し・チェックイン/アウト・個人負担・領収書番号まで含む **実旅行の行動台帳** を、現行 CLI モデルで表現する検証用データです。

Itinerary は **not a venue** — 場所（POI）ではなく、旅行中に順序を持つ **行動** を表す最小単位です。計画時は予定、旅行後は実績として扱えます。出発・高速道路・給油・部屋食・レンタカー返却・帰宅なども Itinerary として登録します。理由は、**費用（Expense）・備考・チェックリスト・将来の Note / Photo** を、時系列上の行動に結びつけるためです。

設計原則の詳細: [`docs/specifications/itinerary-model.md`](../../docs/specifications/itinerary-model.md) / [`ordering-model.md`](../../docs/specifications/ordering-model.md)

| 項目 | 値 |
|---|---|
| Trip | 沖縄 瀬底 4日間 |
| 期間 | 2026-04-26 〜 2026-04-29 |
| Itinerary | 58 件 |
| Expense | 49 件 |
| 合計（JPY） | ¥561,780（PDF 会計合計と一致） |
| Checklist | 4 件 |

## なぜ観光地以外も Itinerary か

PDF / Excel 台帳では、スケジュール行と会計行が混在します。CLI では次のように分けます。

| 台帳上の性質 | CLI |
|---|---|
| いつ・何をするか（出発、高速道路、チェックイン、買い出し、給油、返却、帰宅…） | **Itinerary**（行動単位） |
| 金額・領収書・個人負担 | **Expense**（対応 Itinerary 配下） |

Itinerary 例（すべて `place_id` 不要、`location` は任意）:

- `出発` / `高速道路 東浦→セントレア` / `P1 G Parking`
- `チェックイン` / `NU045 NGO ⇒ OKA`
- `夕食の買い出し` / `夕食 部屋`（location なし可）
- `フェリー乗船` / `ガソリン満タン返し` / `レンタカー返却` / `帰宅`

**1 行 = 1 Itinerary ではありません。** 同一の買い物・食事に複数レシートがある場合は、1 Itinerary に複数 Expense をぶら下げます（例: Day 4 `07:50 土産屋さん` → Expense 4 件）。

## 投入

リポジトリルートから:

```bash
bash samples/okinawa_sesoko_2026/seed.sh
```

`caglla.db` をリセットしたうえで Trip ID `1` を作成します（約 1〜2 分）。

## 確認コマンド

```bash
cargo run -- trip stats 1
cargo run -- trip export-md 1
cargo run -- trip export 1 --output /tmp/okinawa-export.json
cargo run -- trip validate-export /tmp/okinawa-export.json
cargo run -- trip import /tmp/okinawa-export.json
```

## seed 化ルール（要約）

### Itinerary

- PDF の **スケジュール行** を原則 Itinerary として登録（観光地に限らない）
- 時刻あり → `--time`、なし → `sort_order` のみ（**両方指定可**。v1.9.0 以降、一覧・export は **Sequence-first** — [ordering-model.md](../../docs/specifications/ordering-model.md)）
- `--location` は任意（区間名・施設名・自宅など）
- 買い物の追加購入・レシート分割行などは **同一 Itinerary に Expense を追加**（例: 昼食追加、土産屋さんの複数レシート）

### Expense

- 金額がある行は対応 Itinerary 配下に登録
- `旅費` / `食費` / `個別：…` は `note` に保持
- `個別：知弘` / `個別：節子` は `paid_by_name` にも反映
- 領収書 `R-xxx` は `note` に `領収書: R-xxx` 形式で保持

### 意図的に省略したもの

- 金額なし行（ロイズ R-033、有料道路「700円ぐらい？」など）
- Note エンティティの大量投入（備考は Itinerary / Expense `note` に集約）
- Participant / Settlement / Expense category

## ファイル

| ファイル | 説明 |
|---|---|
| `seed.sh` | CLI でデータ投入 |
| `expected-export-v3.json` | seed 後 export の正規化 golden file（metadata 除く） |

### golden file の再生成

```bash
bash samples/okinawa_sesoko_2026/seed.sh
cargo run -- trip export 1 --output /tmp/okinawa-export.json
jq '{
  schema_version: .schema_version,
  trip: (.trip | {name, start_date, end_date}),
  days: .days,
  checklist_items: [.checklist_items[] | {title, is_done, sort_order}],
  notes: .notes
}' /tmp/okinawa-export.json > samples/okinawa_sesoko_2026/expected-export-v3.json
```

## 検証

`tests/okinawa_sesoko_seed_cli.rs` が seed → export → golden 比較と `validate-export` を実行します。
