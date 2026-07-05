# Public examples

Travel Ledger の **最小例と読み方** です。フィールド定義の正本は [export-schema.md](../specifications/export-schema.md)。Proposal 概念は [proposals.md](proposals.md)、AI 向け作法は [ai-json-generation-guide.md](ai-json-generation-guide.md) を参照してください。

## Public JSON files（v4.7.6+）

**見れば分かる** schema v8 Trip の実ファイル:

| ファイル | 内容 |
|---|---|
| [examples/schema-v8-minimal-trip.json](examples/schema-v8-minimal-trip.json) | 最小構成 |
| [examples/schema-v8-okinawa-sesoko-trip.json](examples/schema-v8-okinawa-sesoko-trip.json) | 沖縄瀬底（短縮） |
| [examples/schema-v8-with-reservations-expenses-notes.json](examples/schema-v8-with-reservations-expenses-notes.json) | reservations / expenses / notes |

詳細: [examples/README.md](examples/README.md)

**Non-normative**（Proposal / Fragment — validate-export 対象外）: [examples-non-normative/](examples-non-normative/)

---

## 三種類の JSON

| 種類 | 状態 | validate-export |
|---|---|---|
| **schema v8 Trip** | 採用済み正式データ | **対象** |
| **Trip Proposal Envelope** | 旅行全体の未採用案 | 対象外 |
| **Proposal Fragment** | 既存 Trip への部分提案 | 対象外 |

採用前の Proposal / Fragment を schema v8 Trip に混ぜない。

---

## schema v8 Trip — 最小例

**条件:** 実日付確定、採用済み、export 契約に準拠。

**Primary:** [examples/schema-v8-minimal-trip.json](examples/schema-v8-minimal-trip.json) — `trip validate-export` 通過済み。

以下は narrative 用の短縮表記（`trip.id` 等は file 側を参照）:

```json
{
  "schema_version": 8,
  "generator": "travel-ledger-cli",
  "generator_version": "4.7.x",
  "exported_at": "2026-04-01T12:00:00Z",
  "trip": {
    "name": "Okinawa 3-day family trip",
    "start_date": "2026-04-26",
    "end_date": "2026-04-28",
    "summary": "Adopted plan: north Okinawa, rental car."
  },
  "days": [
    {
      "day_number": 1,
      "summary": "Arrival and check-in",
      "itineraries": [
        {
          "title": "Flight to Naha",
          "start_time": "10:00",
          "sort_order": 1,
          "category": "flight"
        }
      ]
    }
  ],
  "notes": [],
  "participants": [],
  "checklist_items": []
}
```

**読み方:**

- `schema_version: 8` — 現行 canonical export
- `start_date` / `end_date` — **必須**（採用済み Trip）
- `days[].itineraries[]` — nested モデルの中核

**正本サンプル:** [samples/okinawa_sesoko_2026/](../../samples/okinawa_sesoko_2026/) — 実データに近い narrative sample。`trip export` 出力と golden 比較用。

**検証:**

```bash
trip validate-export path/to/export.json
```

`trip validate-export` は **schema v8 Trip JSON のみ** が対象。Proposal / Fragment は通さない。

---

## Trip Proposal Envelope — 概念例

**条件:** 旅行全体の未採用案。日付未定可。**schema v8 ではない。**

**Primary:** [examples-non-normative/trip-proposal-envelope.example.json](examples-non-normative/trip-proposal-envelope.example.json)

以下は narrative 用の短縮表記:

```json
{
  "_concept": "Trip Proposal Envelope — NOT schema v8, NOT for validate-export",
  "metadata": {
    "proposal_id": "prop-2026-okinawa-draft-01",
    "created_at": "2026-03-01T09:00:00Z",
    "source": "ai",
    "provider": "example-model"
  },
  "proposal": {
    "title": "Okinawa family trip (draft)",
    "destination": "Okinawa, Japan",
    "date_policy": "flexible_dates",
    "candidate_days": [
      { "label": "Day 1", "summary": "Arrival — Naha or north", "date": null }
    ],
    "notes": "Dates not confirmed. Compare with hotel availability."
  },
  "materialize_hints": {
    "required_decisions": [
      "Confirm start_date and end_date",
      "Choose north vs south base"
    ],
    "missing_fields": ["confirmed flight times", "hotel booking"],
    "warnings": []
  }
}
```

