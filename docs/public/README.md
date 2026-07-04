# Travel Ledger — Public documentation

Travel Ledger を **外部に説明するための入口** です。実装の詳細は [specifications/](../specifications/) と [command-reference.md](../command-reference.md) を参照してください。

| Document | 内容 |
|---|---|
| [travel-ledger.md](travel-ledger.md) | Travel Ledger とは何か / 解決する問題 |
| [schema.md](schema.md) | schema v8（canonical）と schema v3+（歴史） |
| [proposals.md](proposals.md) | Trip Proposal Envelope / materialize 概要（v4.7.2 concept spec 参照） |

関連:

- [v4.7.0 concept review](../specifications/v4.7.0-schema-publication-travel-ledger-public-direction-concept-review.md) — 新章の方向性
- [v4.7.2 Trip Proposal Envelope spec](../specifications/v4.7.2-trip-proposal-envelope-concept-spec.md) — Envelope 概念正本
- [future-roadmap-planning-memo.md](../future-roadmap-planning-memo.md) — 将来機能のブレインストーミング（非確約）
- [export-schema.md](../specifications/export-schema.md) — export スキーマ正本（実装者向け）

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
| **Proposal** | schema v8 の外側の候補案 — [Trip Proposal Envelope](proposals.md)（v4.7.2） |

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
1. 本 README          — 全体像
2. travel-ledger.md   — 何を解決するか
3. schema.md          — v8 / v3+ の用語
4. proposals.md       — Trip Proposal Envelope / materialize 概要
5. export-schema.md   — フィールドレベル（実装者向け）
```

---

## v4.7.x ドキュメント計画

| Version | 内容 |
|---|---|
| v4.7.0 | public direction concept review — **完了** |
| v4.7.1 | public docs outline — **完了** |
| **v4.7.2** | **Trip Proposal Envelope concept spec** |
| v4.7.3 | Proposal Fragment concept spec |
| v4.7.4 | materialize gate concept |

実装・schema 変更は v4.7.2 の scope 外です。
