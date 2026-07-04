# Expense Post-Implementation Review（責務整理 — 実装後レビュー）

Caglla.Travel CLI の **Travel Ledger Model** における **Expense** の責務を、**v1.5.0 実装後**（export v3 / canonical sample / stats / export-md 含む）に整理・検証するレビューです。

**v1.22.0 時点: 仕様整理のみ（v1 Hardening 第四弾）。** 本書は実装変更を伴わない。改善候補は §12 に記録する。

| ドキュメント | 役割 |
|---|---|
| [expense-model.md](expense-model.md) (v1.5.0) | 設計草案 + 初回実装 — **上書きしない** |
| [travel-ledger-responsibilities.md](travel-ledger-responsibilities.md) (v1.10.0) | Summary / Remark / Note / Reservation / Expense の横断比較 |
| [reservation-responsibilities-review.md](reservation-responsibilities-review.md) (v1.19.0) | Reservation と Expense の独立性 |
| **本書** (v1.22.0) | **実装後**の責務定義 — Transaction Record Layer、Budget / Estimate 分離 |

関連: [note-post-implementation-review.md](note-post-implementation-review.md) / [summary-post-implementation-review.md](summary-post-implementation-review.md) / [export-schema.md](export-schema.md) / [long-term-version-strategy.md](../long-term-version-strategy.md)

設計系列:

```text
v1.5.0   expense-model.md + DB / CLI
v1.6.0   Export schema v3（nested expenses[]）
v1.7.0   Canonical sample + export-md / stats / duplicate
v1.22.0  Post-Implementation Review  ← this document
```

---

## 1. Goals / Non-goals

### Goals（Expense が担うべきこと）

| 課題 | 解決イメージ |
|---|---|
| **金銭の記録** | Itinerary 文脈で、いくら・何通貨かを残す |
| **台帳としての参照** | Trip / Day / Itinerary から支出を辿れる |
| **集計の一次データ** | `trip stats`、将来 Travel Book の費用セクション |
| **バックアップ・移行** | export v3 `expenses[]`、import roundtrip |
| **精算の土台** | 将来 Participant / Settlement の入力（v1.x は記録のみ） |

### レビュー結論（Goals の核心）

```text
Expense は Travel Ledger の Transaction Record Layer である。
金銭が動いた事実（Actual Money）を Itinerary に紐づけて記録する。
```

v1.x の Expense は **予算・見積・精算結果** ではない。**支出ログ（actual transaction record）** として成立している。

### Non-goals

| 概念 | 理由 | 正しい置き場 |
|---|---|---|
| **旅行予算・見積** | 計画額（未発生） | **Budget**（将来） |
| **精算結果** | 誰が誰にいくら払うか | **Settlement**（将来 v3） |
| **按分・受益者** | 誰の費用かの構造化 | **Beneficiary / Shared Expense**（将来 v3） |
| **予約・確認番号** | 手続き正本 | **Reservation** |
| **自由記述メモ** | 金額なし補足 | **Note** |
| **行内短文** | 旅程表備考 | **Remark** |
| **為替換算** | 表示通貨への変換 | **Exchange / Conversion**（将来） |
| **領収書画像** | メディア | **Photo / Attachment**（将来） |

---

## 2. Expense は何を表現するか

### 候補の整理

| 候補 | v1.x での該当 | 判定 |
|---|---|---|
| **支出** | はい | **正** — ユーザー向けラベルの基本 |
| **予算** | いいえ | Budget は別概念（§3） |
| **見積** | いいえ | Estimate は別概念（§3） |
| **実績** | はい | **正** — canonical sample は旅行後の実績台帳 |
| **精算** | いいえ | Settlement は v3 以降 |

### 定義

```text
Expense is a Transaction Record — a monetary amount tied to an Itinerary,
recording money that was spent (or, in v1.x practice, money the user treats
as an expenditure line in the travel ledger).
```

