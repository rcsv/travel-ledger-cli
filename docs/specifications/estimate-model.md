# Estimate / Planned Budget Model Responsibilities Review

Caglla.Travel CLI / 将来 Web 版に向けた **Estimate（事前見積 / Planned Budget）** の責務整理です。

**Responsibilities Review。** 責務・境界・用語の整理が本書の主目的です。DDL・CLI・export の詳細正本は [estimate-entity-design.md](estimate-entity-design.md) / [estimate-implementation-plan.md](estimate-implementation-plan.md) を参照してください。

| ドキュメント | 役割 |
|---|---|
| **本書** | Estimate / Planned Budget の責務・境界・スコープ |
| [estimate-entity-design.md](estimate-entity-design.md) | テーブル・フィールド・CLI・export v6（Entity Design — Phase 1–4 実装済み） |
| [estimate-implementation-plan.md](estimate-implementation-plan.md) | 実装計画（Phase 分割・進捗管理） |
| [estimate-post-implementation-review.md](estimate-post-implementation-review.md) | 実装後レビュー（Phase 5 — 実装済み） |
| [expense-model.md](expense-model.md) (v1.5.0) | Expense = Transaction Record Layer（設計履歴） |
| [expense-post-implementation-review.md](expense-post-implementation-review.md) (v1.22.0) | Expense = Actual Money。Estimate 分離の既存結論 |
| [itinerary-model.md](itinerary-model.md) (v1.8.0+) | Itinerary = 行動単位。子エンティティの親 |
| [planning-design-principles.md](planning-design-principles.md) (v2.0.1) | 入力過多を避ける判断軸 |
| [long-term-version-strategy.md](../long-term-version-strategy.md) | 製品ロードマップ |

関連: [travel-ledger-responsibilities.md](travel-ledger-responsibilities.md) / [reservation-model.md](reservation-model.md) / [export-schema.md](export-schema.md) / [ordering-model.md](ordering-model.md)

設計系列（想定）:

```text
Responsibilities Review   → estimate-model.md（本書）
Entity Design             → estimate-entity-design.md
Implementation Plan       → estimate-implementation-plan.md
Implementation            → Phase 1–4 実装済み
                             Phase 5 完了（Post-Implementation Review）
Post-Implementation Review → estimate-post-implementation-review.md
```

---

## Purpose

旅行 **前** の計画共有で、同行者に「だいたいいくらかかりそうか」を伝え、Trip 単位で **概算合計** を把握できるようにする。

```text
結局この旅行はいくらくらいかかるんだっけ？
```

