# Reservation Entity Design（設計草案）

Caglla.Travel における **Reservation エンティティ** の将来表現 — フィールド、種別、拡張戦略の設計メモです。

**v1.12.0 時点: 仕様整理のみ。** DB migration、CLI、export schema の変更は行いません。

| ドキュメント | 役割 |
|---|---|
| [reservation-model.md](reservation-model.md) | 責務・境界（What it is / is not） |
| **本書** | フィールド・種別・拡張戦略（How we might model it） |
| [reservation-implementation-plan.md](reservation-implementation-plan.md) (v1.13.0) | 実装計画（If we build it, how） |

関連: [Itinerary モデル](itinerary-model.md) / [Travel Ledger Responsibilities](travel-ledger-responsibilities.md) / [Export Schema](export-schema.md) / [Reservation Implementation Plan](reservation-implementation-plan.md)

---

## 1. Web 版 Caglla.Travel からの教訓

過去の Web 版には Reservation スキーマが存在していた。

### 当時の構造

```text
Reservation
 ├─ reservation_type
 ├─ provider / reservation site
 ├─ confirmation code
 └─ type-specific information
```

予約種別ごとに追加情報を持つ思想があった。

```text
hotel
flight
restaurant
rental_car
activity
```

### 評価

| 観点 | 結論 |
|---|---|
| **Itinerary 配下・Trip 集約** | 有効 — 継承する（[reservation-model.md §3](reservation-model.md#3-保存と表示二層構え)） |
| **Venue / POI 分離** | 有効 — 施設情報は POI 側、Reservation は予約情報に集中 |
| **種別ごとの拡張余地** | 有効 — ただし CLI 初手から **専用テーブル分割はしない** |
| **予約サイト内部分類** | Web ではアイコン表示用 — CLI では優先度低（§3） |

CLI / Travel Ledger では、思想は継承しつつ **Core + 将来拡張** の段階的設計を優先する。

---

## 2. 維持する方針（Web 版から）

```text
Reservation は Itinerary 配下
Trip 単位で予約一覧として集約表示できる
Venue / POI 情報は Reservation に複製しない
Reservation は予約・確認情報に集中する
reservation_type による意味付けを持つ
種別ごとの拡張余地は残す
```

利用者操作は **Itinerary 中心**（[reservation-model.md §9](reservation-model.md#9-設計原則--itinerary-中心)）。Reservation は Itinerary 詳細から追加する補助 entity とする。

---

## 3. 見直す方針（Web 版から）

Web 版では予約サイトを内部分類していた。

```text
楽天トラベル / じゃらん / Booking.com / JAL / ANA / …
```

主目的は **画面上のアイコン表示** であった。

CLI / Travel Ledger ではこの taxonomy は優先度が低い。現時点では以下で十分とする。

| フィールド | 用途 |
|---|---|
| `reservation_site_name` | 予約サイト・チャネル名（自由記述） |
| `reservation_site_url` | 予約確認ページ URL |

アイコン表示や provider taxonomy は **将来の GUI** で判断してよい。Core に固定 enum を設けない。

---

## 4. Reservation Core

全 `reservation_type` 共通のフィールド案。実装時のテーブル / export 型のたたき台。

| フィールド | 型（案） | 説明 |
|---|---|---|
| `id` | integer | 主キー（実装時採番） |
| `itinerary_id` | integer | 親 Itinerary（必須） |
| `reservation_type` | enum / text | 種別（§5） |
| `provider_name` | text | 事業者名（航空会社、ホテル名、レストラン名など） |
| `confirmation_code` | text | 予約番号・確認コード |
| `reservation_site_name` | text | 予約サイト名（任意） |
| `reservation_site_url` | text | 予約確認 URL（任意） |
| `start_at` | datetime / text | 利用開始（check-in、pickup、予約時刻など） |
| `end_at` | datetime / text | 利用終了（check-out、return など） |
| `contact_name` | text | 連絡先名（任意） |
| `contact_phone` | text | 電話（任意） |
| `contact_email` | text | メール（任意） |
| `website` | text | 事業者・施設の公式 URL（Venue とは別 — 予約関連リンク） |
| `remark` | text | 短文補足（人数、コース、ETC 要否など） |

### `start_at` / `end_at`

**時間的な幅** を持つ予約の表現に使う。

```text
宿泊     → check-in / check-out
レンタカー → pickup / return
駐車場   → 入庫 / 出庫
レストラン → 予約時刻（end_at は省略可）
```

Itinerary の `start_time`（任意ラベル、`HH:MM`）や Sequence-first の `sort_order` とは **別軸**。Reservation の区間は **契約・予約上の時間幅** を表す。

日時の形式（タイムゾーン、日付のみ可など）は実装フェーズで決定する。

---

## 5. `reservation_type`（enum 候補）

本段階では **列挙候補の整理のみ**。DB enum や CLI バリデーションは実装しない。

```text
hotel
flight
restaurant
rental_car
activity
parking
ticket
other
```

| 値 | 想定用途 |
|---|---|
| `hotel` | 宿泊予約 |
| `flight` | 航空券 |
| `restaurant` | レストラン・食事予約 |
| `rental_car` | レンタカー |
| `activity` | アクティビティ・施設利用 |
| `parking` | 駐車場予約 |
| `ticket` | チケット・入場券 |
| `other` | 上記に当てはまらない予約 |

---

## 6. Core で足りる種別

以下は **Core fields のみ** でおおむね表現できる可能性が高い。

```text
hotel
restaurant
activity
parking
ticket
```

### 例: hotel

```text
reservation_type     = hotel
provider_name        = Hilton Okinawa Sesoko Resort
confirmation_code    = ABC123
start_at             = 2026-04-26 16:40  （check-in）
end_at               = 2026-04-29 10:00  （check-out）
reservation_site_name = 楽天トラベル / Hilton 直予約 など
remark               = 部屋タイプ、ゲスト名など
```

Venue（施設住所・電話）は **複製しない**。Itinerary `location` または将来の Venue 参照で辿る。

### 例: restaurant

```text
reservation_type     = restaurant
provider_name        = AMAHAJI
confirmation_code    = （予約番号）
start_at             = 2026-04-27 19:30
remark               = 5名、コース名など
```

---

## 7. 拡張が必要になりやすい種別

以下は Core fields **だけでは不足しやすい**。

```text
flight
rental_car
```

### flight — 拡張候補フィールド

```text
airline
flight_number
departure_airport
arrival_airport
departure_time
arrival_time
terminal
seat_number
booking_class
```

### rental_car — 拡張候補フィールド

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

Core の `start_at` / `end_at` と一部重複しうるが、空港コード・車両クラスなどは **種別固有** として拡張レイヤに置く想定。

**v1.12.x では専用テーブルは作らない**（§8）。

---

## 8. 専用テーブルに進まない理由

現段階では以下のような **種別専用テーブル** には進まない。

```text
flight_reservations
hotel_reservations
restaurant_reservations
rental_car_reservations
```

| 理由 | 説明 |
|---|---|
| 実装フェーズではない | v1.12.0 は設計文書のみ |
| Core で多くを表現可能 | hotel / restaurant / activity 等は Core で足りる見込み |
| 拡張候補は一部に限定 | flight / rental_car が明確な拡張対象 |
| schema 肥大化 | 早期分割は migration・export・CLI が重くなる |
| 入力体験 | 種別ごとテーブルは CLI 初手の UX を複雑化する |

Web 版の type-specific tables の **思想**（種別固有フィールド）は残し、**物理テーブル分割のタイミング**は実装フェーズで判断する。

---

## 9. 拡張実装の将来候補（未決定）

flight / rental_car などの種別固有データの格納方式は **本段階では決定しない**。候補のみ列挙する。

| 方式 | 概要 | 長所 | 短所 |
|---|---|---|---|
| **A. Core + `details_json`** | 1 テーブル + JSON 列 | シンプル、種別追加が軽い | クエリ・検証が弱い |
| **B. Core + subtype tables** | `flight_reservations` 等 | 型安全、Web 版に近い | schema 重い |
| **C. Export only nested details** | DB は Core、export でネスト | バックアップ表現が豊富 | DB と export の乖離 |
| **D. CLI Core only、GUI 詳細** | CLI は最小、GUI で拡張 | CLI 入力が簡潔 | CLI 利用者は詳細不足 |

実装着手時に、export roundtrip・`trip doctor`・しおり生成の要件と合わせて選択する。

---

## 10. Venue 参照方針

Reservation Core に **施設の複製**（住所・緯度経度・電話の正本）は持たない。

```text
Venue（将来）
- 施設名、住所、電話、website、緯度経度、POI ID

Reservation
- 予約番号、確認、利用期間、連絡先（予約窓口）
```

Itinerary の `location`（任意テキスト）は移行期の表示用。将来は Venue 参照 + Reservation の組み合わせが目標。

---

## 11. Routing は対象外

[Routing](reservation-model.md#8-routing-は対象外) は引き続き Reservation の対象外。

```text
飛行機移動 / ドライブ / 徒歩 / タクシー / 高速道路
```

空間的な幅（出発地・到着地・経路）は **将来の独立モデル**。Reservation は **時間的な幅**（予約・契約区間）は表現できるが、**移動経路そのもの** は表現しない。

移動は現行どおり **Itinerary 行**（`category: transport` 等）で表す。

---

## 12. 将来フック（実装フェーズ — 本書では未実装）

| 領域 | 方針案 |
|---|---|
| **CLI** | `reservation add/list/show/update/delete` — Itinerary スコープ |
| **Export** | schema v4 または v3 拡張 — `days[].itineraries[].reservations[]` |
| **しおり** | Trip-level Reservation セクション（集約表示） |
| **doctor** | 予約必須 Itinerary に Reservation 欠落を検出 |
| **canonical sample** | 旅行前しおり検証用に okinawa へ段階投入（別バージョン） |

詳細: **[Reservation Implementation Plan](reservation-implementation-plan.md)**（v1.13.0）。

---

## 13. v1.12.0 スコープ（本書）

### 実施する

| 項目 | 内容 |
|---|---|
| 仕様書 | 本ドキュメント |
| 索引 | [specifications/README.md](README.md) |
| 参照 | [reservation-model.md](reservation-model.md)、[itinerary-model.md](itinerary-model.md) |

### 実施しない

```text
DB migration
Reservation table 実装
CLI 追加
export / import schema 変更
Markdown export 変更
canonical sample 更新
```

---

## 14. 用語

| 用語 | 意味 |
|---|---|
| **Reservation Core** | 全種別共通のフィールド集合（§4） |
| **reservation_type** | 予約種別の列挙（§5） |
| **Type extension** | flight / rental_car 等の種別固有フィールド（§7） |
| **details_json** | 拡張候補 A — JSON による種別詳細 |
| **Remark** | Itinerary 行の `itinerary_items.note` — Reservation とは別 |

---

## 15. 実装参照（現行）

| 概念 | 状態 |
|---|---|
| Reservation entity | **未実装** |
| 責務・境界 | [reservation-model.md](reservation-model.md) |
| フィールド設計 | **本書** |
| 実装計画 | [reservation-implementation-plan.md](reservation-implementation-plan.md) |
| Remark（暫定） | `itinerary_items.note` |
