# Shared Expense Release Review

Caglla CLI **v3.0.0 Shared Expense** の正式リリース後点検です。Epic #13 設計系列 #30〜#35 が意図どおり完了しているか、ドキュメント・リリース状態・non-goals を確認します。

**本書は documentation-only。** Rust 実装・Cargo bump・tag / Release は行わない。

| ドキュメント | 役割 |
|---|---|
| [shared-expense-model.md](shared-expense-model.md) (#30) | Responsibilities Review |
| [shared-expense-entity-design.md](shared-expense-entity-design.md) (#31) | Entity Design |
| [shared-expense-implementation-plan.md](shared-expense-implementation-plan.md) (#32) | Implementation Plan |
| PR #39 | Implementation (#33) |
| [shared-expense-post-implementation-review.md](shared-expense-post-implementation-review.md) (#34 / PR #40) | Post-Implementation Review |
| [v3.0.0-notes.md](../releases/v3.0.0-notes.md) (#35 / PR #41) | Release |
| **本書** | **Post-release 点検** |

設計系列（Epic #13）:

```text
#30 Responsibilities Review        → shared-expense-model.md
#31 Entity Design                  → shared-expense-entity-design.md
#32 Implementation Plan             → shared-expense-implementation-plan.md
#33 Implementation                 → PR #39 (merge e92692b)
#34 Post-Implementation Review     → PR #40 (merge 376bd92)
#35 Release v3.0.0                 → PR #41 (merge 046ef3a), tag v3.0.0
```

---

## Purpose

1. v3.0.0 が #30〜#35 の設計系列どおりに **完了** していることを確認する。
2. Release notes・主要ドキュメント・export schema 記述の **整合** を点検する。
3. Shared Expense non-goals（Settlement 等）が **守られている** ことを確認する。
4. 残 follow-up を **v3.x / future / doc maintenance** に分類する。

**今回やらないこと:** Settlement 実装、v4 Travel Book 設計、Cargo bump、tag / Release。

---

## Release State

| 項目 | 確認 | 判定 |
|---|---|---|
| `Cargo.toml` / `Cargo.lock` | `3.0.0` | ✅ |
| Git tag | `v3.0.0` | ✅ |
| GitHub Release | https://github.com/rcsv/travel-ledger-cli/releases/tag/v3.0.0 | ✅ |
| Release workflow | linux / macos / windows バイナリ 3 件 | ✅ |
| `make check` | PASS（本レビュー時点） | ✅ |
| Working tree | clean | ✅ |
| Export schema（コード） | `TRIP_EXPORT_SCHEMA_VERSION = 5` | ✅ |

---

## Design Series Completion

| Issue | 成果物 | PR / tag | 判定 |
|---|---|---|---|
| #30 | shared-expense-model.md | #36 | ✅ |
| #31 | shared-expense-entity-design.md | #37 | ✅ |
| #32 | shared-expense-implementation-plan.md | #38 | ✅ |
| #33 | migration + CLI + export v5 | #39 | ✅ |
| #34 | post-implementation review + update 排他 fix | #40 | ✅ |
| #35 | bump + release notes + tag | #41, `v3.0.0` | ✅ |

PR #40 追補: `expense update` でも `--shared-with` と `--beneficiary` を reject（add 側と挙動一致）。v3.0.0 tag **前** に master へ反映済み。

---

## Release Notes Review

[v3.0.0-notes.md](../releases/v3.0.0-notes.md) は実装・設計と整合しています。

| 観点 | 記載 | 判定 |
|---|---|---|
| DB 拡張 | `paid_by_participant_id`, `expense_beneficiaries` | ✅ |
| personal / shared 判定 | beneficiary 行数、明示列なし | ✅ |
| CLI opt-in | add/update オプション、最小パス維持 | ✅ |
| add / update 排他 | `--shared-with` と `--beneficiary` | ✅ |
| Export v5 | `paid_by_participant_ref`, `beneficiaries[]` | ✅ |
| v4 import 互換 | 省略 = personal | ✅ |
| Participant delete | SET NULL + beneficiary DELETE | ✅ |
| diff / export-md / doctor | 各統合 | ✅ |
| Non-goals | Settlement, `trip expense-summary`, `share_ratio` 等 | ✅ |
| Upgrade / downgrade | migration 自動、v2.x downgrade 注意 | ✅ |

---

## Documentation Consistency

### 整合しているもの

| ドキュメント | 内容 | 判定 |
|---|---|---|
| [README.md](../../README.md) | 最新 Release → v3.0.0、Settlement 未対応 | ✅ |
| [releases/README.md](../releases/README.md) | v3.0.0 行 | ✅ |
| [long-term-version-strategy.md](../long-term-version-strategy.md) | v3.0.0 リリース済み、v3.x defer 明記 | ✅ |
| [export-import.md](../export-import.md) | 現行 schema v5、v1–v5 import、v4 互換、Expense diff v5+ | ✅ |
| [export-schema.md](export-schema.md) §Schema versions / §Top-level v5 | v5 フィールド定義 | ✅ |
| [command-reference.md](../command-reference.md) | Shared Expense CLI 例 | ✅ 軽微ギャップ（下記） |
| [shared-expense-post-implementation-review.md](shared-expense-post-implementation-review.md) | 実装整合・non-goals | ✅ 一部時点表現が Release 前のまま（下記） |

### 軽微な doc drift（Release blocker ではない）

| 箇所 | 内容 | 推奨 |
|---|---|---|
| [export-schema.md](export-schema.md) §validate-export | 「v4 追加検証（**現行 export**）」— 現行は v5 | v5 節追加・diff 節更新（Maintenance） |
| [export-schema.md](export-schema.md) §trip diff | Expense diff **非対象** と記載 — v3.0.0 で v5+ 対応済み | 同上 |
| [export-schema.md](export-schema.md) §将来バージョン | Shared Expense **未着手** — v5 で recording 済み | Settlement 行のみ defer と明確化 |
| [command-reference.md](../command-reference.md) | 排他は「`add` では」と記載 — update も reject | 「add / update ともに」へ修正 |
| [shared-expense-post-implementation-review.md](shared-expense-post-implementation-review.md) | 「v3.0.0-notes.md は #35 で作成」等の **Release 前** 表現 | 履歴 doc として許容。必要なら footnote |
| [foundation-hardening-review.md](foundation-hardening-review.md) | v3 前点検 — schema v4 現行表記 | 意図的に tag なし総括。v3 後 follow-up doc 可 |

**一次参照:** export 構造は [export-schema.md](export-schema.md) 冒頭 + [export-import.md](../export-import.md)。手順と v4 互換の説明は **export-import.md が v3.0.0 release で更新済み**。

---

## v4 Import Compatibility

| 観点 | ドキュメント | コード | 判定 |
|---|---|---|---|
| v4 export import 継続 | v3.0.0-notes §Migration、export-import §旧フォーマット | import パス v4 ルーティング | ✅ |
| v5 フィールド省略 = personal | v3.0.0-notes、export-schema | import 時 optional 省略 | ✅ |
| v4 validate-export | v5 ref 検査スキップ | `validate-export` 分岐 | ✅ |
| テスト | `participant_cli`, `trip_import_cli`, roundtrip | CI | ✅ |

---

## Non-goals Verification

### コード（`src/`）

| 除外項目 | 確認 | 判定 |
|---|---|---|
| Settlement / transfer CLI | `settlement` / `expense-summary` コマンドなし | ✅ |
| `share_ratio` / weighted split | DB 列・CLI なし | ✅ |
| `--paid-by` 単独エイリアス | 未実装 | ✅ |
| 独立 Shared Expense エンティティ | Expense 拡張のみ | ✅ |
| Person / Traveler Profile | 未着手 | ✅ |

### ドキュメント

| 除外項目 | 記載箇所 | 判定 |
|---|---|---|
| Settlement 計算 CLI | v3.0.0-notes §Non-goals、long-term §v3.x defer | ✅ |
| `trip expense-summary` | 同上 | ✅ |
| v4 Travel Book | long-term §v5 — 未着手 | ✅ |

---

## Follow-up Classification

### v3.x（Shared Expense 拡張）

| # | 内容 |
|---|---|
| 1 | `expense settlement` — transfer 計算 |
| 2 | `trip expense-summary` — read-only 集計 |
| 3 | `share_ratio` / `share_amount` |
| 4 | `--paid-by` エイリアス |
| 5 | `participant_ref { name, sort_order }` |
| 6 | `paid_by_name` → Participant backfill CLI |

### doc / test maintenance（v3.0.x パッチ可）

Post-Implementation Review §Non-blocking Follow-ups から継続:

| # | 内容 |
|---|---|
| 1 | diff Expense payer/beneficiaries 専用 unit テスト |
| 2 | export-md Paid by / Shared assertion |
| 3 | validate-export v5 ambiguous ref CLI integration |
| 4 | trip duplicate + shared fields integration |
| 5 | okinawa canonical payer/beneficiary 例示（任意） |
| 6 | export-schema.md validate-export / trip diff / 将来節の v5 更新 |
| 7 | command-reference 排他表記（add/update） |

### future（v3.x スコープ外）

| # | 内容 |
|---|---|
| 1 | v5 Travel Book — Shared Expense 専用レイアウト |
| 2 | v6 Travel Journal |
| 3 | Person / Traveler Profile |

---

## Conclusion

```text
v3.0.0 Shared Expense は Epic #13 設計系列 #30〜#35 どおり完了している。
Release state（tag v3.0.0、Release notes、Cargo 3.0.0、make check）は確認済み。
non-goals（Settlement / trip expense-summary）はコード・Release notes の双方で守られている。
残 follow-up は v3.x 機能拡張または doc/test maintenance に分類可能で、
v3.0.0 リリースを覆す問題はない。
```

次フェーズ候補: v3.x Settlement / expense-summary の Responsibilities Review、または doc maintenance Issue。

---

## Completion Criteria

| # | 条件 | 状態 |
|---|---|---|
| 1 | v3.0.0 release state 確認 | ✅ |
| 2 | follow-up を v3.x / future に分類 | ✅ §Follow-up Classification |
| 3 | `make check` PASS | ✅ |
| 4 | `Cargo.toml` / `Cargo.lock` = 3.0.0 | ✅ |
| 5 | tag / Release を新規作成しない | ✅ |
| 6 | Rust 実装・migration 追加なし | ✅ |
