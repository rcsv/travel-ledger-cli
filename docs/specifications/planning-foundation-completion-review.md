# Planning Foundation Completion Review（v1 総括 — 実装後レビュー）

Caglla.Travel CLI の **v1 Planning Foundation**（v1.0.0 〜 v1.22.0）を総括し、完成度・成果物・意図的 defer・v2 以降への接続を整理するレビューです。

**位置づけ（2026-06、v2.0.0 リリース後）:**

```text
v1 系 Planning Foundation の完了総括文書
  = Hardening 系列（v1.19–v1.22）の締めくくり
  = v2.0.0 Participant Foundation へ進んだ理由の整理
  = v3 以降 roadmap への接続

tag v1.23.0 / GitHub Release v1.23.0 は作らない
  （v2.0.0 リリース済みの現状に、過去パッチ release を追加しない）
```

本書は **documentation-only** であり、Rust / DB / export 変更を伴わない。

| ドキュメント | 役割 |
|---|---|
| [travel-ledger-responsibilities.md](travel-ledger-responsibilities.md) (v1.10.0) | Travel Ledger 横断責務 |
| [long-term-version-strategy.md](../long-term-version-strategy.md) | 製品メジャー版ロードマップ |
| Hardening 系列 (v1.19–v1.22) | 主要エンティティの実装後責務レビュー |
| [participant-post-implementation-review.md](participant-post-implementation-review.md) (v2.0.0) | v2 Participant Foundation 実装後レビュー |
| **本書** | **v1 Planning Foundation 完成宣言（文書上のクローズ）** |

Hardening 系列:

```text
v1.19.0  Reservation Post-Implementation Review
v1.20.0  Summary Post-Implementation Review
v1.21.0  Note Post-Implementation Review
v1.22.0  Expense Post-Implementation Review
（本書） Planning Foundation Completion Review  ← documentation-only, no v1.23.0 tag
```

---

## 1. レビュー結論

```text
v1 Planning Foundation は完了と言える。
```

当初想定した **旅行計画・実績記録の基盤** は、主要エンティティの実装・export v3・責務整理を経て **実質的に揃った**。本書の目的は新機能追加ではなく、

```text
何が完成したか
何が v1 ではやらなかったか（意図的 defer）
なぜ v2 Participant Foundation へ進んだか
v3 以降へ何を渡すか
```

を **v2.0.0 リリース後の文脈** で明文化することである。

**未実装の改善候補**（Expense diff、Note export-md 等）は **v1 完了を阻害しない**。v2 以降の **既知の non-blocking follow-up** として記録する（[participant-post-implementation-review.md](participant-post-implementation-review.md) §Non-blocking Follow-ups も参照）。

---

## 2. v1 Planning Foundation で完了した範囲

### 2.1 エンティティ

| エンティティ | 責務 | Export | CLI | Hardening |
|---|---|---|---|---|
| **Trip** | ○ | ○ `trip` | ○ CRUD + stats/doctor | モデル文書 |
| **Day** | ○ | ○ `days[]` | ○ list/show/update/swap | [day-model.md](day-model.md) |
| **Itinerary** | ○ | ○ nested | ○ CRUD + timeline | [itinerary-model.md](itinerary-model.md) |
| **Checklist** | ○ | ○ `checklist_items` | ○ + generate | 設計メモのみ ※ |
| **Note** | ○ | ○ `notes[]` | ○ CRUD | [note-post-implementation-review.md](note-post-implementation-review.md) |
| **Summary** | ○ | ○ trip/day summary | ○ trip/day update | [summary-post-implementation-review.md](summary-post-implementation-review.md) |
| **Expense** | ○ | ○ nested `expenses[]` | ○ CRUD + list --trip | [expense-post-implementation-review.md](expense-post-implementation-review.md) |
| **Reservation** | ○ | ○ nested `reservations[]` | ○ CRUD + list --trip | [reservation-responsibilities-review.md](reservation-responsibilities-review.md) |
| **Remark** | ○ | ○ itinerary `note` | ○ --note on itinerary | travel-ledger §3 |

