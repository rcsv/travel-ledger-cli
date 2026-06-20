# Itinerary モデル

Caglla CLI における **Itinerary（予定）** の責務とフィールド定義です。  
v1.8.0 で **CLI 側のモデルを正として明文化** します。実装は v1.0.x〜v1.7.0 で既に本仕様に沿って動作しています。

関連: [Day モデル](day-model.md) / [Expense モデル](expense-model.md) / [Estimate モデル](estimate-model.md) / [Note モデル](note-model.md) / [Export Schema](export-schema.md) / [Ordering モデル](ordering-model.md) / [Planning Design Principles](planning-design-principles.md)

検証データ: [沖縄・瀬底 canonical sample](../../samples/okinawa_sesoko_2026/README.md)

---

## 1. 設計原則

横断的な判断軸（Itinerary と Checklist の `is_done` の有無、Note の柔軟性、Reservation / Expense の複数紐づき）は [planning-design-principles.md](planning-design-principles.md) を参照。

### Itinerary is not a venue

**Itinerary は場所（Venue / POI）ではない。**

観光地・施設・POI の管理単位ではなく、旅行者の **行動** を表す最小単位です。

### Itinerary is a unit of travel activity

**Itinerary は旅行中の行動を表す最小単位である。** 計画時には **予定** として、旅行後には **実績** として扱える。

- ユーザーが記録・計画するのは「どこか」だけでなく、「いつ・何をするか」という **時系列上の行動**
- 観光地・レストラン・ホテルなどの POI は、行動の **説明や文脈** になりうるが、Itinerary の本質ではない
- `title`（何をするか）と親 Day（いつ）があれば Itinerary として成立する

### Venue / Place is optional metadata

**場所情報は Itinerary の任意属性である。**

- `location` は自由文字列（住所・施設名・区間名など）で、**任意**
- Google Place / POI ID などの外部参照は **現行 CLI にはない**（将来、任意の補助メタデータとして追加可能）
- 場所が特定できない・特定する必要がない行動（「出発」「部屋で夕食」「帰宅」など）も Itinerary として登録する

### 費用は Itinerary 配下の Estimate と Expense

Itinerary 自身は金額を持たない。

| 種別 | エンティティ | 意味 |
|---|---|---|
| **事前見積** | **Estimate**（未実装） | Planned Money — 旅行前の見込み金額。[estimate-model.md](estimate-model.md) |
| **実績支出** | **Expense** | Actual Money — 支払った金額。[expense-model.md](expense-model.md) |

