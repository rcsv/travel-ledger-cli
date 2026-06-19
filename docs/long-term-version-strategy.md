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

Summary と Reservation は **設計 → 実装（v1.17.0 / v1.18.0）まで完了**。v1 系の残りは **Hardening（責務レビュー・改善候補の文書化）** と、旅行計画表現の磨き込み。

### v1 系に持ち込まない想定（現時点）

```text
Photo
Attachment
Participant
```

まず **Summary** と **Reservation** を完成させ、旅行計画を **完結して表現できる状態** を優先する。

---

## v2 — Participant Foundation

### テーマ

```text
誰と旅行するか
```

### 追加予定

```text
Participant
```

### スコープ

この段階では **精算機能は持ち込まない**。

例:

```text
父 / 母 / 妻 / 長男 / 次男
```

参加者情報を Trip に紐付けられることが目的。

設計系列（GitHub Epic #6）: [participant-model.md](specifications/participant-model.md)（#7）→ [participant-entity-design.md](specifications/participant-entity-design.md)（#8）→ Implementation Plan → export v4。

---

## v3 — Shared Expense

### テーマ

```text
誰が払ったか
誰の費用か
```

### 追加予定

```text
Paid By
Beneficiary
Settlement
```

ここで初めて **Expense が Participant と結び付く**。家族旅行・グループ旅行の精算が可能になる。

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
| Hardening（予定） | 将来 | Generator、canonical、diff 等 |

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