日本語:

```text
Expense = Transaction Record Layer（金銭トランザクション記録層）
Itinerary に紐づく、金額 + 通貨を持つ支出行。
```

### 既存実装・運用からの根拠

| 根拠 | 内容 |
|---|---|
| **expense-model.md** | 「旅行中に発生した金銭的支出の記録」「支出ログ」 |
| **必須フィールド** | `amount` + `currency` のみ — 金額が本質 |
| **canonical sample** | 49 件の実旅行由来金額、領収書番号・費用区分を `note` に記録、合計 ¥561,780 |
| **travel-ledger** | After trip フェーズで Expense を重視（Before trip は Summary / Reservation） |
| **`paid_by_name`** | 立替者ラベル — 精算ではなく **記録** |

Expense は **説明レイヤー（Note / Summary）でも予約レイヤー（Reservation）でもなく**、**金銭データの正本** である。

---

## 3. 見積（Estimate）との関係

### 論点

旅行計画段階でホテル代・航空券代・レンタカー代を **事前に入力** したくなる場合、Expense に載せるべきか。

### 結論

```text
v1.x の Expense = Actual Money（実績・支出として記録する金額）
見積・予想金額 = Expense ではない（**Estimate / Planned Budget** — [estimate-model.md](estimate-model.md)）
```

| 観点 | 説明 |
|---|---|
| **技術的には入力可能** | CLI は日付・金額のみで add できるため、旅行前でも行は作れる |
| **意味論** | 未発生の予想額を Expense に入れると、stats 合計が **実績ではなく見積** になる |
| **canonical の運用** | 旅行 **後** の清算・台帳検証が主目的 — 見積用途ではない |
| **推奨** | 計画段階の「だいたい ○○ 円」は **Note（Observation）** または将来 **Budget**。確定支出のみ **Expense** |

### 境界例

| 文 | 置き場 |
|---|---|
| ホテルは 1 泊 2 万円くらい（予想） | **Note** または将来 **Budget** |
| ヒルトン瀬底 52,000 円（宿泊費を支払った） | **Expense** |
| 航空券 210,240 円（canonical: NU045 の実績） | **Expense** |

**v1.22 では `is_estimate` フラグ等は導入しない。** 意味論を文書で固定する。

---

## 4. Budget との関係

### Expense と Budget は同じものか

**いいえ。** 別エンティティとして分離する。

```text
Budget   = Planned Money（計画・上限・配分）
Expense  = Actual Money（発生・記録した支出）
```

| | **Budget**（将来） | **Expense**（v1.x 実装） |
|---|---|---|
| **時制** | 旅行前〜中の計画 | 主に旅行中〜後の記録 |
| **目的** | 予算管理・超過警告 | 台帳・集計・精算の入力 |
| **親** | Trip / Day / カテゴリ（想定） | **Itinerary のみ** |
| **精算** | 計画 vs 実績の比較 | 実績の正本 |

将来 Trip 予算・Day 予算・カテゴリ予算を導入しても、Expense 行を流用せず **Budget 専用モデル** とする方が、stats・Travel Book・Settlement の意味が崩れない。

---

## 5. Reservation との関係

[v1.19.0 Reservation Responsibilities Review](reservation-responsibilities-review.md) §4.3 の結論を **再確認・維持** する。

```text
Reservation と Expense は独立。同一 Itinerary に両方存在しうる。
```

### 例: ヒルトン瀬底・予約済み・52,000 円

| 情報 | 置き場 | 例フィールド |
|---|---|---|
| 宿泊予約・確認番号・チェックイン手続き | **Reservation** | `provider_name`, `confirmation_code` |
| 実際に支払った宿泊費 52,000 円 | **Expense** | `amount: 52000`, `currency: JPY` |
| 予約時のやり取りメモ | **Note** | body |
| 行内の「要: パスポート」 | **Remark** | `itinerary_items.note` |

