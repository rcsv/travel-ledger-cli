# Specifications

Caglla CLI の内部モデル・設計仕様（実装前の設計メモを含む）。

| ドキュメント | 状態 |
|---|---|
| [planning-design-principles.md](planning-design-principles.md) | Planning 設計原則（Itinerary / Checklist / Note / Reservation / Expense の役割分担 — v2.0.1 後） |
| [day-model.md](day-model.md) | Day モデル（v1.0.9–v1.2.0 反映済み） |
| [itinerary-model.md](itinerary-model.md) | Itinerary モデル（v1.8.0：行動単位、not a venue） |
| [ordering-model.md](ordering-model.md) | Ordering モデル（Sequence-first 原則・v1.9.0 実装済み） |
| [travel-ledger-responsibilities.md](travel-ledger-responsibilities.md) | Summary / Remark / Note / Reservation の責務分離（v1.10.0 文書化） |
| [summary-responsibilities-review.md](summary-responsibilities-review.md) | Summary Responsibilities Review（v1.14.0 設計前責務整理） |
| [summary-post-implementation-review.md](summary-post-implementation-review.md) | Summary Post-Implementation Review（v1.20.0 実装後責務再定義 — v1 Hardening） |
| [reservation-model.md](reservation-model.md) | Reservation モデル（v1.11.0 責務・境界） |
| [reservation-entity-design.md](reservation-entity-design.md) | Reservation Entity Design（v1.12.0 フィールド・種別・拡張） |
| [reservation-implementation-plan.md](reservation-implementation-plan.md) | Reservation Implementation Plan（v1.13.0 実装計画） |
| [reservation-responsibilities-review.md](reservation-responsibilities-review.md) | Reservation Responsibilities Review（v1.19.0 実装後責務整理 — v1 Hardening） |
| [note-model.md](note-model.md) | Note モデル（v1.3.0 CRUD、v1.4.0 export v2） |
| [note-post-implementation-review.md](note-post-implementation-review.md) | Note Post-Implementation Review（v1.21.0 実装後責務再定義 — v1 Hardening） |
| [expense-model.md](expense-model.md) | Expense モデル（v1.5.0 CRUD） |
| [expense-post-implementation-review.md](expense-post-implementation-review.md) | Expense Post-Implementation Review（v1.22.0 実装後責務定義 — v1 Hardening） |
| [estimate-model.md](estimate-model.md) | Estimate / Planned Budget Responsibilities Review（設計フェーズ 1/N — **未実装**） |
| [planning-foundation-completion-review.md](planning-foundation-completion-review.md) | Planning Foundation Completion Review（v1 総括 — Hardening 完結、**tag なし**） |
| [participant-model.md](participant-model.md) | Participant Model Responsibilities Review（v2.0.0 設計フェーズ 1/6）。参加行 vs 将来 Person / Traveler Profile、**count 意味論・`is_self`** |
| [participant-entity-design.md](participant-entity-design.md) | Participant Entity Design（v2.0.0 設計フェーズ 2/6）。`participants` = Trip-scoped participation record + `is_self` |
| [participant-implementation-plan.md](participant-implementation-plan.md) | Participant Implementation Plan（v2.0.0 設計フェーズ 3/6） |
| [participant-post-implementation-review.md](participant-post-implementation-review.md) | Participant Post-Implementation Review（v2.0.0 設計フェーズ 5/6） |
| [shared-expense-model.md](shared-expense-model.md) | Shared Expense Model Responsibilities Review（v3.0.0 設計フェーズ 1/6）。payer / beneficiaries / settlement 境界 |
| [shared-expense-entity-design.md](shared-expense-entity-design.md) | Shared Expense Entity Design（v3.0.0 設計フェーズ 2/6）。`expenses` 拡張・`expense_beneficiaries`・export v5 |
| [shared-expense-implementation-plan.md](shared-expense-implementation-plan.md) | Shared Expense Implementation Plan（v3.0.0 設計フェーズ 3/6）。migration / CLI / export v5 / tests |
| [shared-expense-post-implementation-review.md](shared-expense-post-implementation-review.md) | Shared Expense Post-Implementation Review（v3.0.0 設計フェーズ 5/6） |
| [shared-expense-release-review.md](shared-expense-release-review.md) | Shared Expense Release Review（v3.0.0 リリース後点検 — documentation-only） |
| [foundation-hardening-review.md](foundation-hardening-review.md) | Foundation Hardening Review（v2.0.0 後・v3 前の基盤点検 — documentation-only） |
| [checklist-design-memo.md](checklist-design-memo.md) | Checklist 設計メモ（自動生成・Provenance — 将来設計参考、v1.x 対象外） |
| [travel-support-design-memo.md](travel-support-design-memo.md) | Travel Support 設計メモ（旅行支援情報・Destination・注意喚起 — 将来設計参考、v1.x 対象外） |
| [export-schema.md](export-schema.md) | trip export / import JSON（schema v1–**v5**）。構造定義 — 意味論は itinerary-model 等を参照 |
