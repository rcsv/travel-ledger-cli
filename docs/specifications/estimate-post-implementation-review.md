# Estimate Post-Implementation Review

Caglla.Travel CLI の **Estimate（事前見積 / Planned Budget）** 実装（Phase 1–4）が、設計系列の意図と整合しているかを検証する Post-Implementation Review です。

**documentation-only。** 本書は新機能実装・DB migration・CLI 挙動変更・export schema 変更を伴わない。残課題は §9 Deferred Scope に記録する。

| ドキュメント | 役割 |
|---|---|
| [estimate-model.md](estimate-model.md) | Responsibilities Review — **上書きしない** |
| [estimate-entity-design.md](estimate-entity-design.md) | Entity Design — **上書きしない** |
| [estimate-implementation-plan.md](estimate-implementation-plan.md) | Implementation Plan — **上書きしない** |
| PR #50–#53 | Phase 1–4 Implementation |
| **本書** | **実装後**の責務・データモデル・テスト・残課題の整理 |

設計系列:

```text
Responsibilities Review        → estimate-model.md
Entity Design                  → estimate-entity-design.md
Implementation Plan            → estimate-implementation-plan.md
Phase 1 Implementation         → PR #50 (merge 213206e)
Phase 2 Implementation         → PR #51 (a071f8d)
Phase 3 Implementation         → PR #52 (3ef8e4e)
Phase 4 Implementation         → PR #53 (bbb8d80)
Post-Implementation Review     → this document
Release                        → 未着手
```

関連: [expense-post-implementation-review.md](expense-post-implementation-review.md) / [itinerary-model.md](itinerary-model.md) / [export-schema.md](export-schema.md) / [command-reference.md](../command-reference.md) / [long-term-version-strategy.md](../long-term-version-strategy.md)

---

## 1. Overview

Estimate 実装は Phase 1–4 を通じて、Caglla CLI の旅行計画モデルに **Planned Money（予定費用）** の正本を追加した。

### 到達点（設計上の確定事項）

| 概念 | 判定 |
|---|---|
| **Estimate** | **Planned Money** — 旅行前・計画段階の見込み金額 |
| **Expense** | **Actual Money** — 実績支出（既存。Estimate と混在しない） |
| **配置** | Itinerary 配下の **0..N 明細**（1 Itinerary : N Estimate） |
| **Planned Budget** | 独立エンティティ **ではない**。Trip / Itinerary 単位の予定合計は **Estimate 行の集計** |
| **Phase 1–4** | CRUD / export v6 / stats & export-md / replicate — **実装完了** |

```text
Planned total  = Σ Estimate.amount（通貨別）
Actual total   = Σ Expense.amount（通貨別）
Difference     = 未実装（表示レイヤーで将来導出）
```

**結論:** 当初 [estimate-model.md](estimate-model.md) で定義した責務（Planned vs Actual 分離、Itinerary 配下の構造化見積、集計は derived）が、CLI・DB・export・stats・replicate まで一貫して実装されている。

---

## 2. Phase Summary

