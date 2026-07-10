# Travel Ledger — Public documentation

Travel Ledger を **外部に説明するための入口** です。実装の詳細は [specifications/](../specifications/) と [command-reference.md](../command-reference.md) を参照してください。

| Document | 内容 |
|---|---|
| [travel-ledger.md](travel-ledger.md) | Travel Ledger とは何か / 解決する問題 |
| [schema.md](schema.md) | schema v8（canonical）と schema v3+（歴史） |
| [proposals.md](proposals.md) | Envelope / Fragment / adoption gate 概要 |
| [examples.md](examples.md) | 最小例・gate 前後・validate-export の読み方 |
| [examples/](examples/) | **schema v8 Trip JSON files**（validate-export 対象） |
| [examples-non-normative/](examples-non-normative/) | Proposal / Fragment 概念例（non-normative） |
| [ai-json-generation-guide.md](ai-json-generation-guide.md) | 生成 AI 向け JSON 作法・プロンプト例 |

関連:

- [v4.7.0 concept review](../specifications/v4.7.0-schema-publication-travel-ledger-public-direction-concept-review.md) — 新章の方向性
- [v4.7.8 Proposal implementation planning spec](../specifications/v4.7.8-proposal-implementation-planning.md) — 実装フェーズ・command 候補
- [v4.7.7 Public schema post-review spec](../specifications/v4.7.7-public-schema-post-review.md) — export-schema v8 整合
- [v4.7.6 Public JSON examples spec](../specifications/v4.7.6-public-json-examples-concept-stream-post-review.md) — public JSON files + post-review
- [v4.7.5 Public examples / AI guide spec](../specifications/v4.7.5-public-examples-ai-json-generation-guide.md) — 公開例・AI 生成作法正本
- [v4.7.4 Materialize gate spec](../specifications/v4.7.4-materialize-gate-concept-validation-rules.md) — 採用 gate / validation 正本
- [v4.7.3 Proposal Fragment spec](../specifications/v4.7.3-proposal-fragment-concept-spec.md) — Fragment 概念正本
- [v4.7.2 Trip Proposal Envelope spec](../specifications/v4.7.2-trip-proposal-envelope-concept-spec.md) — Envelope 概念正本
- [future-roadmap-planning-memo.md](../future-roadmap-planning-memo.md) — 将来機能のブレインストーミング（非確約）
- [export-schema.md](../specifications/export-schema.md) — export スキーマ正本（実装者向け）
- [venue-model-introduction-policy.md](../specifications/venue-model-introduction-policy.md) — Venue 導入方針（planning、schema v8 変更前）

---

## Travel Ledger とは

**旅行者が手元に持てる、構造化された正式な旅行データ形式** です。

予約サイト・地図・メモ・メール・チャットに分散しがちな旅行計画を、採用済みの Trip / Day / Itinerary とその配下データとして保持します。

```text
Your travel data should belong to you.
旅行データを、サービスの中ではなく、あなたの手元に。
AIが提案し、人が選び、旅の記録として残る。
```

詳細: [travel-ledger.md](travel-ledger.md)

---

## 構成要素の位置づけ

```text
Travel Ledger schema
  └─ 公式 Trip データ契約（現時点: export schema v8）

travel-ledger-cli（本リポジトリ）
  ├─ reference implementation
  ├─ validator      trip validate-export
  ├─ converter      trip import / export / diff
  └─ doctor         trip doctor / trip advisor

future GUI（未実装）
  └─ schema consumer — service 境界経由で mutation

AI / provider（将来）
  └─ Proposal → 人間が選ぶ → materialize → 正式 Trip
```

| 役割 | 説明 |
|---|---|
| **Travel Ledger** | データ形式の正典 |
| **CLI** | 正本の参照実装。予約サイト・SNS ではない |
| **GUI** | 将来の consumer（v4.6.x で整えた service 層を再利用予定） |
| **Proposal** | schema v8 の外側 — [Envelope / Fragment / gate](proposals.md)（v4.7.2〜v4.7.4） |

---

## Schema の読み方

| 用語 | 意味 |
|---|---|
| **schema v8** | **現行 canonical** — `trip export` が出力する Trip JSON |
| **schema v3+** | nested Trip / Day / Itinerary モデルが確立した **歴史的世代**（現行の正本ではない） |
| **Travel Ledger schema** | 公開 Trip データ契約。現時点では **schema v8** を指す |

