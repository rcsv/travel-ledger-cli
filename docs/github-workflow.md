# GitHub 開発運用

caglla-cli を **小さなプロダクト** として継続開発するための、GitHub 上の Issue / PR / Milestone / Project の使い方です。

CI・Release・Dependabot などインフラ面は [development.md](development.md) を参照してください。

関連: [long-term-version-strategy.md](long-term-version-strategy.md) / [specifications/README.md](specifications/README.md) / [releases/README.md](releases/README.md)

---

## 基本方針

```text
チャットで議論した内容 → Issue / PR / docs/specifications/ に残す
設計系列は 6 フェーズで追跡する
製品メジャー（v2, v3…）は Milestone で区切る
進捗の俯瞰は GitHub Project で行う
```

v1 系（Summary / Reservation 等）で確立した設計系列を、v2 以降も GitHub 上で再現します。

---

## 設計系列（6 フェーズ）

新概念・大きな拡張は、原則として次の順序で進めます。

| # | フェーズ | Label | 主な成果物 | 典型リリース |
|---|---|---|---|---|
| 1 | **Responsibilities Review** | `phase:responsibilities-review` | `*-model.md`, `*-responsibilities-review.md` | documentation-only |
| 2 | **Entity Design** | `phase:entity-design` | `*-entity-design.md` | documentation-only |
| 3 | **Implementation Plan** | `phase:implementation-plan` | `*-implementation-plan.md` | documentation-only |
| 4 | **Implementation** | `phase:implementation` | Rust コード・テスト・export schema | feature release |
| 5 | **Post-Implementation Review** | `phase:post-implementation-review` | `*-post-implementation-review.md` | documentation-only |
| 6 | **Release** | `phase:release` | tag, Release notes, バイナリ | GitHub Release |

### v1 での先例（Reservation）

```text
v1.11.0  Responsibilities Review   → reservation-model.md
v1.12.0  Entity Design             → reservation-entity-design.md
v1.13.0  Implementation Plan        → reservation-implementation-plan.md
v1.18.0  Implementation             → コード + export v3
v1.19.0  Post-Implementation Review → reservation-responsibilities-review.md
```

各フェーズに対応する Issue テンプレートは [`.github/ISSUE_TEMPLATE/`](../.github/ISSUE_TEMPLATE/) にあります。

**documentation-only release** も正式なリリースです。設計・Hardening をパッチバージョンで刻む v1 のやり方を v2 以降も踏襲できます。

---

## Issue の使い方

### 新規 Issue

