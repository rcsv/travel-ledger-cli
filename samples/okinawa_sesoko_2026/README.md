# 沖縄・瀬底 2026 canonical sample (v1)

実旅行データ（`EstimateTrip_20260426.pdf`）由来の canonical sample です。

**このサンプルは観光地一覧ではありません。** 高速道路・駐車場・空港内買い物・レンタカー・給油・買い出し・チェックイン/アウト・個人負担・領収書番号まで含む **実旅行の行動台帳** を、現行 CLI モデルで表現する検証用データです。v4.1.2 以降は同じ seed が **Travel Book（旅行前しおり）fixture** としても使えます。

Itinerary は **not a venue** — 場所（POI）ではなく、旅行中に順序を持つ **行動** を表す最小単位です。計画時は予定、旅行後は実績として扱えます。出発・高速道路・給油・部屋食・レンタカー返却・帰宅なども Itinerary として登録します。理由は、**費用（Expense）・備考・チェックリスト・Note** を、時系列上の行動に結びつけるためです。

設計原則の詳細: [`docs/specifications/itinerary-model.md`](../../docs/specifications/itinerary-model.md) / [`ordering-model.md`](../../docs/specifications/ordering-model.md) / [`travel-ledger-responsibilities.md`](../../docs/specifications/travel-ledger-responsibilities.md)

| 項目 | 値 |
|---|---|
| Trip | 沖縄 瀬底 4日間 |
| 期間 | 2026-04-26 〜 2026-04-29 |
| Itinerary | 58 件 |
| Expense | 49 件 |
| 合計（JPY） | ¥561,780（PDF 会計合計と一致） |
| Receipt Inbox（active） | 5 件（Trash 1 件は別途。いずれも **Actual ではない**） |
| Pending Receipt sum（JPY） | ¥15,980（active の金額付き Receipt のみ。`trip stats` には含まれない） |
| Checklist | 4 件 |
| Trip / Day Summary | 1 + 4（Travel Book fixture） |
| Note（entity） | Trip 2 + Day 4 |
| Reservation | 5 |
| Estimate | 8（Planned cost fixture） |

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

## Receipt Inbox（v3.7.0 workflow）

旅行中は土産物屋や売店のレシートがいくつか発生するが、細かく整理する余裕がない。まず **Receipt Inbox** に「未整理の支払い候補」として登録し、帰宅後に見返して整理する。

```text
Receipt = 未整理の支払い候補
Receipt のままでは Actual ではない
Pending Receipt sum も Actual ではない
assign 後に作成された Expense だけが Actual に入る
```

| Receipt（seed 投入後） | Day | 金額 | 帰宅後の整理イメージ |
|---|---|---|---|
| 美ら海水族館ショップ | 2 | ¥4,860 | Itinerary `水族館に入館`（ID 19）へ **assign** |
| ハナサキマルシェ | 2 | ¥2,340 | Itinerary `スタバる`（ID 26）などへ **assign** |
| 道の駅 許田 | — | ¥3,180 | 立ち寄り Itinerary を後で判断 |
| 那覇空港売店 | 4 | ¥5,600 | 旅行費用に含めるか迷う候補 |
| コンビニのレシート | 4 | 金額未入力 | 金額確認後に **assign** |
| 個人的な雑貨購入 | 3 | ¥1,200 | 旅行共通費にしない → **trash** 済み（seed 内） |

seed では Receipt を Expense 化していないため、`trip stats` / `trip export-md` の Planned / Actual / Difference は **従来どおり Expense のみ**（¥561,780）です。

### Receipt 確認・整理コマンド（seed 後）