詳細: [schema.md](schema.md)

**CLI バージョン（Cargo `4.x`）と `schema_version`（整数）は独立** です。CLI v4.7.x が schema v8 を出力するのは正常です。

---

## 読む順序（推奨）

```text
1. 本 README                    — 全体像
2. travel-ledger.md             — 何を解決するか
3. schema.md                    — v8 / v3+ の用語
4. proposals.md                 — Envelope / Fragment / gate 概要
5. examples/                  — schema v8 JSON files（見れば分かる）
6. examples.md                — narrative・validate-export 読み方
7. examples-non-normative/      — Proposal / Fragment 概念例
8. ai-json-generation-guide.md  — 生成 AI 向け作法（provider 向け）
9. export-schema.md             — フィールドレベル（実装者向け）
```

---

## v4.7.x ドキュメント計画

| Version | 内容 |
|---|---|
| v4.7.0 | public direction concept review — **完了** |
| v4.7.1 | public docs outline — **完了** |
| v4.7.2 | Trip Proposal Envelope concept spec — **完了** |
| v4.7.3 | Proposal Fragment concept spec — **完了** |
| v4.7.4 | materialize gate / validation rules — **完了** |
| v4.7.5 | public examples / AI JSON generation guide — **完了** |
| v4.7.6 | public JSON examples / concept stream post-review — **完了** |
| v4.7.7 | public schema post-review — **完了** |
| v4.7.8 | Proposal implementation planning — **完了** |
| v4.7.9 | Proposal Envelope file validation (P-1) — **完了** |
| **v4.7.10** | **Proposal Envelope show / inspect (P-2)** |
| **v4.7.11** | **Proposal Fragment file validation (P-3)** |
| **v4.7.12** | **Public examples validation guard** |
| **v4.7.13** | **Proposal storage strategy planning (P-4)** |
| **v4.7.14** | **Public examples guard CI isolation hotfix** |
| **v4.7.15** | **Materialize / apply planning (P-5)** |
| **v4.7.16** | **Proposal materialize dry-run (P-6a)** |
| **v4.7.17** | **Proposal materialize --confirm (P-6b)** |
| **v4.7.18** | **Fragment apply dry-run (P-6c)** |
| **v4.7.19** | **Fragment apply --confirm (P-6d)** |
| **v4.7.20** | **P-6 post-implementation review** |
| **v4.7.21** | **Fragment apply add_itinerary field expansion (P-6e)** |
| **v4.7.22** | **Fragment apply add_note dry-run (P-6f)** |
| **v4.7.23** | **Fragment apply add_note --confirm (P-6f)** |
| **v4.7.24** | **Fragment apply add_expense dry-run (P-6g)** |
| **v4.7.25** | **Fragment apply add_expense --confirm (P-6g)** |
| **v4.7.26** | **Fragment apply add_reservation dry-run (P-6h)** |
| **v4.7.31** | **Fragment apply delete_itinerary dry-run (P-6j)** |
| **v4.7.32** | **Fragment apply delete_itinerary --confirm (P-6j)** |
| **v4.7.36** | **P-6k reorder_itinerary --confirm（same-day）** |
| **v4.7.35** | **P-6k reorder_itinerary dry-run（same-day）** |
| **v4.7.41** | **P-6n add_estimate Proposal Fragment planning（documentation-only）** |
| **v4.7.40** | **P-6m reorder / move post-release review（documentation-only）** |
| **v4.7.39** | **P-6l move_itinerary --confirm（cross-day）** |
| **v4.7.38** | **P-6l move_itinerary dry-run（cross-day）** |
| **v4.7.37** | **P-6l cross-day move planning（documentation-only）** |
| **v4.7.34** | **P-6k reorder_itinerary planning（documentation-only）** |
| **v4.7.33** | **P-6j safety / UX hardening for delete_itinerary** |
| **v4.7.30** | **P-6j destructive / structural apply policy** |
| **v4.7.29** | **Fragment apply update_itinerary --confirm (P-6i)** |
| **v4.7.28** | **Fragment apply update_itinerary dry-run (P-6i)** |
| **v4.7.27** | **Fragment apply add_reservation --confirm (P-6h)** |

実装・schema 変更は v4.7.8 の scope 外です（planning のみ）。
