# Reservation Implementation Plan（実装計画）

Caglla.Travel CLI に **Reservation entity を実装する場合** の計画メモです。

**v1.13.0 時点: 仕様整理のみ。** DB migration、CLI、export schema、Markdown export の変更は行いません。

| ドキュメント | 役割 |
|---|---|
| [reservation-model.md](reservation-model.md) (v1.11.0) | 責務・境界（What it is / is not） |
| [reservation-entity-design.md](reservation-entity-design.md) (v1.12.0) | フィールド・種別・拡張戦略（How we might model it） |
| **本書** (v1.13.0) | 実装計画（If we build it, how） |

関連: [Itinerary モデル](itinerary-model.md) / [Expense モデル](expense-model.md) / [Export Schema](export-schema.md) / [Travel Ledger Responsibilities](travel-ledger-responsibilities.md)

---

## 1. Goals

Reservation 実装で解決したいこと。

| 課題 | 解決イメージ |
|---|---|
| **予約確認番号を保存したい** | `confirmation_code` を構造化フィールドとして保持。Remark への埋め込みに依存しない |
| **予約サイト情報を保持したい** | `reservation_site_name` / `reservation_site_url` でチャネル・確認ページを記録 |
| **チェックイン・返却などの期間を管理したい** | `start_at` / `end_at` で契約・予約上の時間幅を表現（Itinerary `start_time` とは別軸） |
| **旅程と予約情報を分離したい** | Itinerary = 行動、Reservation = 予約・確認。同一 Itinerary に 0..N 件の Reservation |
| **しおりで予約一覧を見せたい** | Trip レベル集約表示（保存は Itinerary 配下 — [reservation-model.md §3](reservation-model.md#3-保存と表示二層構え)） |
| **doctor で予約漏れを検出したい** | 予約必須行動に Reservation が無いケースを optional warning（§9） |
| **export / import で予約を持ち運びたい** | schema v3 拡張または v4 で `reservations[]` をネスト（§7） |

実装の主目的は **旅行前の計画共有**（予約番号・連絡先・手続き情報）と **Remark からの段階的移行** です。canonical sample の主眼（旅行後台帳・清算）は維持しつつ、しおり用途を補完します。

---

## 2. Non-goals

Reservation が **扱わない** もの。実装計画でもスコープ外とする。

| 概念 | 理由 |
|---|---|
| **Expense** | 費用記録 — 予約の有無と独立（[reservation-model.md §4](reservation-model.md#4-reservation-と-expense)） |
| **Note** | 自由記述メモ — 予約番号の正本ではない |
| **Venue** | 施設・POI — 住所・緯度経度の正本は Venue / Itinerary `location` 側 |
| **Routing** | 空間的移動経路 — 将来の独立モデル（[reservation-model.md §8](reservation-model.md#8-routing-は対象外)） |
| **GPS トラッキング** | 位置追跡・ログ — Travel Ledger の対象外 |
| **地図描画** | 経路・マップ表示 — CLI / 本計画の対象外 |
| **予約サイト taxonomy（楽天・じゃらん等の固定 enum）** | Web 版アイコン用分類 — CLI 初手では `reservation_site_name` で十分（[reservation-entity-design.md §3](reservation-entity-design.md#3-見直す方針web-版から)） |
| **種別専用テーブル** | `flight_reservations` 等 — v1.x 初手では作らない（§4, §11） |
| **自動予約取得・外部 API 連携** | 手入力 CLI を前提 |

---

## 3. Minimum Viable Reservation（MVR）

初回実装で採用する **最小フィールド集合**。ベースは [Reservation Entity Design §4](reservation-entity-design.md#4-reservation-core)。

| フィールド | 必須 | 型（案） | 説明 |
|---|---|---|---|
| `id` | ✓（DB） | integer | 主キー |
| `itinerary_id` | ✓ | integer | 親 Itinerary（FK — アプリ側 cascade） |
| `reservation_type` | ✓ | text / enum | §5 の候補から 1 つ |
| `provider_name` | 推奨 | text | 事業者名（航空会社、ホテル、レストラン等） |
| `confirmation_code` | 推奨 | text | 予約番号・確認コード |
| `reservation_site_name` | 任意 | text | 予約サイト・チャネル名 |
| `reservation_site_url` | 任意 | text | 予約確認ページ URL |
| `start_at` | 任意 | text | 利用開始（ISO 8601 または日付のみ — 実装時に形式確定） |
| `end_at` | 任意 | text | 利用終了 |
| `contact_name` | 任意 | text | 連絡先名 |
| `contact_phone` | 任意 | text | 電話 |
| `contact_email` | 任意 | text | メール |
| `website` | 任意 | text | 事業者公式 URL（Venue 複製ではない） |
| `remark` | 任意 | text | 短文補足（部屋タイプ、人数、コース名等） |
| `created_at` | ✓（DB） | datetime | 作成日時 |
| `updated_at` | ✓（DB） | datetime | 更新日時 |

### MVR で意図的に含めないもの

| 項目 | 扱い |
|---|---|
| `sort_order` | 初手は **不要**（1 Itinerary あたり件数が少ない想定）。複数 Reservation が増えたら追加検討 |
| flight / rental_car 固有フィールド | MVR 外 — `remark` または将来拡張（§11） |
| `details_json` | 初手は採用しない — 型安全・doctor 検証を優先 |
| Venue 複製フィールド | 持たない |

### CLI 最小入力（案）

```bash
reservation add --itinerary 12 --type hotel \
  --provider "Hilton Okinawa Sesoko" \
  --confirmation ABC123 \
  --start-at "2026-04-26T16:40" --end-at "2026-04-29T10:00"
```

`--type` と `--itinerary` を必須とし、他は任意。Expense の `--amount` 必須パターンと同型の「最小入力 + 任意拡張」です。

---

## 4. Database Design

### 方針: 単一 `reservations` テーブルで開始

```text
reservations
```

のみ。以下は **v1.x 初手では作らない**:

```text
flight_reservations
hotel_reservations
restaurant_reservations
rental_car_reservations
```

理由は [reservation-entity-design.md §8](reservation-entity-design.md#8-専用テーブルに進まない理由) と同様 — schema 肥大化、CLI 入力の複雑化、export roundtrip の重さを避ける。

### テーブル案（DDL 草案 — 未実装）

```sql
CREATE TABLE reservations (
    id                  INTEGER PRIMARY KEY AUTOINCREMENT,
    itinerary_id        INTEGER NOT NULL,
    reservation_type    TEXT NOT NULL,
    provider_name       TEXT,
    confirmation_code   TEXT,
    reservation_site_name TEXT,
    reservation_site_url  TEXT,
    start_at            TEXT,
    end_at              TEXT,
    contact_name        TEXT,
    contact_phone       TEXT,
    contact_email       TEXT,
    website             TEXT,
    remark              TEXT,
    created_at          TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at          TEXT NOT NULL DEFAULT (datetime('now'))
);
```

| 論点 | 方針 |
|---|---|
| FK 制約 | Expense / Note と同型 — **アプリ側** で `itinerary_id` 存在・Trip 所属を検証。cascade delete は Itinerary 削除時に Reservation も削除 |
| `reservation_type` | TEXT + アプリ側バリデーション（enum 候補は §5） |
| 日時 | 初手は **TEXT**（ISO 8601 推奨）。タイムゾーン方針は実装フェーズで Trip `start_date` と整合 |
| インデックス | `itinerary_id` に INDEX（`list --itinerary` / export 用） |
| Migration | 既存 DB へ **CREATE TABLE のみ** — backfill 不要（Expense と同型） |

### Model 層（案）

| 項目 | 方針 |
|---|---|
| 型定義 | `src/models.rs` に `Reservation` struct |
| CRUD | `src/reservation.rs`（新規 — Expense / Note と同パターン） |
| cascade | `itinerary delete` 時に配下 Reservation を削除 |

---

## 5. CLI Design

Expense / Note と同型の **Itinerary 起点 CRUD** を想定。

### コマンド一覧

```text
reservation add
reservation list
reservation show
reservation update
reservation delete
```

### コマンド例

```bash
# 最小入力
reservation add --itinerary 12 --type hotel \
  --provider "Hilton Okinawa Sesoko Resort" \
  --confirmation ABC123

# 期間・サイト情報
reservation add --itinerary 12 --type hotel \
  --provider "Hilton Okinawa Sesoko Resort" \
  --confirmation ABC123 \
  --site-name "楽天トラベル" \
  --site-url "https://example.com/booking/abc123" \
  --start-at "2026-04-26T16:40" \
  --end-at "2026-04-29T10:00" \
  --remark "デラックスツイン、2名"

# 一覧
reservation list --itinerary 12
reservation list --trip 1              # Trip 配下を集約表示

# 詳細・更新・削除
reservation show 3
reservation show 3 --json
reservation update 3 --confirmation XYZ789 --remark "部屋変更済み"
reservation delete 3
```

### owner 指定（案）

| 操作 | owner 指定 |
|---|---|
| `add` | **`--itinerary` 必須** |
| `update` / `delete` | **Reservation ID** で指定 |
| `list` | **`--itinerary` または `--trip` のいずれか 1 つ必須** |
| `list --trip` | 当該 Trip 配下すべてを集約（Day / Itinerary コンテキスト付き表示） |

Trip / Day 直下への `add` は **不可**（Expense と同型）。

### 表示 UX（テキスト出力案）

**`reservation list --itinerary 12`**

```text
ID  Type        Provider                      Confirmation  Start              End
3   hotel       Hilton Okinawa Sesoko Resort  ABC123        2026-04-26 16:40   2026-04-29 10:00
```

**`reservation list --trip 1`**

```text
Day 2 / Itinerary 15 チェックイン
  [3] hotel  Hilton Okinawa Sesoko Resort  ABC123  2026-04-26 16:40 → 2026-04-29 10:00

Day 2 / Itinerary 18 NU045 NGO ⇒ OKA
  [5] flight  JAL  （confirmation なし）  2026-04-26 14:30 → 2026-04-26 17:00
```

**`reservation show 3`**

```text
Reservation #3
  Itinerary:  15  チェックイン  (Day 2)
  Type:       hotel
  Provider:   Hilton Okinawa Sesoko Resort
  Confirmation: ABC123
  Site:       楽天トラベル
  URL:        https://example.com/booking/abc123
  Period:     2026-04-26 16:40 — 2026-04-29 10:00
  Contact:    （なし）
  Remark:     デラックスツイン、2名
```

### その他 CLI 論点

| 論点 | 方針 |
|---|---|
| `--type` | 必須（add 時）。`hotel` / `flight` / … / `other` |
| `--json` | `list` / `show` で対応（Note / Expense と同型） |
| エラーメッセージ | 未知の `reservation_type`、存在しない `itinerary_id` を明示 |
| `main.rs` | トップレベル `reservation` サブコマンドを追加 |

---

## 6. Display Integration

Reservation の **表示** は保存（Itinerary 配下）と分離し、複数の入口から辿れるようにする。

### 表示入口

| 入口 | 用途 | 優先度 |
|---|---|---|
| **`reservation list --trip 1`** | Trip しおり向け **予約一覧**（主表示） | 高 |
| **`reservation list --itinerary 12`** | 編集・確認用の局所一覧 | 高 |
| **`itinerary show 12`** | Itinerary 詳細に紐づく Reservation を **インライン表示** | 高 |
| **`trip export-md 1`** | Markdown しおりに Reservation 節を出力（§8） | 中（export-md と同フェーズ） |

`trip reservation-list` のような **別トップレベルコマンド** は初手では **不要** とする。`reservation list --trip` で足りる（Expense の `expense list --trip` と同型）。

### `itinerary show` 統合（案）

```text
Itinerary #15  チェックイン
  Day:        2  (2026-04-26)
  Title:      チェックイン
  Location:   Hilton Okinawa Sesoko Resort
  Start time: 16:40
  Remark:     （itinerary_items.note — 従来どおり）

  Reservations (1):
    [3] hotel  ABC123  Hilton Okinawa Sesoko Resort
        2026-04-26 16:40 — 2026-04-29 10:00
```

Remark（`itinerary_items.note`）と Reservation は **併記** する。移行期は Remark に予約番号が残っていても表示上は共存可。

### Trip しおり（集約）のイメージ

[reservation-model.md §3](reservation-model.md#3-保存と表示二層構え) の二層構え:

```text
保存:  Itinerary に紐づける
表示:  Trip 単位で予約一覧（reservation list --trip / export-md の Reservation 節）
```

利用者は **関係グラフを編集しない**。Itinerary 詳細から Reservation を追加し、Trip レベルでは一覧として読む。

---

## 7. Export / Import

### 方針: schema v3 拡張を第一候補（未確定）

> **v1.13.0 時点:** 以下は将来実装時の **候補** であり、確定事項ではない。Summary / Reservation / Participant / Photo / Attachment 等のモデル整理が進む中で、実装フェーズに **Export Schema v4** を切る可能性も十分ある。実装着手時に再評価する。

Expense が `days[].itineraries[].expenses[]` にネストされたのと同型に、Reservation を **Itinerary 配下** にネストする案。

```json
{
  "schema_version": 3,
  "days": [
    {
      "day_number": 2,
      "itineraries": [
        {
          "title": "チェックイン",
          "sort_order": 5,
          "reservations": [
            {
              "reservation_type": "hotel",
              "provider_name": "Hilton Okinawa Sesoko Resort",
              "confirmation_code": "ABC123",
              "reservation_site_name": "楽天トラベル",
              "reservation_site_url": null,
              "start_at": "2026-04-26T16:40:00",
              "end_at": "2026-04-29T10:00:00",
              "contact_name": null,
              "contact_phone": null,
              "contact_email": null,
              "website": null,
              "remark": "デラックスツイン"
            }
          ],
          "expenses": []
        }
      ]
    }
  ]
}
```

### 検討観点

| 観点 | 方針案 |
|---|---|
| **schema version** | **v3 拡張**（`reservations[]` 追加、省略時は空配列）を第一候補。破壊的変更が必要なら v4 |
| **backward compatibility** | v3 export に `reservations` キーなし → import 時 **空配列** 扱い（Expense 追加時と同型） |
| **内部 ID** | **`reservation.id` / `itinerary_id` は export しない** — JSON 親子構造で関連を保持 |
| **import 順序** | Trip → Day → Itinerary → Checklist → Note → Expense → **Reservation**（Itinerary 存在後） |
| **`validate-export`** | `reservation_type` 必須、未知 type は warning または error（実装時決定） |
| **`trip duplicate`** | v3 roundtrip 後に Reservation も複製 |
| **v1 / v2 import** | `reservations` なし — 問題なし |
| **top-level `reservations[]`** | **採用しない**（Expense と同じくネストのみ） |

### schema v4 が必要になる条件（将来）

- flight / rental_car の **ネスト詳細オブジェクト**（`flight_details: {}`）を export 必須にする
- `details_json` を export に含め、roundtrip で型を保証する

初手 MVR では **v3 拡張で十分** と見込む。

### リリース分割（案）

| フェーズ | 内容 |
|---|---|
| **実装 Phase 1** | `reservations` テーブル + CLI CRUD |
| **実装 Phase 2** | export v3 拡張 + import + validate-export |
| **実装 Phase 3** | export-md、duplicate roundtrip、canonical sample 段階投入 |
| **将来** | trip diff `reservations[]`、schema v4 |

---

## 8. Markdown Export

対象: **`trip export-md`**

### 方針

Expense / Note と同様、**しおり用途の読みやすい出力** を追加する。データ確認・旅行前共有が主目的。

### 出力案

**Trip レベル — Reservations 節（集約）**

```markdown
## Reservations

| Day | Itinerary | Type | Provider | Confirmation | Period |
|-----|-----------|------|----------|--------------|--------|
| 2 | チェックイン | hotel | Hilton Okinawa Sesoko Resort | ABC123 | 2026-04-26 16:40 — 2026-04-29 10:00 |
| 2 | NU045 NGO ⇒ OKA | flight | JAL | — | 2026-04-26 14:30 — 2026-04-26 17:00 |
```

**Day / Itinerary 節内（インライン）**

```markdown
### 16:40 チェックイン
- **Location:** Hilton Okinawa Sesoko Resort
- **Reservation:** hotel / ABC123 / Hilton Okinawa Sesoko Resort
- **Period:** 2026-04-26 16:40 — 2026-04-29 10:00
```

| 論点 | 方針 |
|---|---|
| Reservation 0 件 | Trip Reservations 節は **省略** または「（なし）」— 実装時に統一 |
| `itinerary_items.note` | 従来どおり Itinerary 行に表示。Reservation とは別 |
| flight 詳細 | MVR では `remark` のみ。将来拡張時にサブ項目追加 |
| 実装タイミング | CLI CRUD + export の **次フェーズ**（Expense: v1.7.x で export-md 対応と同型） |

---

## 9. Doctor Integration

実装は本段階では **不要**。将来 `trip doctor` に載せる場合の検証候補。

### 原則

| 機能 | 初手方針 |
|---|---|
| `trip doctor` | **optional suggestion** 中心 — 旅行を止めない warning |
| `trip stats` | Reservation **件数** のみ（任意） |
| `trip advisor` | 対象外 |

Note / Expense と同様、**Reservation 不足で error にしない**。

### 検証候補

| チェック | 深刻度 | 説明 |
|---|---|---|
| **予約番号欠落** | Info / Suggestion | `confirmation_code` が空の Reservation がある |
| **開始終了逆転** | Warning | `start_at` > `end_at` |
| **期限切れ（旅行後）** | Info | `end_at` が Trip `end_date` より大幅に後（データ入力ミス） |
| **予約必須 Itinerary に Reservation なし** | Suggestion | ヒューリスティック: `category` や title パターン（チェックイン、フライト、レストラン予約等） |
| **Remark に予約番号らしき文字列があるが Reservation なし** | Suggestion | Remark → Reservation 移行を促す |
| **未知の reservation_type** | Warning | DB に不正 type が入っている（import 破損） |
| **Itinerary 孤立 Reservation** | Error | 参照先 Itinerary 不在（整合性 — 通常は発生しない） |

### 予約必須判定（将来・要設計）

自動判定は誤検知しやすい。初手は **保守的** に:

- Itinerary に既に Reservation が 1 件以上ある → 追加チェックのみ
- `category: lodging` 等の明示カテゴリ + Reservation 0 → optional suggestion
- 全 Itinerary への一括要求は **しない**

---

## 10. Canonical Sample Strategy

対象: [`samples/okinawa_sesoko_2026/`](../../samples/okinawa_sesoko_2026/)

### 現状

| 項目 | 状態 |
|---|---|
| 主目的 | **旅行後台帳・清算・export 検証** |
| Reservation | **意図的に省略** — Remark / title / Expense に分散 |
| golden file | `expected-export-v3.json` — Reservation キーなし |

### 適用方針

| 段階 | 方針 |
|---|---|
| **Reservation 実装直後** | okinawa sample への **大量投入はしない** |
| **最小検証** | 別途 **小さな fixture** または seed の **オプション引数** で 2〜3 件の Reservation を追加するテストを検討 |
| **段階投入（将来）** | ホテル 1 件 + フライト 1 件 + レストラン 0〜1 件程度を **代表例** として追加。golden 更新が必要 |
| **旅行前しおり検証** | `trip export-md` に Reservation 節が出ることを **別サンプル**（小規模 Trip）で検証する方が安全 |

### 大量投入が不要な理由

1. canonical sample の価値は **58 Itinerary / 49 Expense の清算整合** にある
2. Reservation 追加は golden file・seed 時間・README の説明負荷が増える
3. Remark に予約情報が残っている現状は **移行期の現実的なデータ** として残す価値がある

### 将来の seed 例（参考のみ）

```bash
# Day 2 チェックイン Itinerary（seed 後 ID は環境依存 — 実装時は名前解決）
reservation add --itinerary <checkin_id> --type hotel \
  --provider "Hilton Okinawa Sesoko Resort" \
  --confirmation "（PDF 由来の番号）" \
  --start-at "2026-04-26T16:40" --end-at "2026-04-29T10:00"
```

実装フェーズで **代表 2〜3 件** を入れるか、**`samples/reservation_demo/`** を新設するかを決定する。

---

## 11. Future Expansion

v1.x 初手 MVR を超える拡張候補。[reservation-entity-design.md §7–9](reservation-entity-design.md#7-拡張が必要になりやすい種別) と整合。

### flight

Core だけでは不足しやすいフィールド:

```text
airline
flight_number
departure_airport
arrival_airport
departure_time      # start_at と役割分担を実装時に整理
arrival_time
terminal
seat_number
booking_class
```

**初手:** `remark` に便名・座席を書く運用を許容。  
**将来:** `details_json` または export ネスト `flight_details` — **専用テーブルは v1.x では作らない**。

### rental_car

```text
pickup_location
return_location
pickup_time
return_time
vehicle_class
fuel_policy
insurance
etc_card_required
```

**初手:** `start_at` / `end_at` + `remark`（ETC 要否は既に Remark 運用あり）。  
**将来:** rental_car 詳細は拡張レイヤ（JSON または export のみネスト）。

### 拡張方式の選択（実装時）

| 方式 | 採用タイミング案 |
|---|---|
| **A. Core + `details_json`** | flight / rental_car を CLI で構造化入力したくなったとき |
| **B. subtype tables** | Web 版連携・複雑クエリが必要になったとき（v1.x 非推奨） |
| **C. Export only nested** | DB は MVR、バックアップだけ詳細保持 |
| **D. CLI Core only、GUI 詳細** | GUI 開発時 |

### Routing・Venue

- **Routing** — 引き続き Reservation 対象外
- **Venue 参照** — Reservation に施設複製はしない。将来 `venue_id` は Itinerary 側で保持

---

## 12. 実装フェーズ（参考ロードマップ）

本書は計画のみ。実装時の分割案。

| Phase | 内容 | 依存 |
|---|---|---|
| **0** | 本仕様 + entity design（完了: v1.11–v1.12） | — |
| **1** | 本実装計画（v1.13.0） | Phase 0 |
| **2** | DB migration + Model + CRUD テスト | Phase 1 |
| **3** | CLI（add/list/show/update/delete + `--json`） | Phase 2 |
| **4** | export v3 拡張 + import + validate-export | Phase 3 |
| **5** | itinerary show 統合 + export-md | Phase 4 |
| **6** | doctor optional checks + 小規模 sample | Phase 5 |
| **7** | flight / rental_car 拡張方式の決定 | 利用フィードバック後 |

---

## 13. v1.13.0 スコープ（本書）

### 実施する

| 項目 | 内容 |
|---|---|
| 仕様書 | 本ドキュメント |
| 索引 | [specifications/README.md](README.md) |
| 参照 | reservation-model、reservation-entity-design、itinerary-model 等 |

### 実施しない

```text
DB migration
reservations テーブル実装
CLI コマンド追加
export / import schema 変更
Markdown export 変更
trip doctor 変更
canonical sample 更新
```

---

## 14. 用語

| 用語 | 意味 |
|---|---|
| **MVR** | Minimum Viable Reservation — 初回実装の最小フィールド集合（§3） |
| **Reservation Core** | 全種別共通フィールド（[entity design §4](reservation-entity-design.md#4-reservation-core)） |
| **二層構え** | 保存 Itinerary 配下 / 表示 Trip 集約 |
| **Remark** | `itinerary_items.note` — Reservation とは別 |

---

## 15. 実装参照（現行）

| 概念 | 状態 |
|---|---|
| Reservation entity | **未実装** |
| 責務・境界 | [reservation-model.md](reservation-model.md) |
| フィールド設計 | [reservation-entity-design.md](reservation-entity-design.md) |
| 実装計画 | **本書** |
| 類似実装（参考） | `src/expense.rs`, `src/note.rs` |
| Remark（暫定） | `itinerary_items.note` |