現状、Itinerary の `note` に金額を書く運用では **集計できない**。Expense に事前入力すると **実績と混ざり**、`trip stats` 等の意味が崩れる（[expense-post-implementation-review.md §3](expense-post-implementation-review.md#3-見積estimateとの関係)）。

Estimate / Planned Budget は **Planned Money（計画・見積）** の正本とし、Expense（Actual Money）とは **別エンティティ** として設計する。

---

## Background

### きっかけ

`itinerary replicate`（master 済み）により、複数日に繰り返す定型予定（ホテル朝食・出発・帰館・ラウンジ夕食など）を効率よく複製できるようになった。

一方、旅行前の共有では次のような **見積** が必要になる。

```text
ホテル朝食はだいたい 5人で 14,000円
レンタカー代はこのくらい
水族館チケットはこのくらい
夕食はこのくらい
```

### 現状の Travel Ledger（金銭まわり）

```text
Trip
 └─ Day
      └─ Itinerary
           ├─ Expense        ← 実装済み（Actual Money）
           ├─ Reservation    ← 実装済み（予約・確認。amount 列なし）
           └─ Note           ← 実装済み（金額なし自由記述）
```

| 手段 | 限界 |
|---|---|
| Itinerary `note` | 集計不可、通貨・構造なし |
| Note エンティティ | 同上 — Annotation Layer |
| Expense | **意味論上は実績**。計画段階の入力は stats / 台帳を汚す |
| Reservation | 予約手続き正本。金額の正本ではない |

### Expense 側の既存結論（維持）

[expense-post-implementation-review.md](expense-post-implementation-review.md) v1.22.0:

```text
Budget   = Planned Money（計画・上限・配分）— 将来
Expense  = Actual Money（発生・記録した支出）
Estimate = 本書で扱う Planned Money の **Itinerary 配下の見積行**
```

Expense 行を流用せず、**Estimate 専用モデル** とする方針は v1.22 から変えない。

---

## Terminology

| 用語 | 意味 | 本書での扱い |
|---|---|---|
| **Estimate** | Itinerary に紐づく **事前見積金額** | **推奨エンティティ名・CLI 接頭辞** |
| **Planned Budget** | 計画段階の金銭全般（Trip 上限含む） | Estimate 明細の **集計結果** として扱う。**独立エンティティではない** — 詳細は [estimate-entity-design.md §Estimate line items](estimate-entity-design.md#estimate-line-items--itinerary-配下の-0n-明細) |
| **Planned Expense** | — | **採用しない** — Expense（実績）と混同しやすい |
| **Budget Item** | — | 会計・予算管理寄り。v1 系スコープ外 |

ドキュメント上は **Estimate / Planned Budget** と併記する。実装済みのテーブル名・CLI:

```text
テーブル: estimates
CLI:      estimate add / list / show / update / delete
```

---

## Goals / Non-goals

### Goals

| 課題 | 解決イメージ |
|---|---|
| **旅行前の概算共有** | Itinerary ごとに「見込み ○○ 円」を構造化して記録 |
| **Trip 単位の Planned total** | Estimate 合計を集計（**Phase 3 実装済み** — `trip stats` / `export-md`） |
| **Planned vs Actual** | Expense 合計と並べて表示（**Phase 3 実装済み** — Difference 計算は未実装） |
| **replicate 連携** | 定型予定複製時に **見積も一緒に持ち運ぶ**（**Phase 4 実装済み**） |

### Non-goals（現時点で対象外）

| 概念 | 理由 |
|---|---|
| Trip / Day 直下 Estimate | 行動単位 Itinerary に寄せる（Expense と同型） |
| payer / beneficiary / 按分 | Shared Expense 領域。Estimate は **誰が払うか** より **いくら見込むか** |
| unit_amount × quantity | 会計システム化。初期は **1 行 = 1 見積総額** |
| 為替換算 | Expense と同様 v1 系非対象 |
| 領収書・精算 | Expense / Settlement |
| Expense との自動差分フィールド | 集計レイヤーで導出。専用列は持たない |
| tax / service charge 内訳 | 初期不要 |

---

## Entity placement

### 親子関係（第一候補）

```text
Trip
 └─ Day
      └─ Itinerary
           ├─ Estimate        ← 実装済み（Planned Money / 予定費用）
           ├─ Expense           ← Actual Money
           ├─ Reservation
           └─ Note
```

| 階層 | Estimate |
|---|---|
| Trip 直下 | **なし**（初期） |
| Day 直下 | **なし**（初期） |
| Itinerary 配下 | **あり** — `estimates.itinerary_id` |

Itinerary は「旅行中の行動」であり、**その行動に見込む費用** を Estimate として載せるのが自然（[itinerary-model.md](itinerary-model.md)）。

**1 Itinerary : N Estimate** を許容する（例: 朝食代 + ドリンク代を分けて見積）。Expense と同様、複数行は日常的。

1 行の Estimate は **1 つの見込み項目**（入館料、駐車場など）を表す。Itinerary 全体の予定合計（Planned subtotal）は **明細の集計** であり、Itinerary 上の単一 `planned_amount` フィールドにはしない。水族館・レンタカー・ホテルなど、1 行動に複数の予定費用が付く例は [estimate-entity-design.md §Estimate line items](estimate-entity-design.md#estimate-line-items--itinerary-配下の-0n-明細) を参照。

---

## Estimate vs Expense

### 比較表

| 観点 | Estimate / Planned Budget | Expense |
|---|---|---|
| **時点** | 旅行前・計画時 | 旅行中・旅行後 |
| **意味** | 見積・予定金額 | 実績支出 |
| **用途** | 予算共有・概算把握 | 記録・精算・振り返り |
| **Travel Ledger 層** | **Planned Money**（実装済み） | **Transaction Record / Actual Money** |
| **支払者** | 原則不要（将来検討可） | `paid_by_name` / `paid_by_participant_id` |
| **負担者** | 原則不要（将来検討可） | Shared Expense（beneficiaries） |
| **`trip stats` 合計** | **Planned total**（Phase 3 実装済み） | **Actual total**（現行） |
| **replicate** | **コピーする**（Phase 4 実装済み） | コピーしない |
| **Reservation** | 予約情報とは独立 | 実績金額の正本。Reservation に amount なし |

### 境界例

| 文 | 置き場 |
|---|---|
| ホテル朝食、5人でだいたい 14,000円（旅行前の共有） | **Estimate** |
| 朝食 13,750円をレストランで支払った | **Expense** |
| ホテル予約確認番号 ABC123 | **Reservation** |
| 朝食は 7:00 から（時刻メモ） | Itinerary `start_time` または **Note** |
| 朝食会場はロビー（非金額） | Itinerary `note` または **Note** |

同一 Itinerary に **Estimate + Expense + Reservation** が **共存** しうる。金額の **計画正本** は Estimate、**実績正本** は Expense。

---

## Estimate vs Note / Remark

| | Estimate | Note | Itinerary `note` |
|---|---|---|---|
| **金額** | **必須**（amount + currency） | なし | なし |
| **集計** | 可 | 不可 | 不可 |
| **責務** | Planned Money | Annotation | Remark（短い備考） |

「14,000円くらい」は Estimate。「アレルギー確認」は Note。

---

## Initial model sketch（設計時の最小案 — 実装済み）

設計段階の第一候補。実装 DDL は [estimate-entity-design.md](estimate-entity-design.md) が正本。

```sql
CREATE TABLE estimates (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    itinerary_id    INTEGER NOT NULL,
    title           TEXT,
    amount          INTEGER NOT NULL,
    currency        TEXT NOT NULL,
    note            TEXT,
    sort_order      INTEGER NOT NULL DEFAULT 0,
    created_at      TEXT NOT NULL,
    updated_at      TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_estimates_itinerary
    ON estimates(itinerary_id);
```

| カラム | 必須 | 説明 |
|---|---|---|
| `itinerary_id` | ✓ | 親 Itinerary |
| `title` | — | 項目名（例: `ホテル朝食`）。省略可 |
| `amount` | ✓ | **最小通貨単位** INTEGER（Expense と同型） |
| `currency` | ✓ | ISO 4217（例: `JPY`, `USD`） |
| `note` | — | 補足（例: `5人分`） |
| `sort_order` | ✓ | 同一 Itinerary 内の並び |

### amount の表現

Expense と同じ方針を踏襲する（[expense-model.md §amount](expense-model.md)）。

| 通貨 | 例 | DB 値 |
|---|---|---|
| JPY | 14,000 円 | `14000` |
| USD | 12.50 ドル | `1250`（セント） |

初期実装では **1 Estimate 行 = 1 見込み項目の総額**（`amount` + `currency`）を持つ。Itinerary あたり **複数行** を許容し、Planned Budget（Trip / Itinerary 単位の予定合計）は **行の集計** として導出する。以下は **対象外**:

```text
unit_amount, quantity, participant_id, payer, beneficiaries,
tax, service_charge, planned_vs_actual_delta
```

---

## CLI（実装済み）

```bash
caglla estimate add --itinerary 12 --amount 14000 --currency JPY --title "ホテル朝食"
caglla estimate list --itinerary 12
caglla estimate list --trip 1
caglla estimate show 3
caglla estimate update 3 --amount 15000
caglla estimate delete 3
```

| ルール | 内容 |
|---|---|
| 親 | **Itinerary のみ**（Expense と同型） |
| `--amount` / `--currency` | 必須 |
| Shared Expense 系オプション | **なし** |

詳細: [command-reference.md](../command-reference.md)

---

## Aggregation（Phase 3 実装済み / Trip Difference v3.3.0 / Itinerary Difference v3.4.0）

### Trip 単位（`trip stats` / `export-md` Overview）

```text
Planned total:
  JPY 180,000

Actual total:
  JPY 172,500

Difference:
  -7,500          ← v3.3.0 実装済み（derived 集計）
```

- **Planned total** = Trip 配下 Estimate の合計（通貨別） — **実装済み**
- **Actual total** = `trip stats` / `export-md` の Expense 合計 — **実装済み**
- **Difference** = `Actual − Planned`（通貨別、derived） — **v3.3.0 実装済み**

### Itinerary カード（export-md — v3.4.0 実装済み）

旅行前（Estimate のみ）:

```text
ホテルで朝食
予定費用: （明細表）
```

旅行後（Estimate + Expense あり）:

```text
ホテルで朝食
予定費用: （明細表）
Expenses: （明細）
Planned total:
  JPY 14,000
Actual total:
  JPY 13,750
Difference:
  JPY -250
```

Itinerary に紐づく Estimate 行の合計 vs Expense 行の合計を比較（1 Itinerary : N 行の場合は合算）。**gate:** 当該 Itinerary 内に Estimate と Expense が両方ある場合のみ Difference サマリーを表示。

---

## Relationship to `itinerary replicate`

[itinerary replicate](itinerary-model.md#14-itinerary-の複製itinerary-replicate)（Phase 4 実装済み）:

| コピーする | コピーしない |
|---|---|
| Itinerary 本体、Itinerary-level notes、**Estimate（予定費用）** | Expense（実績支出）、Reservation（予約実体） |

Estimate は予定費用なのでコピーする。Expense は実績支出なのでコピーしない。Reservation は予約実体なのでコピーしない。

定型パターン（朝食・出発・帰館・夕食）を複数日に撒く際、**見積も一緒に持ち運ぶ** のが自然。`copy_estimates_for_itinerary` で `title` / `amount` / `currency` / `note` / `sort_order` を維持し、新 ID で INSERT する。

---

## Relationship to Reservation

[reservation-model.md](reservation-model.md) / [expense-post-implementation-review.md §5](expense-post-implementation-review.md):

- Reservation = 予約・確認の構造化情報（**amount 列なし**）
- Expense = 実績金額の正本
- **Estimate = 計画金額の正本**（実装済み）

例: 水族館 — Reservation に確認番号、Estimate に「チケット代 見込 8,000円」、旅行後 Expense に実際の支払額。

---

## Open questions（確定済み / 残課題）

| # | 論点 | 状態 |
|---|---|---|
| 1 | CLI 名 `estimate` vs `planned-budget` | **`estimate` 採用**（Phase 1） |
| 2 | export schema バージョン | **schema v6** — `days[].itineraries[].estimates[]`（Phase 2） |
| 3 | `trip stats` 拡張 | **Phase 3 実装済み** — Planned total（通貨別） |
| 4 | export-md | **Phase 3 実装済み** — Itinerary 下に予定費用表、Overview に Planned / Actual |
| 5 | `itinerary replicate` | **Phase 4 実装済み** — Estimate デフォルトコピー |
| 6 | Participant 人数との連動 | 初期は `note` に「5人分」等。自動按分は非対象 |
| 7 | Trip 全体予算上限 | Estimate 合計とは別概念。将来 **Budget** エンティティ |

---

## Deferred scope summary

Phase 1–5 は **完了**。現時点で **未実装** の範囲（詳細は [estimate-post-implementation-review.md §9](estimate-post-implementation-review.md#9-deferred-scope)）:

```text
- Difference 計算（Planned vs Actual の差分表示）
- Budget 独立エンティティ（Trip 全体予算上限）
- payer / beneficiary / participant 連動
- unit_amount × quantity
- 為替換算
- --without-estimates（replicate 時に Estimate をコピーしないオプション）
- doctor / advisor での Estimate 活用
- GUI / Web 版での Planned vs Actual 表示
- release 作業
```

次ステップ: **Release**（別 PR）。

---

## References

| 用途 | パス |
|---|---|
| Expense 責務（Actual） | [expense-post-implementation-review.md](expense-post-implementation-review.md) |
| Itinerary 親子 | [itinerary-model.md](itinerary-model.md) |
| replicate 現行 | [itinerary-model.md §14](itinerary-model.md#14-itinerary-の複製itinerary-replicate) |
| 入力過多回避 | [planning-design-principles.md](planning-design-principles.md) |
| ロードマップ | [long-term-version-strategy.md](../long-term-version-strategy.md) |
