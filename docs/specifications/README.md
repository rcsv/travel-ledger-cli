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
| [estimate-model.md](estimate-model.md) | Estimate / Planned Budget Responsibilities Review（Phase 1–4 実装済み） |
| [estimate-entity-design.md](estimate-entity-design.md) | Estimate Entity Design（DDL・CLI・export v6 — 実装済み） |
| [estimate-implementation-plan.md](estimate-implementation-plan.md) | Estimate Implementation Plan（Phase 1–5 完了） |
| [estimate-post-implementation-review.md](estimate-post-implementation-review.md) | Estimate Post-Implementation Review（Phase 5 — 実装後責務・テスト整理） |
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
| [export-schema.md](export-schema.md) | trip export / import JSON（schema v1–**v7**）。構造定義 — 意味論は itinerary-model 等を参照 |
| [v3.2.0-db-status-implementation-plan.md](v3.2.0-db-status-implementation-plan.md) | v3.2.0 Database Status Implementation Plan（Phase 1 完了 — PR #58） |
| [v3.2.0-db-status-post-implementation-review.md](v3.2.0-db-status-post-implementation-review.md) | v3.2.0 Database Status Post-Implementation Review |
| [v3.2.1-module-layout-implementation-plan.md](v3.2.1-module-layout-implementation-plan.md) | v3.2.1 Module Layout Implementation Plan（Phase 1 完了 — PR #59） |
| [v3.2.1-module-layout-post-implementation-review.md](v3.2.1-module-layout-post-implementation-review.md) | v3.2.1 Module Layout Post-Implementation Review |
| [v3.3.0-planned-vs-actual-implementation-plan.md](v3.3.0-planned-vs-actual-implementation-plan.md) | v3.3.0 Planned vs Actual Difference Implementation Plan（Phase 1–2 完了 — PR #60） |
| [v3.3.0-planned-vs-actual-post-implementation-review.md](v3.3.0-planned-vs-actual-post-implementation-review.md) | v3.3.0 Planned vs Actual Difference Post-Implementation Review |
| [v3.4.0-itinerary-planned-vs-actual-implementation-plan.md](v3.4.0-itinerary-planned-vs-actual-implementation-plan.md) | v3.4.0 Itinerary-level Planned vs Actual Difference Implementation Plan（Phase 1 完了 — PR #61 / #62） |
| [v3.4.0-itinerary-planned-vs-actual-post-implementation-review.md](v3.4.0-itinerary-planned-vs-actual-post-implementation-review.md) | v3.4.0 Itinerary-level Planned vs Actual Difference Post-Implementation Review |
| [v3.5.0-receipt-inbox-concept-design.md](v3.5.0-receipt-inbox-concept-design.md) | v3.5.0 Receipt Inbox Concept Design（**v3.5.0 リリース済み** — documentation-only） |
| [v3.6.0-receipt-inbox-metadata-only-implementation-plan.md](v3.6.0-receipt-inbox-metadata-only-implementation-plan.md) | v3.6.0 Receipt Inbox Metadata-only Implementation Plan（**v3.6.0 リリース済み**） |
| [v3.6.0-receipt-inbox-metadata-only-post-implementation-review.md](v3.6.0-receipt-inbox-metadata-only-post-implementation-review.md) | v3.6.0 Receipt Inbox Metadata-only Post-Implementation Review（**v3.6.0 リリース済み**） |
| [v3.7.0-receipt-assignment-and-trash-workflow-design.md](v3.7.0-receipt-assignment-and-trash-workflow-design.md) | v3.7.0 Receipt Assignment and Trash Workflow Design（**v3.7.0 リリース済み**） |
| [v3.7.0-receipt-assignment-and-trash-implementation-plan.md](v3.7.0-receipt-assignment-and-trash-implementation-plan.md) | v3.7.0 Receipt Assignment and Trash Implementation Plan（**v3.7.0 リリース済み**） |
| [v3.7.1-receipt-inbox-post-implementation-review.md](v3.7.1-receipt-inbox-post-implementation-review.md) | v3.7.1 Receipt Inbox Post-Implementation Review（Okinawa sample + export fix — **v3.7.1 リリース済み**） |
| [v3.8.0-roadmap-realignment-after-receipt-inbox.md](v3.8.0-roadmap-realignment-after-receipt-inbox.md) | v3.8.0 Roadmap Realignment after Receipt Inbox（**v3.8.0 documentation-only**） |
| [v3.9.0-config-and-db-path-foundation-implementation-plan.md](v3.9.0-config-and-db-path-foundation-implementation-plan.md) | v3.9.0 Config and DB Path Foundation Implementation Plan（**v3.9.0 Phase 1**） |
| [v3.10.0-db-use-concept-design.md](v3.10.0-db-use-concept-design.md) | v3.10.0 DB Use Concept Design（**v3.10.0 documentation-only**） |
| [v3.11.0-db-use-implementation-plan.md](v3.11.0-db-use-implementation-plan.md) | v3.11.0 DB Use Implementation Plan（**v3.11.0** — `db use` / `db use --clear`） |
| [v4.0.0-travel-book-concept-design.md](v4.0.0-travel-book-concept-design.md) | v4.0.0 Travel Book Concept Design（**v4.0.0 documentation-only**） |
| [v4.1.0-travel-book-chapter-structure-design.md](v4.1.0-travel-book-chapter-structure-design.md) | v4.1.0 Travel Book Chapter Structure Design（**v4.1.0 documentation-only**） |
| [v4.1.1-okinawa-travel-book-sample-enrichment-plan.md](v4.1.1-okinawa-travel-book-sample-enrichment-plan.md) | v4.1.1 Okinawa Travel Book Sample Enrichment Plan（**v4.1.1 documentation-only**） |
| [v4.1.2-okinawa-travel-book-sample-enrichment-implementation-plan.md](v4.1.2-okinawa-travel-book-sample-enrichment-implementation-plan.md) | v4.1.2 Okinawa Sample Enrichment Implementation Plan（**v4.1.2**） |
| [v4.2.0-export-md-layout-improvement-implementation-plan.md](v4.2.0-export-md-layout-improvement-implementation-plan.md) | v4.2.0 export-md Layout Improvement Implementation Plan（**v4.2.0**） |
| [v4.2.1-travel-book-export-md-post-release-review.md](v4.2.1-travel-book-export-md-post-release-review.md) | v4.2.1 Travel Book export-md Post-Release Review（**v4.2.1 documentation-only**） |
| [v4.2.2-travel-book-markdown-polish-implementation-plan.md](v4.2.2-travel-book-markdown-polish-implementation-plan.md) | v4.2.2 Travel Book Markdown Polish Implementation Plan（**v4.2.2 リリース済み**） |
| [v4.3.0-reservation-summary-display-refinement-design.md](v4.3.0-reservation-summary-display-refinement-design.md) | v4.3.0 Reservation / Summary Display Refinement Design（**v4.3.0**） |
| [v4.3.0-reservation-summary-display-refinement-implementation-plan.md](v4.3.0-reservation-summary-display-refinement-implementation-plan.md) | v4.3.0 Reservation / Summary Display Refinement Implementation Plan（**v4.3.0 リリース済み**） |
| [v4.3.1-reservation-summary-display-post-release-review.md](v4.3.1-reservation-summary-display-post-release-review.md) | v4.3.1 Reservation / Summary Display Post-Release Review（**v4.3.1 documentation-only**） |
| [v4.3.2-travel-book-planned-cost-polish-implementation-plan.md](v4.3.2-travel-book-planned-cost-polish-implementation-plan.md) | v4.3.2 Travel Book Planned Cost Polish Implementation Plan（**v4.3.2**） |
| [v4.4.0-travel-book-presentation-model-review.md](v4.4.0-travel-book-presentation-model-review.md) | v4.4.0 Travel Book Presentation Model Review（**v4.4.0**） |
| [v4.4.1-category-display-name-in-travel-book-implementation-plan.md](v4.4.1-category-display-name-in-travel-book-implementation-plan.md) | v4.4.1 Category Display Name in Travel Book Implementation Plan（**v4.4.1**） |
| [v4.4.2-travel-book-presentation-helper-review.md](v4.4.2-travel-book-presentation-helper-review.md) | v4.4.2 Travel Book Presentation Helper Review（**v4.4.2**） |
| [v4.4.3-travel-book-presentation-helpers-extraction-plan.md](v4.4.3-travel-book-presentation-helpers-extraction-plan.md) | v4.4.3 Travel Book Presentation Helpers Extraction Plan（**v4.4.3**） |
| [v4.4.4-travel-book-presentation-helpers-extraction-phase-2.md](v4.4.4-travel-book-presentation-helpers-extraction-phase-2.md) | v4.4.4 Travel Book Presentation Helpers Extraction Phase 2（**v4.4.4**） |
| [v4.4.5-travel-book-presentation-extraction-review.md](v4.4.5-travel-book-presentation-extraction-review.md) | v4.4.5 Travel Book Presentation Extraction Review（**v4.4.5**） |
| [v4.4.6-travel-book-presentation-helpers-extraction-phase-3.md](v4.4.6-travel-book-presentation-helpers-extraction-phase-3.md) | v4.4.6 Travel Book Presentation Helpers Extraction Phase 3（**v4.4.6**） |
| [v4.4.7-travel-book-presentation-helpers-final-review.md](v4.4.7-travel-book-presentation-helpers-final-review.md) | v4.4.7 Travel Book Presentation Helpers Final Review（**released**） |
| [v4.4.8-travel-book-presentation-helper-cleanup.md](v4.4.8-travel-book-presentation-helper-cleanup.md) | v4.4.8 Travel Book Presentation Helper Cleanup（**released**） |
| [v4.5.0-receipt-inbox-responsibilities-review.md](v4.5.0-receipt-inbox-responsibilities-review.md) | v4.5.0 Receipt Inbox Responsibilities Review（**released**） |
| [v4.7.5-public-examples-ai-json-generation-guide.md](v4.7.5-public-examples-ai-json-generation-guide.md) | v4.7.5 Public examples / AI JSON generation guide |
| [v4.7.4-materialize-gate-concept-validation-rules.md](v4.7.4-materialize-gate-concept-validation-rules.md) | v4.7.4 Materialize gate concept / validation rules |
| [v4.7.3-proposal-fragment-concept-spec.md](v4.7.3-proposal-fragment-concept-spec.md) | v4.7.3 Proposal Fragment concept specification（**released**） |
| [v4.7.2-trip-proposal-envelope-concept-spec.md](v4.7.2-trip-proposal-envelope-concept-spec.md) | v4.7.2 Trip Proposal Envelope concept specification（**released**） |
| [v4.7.1-public-readme-schema-docs-outline.md](v4.7.1-public-readme-schema-docs-outline.md) | v4.7.1 Public README / schema docs outline（**released**） |
| [v4.7.0-schema-publication-travel-ledger-public-direction-concept-review.md](v4.7.0-schema-publication-travel-ledger-public-direction-concept-review.md) | v4.7.0 Schema-publication / Travel Ledger public direction concept review（**released**） |
| [v4.6.43-release-workflow-asset-upload-follow-up.md](v4.6.43-release-workflow-asset-upload-follow-up.md) | v4.6.43 Release workflow asset upload follow-up（**released**） |
| [v4.6.42-reservation-write-service-phase-r5-adapter-cleanup.md](v4.6.42-reservation-write-service-phase-r5-adapter-cleanup.md) | v4.6.42 Reservation write service Phase R-5 adapter cleanup（**released**） |
| [v4.6.41-reservation-write-service-phase-r2-r3.md](v4.6.41-reservation-write-service-phase-r2-r3.md) | v4.6.41 Reservation write service Phase R-2+R-3（**released**） |
| [v4.6.40-reservation-write-service-migration-plan.md](v4.6.40-reservation-write-service-migration-plan.md) | v4.6.40 Reservation write service migration plan（**released**） |
| [v4.6.39-reservation-write-path-boundary-review.md](v4.6.39-reservation-write-path-boundary-review.md) | v4.6.39 Reservation write path boundary review（**released**） |
| [v4.6.38-note-write-service-phase-n5-closeout.md](v4.6.38-note-write-service-phase-n5-closeout.md) | v4.6.38 Note write service Phase N-5 closeout（**released**） |
| [v4.6.37-note-write-service-phase-n2-n3.md](v4.6.37-note-write-service-phase-n2-n3.md) | v4.6.37 Note write service Phase N-2+N-3（**released**） |
| [v4.6.36-note-write-service-migration-plan.md](v4.6.36-note-write-service-migration-plan.md) | v4.6.36 Note write service migration plan（**released**） |
| [v4.6.35-note-write-path-boundary-review.md](v4.6.35-note-write-path-boundary-review.md) | v4.6.35 Note write path boundary review（**released**） |
| [v4.6.34-expense-write-adapter-cleanup.md](v4.6.34-expense-write-adapter-cleanup.md) | v4.6.34 Expense write adapter cleanup（**released**） |
| [v4.6.33-expense-write-service-phase-w2-w3.md](v4.6.33-expense-write-service-phase-w2-w3.md) | v4.6.33 Expense write service Phase W-2+W-3（**released**） |
| [v4.6.32-expense-write-service-migration-plan.md](v4.6.32-expense-write-service-migration-plan.md) | v4.6.32 Expense write service migration plan（**released**） |
| [v4.6.31-expense-write-path-migration-plan.md](v4.6.31-expense-write-path-migration-plan.md) | v4.6.31 Expense write path migration plan（**released**） |
| [v4.6.30-expense-write-path-boundary-review.md](v4.6.30-expense-write-path-boundary-review.md) | v4.6.30 Expense write path boundary review（**released**） |
| [v4.6.29-itinerary-show-aggregate-migration-plan.md](v4.6.29-itinerary-show-aggregate-migration-plan.md) | v4.6.29 Itinerary show aggregate migration plan（**released**） |
| [v4.6.28-itinerary-show-aggregate-boundary-review.md](v4.6.28-itinerary-show-aggregate-boundary-review.md) | v4.6.28 Itinerary show aggregate boundary review（**released**） |
| [v4.6.27-expense-output-dto-migration-follow-up-review.md](v4.6.27-expense-output-dto-migration-follow-up-review.md) | v4.6.27 Expense output DTO migration follow-up review（**released**） |
| [v4.6.26-expense-output-dto-migration-phase-2-3.md](v4.6.26-expense-output-dto-migration-phase-2-3.md) | v4.6.26 Expense output DTO migration Phase 2+3（**released**） |
| [v4.6.25-expense-output-dto-migration-plan.md](v4.6.25-expense-output-dto-migration-plan.md) | v4.6.25 Expense output DTO migration plan（**released**） |
| [v4.6.24-expense-dto-context-ownership-review.md](v4.6.24-expense-dto-context-ownership-review.md) | v4.6.24 Expense DTO context ownership review（**released**） |
| [v4.6.23-read-only-helper-context-review.md](v4.6.23-read-only-helper-context-review.md) | v4.6.23 Read-only helper context review（**released**） |
| [v4.6.22-read-only-service-boundary-completion-review.md](v4.6.22-read-only-service-boundary-completion-review.md) | v4.6.22 Read-only service boundary completion review（**released**） |
| [v4.6.21-expense-show-service-boundary.md](v4.6.21-expense-show-service-boundary.md) | v4.6.21 Expense show service boundary（**released**） |
| [v4.6.20-reservation-show-service-boundary.md](v4.6.20-reservation-show-service-boundary.md) | v4.6.20 Reservation show service boundary（**released**） |
| [v4.6.19-day-show-service-boundary.md](v4.6.19-day-show-service-boundary.md) | v4.6.19 Day show service boundary（**released**） |
| [v4.6.18-note-show-service-boundary.md](v4.6.18-note-show-service-boundary.md) | v4.6.18 Note show service boundary（**released**） |
| [v4.6.17-checklist-show-service-boundary.md](v4.6.17-checklist-show-service-boundary.md) | v4.6.17 Checklist show service boundary（**released**） |
| [v4.6.16-read-only-service-boundary-follow-up-review.md](v4.6.16-read-only-service-boundary-follow-up-review.md) | v4.6.16 Read-only service boundary follow-up review（**released**） |
| [v4.6.15-checklist-list-service-boundary.md](v4.6.15-checklist-list-service-boundary.md) | v4.6.15 Checklist list service boundary（**released**） |
| [v4.6.14-expense-list-service-boundary.md](v4.6.14-expense-list-service-boundary.md) | v4.6.14 Expense list service boundary（**released**） |
| [v4.6.13-reservation-list-service-boundary.md](v4.6.13-reservation-list-service-boundary.md) | v4.6.13 Reservation list service boundary（**released**） |
| [v4.6.12-note-list-service-boundary.md](v4.6.12-note-list-service-boundary.md) | v4.6.12 Note list service boundary（**released**） |
| [v4.6.11-read-only-service-boundary-review.md](v4.6.11-read-only-service-boundary-review.md) | v4.6.11 Read-only service boundary review（**released**） |
| [v4.6.10-itinerary-show-service-boundary.md](v4.6.10-itinerary-show-service-boundary.md) | v4.6.10 Itinerary show service boundary（**released**） |
| [v4.6.9-itinerary-timeline-service-boundary.md](v4.6.9-itinerary-timeline-service-boundary.md) | v4.6.9 Itinerary timeline service boundary（**released**） |
| [v4.6.8-itinerary-list-service-boundary.md](v4.6.8-itinerary-list-service-boundary.md) | v4.6.8 Itinerary list service boundary（**released**） |
| [v4.6.7-day-list-service-boundary.md](v4.6.7-day-list-service-boundary.md) | v4.6.7 Day list service boundary（**released**） |
| [v4.6.6-trip-show-service-boundary.md](v4.6.6-trip-show-service-boundary.md) | v4.6.6 Trip show service boundary（**released**） |
| [v4.6.5-read-only-service-boundary-expansion.md](v4.6.5-read-only-service-boundary-expansion.md) | v4.6.5 Read-only service boundary expansion（**released**） |
| [v4.6.4-read-only-service-boundary-pilot.md](v4.6.4-read-only-service-boundary-pilot.md) | v4.6.4 Read-only service boundary pilot（**released**） |
| [v4.6.3-command-handler-split-phase-1.md](v4.6.3-command-handler-split-phase-1.md) | v4.6.3 Command handler split Phase 1（**released**） |
| [v4.6.2-sqlite-migration-strategy-review.md](v4.6.2-sqlite-migration-strategy-review.md) | v4.6.2 SQLite migration strategy review（**released**） |
| [v4.6.1-sqlite-fk-orphan-data-hardening-review.md](v4.6.1-sqlite-fk-orphan-data-hardening-review.md) | v4.6.1 SQLite FK / orphan data hardening review（**released**） |
| [v4.6.0-trip-stats-days-semantics-fix.md](v4.6.0-trip-stats-days-semantics-fix.md) | v4.6.0 TripStats.days semantics fix（**released**） |
| [v4.5.1-doctor-advisor-receipt-utilization-implementation-plan.md](v4.5.1-doctor-advisor-receipt-utilization-implementation-plan.md) | v4.5.1 doctor / advisor Receipt Utilization Implementation Plan（**released**） |
