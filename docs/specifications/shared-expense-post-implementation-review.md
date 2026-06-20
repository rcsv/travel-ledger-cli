# Shared Expense Post-Implementation Review

Caglla CLI v3.0.0 **Shared Expense** の実装（Issue #33 / PR #39）が、設計系列 #30 / #31 / #32 の意図と整合しているかを検証する Post-Implementation Review です。

**v3.0.0 時点: 仕様整理・リリース判定が主目的。** 本書は大きな実装変更を伴わない。改善候補は §Known Gaps / §Non-blocking Follow-ups に記録する。

| ドキュメント | 役割 |
|---|---|
| [shared-expense-model.md](shared-expense-model.md) (#30) | Responsibilities Review — **上書きしない** |
| [shared-expense-entity-design.md](shared-expense-entity-design.md) (#31) | Entity Design — **上書きしない** |
| [shared-expense-implementation-plan.md](shared-expense-implementation-plan.md) (#32) | Implementation Plan — **上書きしない** |
| PR #39 (`7f3424a`, merge `e92692b`) | Implementation — migration + CLI + export v5 |
| **本書** (#34) | **実装後**の整合性レビュー・Release 判定 |

設計系列（Epic #13）:

```text
#30 Responsibilities Review        → shared-expense-model.md
#31 Entity Design                  → shared-expense-entity-design.md
#32 Implementation Plan             → shared-expense-implementation-plan.md
#33 Implementation                 → PR #39 (merge e92692b)
#34 Post-Implementation Review     → this document
#35 Release v3.0.0                 → 次フェーズ
```

---

## Purpose

1. Issue #33 の実装が [shared-expense-implementation-plan.md](shared-expense-implementation-plan.md) の範囲内に収まっていることを確認する。
2. 設計前提（Transaction Record 拡張、beneficiary 行数による personal/shared 判定、Participant 削除時 cascade）がコード・CLI・export・doctor で一貫していることを確認する。
3. テスト・ドキュメント・canonical sample の不足を整理し、**Release blocker の有無**を判定する。
4. Issue #35 Release v3.0.0 に進める状態かを明記する。

---

## Source Documents

| 種別 | 参照 |
|---|---|
| ワークフロー | [github-workflow.md](../github-workflow.md) |
| 設計系列 | 上表の shared-expense-model / entity-design / implementation-plan |
| Participant 前提 | [participant-entity-design.md](participant-entity-design.md) |
| データモデル | [data-model.md](../data-model.md) |
| Export / Import | [export-import.md](../export-import.md), [export-schema.md](export-schema.md) |
| CLI | [command-reference.md](../command-reference.md) |
| Markdown | [markdown-export.md](../markdown-export.md) |
| 開発 | [development.md](../development.md) |
| ロードマップ | [long-term-version-strategy.md](../long-term-version-strategy.md) |
| 実装 | `src/expense.rs`, `src/participant.rs`, `src/trip.rs`, `src/diff.rs`, `src/markdown.rs`, `src/doctor.rs`, `src/advisor.rs`, `src/models.rs` |

---

## Implementation Summary

PR #39（merge `e92692b`, implementation `7f3424a`）で以下を `master` に反映した（19 files, +1949 / −140）。

| 領域 | 内容 |
|---|---|
| **DB** | `expenses.paid_by_participant_id INTEGER NULL`、`expense_beneficiaries` テーブル + indexes |
| **Domain** | `src/expense.rs` — migration、beneficiary CRUD、payer 列 read/write、import/export ref 解決 |
| **CLI** | `expense add/update` — `--paid-by-participant`, `--beneficiary`, `--shared-with all`, `--clear-paid-by`, `--clear-beneficiaries` |
| **Participant 連携** | `delete_participant` — payer SET NULL + beneficiary DELETE |
| **Export** | `schema_version: 5`、`paid_by_participant_ref`、`beneficiaries[]` |
| **Import** | v5 復元、v4/v3 互換（新フィールド省略 = personal）、同名 ref ambiguity error |
| **validate-export** | v5 ref 検証、v4 は v5 専用ルールスキップ |
| **diff** | Expense payer / beneficiaries 変更検出（schema ≥ 5 同士のみ shared フィールド比較） |
| **export-md** | Expense 行に `Paid by:` / `Shared:` 表示 |
| **doctor / advisor** | 3 warning code（DuplicateParticipantNames, SharedExpenseSingleBeneficiary, PaidByNameParticipantMismatch） |
| **Tests** | unit（`expense.rs`, `participant.rs`, `doctor.rs`, `db.rs`）、integration（`expense_cli.rs`, `export_roundtrip_cli.rs` 等） |
| **Docs** | command-reference、export-schema、golden `expected-export-v3.json` → schema 5 |

**意図的に未実装:** Settlement 計算 CLI、永続 Settlement、`share_ratio` / `share_amount`、`--paid-by` エイリアス、`trip expense-summary`、Cargo.toml version bump（#35 defer）。

---

## Scope Review

### Issue #33 実装範囲（Implementation Plan 対照）

| 項目 | 計画 | 実装 | 判定 |
|---|---|---|---|
| `paid_by_participant_id` 列 | §Table Plan | `migrate_expenses_shared_expense` | ✅ |
| `expense_beneficiaries` テーブル | §DDL | CREATE TABLE + indexes | ✅ |
| FK なし | §論点整理 | DDL に FK なし | ✅ |
| migration 冪等 | §Migration | 列存在チェック + `CREATE IF NOT EXISTS` | ✅ |
| personal/shared 明示列なし | §Domain | beneficiary 行数で判定 | ✅ |
| Participant 削除 cascade | §Participant 削除 | SET NULL + DELETE | ✅ |
| CLI opt-in | §CLI Plan | 新オプションのみ structured 入力 | ✅ |
| 既存 `expense add --amount --currency` | §変更なし | 回帰テスト `cli_expense_add_and_show` | ✅ |
| add: `--shared-with` と `--beneficiary` 排他 | §Open Q #10 | `cannot combine --shared-with and --beneficiary` | ✅ |
| update: beneficiary 全置換 | §Open Q #11 | `--clear-beneficiaries` + 置換ヘルパ | ✅ |
| export schema v5 | §Export Plan | `TRIP_EXPORT_SCHEMA_VERSION = 5` | ✅ |
| import v5 + v4 互換 | §Import Plan | ref 解決 + 省略パス | ✅ |
| validate-export v5 | §Validation | ambiguous ref error | ✅ unit |
| diff Expense | §Diff Plan | payer / beneficiaries field change | ✅ 専用 unit テストなし |
| export-md | §Markdown Plan | `format_expense_markdown_line` | ✅ 専用 shared テストなし |
| doctor 3 codes | §Doctor Plan | `collect_shared_expense_issues` | ✅ |
| trip duplicate | §Risk Notes | export → import 経由（ID remap 含む） | ✅ shared 専用テストなし |
| docs | §Docs Plan | command-reference, export-schema 更新 | ✅ |
| Cargo bump | §Non-goals | 未 bump — #35 で実施 | ✅ 意図どおり |

### スコープ外への踏み込み（なし）

| 除外項目 | コード確認 |
|---|---|
| Settlement / transfer 計算 | 未実装 |
| `share_ratio` / weighted split | 未実装 |
| `--paid-by` 単独エイリアス | 未実装 |
| `trip expense-summary` | 未実装 |
| 独立 Shared Expense エンティティ | なし — Expense 拡張のみ |
| `paid_by_name` 削除 | 列維持 |

**結論:** 実装範囲は Implementation Plan に収まっている。

---

## Design Consistency Review

### 設計前提の実装対照

| 前提 | 実装 | 判定 |
|---|---|---|
| Expense = Transaction Record のまま拡張 | `expenses` 列 + `expense_beneficiaries` 子テーブル | ✅ |
| personal デフォルト — beneficiary 0 件 | `expense_is_shared` / JSON `shared` 派生 | ✅ |
| v3.0.0 均等按分のみ | beneficiary ID リストのみ（ratio 列なし） | ✅ |
| Participant 削除 — payer SET NULL | `clear_paid_by_for_participant` | ✅ |
| Participant 削除 — beneficiary DELETE | `delete_beneficiaries_for_participant` | ✅ |
| shared → personal へ戻り得る | beneficiary 全削除で personal | ✅ |
| export ref = Participant.name | `paid_by_participant_ref`, `beneficiaries[].participant_ref` | ✅ |
| 同名 Participant + ref → error | import / validate-export | ✅ |
| structured 入力は Participant 必須 | `no participants registered for this trip` | ✅ |
| `--paid-by-name` のみは 0 件 Trip OK | CLI 分岐 | ✅ |
| `paid_by_name` と structured payer 同期 | add/update で Participant.name を `paid_by_name` に反映 | ✅ |
| doctor: ID を正、`paid_by_name` 不一致は warning | `PaidByNameParticipantMismatch` | ✅ |

**結論:** Entity Design / Model の設計前提はコード全体で一貫している。

---

## DB / Migration Review

| 観点 | 実装 | 判定 |
|---|---|---|
| 列追加 | `paid_by_participant_id INTEGER NULL` | ✅ |
| 新テーブル | `expense_beneficiaries` + 2 indexes | ✅ |
| 冪等 migration | `test_migrate_expenses_shared_expense_idempotent` | ✅ |
| 既存行 | NULL + beneficiary 0 件 — v2 意味維持 | ✅ `test_migrate_adds_paid_by_participant_id` |
| init_db 連携 | `db.rs` → `migrate_expenses_shared_expense` | ✅ |
| init_db テスト | `test_init_db_creates_expense_beneficiaries_table` | ✅ |
| itinerary / trip delete cascade | beneficiary 行も連鎖削除 | ✅ 既存 cascade テスト継続 |
| rollback / down migration | なし — 慣習どおり | ✅ |

**結論:** migration は v2.0.1 既存 DB を壊さず、新構造を安全に追加している。

---

## CLI Review

### 後方互換

```bash
expense add --itinerary 12 --amount 1500 --currency JPY
expense add --itinerary 12 --amount 980 --currency JPY --paid-by-name 太郎
```

Participant 0 件 Trip でも上記は従来どおり動作（`cli_expense_add_and_show`）。

### 新オプション

| オプション | add | update | 実装確認 | 判定 |
|---|---|---|---|---|
| `--paid-by-participant` | ✓ | ✓ | `cli_expense_add_with_paid_by_participant_and_beneficiaries` | ✅ |
| `--beneficiary`（繰り返し） | ✓ | ✓ | 同上 | ✅ |
| `--shared-with all` | ✓ | ✓ | `cli_expense_add_shared_with_all` | ✅ |
| `--clear-paid-by` | — | ✓ | `test_clear_paid_by_and_beneficiaries_on_update` | ✅ |
| `--clear-beneficiaries` | — | ✓ | `cli_expense_update_clear_beneficiaries` | ✅ |
| add 排他 | — | — | `cli_expense_rejects_shared_with_and_beneficiary_on_add` | ✅ |
| structured + 0 participants | — | — | `cli_expense_rejects_structured_without_participants` | ✅ |

### human / JSON 出力

| 出力 | 方針 | 実装 | 判定 |
|---|---|---|---|
| `expense show` human | Paid By + Shared 行 | `print_expense_detail` | ✅ |
| `expense list` human | Paid By 列 | `print_expense_list` | ✅ |
| `--json` | `paid_by_participant_id`, `shared`, `beneficiaries[]` | `expense_to_json` | ✅ `cli_expense_list_json` |

**結論:** CLI は opt-in 拡張であり、最小パスは変更されていない。

---

## Export / Import Review

| 観点 | 実装 | 判定 |
|---|---|---|
| `schema_version: 5` | `TRIP_EXPORT_SCHEMA_VERSION = 5` | ✅ |
| `paid_by_participant_ref` | `ExportExpenseV3` optional 拡張 | ✅ |
| `beneficiaries[]` | `ExportExpenseBeneficiaryV5` | ✅ |
| internal id 非 export | export は name ref のみ | ✅ |
| v5 roundtrip | `test_import_export_roundtrip_v5_fields` + CLI roundtrip | ✅ |
| v4 import 互換 | 新フィールド省略 = personal | ✅ `cli_participant_export_v4_roundtrip` 等 |
| v3 import 互換 | 既存 legacy テスト継続 | ✅ |
| ambiguous ref | import / validate error | ✅ unit（CLI integration 薄い） |
| validate-export v4 | v5 専用 ref 検査スキップ | ✅ 設計どおり |
| canonical sample | `expected-export-v3.json` → schema 5 | ✅ payer/beneficiary 例示なし（計画どおり任意） |

### Open Question #1（型戦略）— 実装での確定

`ExportExpenseV3` に optional v5 フィールドを追加し、`TRIP_EXPORT_SCHEMA_VERSION = 5` に昇格。`ExportExpenseV5` 新設は採用せず — 計画の推奨案どおり。

**結論:** export v5 化は v4 import を壊していない。

---

## Participant Delete Review

| 操作 | 期待 | テスト | 判定 |
|---|---|---|---|
| payer SET NULL | `paid_by_participant_id = NULL` | `test_participant_delete_clears_payer_and_beneficiaries`, `test_delete_participant_clears_expense_refs` | ✅ |
| beneficiary DELETE | 行削除 | 同上 | ✅ |
| shared → personal | beneficiary 0 件 | delete 後 assert | ✅ |

**結論:** Participant 削除時の cascade は Entity Design §3 と一致。

---

## Diff / Export-md / Doctor Review

### trip diff (`src/diff.rs`)

| 観点 | 実装 | 判定 |
|---|---|---|
| Expense added / removed | 既存 `expense_key` マッチング | ✅ |
| payer 変更 | `expense_modified` field `payer` | ✅ 実装あり |
| beneficiaries 変更 | field `beneficiaries` | ✅ 実装あり |
| v4 同士比較 | shared フィールド比較スキップ | ✅ `schema_supports_shared_expense_diff` |
| 専用 unit テスト | — | ⚠️ 未追加（§Known Gaps） |

### export-md (`src/markdown.rs` / `expense.rs`)

| 観点 | 実装 | 判定 |
|---|---|---|
| Paid by 表示 | `format_expense_markdown_line` — `Paid by: {name}` | ✅ |
| Shared 表示 | `Shared: {names}` | ✅ |
| personal expense | payer のみ / beneficiary なし | ✅ 既存テストで間接確認 |
| shared 専用 assertion | — | ⚠️ 未追加 |

### doctor / advisor

| Code | 重大度 | 条件 | 判定 |
|---|---|---|---|
| `DuplicateParticipantNames` | warning | 同名 Participant 2 件以上 | ✅ |
| `SharedExpenseSingleBeneficiary` | warning | beneficiaries 1 名のみ | ✅ |
| `PaidByNameParticipantMismatch` | warning | structured payer と `paid_by_name` 不一致 | ✅ |

advisor は上記に対する advice / try ヒントを提供。export-md / stats には doctor warning を反映しない — 計画 Open Q #5 どおり。

**結論:** diff / export-md / doctor は設計方針と一致。diff / export-md のテスト薄さは non-blocking。

---

## Test Coverage Review

### カバレッジ一覧

| 観点 | テスト所在 | 判定 |
|---|---|---|
| migration 冪等 | `test_migrate_expenses_shared_expense_idempotent` | ✅ |
| init_db creates table | `test_init_db_creates_expense_beneficiaries_table` | ✅ |
| payer + beneficiaries CRUD | `test_create_expense_with_payer_and_beneficiaries` | ✅ |
| duplicate beneficiary | `test_duplicate_beneficiary_rejected` | ✅ |
| participant delete cascade | `test_participant_delete_clears_payer_and_beneficiaries`, `test_delete_participant_clears_expense_refs` | ✅ |
| clear payer / beneficiaries update | `test_clear_paid_by_and_beneficiaries_on_update` | ✅ |
| export v5 unit roundtrip | `test_import_export_roundtrip_v5_fields` | ✅ |
| CLI add/update/shared/clear | `tests/expense_cli.rs` | ✅ |
| export v5 CLI roundtrip | `export_roundtrip_cli.rs`, `participant_cli.rs` | ✅ |
| v4 import 互換 | `participant_cli.rs`, `trip_import_cli.rs` | ✅ |
| doctor shared warnings | `test_doctor_detects_shared_expense_warnings` | ✅ |
| diff payer/beneficiaries | **専用テストなし** | ⚠️ |
| export-md Paid by / Shared | **専用テストなし** | ⚠️ |
| validate-export v5 ambiguous ref | **CLI integration なし** | ⚠️ unit のみ |
| trip duplicate + shared fields | export/import 経路、**shared 専用なし** | ⚠️ |
| canonical payer/beneficiary 例示 | okinawa sample に未追加 | ⚠️ 任意（計画どおり） |

### merge 前確認（8 項目 — 再掲）

| # | 確認項目 | 結果 |
|---|---|---|
| 1 | clean master で `make check` PASS | ✅ |
| 2 | v2.0.1 DB migration 後、既存 Expense 維持 | ✅ |
| 3 | Participant 0 件 Trip で `expense add` | ✅ |
| 4 | v4 import compatibility | ✅ |
| 5 | v5 export/import roundtrip | ✅ |
| 6 | Participant delete — SET NULL / DELETE | ✅ |
| 7 | 排他・update 全置換 | ✅ |
| 8 | doctor / diff / export-md | ✅ 実装確認（diff/export-md テスト薄い） |

---

## Known Gaps

実装バグではなく、テスト・ドキュメントの薄い箇所。

1. `diff.rs` に Expense payer / beneficiaries 変更の **専用 unit テスト**がない。
2. `export-md` の `Paid by:` / `Shared:` を assert するテストがない。
3. `validate-export` の v5 ambiguous `participant_ref` を叩く **CLI integration** テストがない。
4. `trip duplicate` で payer / beneficiary が remap される **shared 専用**テストがない（export/import 経路は間接カバー）。
5. canonical sample（okinawa）に payer / beneficiary 例示がない — 計画上 **任意**。
6. `v3.0.0-notes.md` draft が未作成 — **Issue #35** で正式化。

いずれも **Release blocker ではない**。

---

## Release Blockers

**なし。**

PR #39 merge 時点・本レビュー時点で、v3.0.0 Shared Expense のリリースを妨げる仕様ズレ・データ破壊・v4 import 互換破壊・migration 不備は確認されなかった。

---

## Non-blocking Follow-ups

| # | 内容 | 推奨タイミング |
|---|---|---|
| 1 | diff Expense payer / beneficiaries の unit テスト | v3.0.x パッチ or Maintenance Issue |
| 2 | export-md shared expense 行の assertion | 同上 |
| 3 | validate-export v5 ambiguous ref の CLI integration | 同上 |
| 4 | trip duplicate + shared fields の integration | Maintenance |
| 5 | okinawa canonical sample に payer / beneficiary 例示 | 任意 |
| 6 | [export-import.md](../export-import.md) の schema 説明を v5 中心に更新 | **Issue #35** docs |
| 7 | [v3.0.0-notes.md](../releases/v3.0.0-notes.md) の Release Notes 作成 | **Issue #35** |

---

## Deferred Scope

### v3.x（計画どおり未実装）

```text
trip expense-summary（read-only 集計）
expense settlement（transfer 計算）
share_ratio / share_amount
participant_ref { name, sort_order }
--paid-by エイリアス
paid_by_name → Participant backfill CLI
```

### v5 Travel Book / v6 Journal

export-md 以上の Shared Expense 専用レイアウト — 製品バージョンロードマップどおり未着手。

### Person / Traveler Profile

`persons` テーブル、`participants.person_id` — 未実装。Participant `id` は v3 Shared Expense の参照先として機能している。

---

## Release Readiness

### 判定

| 項目 | 判定 |
|---|---|
| v3.0.0 release blocker | **なし** |
| docs / release notes | export-schema / command-reference 更新済み。**v3.0.0-notes.md は #35 で作成** |
| Cargo.toml / Cargo.lock version bump | **Issue #35 で実施**（PR #39 では意図的に未 bump — 妥当） |
| tag / GitHub Release | **Issue #35 で実施** |
| **Issue #35 に進めるか** | **はい — 進めてよい** |

### Release #35 で行うこと（参考）

1. `Cargo.toml` / `Cargo.lock` を `3.0.0` に bump。
2. `v3.0.0-notes.md` を正式 Release Notes として作成。
3. Git tag + GitHub Release 作成。
4. （任意）Non-blocking follow-up を Maintenance Issue 化。

### レビュー結論（一文）

```text
Shared Expense 実装は設計系列 #30–#33 および PR #39 と整合しており、
v3.0.0 Release（Issue #35）に進める状態である。
```

---

## Completion Criteria

| # | 条件 | 状態 |
|---|---|---|
| 1 | 本書 `shared-expense-post-implementation-review.md` が存在 | ✅ |
| 2 | #30 / #31 / #32 / #33 / PR #39 との整合確認 | ✅ |
| 3 | release blocker の有無が明確 | ✅ なし |
| 4 | non-blocking follow-up 整理 | ✅ §Non-blocking Follow-ups |
| 5 | deferred scope 整理 | ✅ §Deferred Scope |
| 6 | Issue #35 進行可否の明記 | ✅ 進めてよい |
| 7 | 関連 doc からのリンク | ✅ README 等（本 PR） |
| 8 | 大きな実装変更なし | ✅ documentation-only |
| 9 | `make check` PASS | ✅ |

---

## Next phase notes（Release #35）

Post-Implementation Review 完了後、Issue #35 `[Release] v3.0.0` でバージョン bump・正式 Release Notes・tag を行う。本書の §Non-blocking Follow-ups は Release をブロックしない。