※ Checklist 専用 Hardening は未実施 — v1 完了の阻害要因ではない。

### 2.2 横断機能（v1.22 時点）

| 機能 | 状態 |
|---|---|
| **export / import** | schema v3 現行（v2.0.0 以降は v4 — §5） |
| **validate-export** | v3 対応 |
| **trip duplicate** | v3 roundtrip |
| **trip diff** | Note / Summary / Reservation（Expense は改善候補） |
| **trip export-md** | Itinerary / Expense / Checklist / Reservation / Summary |
| **trip stats / doctor / advisor** | 計画点検・集計 |
| **ordering** | Sequence-first (v1.9) |
| **canonical sample** | okinawa_sesoko_2026 |

### 2.3 Travel Planning Model

```text
Trip
 └─ Day
      └─ Itinerary
           ├─ Expense
           ├─ Note（entity）
           └─ Reservation
Trip 配下: Checklist
Itinerary 行内: Remark
Trip / Day: Summary
```

**設計原則:** Itinerary is not a venue · Sequence-first · Local-first SQLite · Day 日付は導出。

### 2.4 Travel Ledger Architecture（v1 Hardening で確定）

```text
Reservation    = Booking Record Layer
Expense        = Transaction Record Layer

Remark         = Inline Annotation
Note           = Annotation Layer
Summary        = Abstract Layer
Travel Journal = Story Layer（将来）

Checklist      = Preparation Layer（Trip 配下）
```

---

## 3. v1 系でやらなかったこと（意図的 defer）

```text
未実装 ≠ v1 未完成
```

| 項目 | 先送り先 | 備考 |
|---|---|---|
| **Participant** | **v2.0.0 で実装済み** | 本書執筆時点では v2 完了 |
| **Shared Expense / Settlement** | v3 | Expense は Actual のみ |
| **Budget / Estimate** | 将来 | Expense ≠ Planned Money |
| **Person / Traveler Profile** | 将来 Root | v2 Participant は参加行のみ |
| **Travel Book**（Rich MD/PDF） | 製品 v5 | export-md が土台 |
| **Travel Journal** | 製品 v6 | Story Layer 責務のみ |
| **Photo / Attachment** | 製品 v6 | |
| **Venue**（POI 正本） | 将来 | Itinerary `location` で足りる |
| **Identity / Cloud / Platform** | v7–v9 | |

---

## 4. v2.0.0 で解決したこと（Participant Foundation）

v1 完了総括の時点で **次の製品テーマ** とされていた Participant は、**v2.0.0**（[v2.0.0-notes.md](../releases/v2.0.0-notes.md)）で実装された。

| 項目 | v2.0.0 の内容 |
|---|---|
| **`participants` テーブル** | Trip スコープの参加行、`is_self` |
| **CLI** | `participant add/list/show/update/delete` |
| **Export schema v4** | top-level `participants[]` |
| **Import** | v4 復元 + **v3 互換**（participants 省略可） |
| **diff / export-md / stats / doctor** | Participant 対応 |
| **count semantics** | self 1 件時のみ `companion_count` 算出 |

**v2 であえてやらなかったこと:** Person / Traveler Profile、`person_id`、Expense FK、Settlement、Reservation guest linking。

詳細: [participant-post-implementation-review.md](participant-post-implementation-review.md)

---

## 5. Export schema の進化

| Version | 内容 | 時期 |
|---|---|---|
| **v1** | Trip + flat itinerary + checklist | 初期 |
| **v2** | + `notes[]` | v1.4 |
| **v3** | + nested `days[]` + `expenses[]` + `reservations[]` + summary | v1.6 — **v1 Planning Foundation の export 正本** |
| **v4** | + top-level `participants[]` + `is_self` | **v2.0.0** — 現行 export |

- v3 JSON は v2.0.0+ CLI で **引き続き import 可能**
- canonical sample（`okinawa_sesoko_2026`）は v2.0.0 以降 **schema 4 + `participants: []`** を期待
- 一次仕様: [export-schema.md](export-schema.md)