1. [Issues → New issue](https://github.com/rcsv/caglla-cli/issues/new/choose) からテンプレートを選択
2. **Phase label** と **Domain label**（下記）を付与
3. 対象 **Milestone** を設定
4. 成果物ドキュメントのパスを Issue 本文に記載

### Epic（親 Issue）

大きなテーマ（例: Participant Foundation）は **Epic Issue** として 1 本立て、6 フェーズ分の子 Issue をリンクします。

```markdown
## Design series

- [ ] #N Responsibilities Review
- [ ] #N+1 Entity Design
- [ ] #N+2 Implementation Plan
- [ ] #N+3 Implementation
- [ ] #N+4 Post-Implementation Review
- [ ] #N+5 Release v2.x.0
```

### 既知ギャップ・Maintenance

v1 完了時点の改善候補（Expense diff、Note export-md 等）は **Maintenance** テンプレートで Issue 化できます。v2 Epic と並行可能なパッチとして扱います。

---

## Pull Request

[PR テンプレート](../.github/pull_request_template.md) に従い、次を記載します。

- 関連 Issue（`Closes #123`）
- フェーズ（仕様 PR か実装 PR か）
- `make check` の結果
- ドキュメント更新の有無

### ブランチ命名（推奨）

```text
spec/participant-responsibilities-review
spec/participant-entity-design
feat/participant-crud
docs/v2.0.0-release-notes
fix/expense-list-filter
```

### マージ方針

- `master` への merge で [Rust CI](../.github/workflows/rust.yml) が走る
- 実装 PR は CI 成功を必須とする
- 仕様-only PR も CI は走るが、ドキュメントのみなら fmt/clippy/test は通常パスする

---

## Labels

### Phase（設計系列）

| Label | 用途 |
|---|---|
| `phase:responsibilities-review` | 責務整理 |
| `phase:entity-design` | エンティティ設計 |
| `phase:implementation-plan` | 実装計画 |
| `phase:implementation` | コード実装 |
| `phase:post-implementation-review` | 実装後レビュー |
| `phase:release` | リリース作業 |

### Type

| Label | 用途 |
|---|---|
| `type:spec` | 仕様・設計ドキュメント |
| `type:hardening` | 実装後責務レビュー（documentation-only 含む） |
| `type:infra` | CI・依存・リファクタ |

### Domain（概念・領域）

| Label | 用途 |
|---|---|
| `domain:participant` | v2 Participant |
| `domain:expense` | Expense / Shared Expense |
| `domain:reservation` | Reservation |
| `domain:summary` | Summary |
| `domain:note` | Note |
| `domain:itinerary` | Itinerary / Day / Trip |
| `domain:checklist` | Checklist |
| `domain:export` | export/import/diff/schema |
| `domain:travel-book` | しおり（export-md, PDF 等） |
| `domain:photo` | Photo（将来 v6） |
| `domain:attachment` | Attachment（将来 v6） |
| `domain:planning-foundation` | v1 基盤 |

既存の `bug`, `documentation`, `enhancement`, `dependencies` 等も引き続き使用します。

---

## Milestones

**製品メジャーテーマ** を Milestone で表します。CLI パッチバージョン（v1.23.0 等）とは一致しない場合があります（[long-term-version-strategy.md](long-term-version-strategy.md) 参照）。

| Milestone | テーマ | 状態 |
|---|---|---|
| **v1 Planning Foundation** | Trip / Day / Itinerary / Ledger 基盤 | 文書でクローズ（[planning-foundation-completion-review.md](specifications/planning-foundation-completion-review.md)、**tag なし**） |
| **v2 Participant Foundation** | 同行者を Trip に紐付け | **v2.0.0 リリース済み** |
| **v3 Shared Expense** | paid_by / beneficiary / settlement | 将来 |
| **v5 Travel Book** | Rich MD/PDF しおり | 将来 |
| **v6 Travel Journal** | Photo / Attachment | 将来 |

各 Milestone 内の Issue は Phase label で設計系列を追跡します。

---

## GitHub Project

リポジトリにリンクした Project **「Caglla CLI Development」** で俯瞰します。

| フィールド | 用途 |
|---|---|
| **Status** | Backlog → Design → Plan → Implement → Review → Release → Done |
| **Phase** | 6 フェーズと対応 |
| **Domain** | Participant / Expense / … |

### 運用

1. 新規 Epic または Phase Issue 作成時に Project へ追加（`gh project item-add` または UI）
2. Status をフェーズに合わせて更新
3. Milestone 完了時に Epic を Done へ

Project URL: [Caglla CLI Development (Project #4)](https://github.com/users/rcsv/projects/4)

---

## Release との接続

Release フェーズでは **Release** Issue テンプレートで作業を起票します。

1. `docs/releases/vX.Y.Z-notes.md` を作成
2. [releases/README.md](releases/README.md) に行を追加
3. `make check` + CI 確認
4. タグ `vX.Y.Z` を push → [release.yml](../.github/workflows/release.yml) がバイナリを生成

documentation-only release でも tag push と Release notes は **必須** です（v1.19–v1.22 Hardening と同様）。Planning Foundation 完了総括（本 Milestone の文書クローズ）は **tag を作らない**。

---

## v2 Participant Foundation — 推奨 Issue 系列

[planning-foundation-completion-review.md](specifications/planning-foundation-completion-review.md) §8 より:

```text
1. Epic: Participant Foundation (v2)
2. Responsibilities Review  → participant-model.md
3. Entity Design            → participant-entity-design.md
4. Implementation Plan      → participant-implementation-plan.md
5. Implementation           → CRUD + export v4
6. Post-Implementation Review
7. Release v2.0.0
```

v2 スコープ外（精算ロジック等）は Issue 本文の Non-goals / defer に明記し、**v3 Shared Expense** Milestone へリンクします。

---

## Cursor / ChatGPT への依頼

チャット上で設計・実装を進める場合も、**GitHub 上に残る状態** を前提に依頼してください。

### 依頼前に用意するもの

| 項目 | 内容 |
|---|---|
| **対象 Issue** | 番号と URL（例: #7 Responsibilities Review） |
| **Milestone** | 製品テーマ（例: v2 Participant Foundation） |
| **Phase** | 6 フェーズのどこか |
| **スコープ境界** | 今回やること / **やらないこと**（Non-goals） |
| **成果物パス** | 例: `docs/specifications/participant-model.md` |
| **参照ドキュメント** | 関連する specifications / release notes |

### 依頼文の例

```text
Issue #7（Responsibilities Review）に沿って participant-model.md を作成してください。
Milestone: v2 Participant Foundation
今回やらないこと: 精算ロジック、DB migration、CLI 実装
参照: travel-ledger-responsibilities.md, planning-foundation-completion-review.md §6
完了条件: PR 作成、make check PASS、Issue へのリンク
```

### 完了時に期待すること

1. 変更は **PR** 経由（直接 master へ push しない）
2. PR 本文に Related issue・Target milestone・`make check` 結果を記載
3. 仕様変更は `docs/specifications/` に残し、必要なら Issue を更新
4. 設計系列を進める場合は **次フェーズ用 Issue** が既にあればリンクする

---

## gh CLI クイックリファレンス

```bash
# Issue 作成（テンプレートは Web UI 推奨）
gh issue create --repo rcsv/caglla-cli --title "[Epic] Participant Foundation (v2)" \
  --label "domain:participant,type:spec" --milestone "v2 Participant Foundation"

# Milestone 一覧
gh api repos/rcsv/caglla-cli/milestones --jq '.[].title'

# Project に Issue を追加
gh project item-add <PROJECT_NUMBER> --owner rcsv --url https://github.com/rcsv/caglla-cli/issues/<N>
```

---

## 改訂

本ドキュメントは運用に合わせて更新します。ラベル・Milestone・Project フィールドの正は GitHub 上の実体を優先します。
