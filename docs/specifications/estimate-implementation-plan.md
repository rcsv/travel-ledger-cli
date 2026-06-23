# Estimate Implementation Plan

Caglla.Travel CLI に **Estimate（事前見積 / Planned Budget）** を実装するための計画です。

**Implementation Plan。** Phase 1–5 まで **完了**（Post-Implementation Review 作成済み）。**次ステップ: Release**（version bump / release notes — 別 PR）。

| ドキュメント | 役割 |
|---|---|
| [estimate-model.md](estimate-model.md) | 責務・境界（Planned Money vs Actual Money） |
| [estimate-entity-design.md](estimate-entity-design.md) | DDL・CLI・export v6・Estimate 配列保持の設計根拠 |
| **本書** | 実装計画・Phase 進捗（If we build it, how） |
| [estimate-post-implementation-review.md](estimate-post-implementation-review.md) | 実装後レビュー（Phase 5 — 作成済み） |

関連: [expense-model.md](expense-model.md) / [itinerary-model.md](itinerary-model.md) / [export-schema.md](export-schema.md) / [command-reference.md](../command-reference.md) / [ordering-model.md](ordering-model.md)

設計系列（想定）:

```text
Responsibilities Review   → estimate-model.md
Entity Design             → estimate-entity-design.md
Implementation Plan       → estimate-implementation-plan.md（本書）
Phase 1 Implementation    → CRUD / migration / CLI — 実装済み（PR #50）
Phase 2 Implementation    → export v6 / validate / diff — 実装済み（PR #51）
Phase 3 Implementation    → trip stats / export-md — 実装済み（PR #52）
Phase 4 Implementation    → itinerary replicate Estimate コピー — 実装済み（PR #53）
Phase 5                     → Post-Implementation Review — 作成済み
Next                      → Release（未着手）
```

---

## Purpose

[estimate-entity-design.md](estimate-entity-design.md) で固めた設計を、実装 PR で **迷わない工事手順書** に落とし込む。

```text
Estimate = Planned Money（Itinerary 配下の事前見積）を
migration → money 共通化 → domain → CLI → tests → docs の順で安全に導入する。
export v6 / trip stats / export-md / replicate は Phase 2 以降に段階分割する。
```

### 設計前提（必読）

| # | 前提 | 出典 |
|---|---|---|
| 1 | Estimate = **Planned Money**。Expense = **Actual Money** — 別エンティティ | [estimate-model.md](estimate-model.md) |
| 2 | 親は **Itinerary のみ**（Trip / Day 直下なし） | Entity Design §Entity Definition |
| 3 | 1 Itinerary : N Estimate | 同上 |
| 4 | 必須入力は **`amount` + `currency`**。`title` / `note` は任意 | Entity Design §2 |
| 5 | amount / currency は Expense と **同一方針**（最小通貨単位 INTEGER） | Entity Design §3 |
| 6 | FK **なし** + アプリ側 cascade（Expense / Note と同型） | Entity Design §5 |
| 7 | replicate 時 Estimate コピーは **Phase 4** | Entity Design §10 |
| 8 | export `estimates[]` + schema v6 は **Phase 2** | Entity Design §8 |

---

## Phase Overview

Phase 1–5 は **すべて完了**。以下は実装履歴の整理。