**両方に属するのではなく、責務が異なる別レコード** である。金額の正本は **Expense のみ**。Reservation に amount 列は **持たない**（v1.18 設計どおり）。

### canonical sample との関係

`okinawa_sesoko_2026` は v1.7 時点のデータで **Reservation 行は含まない**（Expense 49 件のみ）。旅行後台帳として Expense が充実している例であり、Reservation 追加後も **共存モデルは変わらない**。

---

## 6. Currency の責務

### v1.x 実装

| 項目 | 実装 | レビュー |
|---|---|---|
| 保存 | `currency TEXT NOT NULL`、3 文字英字、大文字正規化 | **妥当** |
| 未知通貨 | **許可**（`XXX` 等、形式が合えば通す） | **妥当** — 過度な制約を避ける |
| ISO 4217 一覧 | **未実装** | **v1.22 では不要** — `validate_currency_code()` 差し替え余地 |
| amount | 最小通貨単位の **INTEGER** | **妥当** — 浮動小数点非依存 |
| 小数入力 | CLI で通貨別桁数にパース（JPY=0, USD=2 等） | **妥当** |
| 換算 | **なし** | **意図的** — stats は通貨別合計のみ |

### 将来（Participant / Settlement）との整合

| 論点 | 方針 |
|---|---|
| 多通貨 Trip | 1 Expense = 1 通貨のまま維持 |
| 表示通貨 | Settlement / Travel Book 層で換算（Expense 正本は変えない） |
| ISO 4217 厳格化 | Validator 内部差し替えで対応可 — スキーマ変更不要 |
| `paid_by_name` | v2 Participant 導入までの **表示用ラベル** — 精算ロジックは v3 |

現在の Currency 設計は **v2 / v3 への拡張に耐える** と判断する。

---

## 7. Shared Expense / Participant / Settlement との関係

### v1.x で成立しているか

**はい。** 単独旅行・グループ旅行いずれも **支出ログ** として成立する。

| パターン | v1.x の扱い |
|---|---|
| **単独旅行** | `paid_by_name` 省略可。全 Expense が「その人の支出」 |
| **グループ旅行** | `paid_by_name` で立替者を **文字列記録**（canonical: Alex / Jordan） |
| **割り勘・精算** | **未実装** — 記録のみ、誰の負担かはモデル化しない |

### 将来拡張との境界

```text
v2  Participant        — Trip 参加者の正規参照
v3  Paid By / Beneficiary / Settlement — Expense 行への構造化紐づけ + 精算
```

| 将来概念 | Expense との関係 |
|---|---|
| **payer** | `paid_by_participant_id`（v2 列追加想定） |
| **beneficiary** | `expense_beneficiaries` 中間テーブル（v3） |
| **settlement** | Expense を **入力** とする計算結果エンティティ（Expense そのものではない） |

[expense-model.md](expense-model.md) §4–§5 の移行方針は **有効なまま**。v1.22 は **Itinerary 配下・1 行 = 1 支出** 原則を維持する。

---

## 8. Travel Book との関係

Expense は Travel Book 財務セクションの **一次データ（正本）** として成立する。

### 現行の集約経路

| 機能 | 内容 |
|---|---|
| **`trip stats`** | `expense_count`、通貨別 `expense_totals`（換算なし） |
| **`trip export-md`** | Itinerary 直下に `Expenses:` 一覧 |
| **export v3** | `days[].itineraries[].expenses[]` |

### 将来の Travel Book

```text
総額 / カテゴリ別 / 日別支出
  ← Expense 一次データ + Itinerary category / Day
  ← 必要なら換算は Travel Book Generator 層
```

[summary-post-implementation-review.md](summary-post-implementation-review.md) / [note-post-implementation-review.md](note-post-implementation-review.md) と同様、Travel Book は **並列ソースの編集・集約**:

```text
Summary, Note, Journal, Photo, Reservation, Expense, Itinerary …
        ↓
Travel Book
```

