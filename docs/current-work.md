# Current Work — Caglla CLI / travel-ledger-cli

> **注意:** このファイルは正式仕様ではありません。開発中の **現在地メモ** です。設計・契約の正本は `docs/specifications/` および `docs/releases/` を参照してください。

最終更新: 2026-06-25

---

## 現在フェーズ

**v3.7.0 Receipt assignment and trash workflow**

Receipt Inbox を **assign（Expense 化）** / **trash（ゴミ箱）** / **restore** / **pending sum** で扱える workflow へ拡張するフェーズです。

---

## 最新完了

| 項目 | 内容 |
|---|---|
| Commit | `776bab6` — **Implement Receipt assignment and trash workflow** |
| 実装概要 | `receipt assign` / `trash` / `restore`、`pending sum`、`ignored → trashed` migration、export schema **v8**、v6/v7 import 互換、`validate-export` / `diff` 更新、tests / docs |
| 検証 | `make check` 通過済み（実装 commit 時点） |

設計系列:

```text
Workflow Design        → docs/specifications/v3.7.0-receipt-assignment-and-trash-workflow-design.md
Implementation Plan    → docs/specifications/v3.7.0-receipt-assignment-and-trash-implementation-plan.md
Implementation         → 776bab6
```

---

## 次アクション

**v3.7.0 release preparation**

実装は完了。次はリリース準備（レビュー文書・リリースノート・バージョン bump・索引更新）を進める。

---

## Release preparation checklist

実装完了後、v3.6.0 と同様の系列で release する。

- [ ] **Post-Implementation Review** を作成  
  `docs/specifications/v3.7.0-receipt-assignment-and-trash-post-implementation-review.md`
- [ ] **Release notes** を作成  
  `docs/releases/v3.7.0-notes.md`
- [ ] **`Cargo.toml` / `Cargo.lock`** を `3.7.0` に bump
- [ ] **`README.md`** の最新リリース参照を v3.7.0 に更新
- [ ] **`docs/command-reference.md`** を最終確認（assign / trash / restore / pending summary / schema v8）
- [ ] **`docs/specifications/README.md`** の v3.7.0 ステータスを「リリース済み」へ更新
- [ ] **`docs/long-term-version-strategy.md`** の v3.7 セクションを更新
- [ ] **`make check`** を再実行して通過を確認
- [ ] **Git commit**（release 作業用）
- [ ] **Git tag `v3.7.0`** + **GitHub Release**（必要時）

### Release notes に含める想定の要点

- Added: `receipt assign`, `receipt trash`, `receipt restore`, pending sum（`receipt list`）
- Changed: `receipt ignore` は deprecated alias（`trash` 相当）、export schema **v8**（`trashed_at`）
- Compatibility: v6 / v7 import 互換、`ignored` → `trashed_at` migration
- Unchanged: Receipt / Pending sum は **Actual ではない**（`trip stats` / `export-md` 非混在）
- Deferred: `receipt purge`, standalone `receipt summary`, Expense correction routes

---

## まだ始めないもの（defer / out of scope）

以下は **v3.7.0 release 後** または別テーマとして扱う。今は着手しない。

| テーマ | 理由 |
|---|---|
| **Evidence / Attachment** | 共通証憑レイヤーは別設計・別リリース |
| **`image_path`** | Receipt 専用画像パスは採用しない方針 |
| **OCR** | 自動解析は scope 外 |
| **Balance / Settlement** | 精算・分担は long-term defer |
| **Expense reassign / unassign / trash** | Receipt assign 後の Expense 補正ルートは v3.7.0 defer |
| **`receipt purge`** | Trash からの物理削除は defer |
| **standalone `receipt summary`** | pending sum は `receipt list` に統合済み |
| **Potential Actual display** | 旅行中の補助表示は defer |
| **Settlement warning** | Balance / Settlement 系とセットで将来検討 |
| **Day-level Planned vs Actual** | 別バージョン候補 |
| **Participant sharing** | Shared expense 以外の拡張は defer |

---

## 重要方針（実装済み・維持）

- Receipt は **Actual ではない**
- Pending Receipt sum は **Actual ではない**
- `receipt assign` 後に作成された **Expense だけ** が Actual に入る
- `trip export-md` / `trip stats` / `trip stats --json` に pending Receipt を **混ぜない**
- `receipt assign` は **transaction 必須**、完了後 **Receipt を削除**
- `linked` / `converted` / `receipt link` / `linked_expense_id` は **復活させない**

---

## クイック参照

| 用途 | パス |
|---|---|
| CLI コマンド一覧 | [command-reference.md](command-reference.md) |
| Export / import | [export-import.md](export-import.md) |
| 長期バージョン戦略 | [long-term-version-strategy.md](long-term-version-strategy.md) |
| 仕様索引 | [specifications/README.md](specifications/README.md) |
| リリースノート索引 | [releases/README.md](releases/README.md) |