| Phase | 内容 | PR | 代表 commit |
|---|---|---|---|
| **Phase 1** | `estimates` テーブル migration、`src/money.rs` 共通化、Estimate CRUD CLI、cascade（itinerary / trip delete、db reset） | [#50](https://github.com/rcsv/travel-ledger-cli/pull/50) | `ff434a1` feat: add estimate CRUD CLI (Phase 1) |
| **Phase 2** | export/import **schema v6**（`days[].itineraries[].estimates[]`）、validate-export、trip diff（v6+ 同士） | [#51](https://github.com/rcsv/travel-ledger-cli/pull/51) | `a071f8d` feat: add Estimate export schema v6 support |
| **Phase 3** | `trip stats` Planned total、`trip stats --json` additive fields、`trip export-md` 予定費用表 | [#52](https://github.com/rcsv/travel-ledger-cli/pull/52) | `3ef8e4e` feat: add Estimate planned totals and markdown export |
| **Phase 4** | `itinerary replicate` で Estimate コピー（`copy_estimates_for_itinerary`） | [#53](https://github.com/rcsv/travel-ledger-cli/pull/53) | `bbb8d80` feat: copy estimates during itinerary replicate |

Phase 1 merge commit: `213206e`（PR #50）。補助: `c5947ab` fix: clear estimates table on db reset。

---

## 3. Responsibility Review

[expense-post-implementation-review.md](expense-post-implementation-review.md) の「Expense = Actual Money / Estimate = Planned Money」結論を、Estimate 実装後も維持できているかを確認する。

| 境界 | レビュー | 判定 |
|---|---|---|
| **Estimate vs Expense** | 別テーブル・別 CLI・stats では Planned / Actual を分離表示。Expense に事前見積を入れる必要がなくなった | ✅ |
| **Estimate vs Reservation** | Reservation は amount 列なし。Estimate は予約番号を持たない。replicate でも Reservation はコピーしない | ✅ |
| **Estimate vs Note / Itinerary `note`** | Estimate は amount + currency **必須**。Note / Remark は金額なし。集計対象は Estimate のみ | ✅ |
| **Planned Budget 独立エンティティ** | Trip / Day 直下の budget 列・budget テーブルなし。Planned total は `list_estimates_for_trip` の集計 | ✅ |
| **Itinerary 配下配置** | Expense / Reservation と同型。行動単位の「この予定にいくら見込むか」に整合（[itinerary-model.md](itinerary-model.md)） | ✅ |

**Non-blocking 観察:** `estimate note` 列に「5人分」等の人数情報を書く運用は設計どおり。Participant との構造連動は deferred（§9）。

---

## 4. Data Model Review

### `estimates` テーブル

| カラム | 責務 | 判定 |
|---|---|---|
| `itinerary_id` | 親 Itinerary（必須） | ✅ |
| `title` | 項目名（nullable） | ✅ |
| `amount` | 最小通貨単位 INTEGER（必須） | ✅ |
| `currency` | ISO 4217（必須） | ✅ |
| `note` | 補足（nullable） | ✅ |
| `sort_order` | 同一 Itinerary 内の並び | ✅ |
| `created_at` / `updated_at` | 監査用 | ✅ |

### amount / currency 共通化

Phase 1 で `src/money.rs` を抽出し、`expense.rs` / `estimate.rs` から共用:

- `validate_currency_code`
- `parse_amount_for_currency`
- `format_amount_display` / `format_amount_value`

Expense 回帰は Phase 1 の既存 unit / CLI tests で維持。Estimate も JPY 整数・USD 小数入力を同一ルールで処理。

### 並び順

- 一覧: `sort_order ASC, id ASC`（`list_estimates_for_itinerary` / `list_estimates_for_trip`）
- export-md 表も同一順序

### Cascade / 削除

| 操作 | Estimate の扱い | テスト |
|---|---|---|
| `itinerary delete` | 配下 Estimate 削除 | `cli_estimate_cascade_itinerary_delete`, `test_delete_estimates_for_itinerary_cascade` |
| `trip delete` | Trip 配下 Estimate 削除 | `test_delete_estimates_for_trip_cascade` |
| `db reset` | `estimates` テーブルクリア | `test_reset_db_clears_estimates` |

**方針:** SQLite FK なし + アプリ側 cascade（Expense / Reservation と同型）。実装は `delete_estimates_for_itinerary` / `delete_estimates_for_trip` および itinerary / trip 削除フローから呼び出し。

---

## 5. Export / Import / Diff Review

### schema v6 判断

| 判断 | 理由 | 判定 |
|---|---|---|
| `days[].itineraries[].estimates[]` にネスト | Expense / Reservation と同型。Itinerary コンテキストを export 構造で保持 | ✅ |
| `id` / timestamps を export しない | import 時に新規採番。Expense v3+ と同型 | ✅ |
| v5 export → v6 import | 既存 v5 ファイルは `estimates[]` 省略 = 空配列として import 可能 | ✅ |
| v6 export → v5 import | **不可**（schema バージョン不一致）— 意図どおり | ✅ |

### validate-export

- schema v6+ で Estimate の `amount` / `currency` 形式を検証
- v5 import パスでは Estimate 専用チェックを **スキップ**（`cli_validate_export_v5_import_skips_estimate_checks`）

### trip diff

- `schema_supports_estimate_diff`: schema v6+ 同士でのみ Estimate 比較
- 比較対象: added / removed / amount / currency / title / note / sort_order
- v5 vs v6 混在時は Estimate diff を **行わない**（Expense v5+ ルールと同型）

**テスト:** `test_diff_estimate_added_and_amount_changed`（`src/diff.rs`）、`cli_export_import_reexport_roundtrip_with_estimates`（`tests/export_roundtrip_cli.rs`）

---

## 6. Stats / Markdown Review

Phase 3（PR #52）の結果:

| 項目 | 実装 | 判定 |
|---|---|---|
| `trip stats` Planned total | Trip 配下 Estimate 通貨別合計 | ✅ |
| `trip stats --json` | `estimate_count` / `estimate_totals` を **additive** 追加 | ✅ |
| 既存 `expense_count` / `expense_totals` | 維持。Human label を Actual total に整理 | ✅ |
| `export-md` Itinerary 内 | 見出し「予定費用:」+ 表（title / amount / note） | ✅ |
| Estimate 0 件 | Itinerary セクション・Overview Planned total を **省略** | ✅ |
| Difference | **defer** — docs に明記（entity-design / implementation-plan / estimate-model） | ✅ 意図どおり |

**テスト:** `tests/trip_stats_cli.rs`（4 tests）、`tests/export_md_cli.rs`（Estimate 関連 3 tests）、`src/stats.rs` unit tests、`src/estimate.rs` `test_format_estimates_markdown_section`

---

## 7. Replicate Review

Phase 4（PR #53）の判断:

| 方針 | 実装 | 判定 |
|---|---|---|
| Estimate は予定パターンの一部 → **コピー** | `copy_estimates_for_itinerary` を replicate トランザクション内で呼び出し | ✅ |
| Expense は実績 → **コピーしない** | 既存 `test_replicate_does_not_copy_expense_or_reservation` 維持 | ✅ |
| Reservation は予約実体 → **コピーしない** | 同上 | ✅ |
| `--without-notes` とは独立 | Note 省略時も Estimate はコピー | ✅ |
| `--without-estimates` | **未追加**（将来需要が明確になった場合に検討） | ✅ 意図どおり |
| `--dry-run` | Itinerary / Estimate とも DB 不変 | ✅ |
| コピー先 | 新 `id` / 新 `created_at` / `updated_at`。`title` / `amount` / `currency` / `note` / `sort_order` 維持 | ✅ |
| 独立性 | source / target の update が相互に影響しない | ✅ `test_replicated_estimates_are_independent` |

**テスト:** `test_replicate_copies_estimates`、`test_replicated_estimates_are_independent`、`test_replicate_dry_run_does_not_write`（Estimate 件数不変）、`cli_itinerary_replicate_copies_estimates`、`cli_itinerary_replicate_dry_run_does_not_create_estimates`

---

## 8. Test Coverage Review

`make check` PASS（324 unit + integration tests、okinawa seed 含む）。

### Integration tests

| ファイル | Estimate 関連の主なテスト |
|---|---|
| [tests/estimate_cli.rs](../../tests/estimate_cli.rs) | CRUD、list `--trip`/`--itinerary`、`--json`、cascade、USD 小数、validation |
| [tests/export_roundtrip_cli.rs](../../tests/export_roundtrip_cli.rs) | `cli_export_import_reexport_roundtrip_with_estimates` |
| [tests/validate_export_cli.rs](../../tests/validate_export_cli.rs) | v6 invalid currency、v5 skips estimate checks |
| [tests/trip_stats_cli.rs](../../tests/trip_stats_cli.rs) | Planned total、複数通貨、Estimate なし回帰、`--json` additive fields |
| [tests/export_md_cli.rs](../../tests/export_md_cli.rs) | 予定費用表、0 件省略、null title/note |
| [tests/itinerary_cli.rs](../../tests/itinerary_cli.rs) | replicate Estimate コピー、dry-run 不変 |

### Unit tests

| ファイル | Estimate 関連の主なテスト |
|---|---|
| [src/estimate.rs](../../src/estimate.rs) | CRUD、import/export v3 roundtrip、markdown section、`copy_estimates_for_itinerary`、cascade |
| [src/stats.rs](../../src/stats.rs) | `test_stats_estimate_count_and_totals`、`test_stats_estimate_multi_currency` |
| [src/diff.rs](../../src/diff.rs) | `test_diff_estimate_added_and_amount_changed` |
| [src/itinerary.rs](../../src/itinerary.rs) | replicate + Estimate（§7 参照） |
| [src/db.rs](../../src/db.rs) | `test_reset_db_clears_estimates` |

### カバレッジ上のギャップ（Non-blocking）

| 領域 | 状態 |
|---|---|
| doctor / advisor | Estimate 専用 issue code **なし**（deferred） |
| trip duplicate | Estimate は export → import 経由で間接的に検証。duplicate 専用 Estimate test なし |
| `--without-notes` + Estimate | 明示テストなし（replicate 本体テストで暗黙的にカバー） |

---

## 9. Deferred Scope

Phase 1–4 および本 Review **では実装しない** 範囲:

```text
- Difference calculation（Planned vs Actual 差分の stats / export-md 表示）
- Budget 独立エンティティ（Trip 全体予算上限）
- payer / beneficiary / participant 連動（Estimate 按分）
- unit_amount × quantity
- FX conversion（為替換算）
- --without-estimates（replicate 時に Estimate をコピーしないオプション）
- doctor / advisor での Estimate 活用（件数警告、Planned >> Actual 等）
- GUI / Web 版での Planned vs Actual カード表示
- release 作業（tag / version bump / release notes）
```

次ステップ: **Release**（Implementation Plan 外 — 別 Issue / PR で実施）。

---

## 10. Conclusion

Estimate 実装（Phase 1–4）は、Caglla CLI の旅行計画モデルにおいて **予定費用を構造化し、Trip 全体の Planned Budget（Estimate 集計）を扱う基盤** として成立した。

- **Planned Money** は Estimate 明細が正本、**Actual Money** は Expense が正本 — 境界は維持されている。
- export v6 / stats / export-md / replicate まで、Itinerary 中心の計画モデルと整合する。
- 残課題（Difference、Budget エンティティ、精算連動、doctor、GUI）は §9 に明示し、現行 master の **Release blocker とはしない**。

**Release 判定:** Estimate 機能単体として merge-ready。Cargo.toml version bump と release notes は別作業とする。

---

## References

| 用途 | パス |
|---|---|
| 責務整理 | [estimate-model.md](estimate-model.md) |
| Entity Design | [estimate-entity-design.md](estimate-entity-design.md) |
| Implementation Plan | [estimate-implementation-plan.md](estimate-implementation-plan.md) |
| Expense 対比 | [expense-post-implementation-review.md](expense-post-implementation-review.md) |
| replicate 仕様 | [itinerary-model.md §14](itinerary-model.md#14-itinerary-の複製itinerary-replicate) |
| Export v6 | [export-schema.md](export-schema.md) |
| CLI | [command-reference.md](../command-reference.md) |
