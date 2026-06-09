# Travel Ledger Responsibilities — Summary / Remark / Note / Reservation

Caglla.Travel CLI の **Travel Ledger Model** における、説明・メモ・予約情報の責務分離です。

関連: [Itinerary モデル](itinerary-model.md) / [Note モデル](note-model.md) / [Expense モデル](expense-model.md) / [Export Schema](export-schema.md) / [Ordering モデル](ordering-model.md) / [Summary Responsibilities Review](summary-responsibilities-review.md)

**背景（v1.9.0 時点）:**

```text
v1.8.0  Itinerary is not a venue.
v1.8.1  Itinerary is a unit of travel activity.
v1.9.0  Travel activity ordering is sequence-first.
```

Caglla.Travel は Calendar Model ではなく **Travel Ledger Model** として扱います。基本構造は次のとおりです。

```text
Trip
 └ Day
      └ Itinerary
           └ Expense
```

Note / Summary / Reservation / Checklist / Photo / Attachment の責務を整理し、**v1.x では文書化を優先**、実装は段階的に進めます。

---

## 1. 責務一覧（比較表）

| Concept | Japanese label | Responsibility | v1.x scope |
|---|---|---|---|
| **Trip Summary** | 旅行の概要 | 旅行全体の共有向け説明（同行者・しおり・一覧） | 仕様のみ / 将来実装 |
| **Day Summary** | この日の概要 / 主な行先 | 日別の共有向け要約（印刷・スキャン向き） | 仕様のみ / 将来実装 |
| **Itinerary Remark** | 備考 | 個別行動の短い補足（旅程表の行に載せる） | **既存** `itinerary_items.note` |
| **Note entity** | メモ / 詳細メモ | 自由記述、検討、記録、振り返り（長文・複数件可） | **既存 CRUD**、責務は本書で再整理 |
| **Reservation** | 予約情報 | 予約・確認・手続きに必要な構造化情報 | 仕様のみ / 将来 entity |
| **Expense** | （支出） | 金額・通貨・領収書（Itinerary 配下） | 実装済み — [expense-model.md](expense-model.md) |
| **Checklist** | チェックリスト | 準備・忘れ物防止（Trip 配下） | 実装済み — 将来設計: [checklist-design-memo.md](checklist-design-memo.md)、[travel-support-design-memo.md](travel-support-design-memo.md) |

英語上の責務:

```text
Summary:  Readable overview for sharing, printing, and scanning.
Remark:   Short inline supplement attached to one itinerary item.
Note:     Long-form free text for context, planning, reflection, or records.
Reservation: Structured booking / confirmation data to execute an activity.
```

日本語 UI の最小セット: **概要** / **備考** / **メモ**

---

## 2. Summary と Note は分ける

詳細な責務整理: **[Summary Responsibilities Review](summary-responsibilities-review.md)**（v1.14.0）。

これまで「Trip Note」「Day Note」と呼んでいた用途の一部は、実際には **Summary** と呼ぶ方が自然です。

### 2.1 Trip Summary / Description

**What it is:** 旅行全体を一言〜数行で説明する **共有・印刷・一覧表示向けの要約**。

**What it is not:** 長文の検討メモ、予約番号一覧、清算メモ（それらは Note / Reservation の領域）。

**Examples:**

```text
GWちょっと手前で行くことで、飛行機の料金を格安に抑える。
夏前の過ごしやすい沖縄４日間。
```

**GUI ラベル候補:** 旅行の概要 / 旅行説明 / 概要

**v1.x:** フィールド・CLI 未実装。仕様として必要性を明文化。

**Future:** Trip 表紙、一覧、共有 PDF、しおり export の冒頭セクション。

### 2.2 Day Summary

**What it is:** その日の主な行先・過ごし方を **短く要約** するもの。長文 Note ではない。

**Examples（計画共有資料より）:**

```text
■ 主な行先

Day 1  首里城、瀬底島（瀬底大橋）
Day 2  海洋博公園（美ら海水族館、ドリームセンター）、古宇利島、沖縄ハナサキマルシェ
Day 3  伊江島（リリーフィールド、ハイビスカス園、城山、湧出、ニャティア洞）、瀬底ビーチ
Day 4  御菓子御殿、万座毛、美浜アメリカンビレッジ
```

**GUI ラベル候補:** この日の概要 / 主な行先 / Day Summary

**v1.x:** フィールド・CLI 未実装。

**Future:** 旅程表の Day 見出し直下、A4 印刷、同行者共有。

Day Summary は詳細旅程を読まなくても「その日がどんな日か」を把握できる **スキャン向け情報** です。

---

## 3. Itinerary Remark（`itinerary_items.note`）

