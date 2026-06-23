# Caglla.Travel 長期バージョン戦略

Caglla.Travel（CLI / 将来 Web）の **メジャーバージョンごとの到達イメージ** を整理したロードマップメモです。

**本書の位置付け:**

- 今後の設計議論・優先順位判断の **参考資料**
- **実装指示ではない** — スケジュール・スコープの確約でもない
- リポジトリの **パッチリリース**（v1.14.0、v1.16.0 等の documentation-only release）とは **別軸** の「製品メジャー版」想定

関連: [Travel Ledger Responsibilities](specifications/travel-ledger-responsibilities.md) / [Summary Implementation Plan](specifications/summary-implementation-plan.md) / [Reservation Implementation Plan](specifications/reservation-implementation-plan.md) / [data-model.md](data-model.md)

---

## 基本方針

メジャーバージョンは **技術的な区切り** ではなく、

```text
ユーザーが何をできるようになるか
```

で区切る。

---

## v1 — Planning Foundation

**現在の到達点（および v1 系で完結させたい基盤）。**

### 目的

```text
旅行計画を立てられる
旅行実績を記録できる
```

### 主な要素

```text
Trip
Day
Itinerary
Checklist
Expense
Note
Remark
Summary        ← v1.17.0 実装済み
Reservation    ← v1.18.0 実装済み
```

Summary と Reservation は **設計 → 実装（v1.17.0 / v1.18.0）まで完了**。v1 系 Hardening（v1.19–v1.22）の後、**Planning Foundation 完了総括** を [planning-foundation-completion-review.md](specifications/planning-foundation-completion-review.md) に文書化（**tag v1.23.0 は作らない** — v2.0.0 リリース後に landing）。

**v1 Planning Foundation** は v1.22 + Hardening 系列で実質完了。**製品 v2 Participant Foundation** は [v2.0.0](releases/v2.0.0-notes.md) でリリース済み。

### v1 系に持ち込まない想定（v1 クローズ時点 — 意図的 defer）

```text
Photo
Attachment
Participant          ← v2.0.0 で実装済み（本節は v1 当時の defer 記録）
Travel Journal（実装）
Budget / Settlement
```

v1 完了後、製品の次テーマは **v2 Participant Foundation**（§v2、**v2.0.0 リリース済み**）であった。v2 完了後の次テーマは **v3 Shared Expense**（§v3、**v3.0.0 リリース済み**）。v3.0.0 後の v3 系機能追加として **Estimate / Planned Budget** は **v3.1.0 リリース済み**（§v3.1）。

---

## v2 — Participant Foundation（**v2.0.0 リリース済み**）

### テーマ

```text
誰と旅行するか
```

### 実装内容（v2.0.0）

```text
Participant（Trip-scoped participation record）
```

リリースノート: [v2.0.0-notes.md](releases/v2.0.0-notes.md)