```bash
# Pending Receipt summary（Actual ではない）
cargo run -- receipt list --trip 1

# 旅行費用として計上するものを Expense 化（Receipt は削除される）
cargo run -- receipt assign 1 --itinerary 19   # 美ら海水族館ショップ → 水族館に入館
cargo run -- receipt assign 2 --itinerary 26   # ハナサキマルシェ → スタバる

# 旅行共通費にしないものは Trash へ（個人雑貨は seed 内で ID 6 を trash 済み）
cargo run -- receipt trash 4
cargo run -- receipt restore 6
cargo run -- receipt list --trip 1 --trashed
```

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
- `個別：Alex` / `個別：Jordan` は `paid_by_name` にも反映
- 領収書 `R-xxx` は `note` に `領収書: R-xxx` 形式で保持

### 意図的に省略・制限したもの

- 金額なし行（ロイズ SAMPLE-RECEIPT-033、有料道路「700円ぐらい？」など）
- Itinerary **Remark** / Expense `note` 以外の長文は Note entity に分離（v4.1.2 で Trip/Day Note を少数追加）
- Participant / Settlement / Expense category
- Evidence / Attachment / `image_path` / OCR
- Receipt Inbox の変更（Travel Book には出さない — 未整理候補のまま）

Receipt Inbox は **未整理候補のみ** 投入。assign 前は Expense にしない（Actual に混ぜない）。

## 二重役割（v4.1.2）

| 役割 | 内容 |
|---|---|
| **行動台帳・会計** | Itinerary 58 / Expense 49 / Receipt Inbox — 件数・Actual 合計は不変 |
| **Travel Book fixture** | Summary / Note / Reservation / Estimate — [v4.1.1 plan](../../docs/specifications/v4.1.1-okinawa-travel-book-sample-enrichment-plan.md) / [v4.1.2 implementation](../../docs/specifications/v4.1.2-okinawa-travel-book-sample-enrichment-implementation-plan.md) |

Estimate は **Planned cost 表示検証用 fixture**（当初予定の主張ではない）。Reservation の confirmation は台帳根拠があるもののみ（P1 G `EXAMPLE-PARKING-001`）。

v4.2.2 以降、Reservation の remark と Estimate の note は **Travel Book 本文向けのユーザー文言** に差し替え済み。fixture / canonical sample / ledger 由来の内部説明は本 README に残し、`trip export-md` 本文には出しません。

## ファイル

| ファイル | 説明 |
|---|---|
| `seed.sh` | CLI でデータ投入（v1） |
| `regenerate-golden.sh` | seed 後 golden 再生成 |
| `expected-export-v3.json` | seed 後 export の正規化 golden file（metadata 除く） |
| `expected-export-md.md` | seed 後 `trip export-md` の golden（`Generated at` はテストで正規化） |

Travel Book 出力の post-release review: [v4.2.1-travel-book-export-md-post-release-review.md](../../docs/specifications/v4.2.1-travel-book-export-md-post-release-review.md) / [v4.3.1-reservation-summary-display-post-release-review.md](../../docs/specifications/v4.3.1-reservation-summary-display-post-release-review.md)

Travel Book Markdown polish（v4.2.2）: [v4.2.2-travel-book-markdown-polish-implementation-plan.md](../../docs/specifications/v4.2.2-travel-book-markdown-polish-implementation-plan.md)

### golden file の再生成

```bash
bash samples/okinawa_sesoko_2026/regenerate-golden.sh
```

または手動:

```bash
bash samples/okinawa_sesoko_2026/seed.sh
cargo run -- trip export 1 --output /tmp/okinawa-export.json
jq '{
  schema_version: .schema_version,
  trip: (.trip | {name, start_date, end_date, summary}),
  days: .days,
  checklist_items: [.checklist_items[] | {title, is_done, sort_order}],
  notes: .notes,
  participants: (.participants // []),
  receipts: [
    .receipts[]?
    | {
        day_ref,
        amount,
        currency,
        memo,
        status,
        trashed: (.trashed_at != null)
      }
  ] | sort_by(.memo)
}' /tmp/okinawa-export.json > samples/okinawa_sesoko_2026/expected-export-v3.json
```

## 検証

`tests/okinawa_sesoko_seed_cli.rs` が seed → export → golden 比較と `validate-export` を実行します。
