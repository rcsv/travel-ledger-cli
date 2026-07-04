# Shared Expense Implementation Plan

Caglla.Travel CLI v3.0.0 で **Shared Expense** を実装するための計画です。

**v3.0.0 設計フェーズ 3/6: Implementation Plan のみ。** 本書は DB migration・CLI・export schema・テストコードの変更を伴わない。実装は Issue #33 以降。

| ドキュメント | 役割 |
|---|---|
| [shared-expense-model.md](shared-expense-model.md) (#30) | 責務・境界・v3 スコープ |
| [shared-expense-entity-design.md](shared-expense-entity-design.md) (#31) | DDL・export v5・検証骨格 |
| **本書** (#32) | 実装計画（If we build it, how） |
| [shared-expense-post-implementation-review.md](shared-expense-post-implementation-review.md) (#34) | 実装後レビュー（#33 完了後） |
| [export-schema.md](export-schema.md) | export JSON 構造（#33 で v5 追記） |

関連: [github-workflow.md](../github-workflow.md) / [participant-implementation-plan.md](participant-implementation-plan.md) / [foundation-hardening-review.md](foundation-hardening-review.md) / [command-reference.md](../command-reference.md) / [export-import.md](../export-import.md)

設計系列（Epic #13）:

```text
#30 Responsibilities Review        → shared-expense-model.md
#31 Entity Design                  → shared-expense-entity-design.md
#32 Implementation Plan             → shared-expense-implementation-plan.md（本書）
#33 Implementation                 → migration + CLI + export v5（予定）
#34 Post-Implementation Review
#35 Release v3.0.0
```

---

## Purpose

#30 / #31 で固めた設計を、Issue #33 の実装で **迷わない工事手順書** に落とし込む。

```text
v3.0.0 で Shared Expense recording（payer + beneficiaries）を
migration → domain → CLI → export v5 → tests の順で安全に導入する。
Settlement 計算 CLI・永続 Settlement は v3.0.0 では実装しない。
```

### 設計前提（必読）

| # | 前提 | 出典 |
|---|---|---|
| 1 | Expense = **Transaction Record** のまま拡張 | [shared-expense-model.md](shared-expense-model.md) |
| 2 | 独立 Shared Expense エンティティ **なし** | 同上 |
| 3 | personal デフォルト — **beneficiary 0 件 = personal** | [shared-expense-entity-design.md §personal vs shared](shared-expense-entity-design.md#personal-vs-shared-の判定) |
| 4 | v3.0.0 は **均等按分のみ** | Entity Design §2 |
| 5 | Participant 削除 — payer **SET NULL**、beneficiary 行 **DELETE** | Entity Design §3 |
| 6 | export **schema v5**、`participant_ref` = **name 文字列** | Entity Design §4 |
| 7 | 同名 Participant + ref → import / validate-export **error** | Entity Design §4 / §6 |
| 8 | 既存 `expense add --amount --currency` **変更なし** | Responsibilities Review |
| 9 | `--paid-by` 単独エイリアス **v3.0.0 では導入しない** | Entity Design §5 |

---

## Background

### v2 完了時点

- `participants` CRUD + export **schema v4**
- Expense は `paid_by_name` のみ。Participant との DB リンクなし
- `trip diff` は Expense 差分 **未実装**（[foundation-hardening-review.md](foundation-hardening-review.md) §Maintenance）
- Settlement / beneficiary **なし**

### v3 で追加する価値

| 課題 | 解決 |
|---|---|
| 誰が立て替えたかを構造化 | `paid_by_participant_id` + `--paid-by-participant` |
| 誰の費用か（均等割り） | `expense_beneficiaries` + `--beneficiary` / `--shared-with all` |
| バックアップ・移行 | export **schema v5** + v4 import 互換 |
| グループ旅行の export 可読性 | `paid_by_participant_ref` + `beneficiaries[]` |

### v3.0.0 で意図的にやらないこと

| 項目 | 送り先 |
|---|---|
| `expense settlement` transfer 計算 | v3.x |
| 永続 Settlement / 消込 | v3.x |
| `share_ratio` / `share_amount` | v3.x |
| `--paid-by` 単独エイリアス | v3.x（任意） |
| `participant_ref` オブジェクト（name + sort_order） | v3.x |
| dedicated `trip expense-summary` コマンド | **v3.x**（#32 確定 — §Trip Summary Plan） |
| Person / Traveler Profile | 将来 |
| Budget / Estimate | 非対象 |

---

## Source Documents

| ドキュメント | 本計画が引き継ぐ内容 |
|---|---|
| [shared-expense-model.md](shared-expense-model.md) | 責務、Settlement defer、opt-in UX |
| [shared-expense-entity-design.md](shared-expense-entity-design.md) | DDL、cascade、export v5、Validation、Open Questions 回答 |
| [participant-entity-design.md](participant-entity-design.md) | Participant 正本、`name` 参照 |
| [expense-model.md](expense-model.md) | Itinerary 配下、Transaction Record |
| [export-schema.md](export-schema.md) | v4 現行、schema 昇格パターン |
| [participant-implementation-plan.md](participant-implementation-plan.md) | 計画ドキュメント体裁・実装順序先例 |

---

## Entity Design Open Questions — 本計画での確定

[shared-expense-entity-design.md §Open Questions](shared-expense-entity-design.md#open-questionsimplementation-plan-32-へ):

| # | 論点 | **#32 確定** |
|---|---|---|
| 7 | read-only Trip 集計 | **v3.0.0 では専用コマンドなし**。`expense list --trip --json` / export-md の payer・beneficiary 表示強化のみ。`trip expense-summary` は **v3.x** |
| 8 | `trip diff` 粒度 | **v3.0.0 に含める** — Expense 単位で payer / beneficiaries 変更を検出（§Diff Plan） |
| 10 | `--shared-with all` と `--beneficiary` | **add:** 排他（同時指定エラー）。**update:** beneficiary 指定グループが **あれば全置換** — `--shared-with all` は全 Participant 展開で beneficiaries を **置換** |
| 11 | beneficiary update モード | **全置換**。`--beneficiary` / `--shared-with` 指定時は既存 beneficiary 行を DELETE してから INSERT。`--clear-beneficiaries` で空（personal） |
| 9 | doctor warning 文言 | §Doctor Plan の code + 英語メッセージで確定 |
| 13 | validate-export v4 | v4 ファイルは v5 専用 ref 検査 **スキップ**。multiple self 等 v4 ルールは **継続** |

---

## Implementation Scope

### v3.0.0 で行う（Issue #33）

| 領域 | 内容 |
|---|---|
| **DB** | `expenses.paid_by_participant_id` 列 + `expense_beneficiaries` テーブル migration |
| **Domain** | `Expense` 拡張、`ExpenseBeneficiary` struct、`src/expense.rs` 拡張 |
| **Repository** | beneficiary CRUD、cascade、Participant 削除連携、duplicate |
| **Validation** | Trip 整合、duplicate beneficiary、participant 0 件 Trip の structured 拒否 |
| **CLI** | `expense add/update` opt-in オプション（§CLI Plan） |
| **Participant 連携** | `delete_participant` で payer SET NULL + beneficiary DELETE |
| **Export** | `schema_version: 5`、`paid_by_participant_ref`、`beneficiaries[]` |
| **Import** | v5 復元、v4/v3 互換（新フィールド省略）、ref 解決 |
| **validate-export** | v5 ref 検証、同名 ambiguity error、v4 互換 |
| **diff** | Expense payer / beneficiaries 差分（§Diff Plan） |
| **export-md** | Expense 行に payer / shared 表示（§Markdown Export Plan） |
| **doctor** | Shared expense warnings（§Doctor Plan） |
| **docs** | command-reference、export-schema、export-import |
| **tests** | unit + CLI integration + export roundtrip v4/v5 |

### v3.0.0 で行わない（Non-goals）

| 項目 | 送り先 |
|---|---|
| Settlement 計算 CLI | v3.x |
| `share_ratio` / weighted split | v3.x |
| `--paid-by` エイリアス | v3.x |
| `trip expense-summary` | v3.x |
| Cargo bump / tag / Release | #35 |
| canonical sample payer 例示 | **任意**（#33 後） |
| `paid_by_name` → Participant backfill CLI | 任意 maintenance |

---

## Table / Migration Plan

### 変更概要

```text
expenses
  + paid_by_participant_id INTEGER NULL

expense_beneficiaries（新規）
  id, expense_id, participant_id, sort_order, created_at, updated_at
  UNIQUE (expense_id, participant_id)
```

### DDL（v3.0.0）

[shared-expense-entity-design.md §1–§2](shared-expense-entity-design.md) の DDL をそのまま実装する。

```sql
ALTER TABLE expenses ADD COLUMN paid_by_participant_id INTEGER NULL;

CREATE INDEX IF NOT EXISTS idx_expenses_paid_by_participant
    ON expenses(paid_by_participant_id);

CREATE TABLE IF NOT EXISTS expense_beneficiaries (
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

### 論点整理

| 論点 | 方針 |
|---|---|
| **FK** | **張らない** — Participant / Expense と同型 |
| **冪等性** | `ALTER TABLE` は列存在チェック（`migrate_expenses_shared_expense` パターン — 既存 `migrate_*` と同型） |
| **既存行** | `paid_by_participant_id = NULL`、beneficiary 0 件 — **意味は v2 と同一** |
| **`paid_by_name`** | **削除しない** |
| **rollback** | **しない** — v1/v2 migration 慣習 |
| **down migration** | 作らない |

### Migration 実装案

| 項目 | 案 |
|---|---|
| **関数名** | `migrate_expenses_shared_expense(conn: &Connection) -> Result<()>` |
| **配置** | `src/expense.rs` 内、または `src/db.rs` から呼び出し |
| **呼び出し** | `init_db` パスで `migrate_participants` の **後**、`migrate_indexes` の前後は既存慣習に合わせる |
| **テスト** | `test_init_db_creates_expense_beneficiaries_table`、`test_migrate_adds_paid_by_participant_id` |

---

## Domain / Repository Plan

### Model（`src/models.rs`）

```rust
// 案 — #33 で確定
pub struct Expense {
    // 既存フィールド ...
    pub paid_by_participant_id: Option<i64>,
}

pub struct ExpenseBeneficiary {
    pub id: i64,
    pub expense_id: i64,
    pub participant_id: i64,
    pub sort_order: i32,
    pub created_at: String,
    pub updated_at: String,
}
```

Export 用（v5）:

```rust
pub struct ExportExpenseBeneficiaryV5 {
    pub participant_ref: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort_order: Option<i32>,
}

pub struct ExportExpenseV5 {
    // ExportExpenseV3 相当フィールド ...
    #[serde(skip_serializing_if = "Option::is_none")]
    pub paid_by_participant_ref: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub beneficiaries: Vec<ExportExpenseBeneficiaryV5>,
}
```

**型戦略:** `ExportExpenseV3` を v5 フィールドで拡張するか、`ExportExpenseV5` を新設して alias するか — #33 で選択。**推奨:** 既存 `ExportExpenseV3` に optional フィールド追加 + `TRIP_EXPORT_SCHEMA_VERSION = 5`（serde `skip_serializing_if` で v4 出力互換は **使わない** — v3 CLI は v5 のみ export）。

定数:

```rust
pub const TRIP_EXPORT_SCHEMA_VERSION: i32 = 5;
pub const TRIP_EXPORT_SCHEMA_VERSION_V4: i32 = 4;
// is_supported_export_schema_version に 5 を追加
```

### Repository 操作（`src/expense.rs` 拡張）

| 関数 | 用途 |
|---|---|
| `create_expense(..., paid_by_participant_id?, beneficiaries?)` | add — トランザクション内で beneficiary INSERT |
| `update_expense(..., paid_by_participant_id?, beneficiaries?, clear_paid_by, clear_beneficiaries)` | update |
| `list_beneficiaries_for_expense(conn, expense_id)` | show / list / export |
| `set_expense_beneficiaries(conn, expense_id, participant_ids[])` | 全置換ヘルパ |
| `delete_beneficiaries_for_expense(conn, expense_id)` | cascade / clear |
| `delete_beneficiaries_for_participant(conn, participant_id)` | participant delete |
| `clear_paid_by_for_participant(conn, participant_id)` | participant delete — SET NULL |
| `duplicate_expense_beneficiaries(conn, src_expense_id, dst_expense_id, participant_id_map)` | trip duplicate（Participant ID remap 後） |
| `resolve_participant_for_expense_trip(conn, itinerary_id, id_or_name) -> Result<i64>` | CLI / import 名解決 |
| `validate_expense_participants_trip_scope(...)` | Trip 外拒否 |

### Participant 削除（`src/participant.rs` 拡張）

`delete_participant` を **トランザクション内** で拡張:

```text
1. clear_paid_by_for_participant(id)     — expenses SET paid_by_participant_id = NULL
2. delete_beneficiaries_for_participant(id)
3. DELETE FROM participants WHERE id = ?
```

optional: 削除前に影響件数を stderr に表示（UX — #33 で任意）。

### personal / shared 判定ヘルパ

```rust
pub fn expense_is_shared(beneficiary_count: usize) -> bool {
    beneficiary_count > 0
}
```

---

## CLI Plan

既存 `expense` サブコマンドを **後方互換** で拡張する。

### 変更なし（最小パス）

```bash
expense add --itinerary 12 --amount 1500 --currency JPY
expense add --itinerary 12 --amount 980 --currency JPY --paid-by-name 太郎
```

### 新規オプション

| オプション | add | update | 意味 |
|---|---|---|---|
| `--paid-by-participant <id\|name>` | ✓ | ✓ | 構造化 payer。`paid_by_name` を Participant.name に **同期** |
| `--beneficiary <id\|name>` | ✓（繰り返し） | ✓（繰り返し） | beneficiary **全置換** の入力 |
| `--shared-with all` | ✓ | ✓ | Trip 全 Participant を beneficiary に **置換** |
| `--clear-paid-by` | — | ✓ | payer ID + `paid_by_name` を NULL |
| `--clear-beneficiaries` | — | ✓ | beneficiary 全削除 → personal |
| `--paid-by-name` | ✓ | ✓ | **既存維持**。structured payer を触らない |

**導入しない:** `--paid-by`（v3.0.0）、`--split`、`--share-ratio`。

### `expense add`

| 項目 | 方針 |
|---|---|
| **必須** | 従来どおり `--itinerary`, `--amount`, `--currency` のみ |
| **`--paid-by-participant`** | Trip 内 Participant 解決。0 件 Trip → **エラー** |
| **`--beneficiary`** | 1 件以上指定で shared。重複 → **エラー** |
| **`--shared-with all`** | `--beneficiary` と **同時指定不可** |
| **payer 省略** | 許可 — payer unknown |
| **beneficiary 省略** | 許可 — personal |
| **human output** | 従来 + optional `Paid by: Alex (shared: Alex, Jordan)` 一行（#33 で詳細） |

### `expense update`

| 項目 | 方針 |
|---|---|
| **beneficiary 更新** | `--beneficiary` / `--shared-with` 指定時 **全置換** |
| **`--clear-beneficiaries`** | beneficiary のみクリア。他フィールドは触らない |
| **`--clear-paid-by`** | payer のみクリア |
| **競合** | `--clear-beneficiaries` と `--beneficiary` 同時 → **最後のルール優先** ではなく **エラー**（実装単純化） |
| **少なくとも 1 変更** | 既存 update と同型 — 新オプションも変更対象にカウント |

### participant 0 件 Trip

```text
no participants registered for this trip
```

`--paid-by-name` のみ **許可**。

### Participant 名解決

| 入力 | 解決 |
|---|---|
| 数値 | Participant ID — Trip 整合必須 |
| 文字列 | 同一 Trip 内 `name` **完全一致**（trim 後） |
| 同名 2 件以上 | **エラー** — `ambiguous participant name: …` |
| 未一致 | `participant not found: …` |

### エラーメッセージ（案）

| 条件 | メッセージ |
|---|---|
| Trip 外 Participant | `participant does not belong to this trip` |
| duplicate beneficiary | `duplicate beneficiary for expense` |
| `--shared-with` + `--beneficiary` | `cannot combine --shared-with and --beneficiary` |
| structured + 0 participants | `no participants registered for this trip` |
| ambiguous name | `ambiguous participant name: {name}` |

---

## JSON Output Plan

### `expense list --json` / `expense show --json`

```json
{
  "id": 1,
  "itinerary_id": 12,
  "amount": 4000,
  "currency": "JPY",
  "paid_by_name": "Alex",
  "paid_by_participant_id": 3,
  "paid_by_participant_name": "Alex",
  "shared": true,
  "beneficiaries": [
    { "participant_id": 3, "name": "Alex", "sort_order": 0 },
    { "participant_id": 4, "name": "Jordan", "sort_order": 1 }
  ]
}
```

| フィールド | 方針 |
|---|---|
| `paid_by_participant_id` | **CLI JSON のみ** export しない |
| `shared` | 派生 — `beneficiaries.len() > 0` |
| `paid_by_participant_name` | 解決済み表示用（ID あり時） |

---

## Export / Import Plan

### schema version

```text
schema_version: 5
TRIP_EXPORT_SCHEMA_VERSION = 5
```

`is_supported_export_schema_version`: `1 | 3 | 4 | 5` をサポート。import は v1–v5、export は **v5 のみ**。

### Export 追加フィールド

[shared-expense-entity-design.md §4](shared-expense-entity-design.md) 踏襲:

```json
{
  "paid_by_name": "Alex",
  "paid_by_participant_ref": "Alex",
  "beneficiaries": [
    { "participant_ref": "Alex", "sort_order": 0 },
    { "participant_ref": "Jordan", "sort_order": 1 }
  ]
}
```

| 論点 | 方針 |
|---|---|
| ref 省略 | payer unknown / personal |
| `beneficiaries` 省略 | `[]` = personal |
| internal ID | **export しない** |

### Import 順序

```text
Trip → participants[] → days → itineraries → … → expenses
  → resolve paid_by_participant_ref
  → resolve beneficiaries[].participant_ref
  → INSERT expense_beneficiaries
```

### ref 解決（import）

| 状況 | 結果 |
|---|---|
| name 1 件マッチ | ID 設定 + `paid_by_name` 同期（export に無ければ Participant.name） |
| name 0 件 | **import fail** |
| name 2+ 件 | **import fail** — ambiguous |
| ref 省略 | NULL / 空 beneficiaries |

### v4 / v3 互換

| From | 動作 |
|---|---|
| v4 export → v5 import | 新フィールド省略 — **personal / payer unknown** |
| v3 export → v5 import | `participants: []` — Expense は `paid_by_name` のみ |
| v5 export → v4 import | **不可** |

### validate-export（v5）

| チェック | 結果 |
|---|---|
| ref が `participants[]` に存在 | 必須（指定時） |
| 同名 Participant + ref | **error** |
| `paid_by_name` vs ref name 不一致 | **warning** |
| v4 ファイル | v5 ref ルール **スキップ**（v4 participant ルールは継続） |

### Roundtrip テスト

| テスト | 内容 |
|---|---|
| v5 full | payer + beneficiaries roundtrip |
| v4 → import → export v5 | 新フィールド省略からの昇格 |
| ambiguous ref | validate-export fail |
| participant 削除後 export | payer ref NULL、name 残存 |

---

## Diff Plan

[foundation-hardening-review.md](foundation-hardening-review.md) Maintenance 項目を **v3.0.0 で解消** する。

### 現状

`trip diff` は Expense 本体（amount / title 等）の差分が **薄い / 未実装**。v3 で payer / beneficiaries を追加するタイミングで **Expense 差分を実装** する。

### 差分単位

**Itinerary ネスト内の Expense 行** — キー: `(day_number, itinerary sort_order, itinerary title, expense sort_order, expense title, amount, currency)` または既存 diff が Expense をどう扱うかに合わせる。#33 実装時に **既存パターンを調査** してキーを確定。

### 差分カテゴリ（Shared Expense フィールド）

| 種別 | 検出 |
|---|---|
| **payer changed** | `paid_by_participant_ref` または `paid_by_name` 変更 |
| **beneficiaries added** | 新 JSON にのみ存在する ref |
| **beneficiaries removed** | 旧 JSON にのみ存在 |
| **beneficiaries changed** | 集合または sort_order 変更 |
| **personal → shared** | beneficiaries 空 → 非空 |
| **shared → personal** | beneficiaries 非空 → 空 |

### Diff identity（beneficiary）

export に internal id がないため、**`participant_ref` 文字列の集合** をソートして比較。`sort_order` 変更のみは **reordered** または **changed**。

### v4 vs v5 diff

v4 側 Expense に ref なし → payer/beneficiary 差分は **検出しない**（フィールド欠如 = unknown）。v5 同士を主対象とする。

---

## Markdown Export Plan

### Expense 行の拡張（Itinerary 配下 — 既存構造維持）

| 項目 | 方針 |
|---|---|
| **payer** | `paid_by_participant_id` 解決名、なければ `paid_by_name` |
| **shared** | beneficiaries あり → `Shared: Alex, Jordan` |
| **personal** | beneficiaries なし → shared 行 **省略** |
| **Trip 集計セクション** | **v3.0.0 では追加しない**（v3.x / v5 Travel Book） |

例:

```markdown
- ¥4,000 昼食 — Paid by: Alex · Shared: Alex, Jordan
```

---

## Trip Summary Plan

### v3.0.0 確定（Entity Design #7 回答）

| 項目 | v3.0.0 | v3.x |
|---|---|---|
| `trip expense-summary` コマンド | **なし** | 候補 |
| `expense list --trip --json` | payer / beneficiaries **含める** | — |
| export-md | 行内表示 **含める** | — |
| `trip stats` | **変更なし**（金額集計は既存） | — |

**理由:** recording 優先。Settlement 前の read-only 集計コマンドは UX 設計を v3.x に回し、#33 スコープを抑える。

---

## Doctor / Advisor Plan

### v3.0.0 で追加（warning レベル）

| Code | 条件 | Severity |
|---|---|---|
| `SHARED_EXPENSE_SINGLE_BENEFICIARY` | beneficiaries **1 名のみ** | warning |
| `PAID_BY_NAME_PARTICIPANT_MISMATCH` | `paid_by_participant_id` あり、`paid_by_name` ≠ Participant.name | warning |
| `DUPLICATE_PARTICIPANT_NAMES` | 同一 Trip に **同名 Participant** が 2 件以上 | warning |

### advisor try hints（`--with-commands` 時のみ — 任意）

```text
Try: expense update <id> --beneficiary <name> ...
Try: participant list --trip <id>  # resolve ambiguous names
```

info レベルの `PARTICIPANTS_NOT_RECORDED` 等（v2）は **維持**。

---

## Testing Plan

### Unit tests（`src/expense.rs`）

| テスト | 内容 |
|---|---|
| migration | 列・テーブル・index、冪等性 |
| create with payer / beneficiaries | Trip 整合 |
| duplicate beneficiary | エラー |
| personal default | 0 beneficiaries |
| participant delete | payer SET NULL、beneficiary DELETE、shared→personal |
| clear_paid_by / clear_beneficiaries | update ヘルパ |
| resolve participant by name | 成功 / not found / ambiguous |
| trip duplicate | beneficiaries + payer remap |

### Unit tests（`src/participant.rs`）

| テスト | 内容 |
|---|---|
| delete_participant clears expense refs | SET NULL + beneficiary DELETE |

### CLI integration tests（`tests/expense_cli.rs` 拡張）

| テスト | 内容 |
|---|---|
| add minimal | 従来パス **回帰** |
| add --paid-by-participant | payer 設定 |
| add --beneficiary × N | shared |
| add --shared-with all | 全 Participant |
| 0 participants + structured | エラー |
| update --clear-beneficiaries | personal 戻り |
| update --clear-paid-by | |
| `--shared-with` + `--beneficiary` | エラー |
| list/show --json | beneficiaries ブロック |

### Export / import

| テスト | 内容 |
|---|---|
| export v5 | `schema_version: 5`, ref フィールド |
| import v5 roundtrip | payer + beneficiaries |
| import v4 | 省略フィールド — personal |
| validate-export v5 | ambiguous ref error |
| validate-export v4 | v5 ルールスキップ |
| reexport | `tests/export_roundtrip_cli.rs` に v5 + shared expense ケース |

### その他

| テスト | 内容 |
|---|---|
| diff | payer / beneficiaries changed |
| export-md | Paid by / Shared 行 |
| doctor | 3 warning codes |
| trip delete cascade | beneficiaries 連鎖 |
| itinerary delete cascade | 既存 + beneficiaries |

### canonical sample / golden

| 項目 | 方針 |
|---|---|
| **okinawa_sesoko_2026** | payer / beneficiary 例示は **#33 後任意** |
| **golden** | `expected-export-v5.json` は sample 更新時に追加 |

---

## Implementation Sequence

Issue #33 向け推奨順序:

```text
 1. models.rs — Expense 拡張、ExportExpenseV5 型、TRIP_EXPORT_SCHEMA_VERSION = 5
 2. expense.rs — migrate_expenses_shared_expense
 3. db.rs — init_db から migrate 呼び出し
 4. expense.rs — beneficiary repository + payer 列 read/write
 5. participant.rs — delete_participant 拡張（SET NULL + beneficiary DELETE）
 6. expense create/update 内部 API + unit tests
 7. main.rs — expense add/update 新オプション配線
 8. expense list/show — human + JSON 拡張
 9. trip export — schema v5 フィールド
10. trip import — ref 解決 + beneficiary INSERT
11. validate-export — v5 ルール + v4 スキップ
12. trip duplicate — payer/beneficiary ID remap
13. diff — Expense payer/beneficiaries
14. export-md — Paid by / Shared 表示
15. doctor — 3 warning codes
16. docs — command-reference, export-schema, export-import
17. CLI integration tests + export roundtrip v5
18. make check
```

**早めに import v5 を入れる理由:** CLI / validate / roundtrip が ref 解決に依存するため、**9–11 をまとめて** 着手するのが効率的。

---

## Compatibility / Risk Notes

| リスク | 影響 | 緩和 |
|---|---|---|
| **schema v5 昇格** | v4 import ツールが v5 を読めない | 想定内。v5 export は v3 CLI |
| **v4 export import** | 継続必須 | 新フィールド省略パスをテスト |
| **同名 Participant** | ref ambiguity | import error + doctor warning |
| **Participant 削除** | shared→personal | 明示テスト + ドキュメント |
| **FK なし** | 孤児 participant_id | 書き込み時 Trip 検証 |
| **paid_by_name 不一致** | 表示混乱 | doctor warning、ID を正 |
| **trip diff 複雑化** | Expense キー設計 | #33 で既存 diff 調査 |
| **duplicate trip** | ID remap | Participant 複製後に Expense remap |

---

## Deferred Scope

### v3.x

```text
trip expense-summary（read-only 集計）
expense settlement（transfer 計算）
share_ratio / share_amount
participant_ref { name, sort_order }
--paid-by エイリアス
paid_by_name → Participant backfill CLI
```

### v5 Travel Book / v6 Journal

export-md 以上の Shared Expense セクション — 製品バージョンロードマップどおり。

---

## Open Questions（Implementation #33 へ）

| # | 質問 |
|---|---|
| 1 | `ExportExpenseV3` を拡張 vs `ExportExpenseV5` 新設 — 型名の最終判断 |
| 2 | `expense add` human output に shared 行を **常に** 出すか、--json のみか |
| 3 | `trip duplicate` — payer/beneficiary remap を **同一 PR 必須** にするか |
| 4 | diff の Expense マッチングキー — 既存実装調査結果 |
| 5 | doctor warning を export-md / stats に **反映しない** 方針の明文化（表示のみ doctor） |
| 6 | validate-export: v5 ファイルで `schema_version: 4` + v5 フィールド混在を **拒否** するか |
| 7 | Cargo bump — #33 単独 patch か #35 release まで accum か（#35 で決定） |

---

## Completion Criteria

本 Implementation Plan（Issue #32）の完了条件:

| # | 条件 | 状態 |
|---|---|---|
| 1 | `shared-expense-implementation-plan.md` が存在する | 本書 |
| 2 | migration / domain / CLI / export 計画 | 各 § |
| 3 | Entity Design Open Questions 回答 | §Entity Design Open Questions |
| 4 | テスト計画 | §Testing Plan |
| 5 | 実装順序 | §Implementation Sequence |
| 6 | #33 へ Open Questions | §Open Questions |
| 7 | Rust / DB 実装なし | 本フェーズ対象外 |
| 8 | `make check` PASS | PR CI |

---

## Next phase notes（Implementation #33）

#33 では本書 §Implementation Sequence に従い、**1 PR または Epic 内分割 PR** で実装する。

推奨 PR 分割（任意）:

```text
PR A: migration + domain + unit tests
PR B: CLI + participant delete 連携
PR C: export v5 + import + validate-export + roundtrip tests
PR D: diff + export-md + doctor + docs
```

分割する場合も **schema v5 / migration は一貫** させ、中間状態で `make check` green を維持する。

Post-Implementation Review（#34）→ Release v3.0.0（#35）。
