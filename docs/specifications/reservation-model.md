# Reservation モデル（設計草案）

Caglla.Travel CLI / 将来 Web 版に向けた **Reservation** エンティティの仕様メモです。

**v1.11.0 時点: 仕様整理のみ。** DB migration、CLI、export schema の変更は行いません。

関連: [Travel Ledger Responsibilities](travel-ledger-responsibilities.md) / [Itinerary モデル](itinerary-model.md) / [Expense モデル](expense-model.md) / [Note モデル](note-model.md) / [Export Schema](export-schema.md) / [Reservation Entity Design](reservation-entity-design.md) / [Reservation Implementation Plan](reservation-implementation-plan.md)

---

## 1. Reservation の定義

### What it is

```text
Reservation represents booking and confirmation information
required to execute a travel activity.
```

日本語:

```text
旅行行動を実行するために必要な予約・確認情報
```

Reservation は **Itinerary 配下** に属する、**構造化された予約・確保情報** です。

### What it is not

| 概念 | 関係 |
|---|---|
| **Note** | 自由記述メモ — Reservation ではない |
| **Expense** | 費用記録 — 予約の有無とは独立 |
| **Venue** | 場所・POI — 「どこで」であり「予約そのもの」ではない |
| **Remark** | 短文の備考（`itinerary_items.note`）— 予約番号の正本ではない |
| **Routing** | 空間的な移動経路 — 本仕様の対象外（§8） |

---

## 2. モデル上の位置付け

Travel Ledger の基本構造（現行 CLI）:

```text
Trip
 └ Day
      └ Itinerary
           └ Expense
```

将来、Itinerary 配下の補助情報として次を想定します。

```text
Itinerary
 ├─ Venue          （将来 — どこで）
 ├─ Reservation    （将来 — 予約・確認）
 └─ Expense        （現行 — 費用）
```

各概念の責務:

| 概念 | 責務 | 例 |
|---|---|---|
| **Itinerary** | **何をするか**（行動単位） | ヒルトン瀬底チェックイン、JTA045で那覇へ移動、海邦丸で昼食 |
| **Venue** | **どこで行うか** | 施設名、住所、電話、Web、緯度経度、POI（Google Maps POI に近い） |
| **Reservation** | **予約・確保済みの権利や確認情報** | 航空券、ホテル、レンタカー、レストラン、アクティビティ、駐車場、施設利用 |
| **Expense** | **費用記録** | 宿泊費、航空券代、食事代 — 予約の有無と独立 |

**Itinerary is not a venue**（[itinerary-model.md](itinerary-model.md)）は維持します。Venue は Itinerary に **任意で紐づく** 補助情報として将来分離する想定です。

---

## 3. 保存と表示（二層構え）