---

## 6. Canonical sample への影響

[`samples/okinawa_sesoko_2026/`](../../samples/okinawa_sesoko_2026/README.md) — 実旅行由来の **行動台帳 + 清算** 検証データ。

| 項目 | 値 / 用途 |
|---|---|
| Itinerary | 58 件 |
| Expense | 49 件（¥561,780） |
| Participants | v2.0.0 以降 export は空配列可 |
| 用途 | export roundtrip、stats、seed テスト |

Reservation / Summary / Long-form Note の **共存例** は canonical には未投入（各 Hardening 文書で責務定義済み）。

---

## 7. v3 以降へ送ること

| テーマ | 内容 | 参照 |
|---|---|---|
| **v3 Shared Expense** | `paid_by_participant_id`、beneficiary、Settlement | [long-term-version-strategy.md](../long-term-version-strategy.md) §v3 |
| **Person / Traveler Profile** | Root スコープの人物正本 | [participant-model.md](participant-model.md) |
| **Travel Book** | Rich MD/PDF（v5） | export-md 土台 |
| **Travel Journal** | Story Layer 実装（v6） | summary/note Hardening |

v2 Participant の安定 `id` が v3 Expense 拡張の **参照先の正本** となる。

---

## 8. GitHub workflow 導入後の運用整理

PR #18 以降、設計系列を GitHub Issue / PR で追跡する運用が確立した（[github-workflow.md](../github-workflow.md)）。

| フェーズ | Participant Foundation（Epic #6）の例 |
|---|---|
| Responsibilities Review | #7 → `participant-model.md` |
| Entity Design | #8 → `participant-entity-design.md` |
| Implementation Plan | #9 → `participant-implementation-plan.md` |
| Implementation | #10 / PR #24 |
| Post-Implementation Review | #11 / PR #25 |
| Release | #12 → **v2.0.0** tag |

**本書（Planning Foundation 総括）** は v1 Hardening の **文書上の締め** であり、上記 6 フェーズの **Participant 系列とは別タイミング** で master に landing する（v2.0.0 後の retrospective documentation）。

v1 系 Hardening（v1.19–v1.22）は **パッチ release + tag** 付き。本書のみ **tag なし** — 履歴の混乱を避けるため。

---

## 9. v1 内の既知ギャップ（non-blocking）

| # | 項目 | 優先 |
|---|---|---|
| 1 | `trip diff` — Expense | 中 |
| 2 | `export-md` — Long-form Note | 中 |
| 3 | canonical — Reservation / Summary 共存 | 低 |
| 4 | Checklist Hardening レビュー | 低 |
| 5 | `export-import.md` — v4 記述の追従 | 低（export-schema が正） |

v2.0.0 / Participant リリースの **blocker ではない**。

---

## 10. 製品メジャーと CLI パッチの関係

[long-term-version-strategy.md](../long-term-version-strategy.md) の製品メジャー番号と **CLI パッチバージョンは一致しない** 場合がある。

| 製品メジャー | CLI での到達例 |
|---|---|
| v1 Planning Foundation | v1.0–v1.22（実装 + Hardening）+ 本書（総括・tag なし） |
| v2 Participant Foundation | **v2.0.0**（製品メジャー = CLI メジャー） |
| v4 Reservation（製品ロードマップ） | v1.18 で前倒し実装済み |

---

## 11. 参照

| 領域 | ドキュメント |
|---|---|
| Travel Ledger | [travel-ledger-responsibilities.md](travel-ledger-responsibilities.md) |
| ロードマップ | [long-term-version-strategy.md](../long-term-version-strategy.md) |
| Export | [export-schema.md](export-schema.md) |
| v2 Participant | [participant-post-implementation-review.md](participant-post-implementation-review.md) |
| v2.0.0 Release | [v2.0.0-notes.md](../releases/v2.0.0-notes.md) |
| Canonical | [samples/okinawa_sesoko_2026/](../../samples/okinawa_sesoko_2026/README.md) |
| GitHub 運用 | [github-workflow.md](../github-workflow.md) |