Expense は Abstract（Summary）でも Annotation（Note）でもなく、**数値正本の Transaction Record** である。

---

## 9. Travel Ledger におけるレイヤー位置づけ

### 説明系レイヤー（v1.20 / v1.21）

```text
Remark         = Inline Annotation   （Itinerary 行内短文）
Note           = Annotation Layer    （対象への補足、Narrative なし）
Summary        = Abstract Layer      （Trip/Day 俯瞰要旨）
Travel Journal = Story Layer         （体験の語り、将来）
```

### 記録系レイヤー（本レビュー）

```text
Reservation    = Booking Record Layer     （予約・確認の構造化正本）
Expense        = Transaction Record Layer （金銭の支出正本）
```

| レイヤー | 必須情報 | 件数 | 例 |
|---|---|---|---|
| **Reservation** | 予約種別・提供者・確認情報 | 0..N / Itinerary | ヒルトン、確認番号 ABC |
| **Expense** | **amount + currency** | 0..N / Itinerary | ¥52,000 JPY |

### Fact / Record / Transaction のどれか

| ラベル | 評価 |
|---|---|
| **Fact** | Note 領域と混同しやすい — **不採用** |
| **Record** | 広すぎる（Reservation も record）— 単独ラベルとしては弱い |
| **Transaction** | **採用** — 金銭の動きを表す点で最も自然 |

**正式ラベル: Transaction Record Layer**（略: Transaction Record）。

---

## 10. v1.5+ 実装との整合確認

### 結論

```text
Expense を Transaction Record Layer（Actual Money）と定義しても、
v1.5.0〜v1.7.0 実装は破綻していない。
```

### DB

| 項目 | 実装 | レビュー |
|---|---|---|
| `expenses` テーブル | `itinerary_id`, `amount`, `currency`, … | **十分** |
| Trip / Day 直下 | なし | **意図的** — Itinerary アンカー維持 |
| `paid_by_participant_id` | なし | **v1.22 では不要** |
| Budget / Estimate 列 | なし | **不要** |

### CLI

| コマンド | レビュー |
|---|---|
| `expense add --itinerary` | **維持** — 正本の追加経路 |
| `expense list --trip` / `--itinerary` | **維持** — 集約表示 |
| `expense update` / `delete` | **維持** |

### ダミー Itinerary 運用

Trip 共通経費はユーザーが `itinerary add` で「その他経費」等を作成する運用（[expense-model.md](expense-model.md)）— **維持**。自動作成はしない。

---

## 11. Export / Import / Markdown / Diff / Stats レビュー

### Export / Import（schema v3）

```json
{
  "days": [{
    "itineraries": [{
      "expenses": [{
        "amount": 2200,
        "currency": "JPY",
        "paid_by_name": "太郎",
        "expense_date": "2026-04-27"
      }]
    }]
  }]
}
```

| 論点 | 判定 |
|---|---|
| nested `expenses[]` | **維持** — Itinerary 親子が正 |
| 内部 ID 非 export | **維持** |
| import roundtrip / duplicate | **維持**（v1.7+） |
| `validate-export` | currency 必須・形式 — **維持** |

**Export Schema 変更: v1.22 では不要。**

### Markdown export

| 項目 | 判定 |
|---|---|
| Itinerary 下 `Expenses:` | **維持** — Travel Book 素材 |
| 通貨別 Trip 合計 | export-md には未掲載 — stats / 将来 Book で対応 |

### Diff

| 項目 | 判定 |
|---|---|
| `trip diff` Expense 比較 | **v1.x 現状: 未実装**（[expense-model.md](expense-model.md) §9 に候補記載） |
| 責務再定義との関係 | 未実装は **改善候補** であり、定義変更を要求しない |

### Stats