### What it is

Itinerary 1 件に付随する **短い補足**。旅程表の **備考欄** に相当します。

- 1 Itinerary : 0 or 1 remark
- 短文
- timeline / list / export の行表示に載せても邪魔になりにくい

### What it is not

- メモ帳（長文・複数件）→ **Note entity**
- 予約番号・連絡先の構造化保管 → **Reservation**（背景説明は Remark 可）
- 金額・領収書 → **Expense**

### Examples

```text
予約番号
注意事項
営業時間メモ
駐車場メモ
集合場所
電話番号
要: ETCカード
チェックアウトリミット：10:00
一人1500円程度
```

### GUI ラベル

**備考**（推奨）。ドキュメント内部では `remark` / `short note field` と表記してもよい。

### v1.x scope

**既存。** `itinerary add` / `itinerary update` の `--note`、`itinerary_items.note` 列、export v3 の itinerary `note` フィールド。変更なし。

### Future scope

Remark のまま維持。Long-form へ昇格させる自動変換は行わない。

---

## 4. Note entity（Long-form Note）

### What it is

**自由記述・補足・記録・検討・振り返り** 向けの長文メモ。Trip / Day / Itinerary に **0..N 件** 付与可能。

**Examples:**

```text
旅行全体の検討メモ
日別の振り返り
行動単位の詳細記録
雨天時の代替案
次回改善
旅行後の感想
予約時のやり取りメモ（背景・経緯）
```

**性質:**

```text
1 target : 0..N notes
長文可
詳細画面・専用表示向き
旅行前・旅行中・旅行後の文脈を持てる
```

### What it is not

- 共有向けの一行要約 → **Summary**
- 旅程表行の短文 → **Remark**（`itinerary_items.note`）
- 予約番号・チェックイン手順の正本 → **Reservation**

### v1.x scope

- **CLI CRUD 実装済み**（[note-model.md](note-model.md)）
- Export schema v2+ の `notes[]`
- 本書での **責務再整理**（Summary との混同を避ける）
- 初期 GUI では Trip/Day/Itinerary ごとに「概要 + 備考 + メモ」を同時に並べず、以下の割り切りを推奨:

```text
Trip
 ├─ 概要        ← Trip Summary（将来）
 └─ メモ        ← Note entity

Day
 └─ 概要        ← Day Summary（将来）

Itinerary
 └─ 備考        ← Remark（既存 note 列）
```

Long-form Note は Photo / Attachment / Reservation / Checklist との整理が進んだあと、**詳細情報エリア** として拡張する。

### Future scope

- Itinerary-level Note の export-md / しおりへの組み込み
- Photo / Attachment との連携
- 旅行前（plan）と旅行後（reflect）の表示モード分岐

---

## 5. Reservation

詳細仕様: **[Reservation モデル](reservation-model.md)**（v1.11.0 — 責務・境界）、**[Entity Design](reservation-entity-design.md)**（v1.12.0 — フィールド）、**[Implementation Plan](reservation-implementation-plan.md)**（v1.13.0 — 実装計画）。

### What it is

**Itinerary を実行可能にするための予約・確認・手続き情報。** Note の一種ではなく、**構造化 entity** として扱う。

```text
Reservation
- belongs to Itinerary
- represents booking / confirmation information required to execute the travel activity
- can be aggregated at Trip level for guidebook / itinerary booklet export
- is not a Note
- is not an Expense
```

**Examples:**

```text
宿泊予約（ヒルトン沖縄瀬底リゾート）
駐車場予約（セントレア駐車場）
レンタカー（Ks Rent A Car）
航空券（NU045 / NU046）
レストラン（AMAHAJI）
```

計画共有資料の「予約情報」セクション例:

```text
宿泊: ヒルトン沖縄瀬底リゾート
駐車場: セントレア駐車場
レンタカー: Ks Rent A Car
航空券: NU045 / NU046
食事: AMAHAJI
```

しおりに必要な情報の例:

```text
どこに行くか
何時ごろ行くか
いくらぐらいか
何を忘れてはいけないか
予約番号は何か
連絡先はどこか
当日どう手続きするか
```

### What it is not

- 自由記述メモ → **Note**（予約の背景・経緯は Note でよい）
- 金額・領収書 → **Expense**
- 短文の行内補足 → **Remark**（「要: ETCカード」など）

### 保存と表示（二層構え）

```text
保存:  Itinerary に紐づける
表示:  Trip しおりで Reservation 一覧として集約表示する
```

旧 Caglla.Travel Web でも Reservation は Itinerary 配下で、Trip レベル集約表示がありました。この方針を継承します。

### Reservation を Note にしない理由