**ポイント:**

- `date_policy: flexible_dates` — 日付候補のみ、未採用
- `missing_fields` — 不明は **空にせず列挙**
- `_concept` — 説明用（将来 schema 確定時は除去）

詳細: [v4.7.2 spec](../specifications/v4.7.2-trip-proposal-envelope-concept-spec.md)

---

## Proposal Fragment — 概念例

**条件:** 既存 Trip への部分提案。**小さい Trip ではない。**

**Primary:** [examples-non-normative/proposal-fragment.example.json](examples-non-normative/proposal-fragment.example.json)

以下は narrative 用の短縮表記:

```json
{
  "_concept": "Proposal Fragment — NOT schema v8, NOT for validate-export",
  "metadata": {
    "fragment_id": "frag-2026-okinawa-day2-lunch",
    "created_at": "2026-03-15T14:00:00Z",
    "source": "ai",
    "provider": "example-model"
  },
  "target": {
    "target_type": "day",
    "trip_reference": "Okinawa 3-day family trip",
    "day_reference": 2,
    "unresolved_target_hints": null
  },
  "fragment": {
    "intent": "add",
    "candidate_content": {
      "title": "Lunch at local soba shop (candidate)",
      "category": "meal",
      "placement_hint": "after aquarium visit"
    },
    "notes": "Opening hours not verified."
  },
  "adoption_hints": {
    "required_decisions": ["Confirm day 2 schedule slot"],
    "conflicts": [],
    "warnings": ["stale_source: hours may be outdated"]
  }
}
```

**unresolved target の例:**

```json
"target": {
  "target_type": "unresolved",
  "unresolved_target_hints": "Suggested restaurant — assign to a day during review"
}
```

詳細: [v4.7.3 spec](../specifications/v4.7.3-proposal-fragment-concept-spec.md)

---

## Materialize gate — 前後の流れ

**Primary:** [examples-non-normative/materialize-before-after.md](examples-non-normative/materialize-before-after.md)

```text
[Before gate]
  Trip Proposal Envelope  ─┐
  Proposal Fragment       ─┤  schema v8 の外側
                           │  trip export に含まれない
                           │  validate-export 対象外

[Adoption gate]
  human review
  required decisions
  blocking validation / warnings

[After gate]
  schema v8 Trip (new or updated)
  → trip import / ledger
  → trip export
  → trip validate-export  ✓
```

| 入力 | Gate 操作（概念） | 出力 |
|---|---|---|
| Trip Proposal Envelope | materialize | **新規** schema v8 Trip |
| Proposal Fragment | apply | **既存** Trip 更新 |

詳細: [v4.7.4 spec](../specifications/v4.7.4-materialize-gate-concept-validation-rules.md)

---

## validate-export の読み方

```text
schema v8 Trip JSON     →  trip validate-export で conformance 確認
Trip Proposal Envelope  →  対象外（候補案）
Proposal Fragment       →  対象外（候補案）
materialize / apply 後  →  生成された schema v8 Trip が対象
```

第三者実装・CI では `trip validate-export` を **正式 Trip 契約の gate** として扱える。

Proposal を validate-export に通そうとしない — 型が違う。

---

## 関連

- [examples/](examples/) — schema v8 Trip JSON files
- [examples-non-normative/](examples-non-normative/) — Proposal / Fragment 概念例
- [AI JSON generation guide](ai-json-generation-guide.md)
- [Proposals outline](proposals.md)
- [Schema overview](schema.md)
- [v4.7.8 spec](../specifications/v4.7.8-proposal-implementation-planning.md)
- [v4.7.7 spec](../specifications/v4.7.7-public-schema-post-review.md)