**1 Itinerary に複数 Estimate / 複数 Expense** は自然 — [planning-design-principles.md §6](planning-design-principles.md#6-multiple-expenses-under-one-itinerary--natural)（Expense について記載。Estimate も同型）。

### Itinerary is not a task row

Itinerary に **`is_done` は持たない**（意図的）。旅行行動の流れを表し、タスク消化の行ではない — [planning-design-principles.md §2](planning-design-principles.md#2-itinerary-is-not-a-task-row)。確認したい事項は **Checklist**（`is_done` あり）に置く。

---

## 2. エンティティ関係

```text
Trip
 └─ Day (day_number, title, description)
      └─ Itinerary (itinerary_items)
           ├─ Estimate（将来 — [estimate-model.md](estimate-model.md)）
           ├─ Expense
           ├─ Reservation（Itinerary 配下）
           ├─ Note（エンティティ）
           └─ itinerary_items.note（短いメモ列）
```

| エンティティ | 役割 |
|---|---|
| **Trip** | 旅行全体 |
| **Day** | 旅行内の「何日目か」。カレンダー日付は Trip から導出 |
| **Itinerary** | Day 内の **行動**（予定／実績）。順序・時刻・備考・分類を持つ |
| **Estimate** | Itinerary に紐づく **事前見積**（Planned Money — **未実装**） |
| **Expense** | Itinerary に紐づく **実績支出**（Actual Money） |

Itinerary の正規の親は DB 上 `day_id`（`days.id`）。CLI / export では `day`（= `day_number`）を使用する（[Day モデル](day-model.md) 参照）。

---

## 3. Itinerary として成立するもの

### 必須

| 条件 | 説明 |
|---|---|
| 親 Day | `--day N`（Trip 内の N 日目）。内部で `day_id` に解決 |
| `title` | 行動の要約（例: `出発`、`高速道路 東浦→セントレア`、`部屋で夕食`） |

### 任意

`start_time`, `sort_order`, `note`, `location`, `category`, `duration_minutes`, `travel_minutes` および配下の Expense / Note。

**場所（`location`）・POI・金額は必須ではない。**

### 沖縄・瀬底 canonical sample における代表例

実旅行台帳（`EstimateTrip_20260426.pdf`）を CLI モデルで表現した [canonical sample](../../samples/okinawa_sesoko_2026/) では、次のような項目が Itinerary として登録されています。いずれも **POI ではない場合がある** が、旅行の順序を持った **行動** であり、費用・備考・チェックリスト・将来の Note / Photo を結びつける単位として価値があります。

| 行動 | canonical sample での例 | 場所の扱い | category（補助） |
|---|---|---|---|
| 出発 | Day 1 `06:00` 出発、せっちゃんやっちゃんピックアップ | `location`: 粟根家 | `transport` |
| 高速道路 | `高速道路 東浦→セントレア` | 区間名を `location` に | `transport` |
| 駐車場 | `P1 G Parking` | 施設名 | `transport` |
| 空港チェックイン | `チェックイン`（出発・帰路とも） | 空港名 | `flight` |
| フライト | `NU045 NGO ⇒ OKA` | 空港 | `flight` |
| レンタカー受取 | `Toyota Alphard 又は同等車種` | レンタカー会社 | `transport` |
| 買い出し | `夕食の買い出し` | スーパー名 | `shopping` |
| 部屋で夕食 | `夕食 部屋` | **location なし可** | `restaurant` |
| ホテルチェックイン | `チェックイン`（ヒルトン瀬底） | ホテル名 | `hotel` |
| ホテルチェックアウト | `出発`（チェックアウトリミット記載を `note` に） | ホテル名 | `hotel` |
| フェリー乗船 | `フェリー乗船` | ターミナル名 | `transport` |
| 給油 | `ガソリン満タン返し` | スタンド名 | `transport` |
| レンタカー返却 | `レンタカー返却` | 返却場所 | `transport` |
| 帰宅 | Day 4 `23:30` `帰宅` | `location`: 自宅 | `transport` |

これらは **「場所を先に決めてから予定を作る」** のではなく、**「旅行の行動を時系列で計画・記録する」** という入力順序で自然に登録できます。

---

## 4. PDF / Excel 行と Itinerary の対応

旅行会計の PDF や Excel 台帳では、**1 行が必ず 1 Itinerary ではありません。**

### 原則

| 台帳の性質 | CLI での置き方 |
|---|---|
| スケジュール行（いつ・何をしたか） | **Itinerary** |
| 金額行（レシート・領収書・個人負担） | **Expense**（対応する Itinerary 配下） |
| 金額なしの備考行 | Itinerary `note`、Expense `note`、または省略 |

canonical sample の seed 化ルール（[README](../../samples/okinawa_sesoko_2026/README.md)）:

- PDF の **スケジュール行** を原則 Itinerary として登録
- **金額がある行** は対応 Itinerary 配下に Expense として登録
- 買い物の追加購入・レシート分割行などは **同一 Itinerary に Expense を追加**

### 例: 1 Itinerary に複数 Expense（土産屋さん）

同一時刻・同一行動に対して複数レシートや複数費用がある場合、**1 Itinerary に複数 Expense** をぶら下げます。

```text
Itinerary (Day 4, 07:50):
  土産屋さん

Expenses:
  - 土産屋さん（個別：節子）  R-005  ¥3,700
  - 土産屋さん（個別：知弘）  R-021  ¥8,380
  - SAGAWA BOX（個別：知弘）  R-001  ¥3,200
  - 宅配便（個別：知弘）      R-001  ¥286
```

昼食で追加注文があった場合も同様です（`朝食 スタンダードコーヒー` に本体 Expense + `ジュース買い足し` Expense）。

### 意図的に Itinerary にしないもの

canonical sample では次を省略しています（Itinerary / Expense どちらにもしない、または note のみ）:

- 金額なし行（例: ロイズ R-033）
- 概算のみの備考（例: 有料道路「700円ぐらい？」— Itinerary `note` に記載、Expense なし）

---

## 5. コアフィールド

### `ItineraryItem`（`src/models.rs`）

| フィールド | 必須 | 説明 |
|---|---|---|
| `id` | ✓（DB） | 内部 ID |
| `trip_id` | ✓ | 所属 Trip |
| `day` | ✓ | `day_number`（表示・CLI 用） |
| `title` | ✓ | 行動の要約 |
| `note` | — | 短い補足（1 フィールド） |
| `start_time` | — | 開始時刻 `HH:MM` |
| `sort_order` | ✓ | Day 内の行動順序（主情報）。`--order` で明示指定可。未指定時は Day 末尾へ sparse ordering（1000 刻み）で自動採番 |
| `duration_minutes` | — | 滞在・実施の目安時間（分） |
| `travel_minutes` | — | 次の予定までの移動時間の目安（分） |
| `location` | — | 場所の自由記述 |
| `category` | — | 補助分類（後述） |
| `created_at` / `updated_at` | ✓ | タイムスタンプ |

DB 上は `day_id`（`days.id` への FK）を正規の親参照として保持する。`day` 列は export / 互換用に `day_number` を同期保持する。

### CLI 作成の最小例

```bash
# 場所・時刻なしでも作成可能
caglla itinerary add 1 --day 1 "部屋で夕食"

# 時刻・場所・順序は任意で付与
caglla itinerary add 1 --day 1 --time 06:00 --order 1 --location "粟根家" \
  "出発、せっちゃんやっちゃんピックアップ"
```

---

## 6. 場所（`location`）

`location` は **任意の文字列** です。

- 施設名（`ヒルトン瀬底`）、住所（`自宅`）、区間（`東浦→セントレア`）など、ユーザーの入力意図をそのまま保持する
- POI データベースや地図サービスへの参照は **必須ではない**
- 将来、外部 Place ID を任意メタデータとして追加しても、Itinerary の成立条件には含めない

移動区間（高速道路）や自宅など、固定 POI に紐づけにくい行動こそ、`location` を補助的に使う典型例です。

---

## 7. 時刻と並び順

Itinerary の ordering 責務（Sequence-first、各 CLI 出力の統一）は **[Ordering モデル](ordering-model.md)** を正とします。以下は概要のみ。

### 設計原則

| フィールド | 位置づけ |
|---|---|
| **`sort_order`** | Day 内の **行動順序の主情報**（sequence） |
| **`start_time`** | 行動に付随する **任意の時刻ラベル** |

Caglla.Travel は Calendar Event ではなく Travel Activity Unit を扱うため、**Sequence-first** を採用しています。詳細は [ordering-model.md §5](ordering-model.md#5-現行実装) を参照。

### `start_time`

- 形式: `HH:MM`（任意）
- ある場合は Markdown 見出し・timeline 表示・時間集計に使える
- **なくても Itinerary は成立する**（canonical sample でも時刻なしの行がある）
- **順序の主決定因子にしない**

### `sort_order`

- 同一 Day 内での **明示的な行動順序**（主情報）
- `--order`（CLI）で明示指定できる
- 未指定時は対象 Day の末尾へ **sparse ordering** により自動採番される（標準間隔は **1000**）
- `--after` / `--before` 指定時は参照 Itinerary の直後 / 直前へ挿入
- **一覧順の主キー**（`sort_order` → `id`）

### 現行実装

| 出力 | 並び |
|---|---|
| `itinerary list` / `timeline` / `day show` | **Sequence-first**（`sort_order` → `id`） |
| `trip export` v3 | **Sequence-first**（Day 内） |
| `trip export-md` | **Sequence-first**（list と同一） |

詳細は [ordering-model.md §5](ordering-model.md#5-現行実装) を参照。

### `duration_minutes` / `travel_minutes`

- **任意** の目安値
- 移動そのものを別 Itinerary（例: `高速道路 東浦→セントレア`）として表現する運用と併用可能
- `travel_minutes` は「次の予定までの移動」のメモとして使えるが、canonical sample では主に **移動 Itinerary 行** で表現している

---

## 8. 費用（Expense との関係）

Itinerary は **金額フィールドを持たない**。支出はすべて Expense として Itinerary 配下に登録する。

```bash
caglla expense add --itinerary 39 --amount 3700 --currency JPY \
  --title "土産屋さん" --paid-by-name "節子" \
  --note "費用区分: 個別：節子 / 領収書: R-005"
```

| 観点 | 方針 |
|---|---|
| 親子関係 | Expense の親は **Itinerary のみ**（Trip / Day 直下は v1.x では設けない） |
| 複数件 | 1 Itinerary に **複数 Expense 可** |
| 集計 | `trip stats` で Trip 単位の Expense 件数・通貨別合計を表示 |
| Export | schema v3 で `days[].itineraries[].expenses[]` にネスト |

Itinerary に紐づけにくい支出は、ユーザーが明示的に作った Itinerary（例: 「その他経費」）に載せる運用を許容する。**ダミー Itinerary の自動作成は行わない**。

---

## 9. 説明とメモ（Remark / Note / Summary）

| 手段 | GUI ラベル | 用途 |
|---|---|---|
| `itinerary_items.note` | **備考** | 予定 1 件への **短い補足**（Remark） |
| **Note エンティティ** | **メモ** | **長文・複数件** の自由記述 |
| Trip / Day Summary | **概要** | 共有・印刷向け要約（**将来**） |

責務の全体整理: [travel-ledger-responsibilities.md](travel-ledger-responsibilities.md)。**Summary**（将来）: [summary-responsibilities-review.md](summary-responsibilities-review.md)。**Reservation**（将来）: [reservation-model.md](reservation-model.md)（責務）、[reservation-entity-design.md](reservation-entity-design.md)（フィールド設計）、[reservation-implementation-plan.md](reservation-implementation-plan.md)（実装計画）。Note CRUD の詳細: [note-model.md](note-model.md)。

v1.x では Remark と Note entity を **併存** させる。canonical sample では備考の多くを Itinerary `note` / Expense `note` に集約し、Note エンティティの大量投入は省略している（旅行前しおり向けの Summary / Reservation は将来拡張の動機として仕様に記載）。

---

## 10. カテゴリ（`category`）

`category` は Itinerary を **表示・集計・チェックリスト生成** するための **補助分類** です。Itinerary の成立条件ではない。

### 定義済み値（8 種）

| 値 | 表示名 | 用途の例 |
|---|---|---|
| `flight` | フライト | 空港チェックイン、搭乗 |
| `hotel` | ホテル | チェックイン、アウト |
| `restaurant` | 食事 | レストラン、部屋食 |
| `activity` | アクティビティ | 観光・体験 |
| `transport` | 移動 | 出発、高速道路、駐車場、フェリー、給油、帰宅 |
| `shopping` | 買い物 | 買い出し、土産 |
| `beach` | ビーチ | ビーチ訪問 |
| `museum` | 博物館・展示 | 博物館・水族館など |

```bash
caglla itinerary update 2 --category transport
```

### チェックリストとの関係

`ItineraryCategory` にはカテゴリごとの **標準チェックリスト候補** が定義されている（`src/models.rs` の `CategoryDefinition`）。`ChecklistRule` により、Trip 内のカテゴリ組み合わせに応じたチェックリスト追加も行える。

カテゴリは **あとから付与・変更可能**。作成時に未指定でもよい。

自動生成の設計判断: [checklist-design-memo.md](checklist-design-memo.md)。旅行支援・注意喚起の方向性: [travel-support-design-memo.md](travel-support-design-memo.md)（いずれも Web 版知見、v1.x 対象外）。

---

## 11. CLI コマンド（現行）

| コマンド | 説明 |
|---|---|
| `itinerary add <trip_id> --day N <title>` | 作成（`--time`, `--order`, `--location`, `--note` 任意） |
| `itinerary list <trip_id>` | 一覧 |
| `itinerary show <id>` | 詳細 |
| `itinerary update <id>` | 更新（`--category`, `--time`, `--order` 等） |
| `itinerary delete <id>` | 削除 |

Day 操作は [Day モデル](day-model.md) の `day list` / `day show` / `day swap` を参照。

---

## 12. DB スキーマ（抜粋）

```sql
CREATE TABLE itinerary_items (
    id               INTEGER PRIMARY KEY AUTOINCREMENT,
    trip_id          INTEGER NOT NULL,
    day_id           INTEGER REFERENCES days(id),
    day              INTEGER NOT NULL,
    title            TEXT NOT NULL,
    note             TEXT,
    start_time       TEXT,
    sort_order       INTEGER NOT NULL DEFAULT 0,
    duration_minutes INTEGER,
    travel_minutes   INTEGER,
    location         TEXT,
    category         TEXT,
    created_at       TEXT NOT NULL,
    updated_at       TEXT NOT NULL,
    FOREIGN KEY(trip_id) REFERENCES trips(id) ON DELETE CASCADE
);
```

`place_id` 列は **存在しない**（意図的）。

---

## 13. Export / Import（schema v3）

現行 export は `schema_version: 3`。Itinerary は `days[].itineraries[]` にネストする。**JSON フィールド定義・検証ルール** は [Export Schema](export-schema.md) を正とし、本ドキュメントは Itinerary の意味論に集中します。

```json
{
  "day_number": 4,
  "itineraries": [
    {
      "title": "土産屋さん",
      "start_time": "07:50",
      "sort_order": 2,
      "category": "shopping",
      "expenses": [
        {
          "title": "土産屋さん",
          "amount": 3700,
          "currency": "JPY",
          "paid_by_name": "節子",
          "note": "費用区分: 個別：節子 / 領収書: R-005",
          "sort_order": 0
        }
      ]
    }
  ]
}
```

| 論点 | 方針 |
|---|---|
| 内部 ID | export しない |
| `place_id` | フィールドなし（場所は `location` のみ） |
| Expense | Itinerary 配下のみ |

詳細は [Export Schema](export-schema.md)。

---

## 14. Itinerary の複製（`itinerary replicate`）

既存 Itinerary を、指定した複数 Day へ **独立した Itinerary** として複製する CLI 操作です。

```text
itinerary replicate --items 12,13,18,21 --to-days 3-5
```

| 観点 | 方針 |
|---|---|
| **recurring との違い** | Google Calendar 的な繰り返し予定ではない。複製後は各 Itinerary を個別に編集できる |
| **コピーする** | `title`, `note`, `start_time`, `sort_order`, `duration_minutes`, `travel_minutes`, `location`, `category`, Itinerary-level notes（デフォルト） |
| **コピーしない** | Expense, Reservation, `id`, タイムスタンプ |
| **将来コピー候補** | Estimate / Planned Budget（[estimate-model.md](estimate-model.md) — 未実装） |
| **sort_order** | 元 Itinerary の値をそのまま維持（各 Day のリズムを揃える） |
| **制約** | source items は同一 Trip・同一 Day。`--to-days` に source Day を含めない |

`--without-notes` 指定時は Itinerary-level notes（Note エンティティ）のみ抑止し、Itinerary 本体の `note` はコピーする。

詳細: [ordering-model.md](ordering-model.md) / [command-reference.md](../command-reference.md)

---

## 15. 実装参照

| 用途 | パス |
|---|---|
| 型定義 | `src/models.rs`（`ItineraryItem`, `ItineraryCategory`, `ExportItineraryV3`） |
| CRUD | `src/itinerary.rs` |
| Export / Import | `src/trip.rs` |
| Markdown 出力 | `src/markdown.rs` |
| canonical sample | `samples/okinawa_sesoko_2026/` |
| golden テスト | `tests/okinawa_sesoko_seed_cli.rs` |

---

## Appendix: 他系統の設計について

Caglla.Travel Web 版には、Google Places を起点とした別系統の Itinerary 設計（作成時に `place_id` を必須とするスキーマなど）が存在します。

**v1.8.0 では CLI モデルを正とし、Web との互換・統合・マイグレーションは本仕様のスコープ外** です。将来の連携を検討する場合も、まず本ドキュメントの「行動単位としての Itinerary」を基準にします。