Note に押し込むと次が困難になります。

```text
予約番号を探す
電話番号だけ一覧する
チェックイン時刻を出す
航空券だけまとめる
未予約の予定を doctor で検出する
しおりに予約情報セクションを出す
```

### v1.x scope

**仕様のみ。** DB / CLI / export schema 変更なし。一部の予約情報は現状 **Remark** や **Checklist** に分散していることを許容する。

### Future scope

- Reservation entity（種別、予約番号、連絡先、確認 URL、チェックイン時刻など）
- Trip-level 集約表示・しおり export の Reservation セクション
- `trip doctor` による未予約検出

---

## 6. 将来の概念モデル（全体像）

長期的な構造の目標像:

```text
Trip
 ├─ summary / description     ← Trip Summary
 ├─ Note[]
 ├─ Checklist
 └─ Day
      ├─ summary              ← Day Summary
      ├─ Note[]
      └─ Itinerary
           ├─ Expense
           ├─ Reservation
           ├─ Checklist
           ├─ Note[]
           ├─ Photo / Attachment
           └─ remark          ← itinerary_items.note
```

**v1.x で全部実装する必要はない。** 責務整理とドキュメント化を優先する。

---

## 7. v1.x と Future の境界

### v1.x（今回のドキュメント整備）

| 項目 | 内容 |
|---|---|
| 文書化 | Summary / Remark / Note / Reservation の責務 |
| Remark | `itinerary_items.note` を「備考」として位置付け（既存のまま） |
| Summary | Trip / Day Summary の必要性を仕様として明文化 |
| Note entity | 将来拡張を含め責務を再整理（CRUD は既存） |
| Reservation | Note ではない将来 entity として整理 |
| canonical sample | Note 大量投入を省略する理由を明文化 |
| **変更しない** | DB schema / CLI コマンド / export schema |

### v2+ / Future

```text
- Trip summary / description の実装
- Day summary の実装
- Long-form Note entity の階層・表示強化
- Itinerary-level Note のしおり組み込み
- Photo / Attachment
- Itinerary-level Checklist
- Reservation entity
- Reservation の Trip-level 集約表示
- しおり export への Reservation section 追加
```

---

## 8. canonical sample との関係

[`samples/okinawa_sesoko_2026/`](../../samples/okinawa_sesoko_2026/) は実旅行データ由来の **行動台帳 + 清算** の検証用データです。

| 項目 | 値 |
|---|---|
| 期間 | 2026-04-26 〜 2026-04-29 |
| Itinerary | 58 件 |
| Expense | 49 件 |
| 合計 | ¥561,780 |

### 旅行前 vs 旅行後

同じ旅行でも、表示したい情報はフェーズで異なります。

| フェーズ | 目的 | 重視する概念 |
|---|---|---|
| **Before trip** | 計画共有・しおり・同行者説明 | Trip/Day **Summary**、**Reservation** 集約、Remark、Checklist |
| **After trip** | 実績・清算共有 | Itinerary 序列、Expense、Remark（領収書番号など） |

canonical sample は主に **After trip（台帳・export roundtrip）** を検証します。そのため:

- 備考の多くは **Itinerary Remark**（`note`）と **Expense `note`** に集約
- **Note エンティティの大量投入は意図的に省略**（export / CLI の安定検証を優先）
- Trip/Day Summary・Reservation セクションは **将来のしおり生成** の動機として本仕様に記載

計画共有資料（旅行前）にあった「旅行の狙い」「主な行先」「予約情報」は、本仕様の **Summary / Reservation** 整理の実例として参照してください。

---

## 9. 用語

| 用語 | 意味 |
|---|---|
| **Travel Ledger** | 行動台帳。Sequence-first の Itinerary 列 + 紐づく Expense 等 |
| **Summary** | 共有・印刷向けの要約（Trip / Day） |
| **Remark** | Itinerary 行の短い備考（`itinerary_items.note`） |
| **Note entity** | 長文・複数件の自由記述エンティティ |
| **Reservation** | 実行に必要な予約・確認情報（将来の構造化 entity） |

---

## 10. 実装参照（現行）

| 概念 | パス / コマンド |
|---|---|
| Remark | `itinerary_items.note` — `src/itinerary.rs` |
| Note entity | `src/note.rs` — `note add/list/show/update/delete` |
| Checklist | `src/checklist.rs` |
| Expense | `src/expense.rs` |
| Export notes | `export-schema.md` — `notes[]` |
| Summary | **未実装** — [summary-responsibilities-review.md](summary-responsibilities-review.md) |
| Reservation | **未実装** — [reservation-model.md](reservation-model.md) 系列 |