| Phase | 内容 | PR |
|---|---|---|
| **Phase 1** | DB migration + `src/money.rs` + estimate CRUD + CLI + cascade + tests + command-reference | [#50](https://github.com/rcsv/travel-ledger-cli/pull/50) |
| **Phase 2** | export / import **schema v6**、`validate-export`、`trip diff`、Estimate 配列保持設計メモ | [#51](https://github.com/rcsv/travel-ledger-cli/pull/51) |
| **Phase 3** | `trip stats` Planned total、`export-md` 予定費用表示 | [#52](https://github.com/rcsv/travel-ledger-cli/pull/52) |
| **Phase 4** | `itinerary replicate` の Estimate コピー | [#53](https://github.com/rcsv/travel-ledger-cli/pull/53) |
| **Phase 5** | Post-Implementation Review（[estimate-post-implementation-review.md](estimate-post-implementation-review.md)） | 本 PR |

### Phase 1 に含める（推奨 — 最初の実装 PR）

```text
✓ estimates テーブル + migration（冪等）
✓ src/money.rs への amount / currency 共通化（案 A）
✓ src/estimate.rs（CRUD + cascade + 表示）
✓ models.rs に Estimate struct
✓ main.rs CLI（add / list / show / update / delete）
✓ itinerary delete / trip delete cascade 接続
✓ unit tests（estimate.rs 内）+ tests/estimate_cli.rs
✓ docs/command-reference.md 更新（実装済みに変更）
```

### Phase 1 に含めない

```text
✗ export schema v6 / trip export-import 変更
✗ validate-export / trip diff の Estimate 対応
✗ trip stats Planned total
✗ export-md 予定費用
✗ itinerary replicate Estimate コピー
✗ release / tag
✗ estimate-post-implementation-review.md（Phase 5 で作成済み）
```

**理由:** export schema を同時に入れると変更ファイルが `trip.rs` / import / validate / diff に一気に波及し、レビュー・rollback が困難になる。CRUD の正しさを Phase 1 で固めてから export を載せる。

---

## Entity Design Open Questions — 本計画での確定

[estimate-entity-design.md §12](estimate-entity-design.md#12-open-questionsimplementation-plan-で確定):

| # | 論点 | **本計画の確定** |
|---|---|---|
| 1 | `--clear-title` / `--clear-note` | **Phase 1 で採用** — nullable フィールドの明示クリア（§Update Plan） |
| 2 | `estimate list --trip` の Day ヘッダ | Phase 1 は **Expense 同型**（Itinerary ID 列）。Day 名付き表示は Phase 3 任意 |
| 3 | doctor / validate-export | **Phase 2 以降**（export 実装時） |
| 4 | export v6 フィールド名 | `estimates[]` — Entity Design どおり |
| 5 | replicate default | Phase 4 — **Estimate もコピー**（`--without-estimates` は初期不要） |
| 6 | Trip 全体 Budget 上限 | **defer** — Estimate 合計とは別概念 |

---

## Phase 1 — Detailed Plan

### 1.1 Table / Migration Plan

#### DDL（Entity Design どおり）

```sql
CREATE TABLE IF NOT EXISTS estimates (
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

#### 論点整理

| 論点 | 方針 |
|---|---|
| **FK** | **張らない** — Expense / Reservation / Note と同型 |
| **冪等性** | `CREATE TABLE IF NOT EXISTS` + `CREATE INDEX IF NOT EXISTS`（`migrate_participants` パターン） |
| **新規 DB** | `init_db` の `CREATE TABLE` ブロックにも `estimates` を **追加**（`expenses` / `reservations` の直後が自然） |
| **既存 DB** | `migrate_estimates(conn)` で上記 DDL を実行 |
| **rollback / down migration** | **作らない** — 既存 migration 慣習 |

#### Migration 実装案

| 項目 | 案 |
|---|---|
| **関数名** | `migrate_estimates(conn: &Connection) -> Result<()>` |
| **配置** | `src/estimate.rs` 内 |
| **呼び出し** | `src/db.rs` の `init_db` — `migrate_expenses_shared_expense` の **後** |
| **インデックス** | migration 内で `idx_estimates_itinerary` を作成。`migrate_indexes` への重複追加は **不要**（migration 内完結でよい） |

#### 触るファイル

```text
src/db.rs          — init_db: CREATE TABLE estimates + migrate_estimates 呼び出し
src/estimate.rs    — migrate_estimates（新規）
src/main.rs        — mod estimate;
```

---

### 1.2 Money Logic 共通化

Estimate は Expense と同一の amount / currency 方針を使う。

| 関数 | 現状 | Phase 1 |
|---|---|---|
| `validate_currency_code` | `src/expense.rs` | **`src/money.rs` へ移動** |
| `parse_amount_for_currency` | `src/expense.rs` | **`src/money.rs` へ移動** |
| `format_amount_value` | `src/expense.rs` | **`src/money.rs` へ移動** |
| `format_amount_display` | `src/expense.rs` | **`src/money.rs` へ移動** |

#### 方針: **案 A — `src/money.rs` 新設（推奨）**

```text
src/money.rs
  pub(crate) fn validate_currency_code(...)
  pub(crate) fn parse_amount_for_currency(...)
  pub(crate) fn format_amount_value(...)
  pub(crate) fn format_amount_display(...)
```

| 項目 | 方針 |
|---|---|
| **expense.rs** | `use crate::money::*` に切り替え。公開 API（CLI 経由の挙動）は **変更なし** |
| **estimate.rs** | 同上 |
| **テスト** | 既存 `expense.rs` 内の currency / amount unit tests を `money.rs` tests へ **移動または委譲** |
| **差分が大きい場合** | Phase 1 PR の **最初の commit** を money 抽出のみに分けてもよい（同一 PR 内 squash 可） |

案 B（expense 関数を pub(crate) 流用）は **不採用** — Estimate 追加後も Expense / 将来 Budget で再利用するため、早めに切り出す。

---

### 1.3 Domain / Repository Plan

#### Model（`src/models.rs`）

```rust
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Estimate {
    pub id: i64,
    pub itinerary_id: i64,
    pub title: Option<String>,
    pub amount: i64,
    pub currency: String,
    pub note: Option<String>,
    pub sort_order: i64,
    pub created_at: String,
    pub updated_at: String,
}
```

#### `EstimateWithContext` の要否

| 選択 | 説明 |
|---|---|
| **Phase 1: 採用しない（推奨）** | Expense `list --trip` と同型 — `Estimate` 行 + 表示時に `itinerary_id` 列。`ReservationWithContext` は export-md 向けだが Estimate Phase 1 では md 未対応 |
| 将来 | Phase 3 export-md で Day / Itinerary タイトルが必要になれば `EstimateWithContext` を追加検討 |

#### Module（`src/estimate.rs` — 新規）

想定する公開（`pub(crate)`）関数:

```rust
// migration
migrate_estimates(conn) -> Result<()>

// CRUD
add_estimate(conn, itinerary_id, amount_input, currency_input, title, note, sort_order) -> Result<i64>
list_estimates_for_itinerary(conn, itinerary_id) -> Result<Vec<Estimate>>
list_estimates_for_trip(conn, trip_id) -> Result<Vec<Estimate>>
get_estimate(conn, id) -> Result<Estimate>
update_estimate(conn, id, UpdateEstimateParams) -> Result<()>
delete_estimate(conn, id) -> Result<()>

// cascade
delete_estimates_for_itinerary(conn, itinerary_id) -> Result<()>
delete_estimates_for_trip(conn, trip_id) -> Result<()>

// CLI helpers
resolve_estimate_list_target(trip, itinerary) -> Result<EstimateListTarget>
print_estimate_list(conn, target, estimates)
print_estimate_detail(conn, estimate)
estimate_to_json(conn, estimate) -> Result<EstimateJson>
```

**Expense とは混ぜない** — ファイル・テーブル・CLI を分離。内部パターン（`list_*_where`、`with_transaction`、JSON struct）は `expense.rs` を **参照** する。

#### `add_estimate` 要点

- `get_itinerary_item` で親存在確認
- `parse_amount_for_currency` + `validate_currency_code`
- 空文字 `title` / `note` → **NULL**
- `sort_order` 省略 → `0`
- `created_at` / `updated_at` = `now_string()`

#### `list` クエリ

```sql
ORDER BY itinerary_id ASC, sort_order ASC, id ASC
```

Trip 集約時も Expense と同型の subquery:

```sql
itinerary_id IN (SELECT id FROM itinerary_items WHERE trip_id = ?1)
```

---

### 1.4 CLI Plan

#### `main.rs` 追加

```text
mod estimate;

Command::Estimate { action: EstimateAction }

enum EstimateAction {
    Add { itinerary, amount, currency, title, note, sort_order },
    List { trip, itinerary, json },
    Show { id, json },
    Update { id, title, note, amount, currency, sort_order, clear_title, clear_note },
    Delete { id },
}
```

Expense の clap 定義（`ExpenseAction`）を **テンプレート** にする。

#### コマンド仕様（Phase 1 確定）

```bash
caglla estimate add --itinerary 12 --amount 14000 --currency JPY
caglla estimate add --itinerary 12 --amount 14000 --currency JPY --title "ホテル朝食" --note "5人分"
caglla estimate list --itinerary 12
caglla estimate list --trip 1
caglla estimate list --trip 1 --json
caglla estimate show 3
caglla estimate show 3 --json
caglla estimate update 3 --amount 15000
caglla estimate update 3 --title "ホテル朝食 revised"
caglla estimate update 3 --clear-title
caglla estimate update 3 --clear-note
caglla estimate delete 3
```

#### `estimate list` ターゲット検証

| 入力 | 結果 |
|---|---|
| `--itinerary` のみ | OK |
| `--trip` のみ | OK |
| **両方指定** | **エラー** |
| **両方未指定** | **エラー** |

`resolve_estimate_list_target` — `expense::resolve_expense_list_target` と **同型** で実装。

#### `--json`

| コマンド | Phase 1 |
|---|---|
| `list --json` | **採用** — `EstimateListJson { trip_id, itinerary_id, estimates: Vec<EstimateJson> }` |
| `show --json` | **採用** — 単一 `EstimateJson` |

`EstimateJson` には表示用に `amount_display`（フォーマット済み文字列）を含めてもよい（Expense `ExpenseJson` 参照）。

#### 人間向け表示

| フィールド | 未設定時 |
|---|---|
| `title` | `-` または `(Estimate)` |
| `note` | `-`（detail のみ） |

---

### 1.5 Update / Clear Semantics（確定）

Expense は nullable `title` / `note` だが **`--clear-title` / `--clear-note` を持たない**（`--note` 指定時のみ上書き）。Estimate Phase 1 では **明示 clear を採用** する。

| オプション | 動作 |
|---|---|
| `--title "..."` | title を設定 |
| `--clear-title` | title を **NULL** |
| `--note "..."` | note を設定 |
| `--clear-note` | note を **NULL** |
| `--clear-title` + `--title` | **エラー**（同グループ排他） |
| `--clear-note` + `--note` | **エラー** |

| ルール | 内容 |
|---|---|
| 更新 0 件 | **エラー** — Expense update と同型メッセージ |
| `--amount` のみ | 既存 `currency` で parse |
| `--currency` のみ | amount は **変更しない**（Expense 同型） |
| `--amount` + `--currency` | 新 currency で parse |

将来 Expense に `--clear-title` / `--clear-note` を **backport** してもよい（Phase 1 スコープ外）。

---

### 1.6 Cascade Delete Plan

#### 接続箇所

| トリガー | ファイル | 追加呼び出し |
|---|---|---|
| `itinerary delete` | `src/itinerary.rs` `delete_itinerary_item` | `estimate::delete_estimates_for_itinerary(tx, id)?` — **expense の直前または直後**（Note → Expense → Reservation の並びに Estimate を Expense 隣接で挿入） |
| `trip delete` | `src/trip.rs` | `estimate::delete_estimates_for_trip(tx, id)?` — `delete_expenses_for_trip` の **前後**（同一 tx） |

推奨順序（itinerary delete tx 内）:

```text
delete_notes_for_itinerary
delete_estimates_for_itinerary    ← 追加
delete_expenses_for_itinerary
delete_reservations_for_itinerary
DELETE itinerary_items
```

#### `delete_estimate`（単体）

- Estimate 行のみ DELETE
- 親 Itinerary は触らない

---

### 1.7 Phase 1 Tests Plan

#### Unit tests（`src/estimate.rs` `#[cfg(test)]`）

| テスト | 内容 |
|---|---|
| `test_migrate_estimates_idempotent` | 2 回実行してもエラーなし |
| `test_add_estimate_minimal` | amount + currency のみ |
| `test_add_estimate_with_title_note_sort_order` | 任意フィールド |
| `test_add_estimate_invalid_itinerary` | 存在しない itinerary → エラー |
| `test_add_estimate_negative_amount` | 拒否 |
| `test_list_estimates_for_itinerary` | sort_order → id 順 |
| `test_list_estimates_for_trip` | 複数 Day / Itinerary 跨ぎ |
| `test_get_estimate_not_found` | 404 相当 |
| `test_update_estimate_amount_currency` | 部分更新 |
| `test_update_estimate_clear_title_note` | clear オプション |
| `test_update_estimate_no_fields_rejects` | 0 件更新エラー |
| `test_update_estimate_clear_title_and_title_conflict` | 排他 |
| `test_delete_estimate` | 単体削除 |
| `test_delete_estimates_for_itinerary_cascade` | itinerary 配下全削除 |
| `test_delete_estimates_for_trip_cascade` | trip 配下全削除 |

#### Money 共通化 regression（`src/money.rs` または expense 既存 test 維持）

| テスト | 内容 |
|---|---|
| currency validation | 既存 expense tests を **移行後も PASS** |
| JPY / USD amount parse | 同上 |
| `format_amount_display` | 同上 |

#### CLI integration tests（`tests/estimate_cli.rs` — 新規）

Expense CLI test（`tests/expense_cli.rs`）と同型の helper + ケース:

| テスト | 内容 |
|---|---|
| `cli_estimate_add_minimal` | add → show |
| `cli_estimate_add_with_title` | title 付き |
| `cli_estimate_list_itinerary` | `--itinerary` |
| `cli_estimate_list_trip` | `--trip` 集約 |
| `cli_estimate_list_json` | `--json` 構造 |
| `cli_estimate_list_invalid_target` | 両方未指定 / 両方指定 → 非ゼロ exit |
| `cli_estimate_update_amount` | 部分更新 |
| `cli_estimate_update_clear_title` | `--clear-title` |
| `cli_estimate_delete` | 削除後 show 失敗 |
| `cli_estimate_cascade_itinerary_delete` | itinerary delete で Estimate も消える |
| `cli_estimate_usd_decimal_amount` | `--amount 12.50 --currency USD` → 1250 |

---

### 1.8 Phase 1 Documentation Updates

Implementation Plan PR では **実装済みに書かない**。Phase 1 実装 PR で更新:

| ファイル | 更新内容 |
|---|---|
| [command-reference.md](../command-reference.md) | Estimate 節を **実装済み** CLI リファレンスに昇格 |
| [estimate-model.md](estimate-model.md) | 設計系列に Implementation 進行中を反映（任意） |
| [estimate-entity-design.md](estimate-entity-design.md) | Open Questions の Phase 1 確定を **実装済み** メモへ（任意） |
| [itinerary-model.md](itinerary-model.md) | Estimate を「未実装」→「実装済み（CRUD）」— export 未対応は明記 |
| [README.md](README.md) | estimate-implementation-plan 行の状態更新 |

**Phase 1 では触らない:** `export-schema.md`、`export-import.md`、`ordering-model.md`（replicate は Phase 4）

---

## Phase 2 — Export / Import schema v6

### 設計メモ: Estimate 配列保持

Estimate は Itinerary 配下の **0..N 明細** として保持・export する（単一 `planned_amount` に潰さない）。Planned Budget は Estimate 行の集計概念であり、独立エンティティではない。

| 概念 | 保存 / export |
|---|---|
| Estimate | `estimates` テーブル / `days[].itineraries[].estimates[]` |
| Planned Budget | 集計レイヤー（Phase 3 `trip stats` 等） |
| Expense | `expenses[]` — Actual Money |

想定ツリー・JSON 例・空配列方針: [estimate-entity-design.md §Estimate line items](estimate-entity-design.md#estimate-line-items--itinerary-配下の-0n-明細) / [§8 Export](estimate-entity-design.md#8-export--importschema-v6--phase-2-実装済み)

### スコープ

| 領域 | 内容 |
|---|---|
| **schema** | `schema_version: 6` |
| **nested JSON** | `days[].itineraries[].estimates[]` |
| **trip export** | Estimate 行を出力（id / timestamps **なし**） |
| **trip import** | v6 復元、v5 以前は `estimates` 省略 = 空 |
| **validate-export** | amount / currency 必須、currency 形式 |
| **trip diff** | Estimate added / removed / field changed（**v6+ 同士のみ**） |

### 触るファイル（想定）

```text
src/models.rs       — ExportEstimateV6, ExportDay/Itinerary 拡張
src/trip.rs         — export / import パス
src/estimate.rs     — export 変換、import INSERT
src/diff.rs         — Estimate 差分（Expense パターン参照）
docs/export-schema.md
docs/export-import.md
tests/              — export roundtrip v5→v6, v6 roundtrip
```

### v5 互換

| 方向 | 方針 |
|---|---|
| v5 export → v6 import | **可** — estimates 省略 |
| v6 export → v5 import | **不可** |
| 現行 export | **schema v6**（Phase 2 実装済み — PR #51） |

---

## Phase 3 — trip stats / export-md（実装済み）

### trip stats

| 表示 | 算出 |
|---|---|
| **Planned total** | Trip 配下 Estimate 合計（通貨別） |
| **Actual total** | 現行 Expense 合計（既存） |
| **Difference** | **defer**（将来） |

触ったファイル: `src/stats.rs`、`src/markdown.rs`（Overview）、`docs/command-reference.md`

JSON: `estimate_count` / `estimate_totals` を追加。

### export-md

- Itinerary セクションに Estimate 一覧（見出し「予定費用:」）
- Overview に Planned / Actual 合計（通貨別）

触ったファイル: `src/markdown.rs`、`src/estimate.rs`（Markdown 整形）、関連 docs

---

## Phase 4 — itinerary replicate（実装済み）

[estimate-entity-design.md §10](estimate-entity-design.md#10-itinerary-replicatephase-4--実装済み) どおり:

| コピーする | コピーしない |
|---|---|
| Itinerary 本体、Itinerary-level notes、**Estimate** | Expense、Reservation |

### 実装要点

| 項目 | 方針 |
|---|---|
| 関数 | `copy_estimates_for_itinerary` — target Itinerary 作成後に呼び出し |
| ID | 新規採番 |
| `sort_order` | **維持** |
| dry-run | Estimate を作成しない（DB 件数不変をテストで保証） |
| tx | 既存 replicate と **同一トランザクション** |
| CLI | `--without-estimates` は **追加しない**（デフォルトコピー） |

触ったファイル: `src/estimate.rs`、`src/itinerary.rs`、`tests/itinerary_cli.rs`、関連 docs

---

## Phase 5 — Post-Implementation Review（作成済み）

[estimate-post-implementation-review.md](estimate-post-implementation-review.md) を作成（documentation-only）。Phase 1–4 の実装結果・責務レビュー・テスト整理・deferred scope を本書に集約。

---

## Implementation Order（Phase 1 内部）

Phase 1 PR 内の **推奨 commit / 作業順**:

```text
1. src/money.rs 抽出 + expense.rs 差し替え + 既存 test PASS
2. migrate_estimates + init_db + models::Estimate
3. estimate.rs CRUD + cascade helpers + unit tests
4. itinerary.rs / trip.rs cascade 接続
5. main.rs CLI + estimate list target / json
6. tests/estimate_cli.rs
7. docs/command-reference.md
8. make check
```

---

## Risk / Mitigation

| リスク | 対策 |
|---|---|
| money 抽出で Expense 回帰 | 既存 expense unit / CLI tests を Phase 1 で **必ず** 実行 |
| export を Phase 1 に混ぜる誘惑 | 本計画どおり **Phase 2 まで defer** |
| replicate 忘れ | Phase 4 を明示。Phase 1 の cascade だけ先に入れる |
| Planned total の通貨混在 | stats Phase 3 — 通貨別行表示（Expense stats 同型） |

---

## Deferred Scope Summary

Phase 1–5 は **完了**。現時点で **未実装** の範囲（詳細は [estimate-post-implementation-review.md §9](estimate-post-implementation-review.md#9-deferred-scope)）:

```text
- Difference 計算（Planned vs Actual 差分表示）
- Budget 独立エンティティ
- payer / beneficiary / participant 連動
- unit_amount × quantity
- 為替換算
- --without-estimates
- doctor / advisor での Estimate 活用
- GUI / Web 版 Planned vs Actual 表示
- release 作業（tag / version bump / release notes）
```

**次ステップ: Release**（別 PR）。

---

## References

| 用途 | パス |
|---|---|
| 責務整理 | [estimate-model.md](estimate-model.md) |
| Entity Design | [estimate-entity-design.md](estimate-entity-design.md) |
| **Post-Implementation Review** | [estimate-post-implementation-review.md](estimate-post-implementation-review.md) |
| Expense 実装先例 | [expense-model.md](expense-model.md) / `src/expense.rs` |
| Shared Expense 計画体裁 | [shared-expense-implementation-plan.md](shared-expense-implementation-plan.md) |
| Export 現行 v6 | [export-schema.md](export-schema.md) |
| replicate 現行 | [itinerary-model.md §14](itinerary-model.md#14-itinerary-の複製itinerary-replicate) |
