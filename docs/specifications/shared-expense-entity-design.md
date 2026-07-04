# Shared Expense Entity Design

Caglla.Travel CLI / 将来 Web 版に向けた **Shared Expense（v3）** の具体設計です。

**v3.0.0 設計フェーズ 2/6: Entity Design のみ。** 本書は DB migration・CLI・export schema の **実装を伴わない**。実装手順は Issue #32 以降。

| ドキュメント | 役割 |
|---|---|
| [shared-expense-model.md](shared-expense-model.md) (#30) | 責務・境界（What it is / is not） |
| **本書** (#31) | テーブル・フィールド・export v5・検証（How we model it） |
| [shared-expense-implementation-plan.md](shared-expense-implementation-plan.md) (#32) | 実装計画（How to build） |
| [shared-expense-post-implementation-review.md](shared-expense-post-implementation-review.md) (#34) | 実装後レビュー（#33 完了後） |
| [participant-entity-design.md](participant-entity-design.md) (v2.0.0) | `participants` 正本 — v3 の参照先 |
| [expense-model.md](expense-model.md) (v1.5.0) | Expense = Transaction Record Layer |

関連: [export-schema.md](export-schema.md) / [planning-design-principles.md](planning-design-principles.md) / [github-workflow.md](../github-workflow.md)

設計系列（Epic #13）:

```text
#30 Responsibilities Review   → shared-expense-model.md
#31 Entity Design             → shared-expense-entity-design.md（本書）
#32 Implementation Plan       → shared-expense-implementation-plan.md
#33 Implementation            → DB + CLI + export v5
#34 Post-Implementation Review → shared-expense-post-implementation-review.md
#35 Release v3.0.0
```

---

## Purpose

[shared-expense-model.md](shared-expense-model.md) で確定した責務を前提に、v3.0.0 で導入する **Expense 拡張・beneficiary 中間テーブル・export schema v5** の保存形を具体化する。

```text
後続の Implementation Plan / 実装が迷わないよう、
DDL 骨格・Participant 参照・cascade・export フィールド・検証ルールを固める。
```

---

## Source Responsibilities Review

[shared-expense-model.md](shared-expense-model.md) から本設計が引き継ぐ **確定前提**:

| 項目 | 前提 |
|---|---|
| **Expense** | Transaction Record のまま拡張。独立 Shared Expense エンティティ **なし** |
| **payer** | optional `paid_by_participant_id` + `paid_by_name` 共存 |
| **beneficiaries** | v3.0.0 は **均等按分のみ**（`share_ratio` / `share_amount` は defer） |
| **personal / shared** | personal デフォルト。shared は **opt-in** |
| **participant 未登録 Trip** | 従来どおり `expense add`（`paid_by_name` のみ） |
| **Settlement** | recording 優先。永続 Settlement / transfer CLI / 消込は **defer** |
| **Itinerary** | 1 Itinerary : N Expense **維持** |

本書は上記を **破らない** 範囲で DDL・export・検証を確定する。

---

## Entity Definition

### What changes in v3.0.0

```text
Expense（既存行）
  + paid_by_participant_id   NULL  → participants.id（論理 FK）
  + expense_beneficiaries[]  0..N  → 均等 split の対象 Participant

Settlement / shared_expenses テーブル  → 導入しない
```

### エンティティ関係（v3.0.0）

```text
Trip
 ├─ Participant[]
 └─ Day
      └─ Itinerary
           └─ Expense
                ├─ paid_by_participant_id?  → Participant
                └─ expense_beneficiaries[]  → Participant（均等按分）
```

### personal vs shared の判定

**独立列（`is_shared` 等）は持たない。** beneficiary 行の有無のみで判定する（[shared-expense-model.md](shared-expense-model.md) Open Question #4 の回答）。

| `expense_beneficiaries` 行数 | 意味 |
|---|---|
| **0** | **personal**（デフォルト）。shared 精算入力には **載せない** |
| **1** | **shared** — 当該 Participant に 100% 按分（均等 split の特例） |
| **2+** | **shared** — 列挙 Participant 間で amount を均等分割 |

`paid_by_participant_id` の有無は personal/shared 判定に **使わない**。payer だけ指定・beneficiary なし = 「誰が立て替えたかは分かるが、按分は未記録（personal 扱い）」。

---

## 1. `expenses` テーブル拡張

### 追加列

| 列 | 型 | NULL | 意味 |
|---|---|---|---|
| `paid_by_participant_id` | `INTEGER` | **可** | 立替者 / 支払実行 Participant への参照 |

**v3.0.0 で追加しない列:** `is_shared`, `share_mode`, `settlement_id` 等。

### DDL 変更案（migration 参考 — 本フェーズでは実装しない）

```sql
-- migrate_expenses_shared_expense() 内（#33）
ALTER TABLE expenses ADD COLUMN paid_by_participant_id INTEGER NULL;

CREATE INDEX IF NOT EXISTS idx_expenses_paid_by_participant
    ON expenses(paid_by_participant_id);
```

既存列（`paid_by_name` 含む）は **変更・削除しない**。

### 外部キー方針

[participant-entity-design.md](participant-entity-design.md) / [expense-model.md](expense-model.md) と同型の **案 C: SQLite FK 制約なし + アプリ側検証**。

| 理由 | 説明 |
|---|---|
| 既存慣習 | `expenses` / `participants` / `notes` と統一 |
| Participant 削除 | アプリ層で `SET NULL` / beneficiary 行削除を明示（§3） |
| 検証 | create / update 時に Trip 整合をチェック |

論理参照: `paid_by_participant_id` → `participants.id`。Participant は **Expense が属する Itinerary の Trip** と同一 `trip_id` であること。

### Trip 整合の解決経路

```text
expense.itinerary_id
  → itinerary_items.trip_id
  → participants.trip_id（payer / beneficiary 双方）
```

CLI / import / validate-export は上記経路で Trip 外 Participant を **拒否** する。

### `paid_by_name` との共存・優先順位

| 状態 | 正本 | 表示 |
|---|---|---|
| `paid_by_participant_id` **のみ** | Participant ID | `participants.name` |
| `paid_by_name` **のみ** | 文字列（v1/v2 互換） | `paid_by_name` |
| **両方** | **`paid_by_participant_id` が正** | Participant `name`。`paid_by_name` は export 可読性・fallback |
| **両方なし** | payer unknown | `—` / 空 |

#### 書き込み時の同期（推奨 — #32 で実装詳細）

| 操作 | `paid_by_name` の扱い |
|---|---|
| `--paid-by-participant` 指定 | **Participant.name を `paid_by_name` に同期**（denormalized cache） |
| `--paid-by-name` のみ | `paid_by_participant_id` は **触らない**（NULL のまま） |
| `--clear-paid-by` | **両方 NULL** |
| import（ref 解決成功） | Participant.name を `paid_by_name` に **設定**（export に無くても可） |

#### 不一致時

| 層 | 方針 |
|---|---|
| **CLI save** | **許可** — ID を正として保存。`paid_by_name` はユーザー指定があればそのまま残してよい |
| **doctor** | **warning** — `paid_by_name` と Participant.name が異なる（§6） |
| **validate-export** | **warning**（export ファイルの整合チェック）。hard fail にしない |

精算入力（将来）は **`paid_by_participant_id` のみ** を正とし、`paid_by_name` のみの Expense は shared 計算から **除外**（Responsibilities Review 維持）。

---

## 2. `expense_beneficiaries` テーブル

### 役割

1 Expense に対し、**均等按分の対象 Participant** を 0..N 行で保持する。v3.0.0 では **join 行のみ** — `share_ratio` / `share_amount` 列は **持たない**。

### DDL 案

```sql
CREATE TABLE expense_beneficiaries (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    expense_id      INTEGER NOT NULL,
    participant_id  INTEGER NOT NULL,
    sort_order      INTEGER NOT NULL DEFAULT 0,
    created_at      TEXT NOT NULL,
    updated_at      TEXT NOT NULL,
    UNIQUE (expense_id, participant_id)
);

CREATE INDEX IF NOT EXISTS idx_expense_beneficiaries_expense
    ON expense_beneficiaries(expense_id);

CREATE INDEX IF NOT EXISTS idx_expense_beneficiaries_participant
    ON expense_beneficiaries(participant_id);
```

### フィールド

| 列 | 必須 | 説明 |
|---|---|---|
| `id` | はい | 行 ID（CLI 内部用。v3.0.0 では beneficiary 個別 CRUD コマンド **なし**） |
| `expense_id` | はい | 親 Expense |
| `participant_id` | はい | 受益者 Participant |
| `sort_order` | はい | export / 表示順（デフォルト `0`） |
| `created_at` / `updated_at` | はい | 監査列（既存エンティティ同型） |

### composite unique

```text
UNIQUE (expense_id, participant_id)
```

同一 Expense に同一 Participant を **二重登録不可**。CLI / import は **エラー**。

### `sort_order` 要否

**採用する。**

| 理由 | 説明 |
|---|---|
| export 安定性 | `beneficiaries[]` の並びを DB から再現 |
| 既存慣習 | Participant / Expense / Checklist と同型 |
| tie-break | 同一 `sort_order` は **`id ASC`** |

追加時: 未指定なら `0`。Implementation Plan で「既存最大+1」を選んでもよい。

### cascade 方針

| トリガー | `expense_beneficiaries` |
|---|---|
| `expense delete` | 当該 `expense_id` の行を **全削除** |
| `itinerary delete` | Expense cascade に伴い **全削除** |
| `trip delete` | Expense cascade に伴い **全削除** |
| `participant delete` | 当該 `participant_id` の行を **全削除**（§3） |

実装イメージ: `delete_expense_beneficiaries_for_expense` を `expense delete` から呼ぶ。Trip / Itinerary は既存 Expense cascade 経由。

### beneficiary 0 件の意味

**personal（または split 未記録）。** shared ではない。精算 read-only summary（将来）では shared 行のみ集計。

### beneficiary 1 件の意味

**有効な shared。** 当該 Participant に amount の 100% が帰属（均等 split で人数 1）。

| 観点 | 方針 |
|---|---|
| **データ** | **拒否しない** — 「この人だけの費用だが shared として明示したい」ケースを許容 |
| **doctor** | **warning** — 「beneficiary が 1 名のみ」（§6） |

---

## 3. Participant 削除時の扱い

### payer（`expenses.paid_by_participant_id`）

| 項目 | 方針 |
|---|---|
| **DB 操作** | **`SET NULL`**（Participant 行 DELETE 後） |
| **Expense 行** | **保持** — amount / itinerary 等は変更なし |
| **RESTRICT** | **採用しない** — Participant 削除を過度にブロックしない |

### beneficiary（`expense_beneficiaries`）

| 項目 | 方針 |
|---|---|
| **DB 操作** | 当該 `participant_id` の beneficiary 行を **DELETE** |
| **Expense 行** | **保持** |
| **0 件になった場合** | Expense は **personal** に戻る（暗黙） |

### fallback 表示

| 状況 | 表示優先 |
|---|---|
| payer ID あり、Participant 存在 | `participants.name` |
| payer ID NULL、`paid_by_name` あり | `paid_by_name` |
| payer ID NULL、`paid_by_name` NULL | `—` / unknown |
| （将来）削除済み ID が残存した場合 | **起きない** — DELETE 前に SET NULL |

`paid_by_name` は Participant 削除 **後も残る**。削除前に `--paid-by-participant` で同期していれば、**名前は表示可能**。

### `participant delete` の UX（#32）

```text
participant delete <id>
  → paid_by_participant_id を SET NULL（参照 Expense）
  → expense_beneficiaries 行を DELETE
  → participants 行 DELETE
```

削除前に **情報メッセージ**（任意）: 「N 件の Expense の payer、M 件の beneficiary 参照を解除します」— Implementation Plan で詳細化。

### Trip 削除

既存どおり `trip delete` → Expenses 全削除 → beneficiaries も **連鎖削除**。Participant も `delete_participants_for_trip`。

---

## 4. Export schema

### schema v5 が必要か

**はい — `schema_version: 5` を新設する。**

| 理由 | 説明 |
|---|---|
| 新フィールド | `paid_by_participant_ref`, `beneficiaries[]` |
| v4 互換 | v4 importer は v5 未知フィールドを **読まない** — v4 export 生成は v3 実装まで継続 |
| 明確な境界 | Responsibilities Review + Epic #13 補足方針に合致 |

v4 export を **schema bump なしで拡張** する案は **不採用**（optional フィールド追加でも v4 正本 importer の挙動が曖昧になる）。

### v4 import 互換

| 方向 | 方針 |
|---|---|
| **v4 export → v5 import** | **可** — `paid_by_participant_ref` / `beneficiaries` 省略 = personal / payer unknown |
| **v5 export → v4 import** | **不可** — 想定どおり |
| **v3 export → v5 import** | **可** — `participants` 省略 = `[]`。Expense は `paid_by_name` のみ |
| **v1/v2 export** | 従来どおり legacy 経路 |

v5 import は v4 import 能力を **包含** する（participants + nested expenses + 新フィールド）。

### v5 top-level 構造

```json
{
  "schema_version": 5,
  "generator": "caglla-cli",
  "generator_version": "3.0.0",
  "exported_at": "2026-06-08T00:00:00Z",
  "trip": {},
  "participants": [],
  "days": [],
  "checklist_items": [],
  "notes": []
}
```

v4 との差分: `schema_version: 5` + Expense オブジェクト内の optional フィールド（下記）。

### Expense オブジェクト（v5 追加フィールド）

| フィールド | 必須 | 説明 |
|---|---|---|
| `paid_by_name` | 任意 | **v3 から存在** — 維持 |
| `paid_by_participant_ref` | 任意 | payer の Participant 参照（**内部 ID は export しない**） |
| `beneficiaries` | 任意 | beneficiary 配列。省略 = `[]` = personal |

```json
{
  "title": "昼食",
  "amount": 4000,
  "currency": "JPY",
  "paid_by_name": "Alex",
  "paid_by_participant_ref": "Alex",
  "beneficiaries": [
    { "participant_ref": "Alex", "sort_order": 0 },
    { "participant_ref": "Jordan", "sort_order": 1 }
  ],
  "expense_date": null,
  "note": null,
  "sort_order": 0
}
```

`beneficiaries[].sort_order` は **export 時に出力**（省略時 import は配列順 + `0`）。

### `participant_ref` の形式

**確定: 文字列 = `participants[].name` と完全一致（trim 後）。**

```json
"paid_by_participant_ref": "Alex"
"beneficiaries": [ { "participant_ref": "Jordan" } ]
```

| 論点 | 方針 |
|---|---|
| **name のみ** | 通常ケース。Participant entity は **同名許可**（[participant-entity-design.md](participant-entity-design.md)） |
| **同名あいまいさ** | ref 文字列が Trip 内で **2 件以上** マッチ → **import / validate-export エラー** |
| **`sort_order` による解決** | v3.0.0 では **採用しない** — ref は **name 文字列のみ**。同名 Participant は Trip 設計上避ける（doctor **warning**） |
| **オブジェクト形式** `{ "name", "sort_order" }` | **v3.x 拡張候補** — v3.0.0 export では **出さない** |

**name + sort_order 複合 ref** は v3.0.0 では **見送り**。Open Questions #32 に「同名 Participant が増えた場合の v3.x ref 拡張」を残す。

### import 時の解決ルール

```text
1. Trip 作成
2. participants[] INSERT（新 id 採番）
3. days / itineraries / …
4. expenses INSERT
     a. paid_by_participant_ref → name 完全一致で Participant 1 件に解決
        → expenses.paid_by_participant_id + paid_by_name（Participant.name）設定
     b. beneficiaries[] → 各 participant_ref を同規則で解決
        → expense_beneficiaries INSERT
     c. ref 省略 → paid_by_participant_id NULL、beneficiary 0 件
```

| 入力 | 結果 |
|---|---|
| ref のみ、name 一致 1 件 | ID 設定 + `paid_by_name` 同期 |
| ref + export の `paid_by_name` 両方 | ID 正。`paid_by_name` は export 値を **保持**（doctor で不一致 warning 可） |
| ref なし、`paid_by_name` のみ | v2 互換 — ID NULL |
| ref あいまい / 未解決 | **import 失敗**（トランザクション rollback） |

### unresolved participant ref

| 層 | 方針 |
|---|---|
| **trip import** | **hard fail** — `unknown participant_ref: "…" for expense …` |
| **validate-export** | **error** — ファイル内 `participants[]` に存在しない ref |
| **既存 DB** | 該当なし（ID 参照は import 時のみ ref 解決） |

### import 順序（確定）

```text
Trip → participants[] → days → itineraries → checklist → notes → expenses（+ beneficiaries）
```

Participant を Expense より **必ず先** に作成する（v2 と同型）。

### `paid_by_participant_id` を export しない理由

| 理由 | 説明 |
|---|---|
| 既存方針 | Expense / Participant / Reservation の **内部 ID は export しない** |
| 再 import | ID 再採番が前提 — **安定キーは Trip 内 name** |
| roundtrip | `participant_ref` + `participants[]` で復元 |

---

## 5. CLI UX 設計の土台

**本フェーズでは実装しない。** Entity Design と矛盾しない CLI 候補:

### 既存コマンド（変更なし）

```bash
expense add --itinerary 12 --amount 1500 --currency JPY
expense add --itinerary 12 --amount 980 --currency JPY \
  --paid-by-name 太郎
```

`--paid-by-name` は **v3 でも維持**。Participant 未登録 Trip の主経路。

### 新規オプション（v3 — すべて optional）

| オプション | 対象 | 意味 |
|---|---|---|
| `--paid-by-participant <id\|name>` | add / update | 構造化 payer。Trip 内 ID または name 解決 |
| `--beneficiary <id\|name>` | add / update | **繰り返し可**。shared beneficiary 追加 |
| `--shared-with all` | add / update | Trip の **全 Participant** を beneficiary に展開（`--beneficiary` と排他または上書き — #32 で詳細） |
| `--clear-paid-by` | update | `paid_by_participant_id` と `paid_by_name` を **両方 NULL** |
| `--clear-beneficiaries` | update | 当該 Expense の beneficiary 行を **全削除** → personal |

### `--paid-by` について

**v3.0.0 では `--paid-by` 単独エイリアスは導入しない。**

| 理由 | 説明 |
|---|---|
| 曖昧さ | Participant 名解決か `paid_by_name` かが CLI 上不明確 |
| 後方互換 | 既存 `--paid-by-name` と役割分担を明確化 |

将来 `--paid-by` を `--paid-by-participant` の短縮別名にする余地は #32 に残す。

### participant 0 件 Trip

| オプション | 結果 |
|---|---|
| `--paid-by-name` | **許可** |
| `--paid-by-participant` / `--beneficiary` / `--shared-with` | **CLI エラー** — `no participants registered for this trip` |

### `expense list` / `show` / `--json`

表示・JSON に追加（#33）:

```text
paid_by_participant_id   （内部 JSON のみ — list/show --json）
paid_by_name
beneficiaries[]          { participant_id, name, sort_order }
shared                   boolean 派生 — beneficiaries.len() > 0
```

人間可読 `show` では Participant **name** を表示。

### Settlement 関連 CLI

v3.0.0 では **`expense settlement` 等は追加しない**（Responsibilities Review）。

---

## 6. Validation / doctor 候補

### CLI / DB 書き込み（hard fail）

| # | ルール | エラー例 |
|---|---|---|
| 1 | payer / beneficiary の Participant が **Trip 外** | `participant does not belong to this trip` |
| 2 | beneficiary **重複**（同一 Expense + Participant） | `duplicate beneficiary` |
| 3 | `--shared-with all` で participants **0 件** | `no participants registered for this trip` |
| 4 | structured payer/beneficiary で participants **0 件** | 同上 |
| 5 | beneficiary に **share 列** — v3.0.0 では CLI オプション自体 **存在しない** | — |
| 6 | `paid_by_participant_id` 存在しない ID | `participant not found` |

### payer と beneficiaries の関係

| ケース | 方針 |
|---|---|
| payer ∈ beneficiaries | **許可** — 立替者も受益者（一般的） |
| payer 未指定、beneficiary あり | **許可** — shared だが立替者不明 |
| beneficiary 0、payer あり | **許可** — personal + 立替者記録 |

### doctor（warning — 保存は許可）

| code（案） | 条件 |
|---|---|
| `shared_expense_single_beneficiary` | beneficiaries **1 名のみ** |
| `paid_by_name_participant_mismatch` | `paid_by_participant_id` あり、`paid_by_name` が Participant.name と **不一致** |
| `duplicate_participant_names` | 同一 Trip に **同名 Participant** が複数 — ref あいまいさリスク |
| `structured_expense_no_participants` | （該当なし — CLI で拒否） |

### validate-export（v5）

| 条件 | 結果 |
|---|---|
| `paid_by_participant_ref` が `participants[]` に **不一致** | **error** |
| `beneficiaries[].participant_ref` 不一致 | **error** |
| 同名 Participant が 2 人以上 **かつ** ref 使用 | **error**（あいまい） |
| ref 省略、`paid_by_name` のみ | **valid**（v4 互換セマンティクス） |
| `paid_by_name` と ref 先 name **不一致** | **warning** |

### participant 0 件 Trip

| 層 | 方針 |
|---|---|
| **CLI** | structured オプション **拒否**。`paid_by_name` のみ **許可** |
| **doctor** | 特殊ルール **不要** |
| **export v5** | `participants: []`、Expense は `paid_by_name` のみ — **valid** |

---

## 7. Migration 方針

### 起点

v2.0.1（最新リリース）DB — `expenses` に `paid_by_name` のみ。`participants` テーブルあり。

### 手順（#33 実装参考）

```text
1. migrate_expenses_shared_expense()
     ALTER TABLE expenses ADD COLUMN paid_by_participant_id INTEGER NULL
     CREATE INDEX idx_expenses_paid_by_participant
2. CREATE TABLE expense_beneficiaries (...)
     CREATE INDEX ...
3. 既存データ — 自動 backfill なし
```

### 既存行の意味

| 項目 | migration 直後 |
|---|---|
| 全 Expense | `paid_by_participant_id = NULL` |
| beneficiaries | **0 件** — personal 相当 |
| `paid_by_name` | **そのまま** — 意味変更なし |

### destructive migration

**なし。** 列削除・データ上書き・`paid_by_name` クリアは行わない。

### 任意 backfill（非 migration）

`paid_by_name` → Participant 作成 + ID 設定は **別途 CLI**（[shared-expense-model.md](shared-expense-model.md) Deferred）。v3.0.0 migration には **含めない**。

### export / import 移行

| 項目 | 方針 |
|---|---|
| v2.0.x CLI export | migration 後も **v4 export 可能**（#33 まで）— v5 export は v3 CLI |
| DB バージョン | Cargo / schema_version とは **独立** — migration 関数で段階追加 |

---

## Relationships（確定）

### Expense ↔ Participant（payer）

| | |
|---|---|
| **カーディナリティ** | 0..1 Participant per Expense |
| **方向** | Expense → Participant |
| **必須** | **任意** |

### Expense ↔ Participant（beneficiary）

| | |
|---|---|
| **カーディナリティ** | 0..N Participant per Expense |
| **中間** | `expense_beneficiaries` |
| **必須** | **任意**（0 = personal） |

### Participant 削除

§3 参照 — payer `SET NULL`、beneficiary 行 DELETE。

### Itinerary / Day / Trip

Expense の親子は **v1 維持**。beneficiaries は Expense に従属。

---

## Deferred Scope（Entity レベル）

| 項目 | 送り先 |
|---|---|
| `share_ratio` / `share_amount` | v3.x |
| `participant_ref` オブジェクト（name + sort_order） | v3.x（同名 Participant 対策） |
| Settlement 永続テーブル | v3.x |
| `expense settlement` CLI | v3.x |
| read-only Trip 集計コマンド | v3.0.0 **候補** — #32 で要否確定 |
| `trip diff` payer/beneficiary | #32 / Maintenance |
| weighted split export フィールド | v3.x |

---

## v3.0.0 Entity Design Scope

### 本書で確定するもの

| 項目 | 内容 |
|---|---|
| `expenses` 拡張 | `paid_by_participant_id INTEGER NULL` |
| 新テーブル | `expense_beneficiaries`（7 列 + UNIQUE） |
| personal/shared | beneficiary **行数** で判定。明示列なし |
| FK | アプリ検証。SQLite FK なし |
| Participant 削除 | payer SET NULL、beneficiary DELETE |
| export | **schema v5**、`participant_ref` = name 文字列 |
| v4 互換 | v4 export import 継続。新フィールド省略可 |

### 実装フェーズ（#32–#33）で行うもの

- migration SQL / Rust
- CLI オプション実装
- export v5 / import / validate-export
- doctor / diff / tests

---

## Open Questions（Implementation Plan #32 へ）

[shared-expense-model.md](shared-expense-model.md) Open Questions の **回答** と、残る実装詳細:

| # | 質問 | Entity Design 回答 / 残課題 |
|---|---|---|
| 1 | export Participant 参照 | **確定:** `participant_ref` = **name 文字列** |
| 2 | paid_by_name と ID 不一致 | **確定:** save 許可、doctor + validate-export **warning** |
| 3 | Participant 削除 payer FK | **確定:** **SET NULL** |
| 4 | shared 明示列 | **確定:** **持たない** — beneficiary 行数で判定 |
| 5 | `--shared-with all` + is_self 不明 | **確定:** 全 `participants` 行を beneficiary に。**is_self 不問** |
| 6 | schema v5 vs v4 拡張 | **確定:** **v5 新設** |
| 7 | read-only Trip 集計 | **未確定** — v3.0.0 に含めるか #32 で決定（`trip expense-summary` 候補） |
| 8 | `trip diff` 粒度 | **#32** — payer_id / beneficiary 集合の added/removed/changed |
| 9 | doctor warning 一覧 | **確定:** §6 の 3 code + 実装文言は #32 |
| 10 | `--shared-with all` と `--beneficiary` の併用 | **#32** — last wins か排他か |
| 11 | `expense update` で beneficiary **全置換** vs **追加のみ** | **#32** — 推奨: 指定時は **全置換**、`--clear-beneficiaries` で空 |
| 12 | 同名 Participant ref あいまい | **v3.0.0:** import **error**。v3.x で sort_order 付き ref 拡張 |
| 13 | validate-export v4 ファイル | v4 schema **warning** のみ（v5 フィールド検査スキップ） |

---

## Completion Criteria

本 Entity Design（Issue #31）の完了条件:

| # | 条件 | 状態 |
|---|---|---|
| 1 | `shared-expense-entity-design.md` が存在する | 本書 |
| 2 | `expenses` 拡張方針 | §1 |
| 3 | `expense_beneficiaries` 方針 | §2 |
| 4 | Participant 削除時方針 | §3 |
| 5 | export schema v5 方針 | §4 |
| 6 | CLI UX 土台 | §5 |
| 7 | validation / doctor 候補 | §6 |
| 8 | migration 方針 | §7 |
| 9 | #32 へ Open Questions | §Open Questions |
| 10 | Rust / DB / CLI 実装なし | 本フェーズ対象外 |

---

## Next phase notes（Implementation Plan #32）

#32 では本書を前提に、以下を確定する:

- migration 関数順序・ロールバックテスト
- CLI ヘルプ文言・エラーメッセージ英語域
- export v5 Rust 型・`validate-export` 実装詳細
- `trip diff` / doctor 実装有無
- integration test 一覧（roundtrip v4→v5、participant 削除 cascade）
- read-only Trip 集計を v3.0.0 に含めるかの最終判断

Implementation（#33）→ Post-Implementation Review（#34）→ Release v3.0.0（#35）。