v2 の `participants` は **ある Trip への参加行**（TripParticipant-like）であり、**人そのものの正本ではない**。パスポート・生年月日・マイレージ等の Trip 横断プロフィールは、将来 Root スコープの **Person / Traveler Profile** として検討する（v2.0.0 では未実装）。詳細は [participant-model.md](specifications/participant-model.md#conceptual-model-person-vs-trip-participation)。

### スコープ

この段階では **精算機能は持ち込まない**。Root-level Person / Traveler Profile も **持ち込まない**。

例:

```text
父 / 母 / 妻 / 長男 / 次男
```

参加者情報を Trip に紐付けられることが目的（参加関係のレジストリ）。**Participant は自分を含む旅行参加者全員** を指す（Companion は自分以外）。人数統計では `participant_count` と `companion_count` を混同しない。v2.0.0 では `is_self` 列で「この Trip における自分」をマークする。詳細は [participant-model.md §Participant count semantics](specifications/participant-model.md#participant-count-semantics)。

設計系列（GitHub Epic #6）: [participant-model.md](specifications/participant-model.md)（#7）→ [participant-entity-design.md](specifications/participant-entity-design.md)（#8）→ 設計補正（#21 / #22）→ [participant-implementation-plan.md](specifications/participant-implementation-plan.md)（#9）→ Implementation（#10 / PR #24）→ [participant-post-implementation-review.md](specifications/participant-post-implementation-review.md)（#11）→ Release v2.0.0（#12）。

---

## v3 — Shared Expense（**v3.0.0 リリース済み**）

### テーマ

```text
誰が払ったか
誰の費用か
```

### 実装内容（v3.0.0）

```text
Paid By（structured payer）
Beneficiaries（equal split recording）
Export schema v5
```

リリースノート: [v3.0.0-notes.md](releases/v3.0.0-notes.md)

ここで **Expense が Participant と結び付く**。beneficiary 0 件 = personal、1 件以上 = shared（Settlement 計算 CLI は v3.x defer）。

設計系列（GitHub Epic #13）: [shared-expense-model.md](specifications/shared-expense-model.md)（#30）→ [shared-expense-entity-design.md](specifications/shared-expense-entity-design.md)（#31）→ [shared-expense-implementation-plan.md](specifications/shared-expense-implementation-plan.md)（#32）→ Implementation（#33 / PR #39）→ [shared-expense-post-implementation-review.md](specifications/shared-expense-post-implementation-review.md)（#34 / PR #40）→ Release v3.0.0（#35）。

### v3.x defer

```text
Settlement / transfer calculation
trip expense-summary
share_ratio / weighted split
--paid-by alias
```

---

## v3.1 — Estimate / Planned Budget（**v3.1.0 リリース済み**）

### テーマ

```text
予定費用（Planned Money）の構造化
Itinerary 配下の事前見積
Trip 全体の Planned total（Estimate 集計）
```

### 実装内容（v3.1.0）

```text
Estimate CRUD（Itinerary 配下）
src/money.rs（amount / currency 共通化）
Export schema v6（days[].itineraries[].estimates[]）
validate-export / trip diff（v6+）
trip stats Planned total
trip export-md 予定費用表示
itinerary replicate Estimate コピー
Post-Implementation Review（PR #57）
```

リリースノート: [v3.1.0-notes.md](releases/v3.1.0-notes.md)

**Planned Budget** は独立エンティティ **ではない**。Trip / Itinerary 単位の予定合計は **Estimate 行の derived 集計**。Actual Money は従来どおり Expense が正本。

設計系列: [estimate-model.md](specifications/estimate-model.md) → [estimate-entity-design.md](specifications/estimate-entity-design.md) → [estimate-implementation-plan.md](specifications/estimate-implementation-plan.md) → Phase 1–4（PR #50–#53）→ [estimate-post-implementation-review.md](specifications/estimate-post-implementation-review.md)（PR #57）→ Release v3.1.0。

### v3.1.x defer（Estimate 関連）

```text
Budget 独立エンティティ（Trip 全体予算上限）
Estimate payer / beneficiary / participant 連動
unit_amount × quantity
FX conversion
--without-estimates（replicate）
doctor / advisor での Estimate 活用
GUI / Web 版 Planned vs Actual 表示
```

v3.0.0 の Shared Expense defer（Settlement 等）は **引き続き v3.x defer**。

---

## v3.2 — Database Status（**v3.2.0 リリース済み**）

### テーマ

```text
参照中の caglla.db を明示する
サンプル DB・本番 DB・テスト DB の混同を防ぐ
```

### 実装内容（v3.2.0）

```text
db path（絶対パス表示、open なし）
db status / db status --json（存在確認、table counts、trip export schema version）
open 前 dispatch（未存在 DB を作成しない）
db reset 既存挙動維持
Post-Implementation Review（v3.2.0-db-status-post-implementation-review.md）
```

リリースノート: [v3.2.0-notes.md](releases/v3.2.0-notes.md)

設計系列: [v3.2.0-db-status-implementation-plan.md](specifications/v3.2.0-db-status-implementation-plan.md) → PR #58 → Post-Implementation Review → Release v3.2.0。

### v3.2.x defer

```text
--db <path> / CAGLLA_DB / db use（DB パス切替・永続設定）
doctor / advisor Estimate 活用
Trip / Itinerary command handler の commands/ 一括移動
```

---

## v3.2.1 — Module Layout（**v3.2.1 リリース済み**）

### テーマ

```text
src/ 内部構成の責務別整理
behavior-preserving refactor
```

### 実装内容（v3.2.1）

```text
cli/ commands/ domain/ storage/ analysis/ io/ output/
main.rs から Clap 定義を cli/args.rs へ
print_json を output::json へ集約
Post-Implementation Review（v3.2.1-module-layout-post-implementation-review.md）
```

リリースノート: [v3.2.1-notes.md](releases/v3.2.1-notes.md)

**挙動非変更** — CLI・JSON schema・DB schema・export/import 形式に変更なし。

設計系列: [v3.2.1-module-layout-implementation-plan.md](specifications/v3.2.1-module-layout-implementation-plan.md) → PR #59 → Post-Implementation Review → Release v3.2.1。

---

## v3.3 — Planned vs Actual Difference（**v3.3.0 リリース済み**）

### テーマ

```text
Trip 単位で予定（Estimate）と実績（Expense）の通貨別差分を表示する
```

### 実装内容（v3.3.0）

```text
Difference = Actual - Planned（通貨別、derived 集計）
trip stats human 出力に Difference:
trip stats --json に difference_totals（additive）
trip export-md Overview に Difference:
DB schema / export schema v6 変更なし
Post-Implementation Review（v3.3.0-planned-vs-actual-post-implementation-review.md）
```

リリースノート: [v3.3.0-notes.md](releases/v3.3.0-notes.md)

**表示 gate:** `estimate_count > 0 && expense_count > 0` のときのみ Difference を出す。Estimate のみ / Expense のみの Trip では省略。

設計系列: [v3.3.0-planned-vs-actual-implementation-plan.md](specifications/v3.3.0-planned-vs-actual-implementation-plan.md) → PR #60 → Post-Implementation Review → Release v3.3.0。

### v3.3.x defer

```text
Budget 独立エンティティ
doctor / advisor での Estimate 活用（v3.5.0 候補）
FX conversion
--db <path> / CAGLLA_DB / db use
trip export JSON への difference 永続化
Shared Expense / Settlement 連動
GUI / Web 版 Planned vs Actual カード表示
```

v3.0.0 の Shared Expense defer（Settlement 等）および v3.1.x の Budget / participant 連動 defer は **引き続き v3.x defer**。

---

## v3.4 — Itinerary-level Planned vs Actual Difference（**計画中**）

### テーマ

```text
Itinerary 単位で予定（Estimate）と実績（Expense）の通貨別差分を export-md に表示する
```

### 実装予定（v3.4.0）

```text
Difference = Actual - Planned（Itinerary 内、通貨別、derived 集計）
gate = itinerary 内 estimate_count > 0 && expense_count > 0
trip export-md Itinerary セクションに Planned / Actual / Difference サマリー
DB schema / export schema v6 変更なし
trip stats / JSON への Itinerary 一覧は追加しない
```

設計系列: [v3.4.0-itinerary-planned-vs-actual-implementation-plan.md](specifications/v3.4.0-itinerary-planned-vs-actual-implementation-plan.md) → Implementation → Post-Implementation Review → Release v3.4.0。

**Difference vs Settlement:** Difference は `Actual − Planned` のみ。事前入金・最終精算バランスは Balance / Settlement 別レイヤー（v3.4.0 非対象）。

### v3.4.x defer

```text
Day 単位 difference
trip stats human / --json への itinerary_differences
doctor / advisor Estimate 活用（v3.5.0 候補）
Budget 独立エンティティ
FX conversion
Balance / Settlement / advance payment / transfer
--db <path> / CAGLLA_DB / db use
commands/ への段階移行
```

---

## v4 — Reservation

### テーマ

```text
予約情報の正式管理
```

### 追加予定

```text
Reservation Entity
Reservation Export
Reservation Display
```

**設計フェーズは v1 系ドキュメントで進行済み**（[reservation-model.md](specifications/reservation-model.md) 系列）。製品メジャー v4 では、旅行計画の **中核情報として Reservation を正式実装** する到達点を想定。

> **注:** v1 系パッチで Summary 実装が先に入る可能性はある。本ロードマップの **番号** と **CLI パッチバージョン** は一致しない場合がある。

---

## v5 — Travel Book

### テーマ

```text
旅のしおり
```

### 追加予定

```text
Rich Markdown Export
PDF Export
Reservation Integration
Summary Integration
```

旅行 **前** の共有資料を生成できる状態を目指す。

---

## v6 — Travel Journal

### テーマ

```text
旅行記録
```

### 追加予定

```text
Photo
Attachment
```

旅行 **後** の記録保存・振り返りを強化する。

---

## v7 — Identity

### テーマ

```text
利用者
```

### 追加予定

```text
User
Authentication
```

**クラウド同期はまだ行わない**。まず「利用者」という概念を導入する。

---

## v8 — Cloud

### テーマ

```text
共有と同期
```

### 追加予定

```text
Cloud Sync
Backup
Sharing
```

複数端末での利用・共有を実現する。

---

## v9 — Platform

### テーマ

```text
Caglla Engine
```

### 追加予定

```text
Desktop Application
Mobile Application
Backend Library
Public API
```

CLI を中心とした **旅行プラットフォーム** へ発展させる。

---

## 一覧（早見表）

| メジャー | テーマ | ユーザーができること（イメージ） |
|---|---|---|
| **v1** | Planning Foundation | 計画・実績の記録、しおりの土台 |
| **v2** | Participant Foundation | 同行者を Trip に登録 |
| **v3** | Shared Expense | 誰が払い・誰の費用かを整理・精算 |
| **v3.1** | Estimate / Planned Budget | Itinerary 配下の予定費用・Planned total |
| **v3.2** | Database Status | 参照中 `caglla.db` の明示・状態確認 |
| **v3.2.1** | Module Layout | `src/` 責務別 module 整理（refactor） |
| **v3.3** | Planned vs Actual Difference | Trip 単位の通貨別差分（stats / export-md） — **v3.3.0 リリース済み** |
| **v3.4** | Itinerary Planned vs Actual | Itinerary 単位差分（export-md） |
| **v4** | Reservation | 予約情報の正式管理 |
| **v5** | Travel Book | 共有用しおり（MD/PDF） |
| **v6** | Travel Journal | 写真・添付付き旅行記 |
| **v7** | Identity | 利用者・アカウント |
| **v8** | Cloud | 同期・バックアップ・共有 |
| **v9** | Platform | デスクトップ / モバイル / API |

---

## v1 系ドキュメント系列（参考）

Summary / Reservation はいずれも **3 段階** で設計を整理している。

```text
Responsibilities Review  →  Entity Design  →  Implementation Plan
```

| 概念 | 責務整理 | Entity Design | Implementation Plan | Hardening Review |
|---|---|---|---|---|
| **Summary** | v1.14.0 | v1.15.0（同梱） | v1.16.0 | v1.20.0 |
| **Reservation** | v1.11.0 | v1.12.0 | v1.13.0 | v1.19.0 |

実装着手順序は本ロードマップの **v1 優先（Summary → Reservation）** と整合させる。

**Summary 実装:** v1.17.0 で Trip / Day Summary を実装済み（[v1.17.0-notes.md](releases/v1.17.0-notes.md)）。

**Reservation 実装:** v1.18.0 で Itinerary 配下 Reservation を実装済み（[v1.18.0-notes.md](releases/v1.18.0-notes.md)）。保存は Itinerary 配下、Trip 一覧は表示・集約ビュー。

**v1 Hardening（documentation-first）:** v1.19.0 より、実装後の責務レビューをパッチリリースで記録する。コード変更は伴わない。

| 段階 | パッチ例 | 内容 |
|---|---|---|
| 設計 | v1.11–v1.13 / v1.14–v1.16 | Reservation / Summary 系列 |
| 実装 | v1.17.0 / v1.18.0 | Summary / Reservation CRUD + export |
| **Hardening** | **v1.19.0** | Reservation 実装後責務レビュー |
| **Hardening** | **v1.20.0** | **Summary 実装後責務再定義（Abstract / Journal 分離）** |
| **Hardening** | **v1.21.0** | **Note 実装後責務再定義（Annotation / Narrative 境界）** |
| **Hardening** | **v1.22.0** | **Expense 実装後責務定義（Transaction Record / Budget 分離）** |
| **Hardening** | **（文書のみ）** | **Planning Foundation 完了総括** — [planning-foundation-completion-review.md](specifications/planning-foundation-completion-review.md)（**tag なし**、v2.0.0 後 landing） |

---

## CLI 方向性（参考）

CLI は当面、**確認・検証・連携** を重視する。

```text
show / list / validate / export / import / diff / export-md / doctor / stats
```

細かな手入力・編集系コマンドは、将来 GUI が成熟した段階で **deprecated / obsolete 候補** になり得る。現行バージョンでは Summary 編集を含む CRUD は引き続きサポートする。

---

## 改訂

本メモはプロダクト判断に応じて更新する。実装・リリースの正は各 [releases/](releases/) ノートおよび [specifications/](specifications/) を参照する。
