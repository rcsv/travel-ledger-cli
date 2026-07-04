# Materialize / apply — before and after

Trip Proposal Envelope と Proposal Fragment が **adoption gate** を通ったあと、schema v8 Trip とどう関係するかを説明します。

概念正本: [v4.7.4 Materialize gate spec](../../specifications/v4.7.4-materialize-gate-concept-validation-rules.md)

---

## Before gate

```text
Trip Proposal Envelope     Proposal Fragment
        │                          │
        │   schema v8 の外側        │
        │   validate-export 対象外  │
        └──────────┬───────────────┘
                   │
            [ human review ]
            required decisions
            blocking validation
            non-blocking warnings
```

| 入力 | 状態 | validate-export |
|---|---|---|
| Trip Proposal Envelope | 旅行全体の未採用案 | 対象外 |
| Proposal Fragment | 既存 Trip への部分提案 | 対象外 |

---

## Gate 操作（概念）

| 入力 | Gate 操作 | 出力 |
|---|---|---|
| Trip Proposal Envelope | **materialize** | **新規** schema v8 Trip |
| Proposal Fragment | **apply** | **既存** schema v8 Trip を更新 |

人間が採用判断するまで、AI 出力や provider draft は正式 Trip にならない。

---

## After gate

```text
schema v8 Trip (new or updated)
  ├─ trip import / ledger
  ├─ trip export
  └─ trip validate-export  ✓
```

gate 通過後に生成・更新された JSON は [examples/](../examples/) と同じ **schema v8 Trip** 契約に従う。

例:

- Envelope `materialize` → [schema-v8-minimal-trip.json](../examples/schema-v8-minimal-trip.json) のような採用済み export
- Fragment `apply` → 既存 Trip（例: [schema-v8-okinawa-sesoko-trip.json](../examples/schema-v8-okinawa-sesoko-trip.json)）への差分反映後、更新された schema v8 export

---

## 混在を防ぐ

```text
✗ Envelope / Fragment を schema v8 Trip にそのまま import しない
✗ 候補案に validate-export を通して「正式」とみなさない
✓ gate 後の schema v8 Trip のみ validate-export
```

---

## Example files

| 段階 | 参照 |
|---|---|
| 候補（全体） | [trip-proposal-envelope.example.json](trip-proposal-envelope.example.json) |
| 候補（部分） | [proposal-fragment.example.json](proposal-fragment.example.json) |
| 採用済み Trip | [examples/](../examples/) |

---

## 関連

- [examples-non-normative/README.md](README.md)
- [examples.md](../examples.md)
- [proposals.md](../proposals.md)
- [ai-json-generation-guide.md](../ai-json-generation-guide.md)