[travel-ledger-responsibilities.md §5](travel-ledger-responsibilities.md#5-reservation) と同方針:

```text
保存:  Itinerary に紐づける
表示:  Trip しおりで Reservation 一覧として集約表示する
```

旧 Caglla.Travel Web でも Itinerary 配下保存 + Trip レベル集約表示がありました。このパターンを継承します。

---

## 4. Reservation と Expense

予約と費用は **別エンティティ** です。同一 Itinerary に両方存在しうる。

### 例: ホテル宿泊

```text
Itinerary: チェックイン（ヒルトン瀬底）

Reservation:
  - 宿泊予約
  - 予約番号
  - チェックイン / チェックアウト（時間幅 — §7）

Expense:
  - 宿泊費
  - 管理費など
```

### 例: コンビニ買い物

```text
Itinerary: 朝食 ローソン

Reservation:
  - なし

Expense:
  - 購入費用
```

Expense のみ存在する行動は多数あります。Reservation は **実行に予約が要る行動** に付与します。

---

## 5. Reservation と Note

| | Reservation | Note entity |
|---|---|---|
| **性質** | 構造化された予約情報 | 自由記述 |
| **件数** | 0..N（将来） | 0..N |
| **例** | 予約番号、チェックイン日時、便名、座席、予約サイト | 次回来るなら朝がおすすめ、雨天時は別プラン、予約時のやり取りの背景 |

予約の **背景・経緯・メモ** は Note でよい。**予約番号・確認情報の正本** は Reservation です。

---

## 6. Reservation と Remark

| | Remark（`itinerary_items.note`） | Reservation |
|---|---|---|
| **GUI** | 備考 | 予約情報 |
| **性質** | 短い補足、旅程表の行内表示向き | 予約そのもの、構造化 |
| **例** | 要ETCカード、集合場所は南口、レシート番号123 | Confirmation ABC123、便名 NU045 |

Remark に予約番号を書く運用は **移行期** として許容できますが、しおり集約・doctor 検出・一覧抽出のため **Reservation entity が正** です。

---

## 7. 時間的な幅

Reservation は **時間的な幅**（期間・区間）を持つ関係を表現できます。

| 種別 | イメージ |
|---|---|
| 宿泊 | `check_in` / `check_out` |
| レンタカー | `pickup` / `return` |
| 駐車場 | 入庫 / 出庫 |
| 航空券 | 出発 / 到着（便情報と併用） |

Itinerary の `start_time`（任意ラベル）や Sequence-first の `sort_order` とは **別軸** です。Reservation の時間幅は **予約・契約上の区間** を表します。

フィールド名・型は将来の実装フェーズで決定します。本仕様では **概念として期間を持てる** ことを明記します。

---

## 8. Routing は対象外

過去の Caglla.Travel Web では **空間的な幅** を持つモデル（Routing）も検討していました。

```text
飛行機移動
ドライブ
散歩
タクシー移動
高速道路
```

これらは **Reservation の責務ではありません**。

| 概念 | 空間 |
|---|---|
| **Routing** | 経路・移動区間（空間的幅） |
| **Reservation** | 予約・確保（権利・確認） |
| **Itinerary** | 行動そのもの（「高速道路 東浦→セントレア」等） |

Routing は **将来の独立モデル** として扱い、本仕様（Reservation model）のスコープ外とします。移動は現行 CLI どおり **Itinerary 行**（`category: transport` 等）で表現します。

---

## 9. 設計原則 — Itinerary 中心

Caglla.Travel Web 版の反省（Venue 関係・Routing 関係を利用者に直接露出し UI が複雑化）を踏まえ、次を原則とします。

**English:**

```text
Caglla.Travel should not expose generic relationship modeling
directly to users.

Complex internal relationships may exist,
but user-facing operations should remain itinerary-centric.
```

**日本語:**

```text
Caglla.Travel は汎用的な関係モデルを
利用者へ直接露出しない。

内部的に複雑な関係を持てる余地は残すが、
利用者操作は Itinerary 中心とする。
```

### 実務上の意味

| 利用者操作 | 方針 |
|---|---|
| 予定の追加・並び替え | **Itinerary** を編集 |
| 予約番号の登録 | 将来: Itinerary 詳細から **Reservation** を追加（グラフ編集 UI ではない） |
| 費用の登録 | Itinerary 配下の **Expense** |
| 場所の指定 | 現行: `location` 任意列 / 将来: **Venue** 参照（Itinerary から辿る） |

Trip しおりの **Reservation 一覧** は **表示の集約** であり、利用者が Trip 全体の関係グラフを編集する UI ではありません。

---

## 10. しおり・doctor との関係（将来）

計画共有（旅行前）で必要な情報の例:

```text
どこに行くか
何時ごろ行くか
いくらぐらいか
何を忘れてはいけないか
予約番号は何か
連絡先はどこか
当日どう手続きするか
```

| 情報 | 主なソース（将来） |
|---|---|
| どこに行くか | Itinerary / Venue / Day Summary |
| 何時ごろ | Itinerary `start_time` + Sequence |
| いくら | Expense 集計 |
| 予約番号・連絡先 | **Reservation**（Trip 集約表示） |
| 忘れ物 | Checklist |
| 補足 | Remark / Note |

`trip doctor` で **予約が必要な Itinerary に Reservation が無い** ケースを検出する余地があります（Future scope）。

---

## 11. 現行 CLI での暫定表現

v1.10.0 まで Reservation entity は **未実装** です。canonical sample（[okinawa_sesoko_2026](../../samples/okinawa_sesoko_2026/)）では:

| 情報 | 現行の置き場 |
|---|---|
| 予約番号・ETC 要否 | Itinerary **Remark** |
| 航空便・ホテル名 | Itinerary **title** / `location` |
| 金額 | **Expense** |
| 準備項目 | **Checklist** |

旅行後台帳・清算検証が主目的のため、Summary / Reservation entity は seed に含めていません（[travel-ledger-responsibilities.md §8](travel-ledger-responsibilities.md#8-canonical-sample-との関係)）。

---

## 12. v1.11.0 スコープ（本書）

### 実施する

| 項目 | 内容 |
|---|---|
| 仕様書 | 本ドキュメント（`reservation-model.md`） |
| 索引 | [specifications/README.md](README.md) |
| 参照整理 | travel-ledger-responsibilities、note-model 等からのリンク |

### 実施しない

```text
DB migration
Reservation entity 実装
CLI 追加
Export / import schema 変更
Markdown export 変更
Sample 更新
```

---

## 13. Future scope（実装フェーズ）

フィールド・種別・拡張戦略: **[Reservation Entity Design](reservation-entity-design.md)**（v1.12.0）。  
実装計画（DB / CLI / export）: **[Reservation Implementation Plan](reservation-implementation-plan.md)**（v1.13.0）。

以下は **仕様メモ**。バージョン・フィールド設計は別途決定します。

| 項目 | 方針案 |
|---|---|
| Reservation 種別 | flight / hotel / rental_car / restaurant / activity / parking / … |
| 主要フィールド | 予約番号、確認コード、連絡先、URL、check_in/out、便名、座席 |
| CLI | `reservation add/list/show/update/delete`（Itinerary スコープ） |
| Export | schema v4 または v3 拡張（要互換検討） |
| しおり | Trip-level Reservation セクション |
| Venue model | 別仕様書（Itinerary 配下、POI 参照） |
| Routing model | 別仕様書（本書対象外） |

---

## 14. 用語

| 用語 | 意味 |
|---|---|
| **Reservation** | 行動を実行するための予約・確認情報（構造化 entity） |
| **Venue** | 行動の場所・POI（将来） |
| **Routing** | 空間的移動経路（将来・本書対象外） |
| **Remark** | Itinerary 行の短い備考（`itinerary_items.note`） |
| **Travel Ledger** | 行動台帳 — Itinerary 序列 + 紐づく Expense 等 |

---

## 15. 実装参照（現行）

| 概念 | パス / 状態 |
|---|---|
| Reservation | **未実装** — 本仕様のみ |
| Remark | `itinerary_items.note` — `src/itinerary.rs` |
| Expense | `src/expense.rs` |
| Note entity | `src/note.rs` |
| 責務一覧 | [travel-ledger-responsibilities.md](travel-ledger-responsibilities.md) |