| 項目 | 判定 |
|---|---|
| 通貨別合計（換算なし） | **Actual Money の集計として妥当** |
| 見積との混在 | 運用で Expense に見積を入れないことで回避 — §3 |

---

## 12. 改善候補（v1.22 では実装しない）

| # | 候補 | 種別 |
|---|---|---|
| 1 | `trip diff` — Expense 比較（itinerary key + sort_order） | diff |
| 2 | **Budget** entity 設計（Trip / Day / category） | 将来 |
| 3 | **Estimate** と Expense の分離フラグ（`is_planned` 等）— Budget 導入時に再検討 | 将来 |
| 4 | `paid_by_participant_id`（v2 Participant） | DB / CLI |
| 5 | `expense_beneficiaries` + Settlement（v3） | 精算 |
| 6 | Trip 表示通貨・換算レート | Conversion |
| 7 | export-md Trip 合計行・カテゴリ別集計 | Travel Book |
| 8 | Receipt Photo 添付 | Photo |
| 9 | ISO 4217 厳格検証（`validate_currency_code` 差し替え） | validation |
| 10 | canonical sample への Reservation + Expense 共存例 | sample |

---

## 13. expense-model から明確化したこと

[expense-model.md](expense-model.md) は **設計履歴として残す**。本書で **文言を強化した解釈**:

| トピック | expense-model（既存） | v1.22（精緻化） |
|---|---|---|
| **Expense の性質** | 支出ログ | **Transaction Record Layer** / **Actual Money** |
| **見積・予算** | 非対象（暗黙） | **Budget / Estimate は別エンティティ** と明示 |
| **Reservation** | 独立（travel-ledger 経由） | **金額正本は Expense のみ** を再確認 |
| **Travel Book** | stats / export-md 言及 | **一次データ** として位置づけ |
| **三層モデルとの関係** | 未記載 | Note / Summary / Journal とは **別系統の記録層** |

### 変わらないもの

- Itinerary 配下のみ（Trip / Day 直下なし）
- `amount` + `currency` 必須
- 換算・Settlement 非対象（v1.x）
- export v3 nested 構造
- `paid_by_name` のみ（Participant 列なし）

---

## 14. v1.22.0 スコープ（本書）

### 実施する

| 項目 | 内容 |
|---|---|
| 仕様書 | 本ドキュメント |
| 索引 | [specifications/README.md](README.md) |
| 参照更新 | travel-ledger-responsibilities、expense-model、long-term-version-strategy |
| v1 Hardening | Expense 実装後責務定義（第四弾） |

### 実施しない

```text
DB migration / schema 変更
CLI 変更
export / import schema 変更
Markdown / diff / stats 実装変更
Budget / Estimate / Settlement 設計の実装
canonical sample 更新
テスト追加
expense-model.md の上書き
```

---

## 15. 用語

| 用語 | 意味 |
|---|---|
| **Transaction Record** | Expense の性質 — 金銭が動いた記録（Actual Money） |
| **Actual Money** | 実績として記録する金額 — v1.x Expense の意味論 |
| **Planned Money** | 予算・見積 — 将来 Budget の領域 |
| **Booking Record** | Reservation の性質 — 予約・確認（金額正本ではない） |
| **Settlement** | 精算結果 — Expense の計算出力（将来 v3） |

---

## 16. 実装参照（v1.5.0+）

| 領域 | パス |
|---|---|
| CRUD / currency / amount | `src/expense.rs` |
| Models | `src/models.rs`（`Expense`, `ExportExpenseV3`） |
| export / import | `src/trip.rs` |
| Stats | `src/stats.rs` |
| Markdown | `src/markdown.rs` |
| 統合テスト | `tests/expense_cli.rs`, `tests/export_roundtrip_cli.rs`, `tests/okinawa_sesoko_seed_cli.rs` |
| Canonical sample | `samples/okinawa_sesoko_2026/` |
| 設計草案 | [expense-model.md](expense-model.md) (v1.5.0) |
