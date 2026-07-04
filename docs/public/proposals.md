# Proposals — outline

AI、旅行業者、ブログ、手入力などから来る **まだ採用していない旅行案** の扱いです。

- [v4.7.2](../specifications/v4.7.2-trip-proposal-envelope-concept-spec.md) — **Trip Proposal Envelope**（旅行全体の未採用案）
- [v4.7.3](../specifications/v4.7.3-proposal-fragment-concept-spec.md) — **Proposal Fragment**（既存 Trip への部分提案）
- [v4.7.4](../specifications/v4.7.4-materialize-gate-concept-validation-rules.md) — **Adoption gate**（採用・validation）

---

## Core rule

```text
schema v8 Trip = 採用済みの正式データ（実日付を持つ）
Proposal       = 候補案 — schema v8 の外側
Materialize    = 人間が採用したときだけ正式 Trip へ変換・反映する gate
```

採用前の提案を schema v8 Trip に無理に入れない。AI 提案の自動取り込みはしない。

---

## Two proposal types

| Type | Scope | 採用後 |
|---|---|---|
| **Trip Proposal Envelope** | 旅行全体の未採用案 | **新規** schema v8 Trip |
| **Proposal Fragment** | 既存 Trip への部分提案 | **既存** Trip へ反映 |

Fragment は「小さい Trip」ではない。

---

## Adoption gate（v4.7.4）

**Adoption gate** は、Proposal / Fragment を正式データにしてよいか人間が判断する **門番** です。

```text
Input   → Trip Proposal Envelope または Proposal Fragment
Gate    → human review / required decisions / validation / warnings
Output  → new Trip | updated Trip | reject | defer
```

| ルート | 操作（概念） | 結果 |
|---|---|---|
| Trip Proposal Envelope | **materialize** | 新しい schema v8 Trip |
| Proposal Fragment | **apply**（候補名） | 既存 schema v8 Trip を更新 |

### Validation（概要）

```text
blocking:     実日付未確定、title 欠如、target Trip 不在 など → 採用を止める
non-blocking: 期限切れ、古い情報、time_overlap の可能性 など → 人間が判断
```

期限切れ・古さは **warning のみ**。自動破棄や import 禁止にはしない。

詳細: [v4.7.4 spec](../specifications/v4.7.4-materialize-gate-concept-validation-rules.md)

---

## Trip Proposal Envelope

```text
Trip Proposal Envelope
  ├─ metadata
  ├─ proposal        — title, destination, date_policy, 候補行程
  └─ materialize hints
```

Gate で確定すること（例）: `start_date` / `end_date`、title、Day 構成、Itinerary 採否。

詳細: [v4.7.2 spec](../specifications/v4.7.2-trip-proposal-envelope-concept-spec.md)

---

## Proposal Fragment

```text
Proposal Fragment
  ├─ metadata
  ├─ target          — trip / day / itinerary / unresolved
  ├─ fragment        — intent, candidate content
  └─ adoption hints  — conflicts, warnings, required decisions
```

Gate で確定すること（例）: target Trip / Day / Itinerary、intent、conflict 対応。

詳細: [v4.7.3 spec](../specifications/v4.7.3-proposal-fragment-concept-spec.md)

---

## Flow

```text
Whole-trip:
  Envelope → review → materialize (gate) → new schema v8 Trip

Partial:
  Fragment → review target/conflicts → apply (gate) → updated schema v8 Trip
```

---

## Expiry

```text
valid_until / created_at + 1 year → soft warning at gate
expired → warning only — not auto-delete or import block
```

---

## Specification roadmap

```text
v4.7.2  Trip Proposal Envelope — 完了
v4.7.3  Proposal Fragment — 完了
v4.7.4  materialize gate / validation rules — 完了
v4.7.5  public examples / AI JSON generation guide — 完了
```

Authoring 例と生成 AI 向け作法: [examples.md](examples.md) · [ai-json-generation-guide.md](ai-json-generation-guide.md)

---

## Out of scope (still)

```text
materialize / apply commands
proposal / fragment import commands
JSON schemas
GUI for proposal review
```

---

## Related

- [Examples](examples.md) — 最小例・validate-export
- [AI JSON generation guide](ai-json-generation-guide.md) — 生成 AI 向け作法
- [Public README](README.md)
- [Travel Ledger](travel-ledger.md)
- [Schema overview](schema.md)
- [v4.7.4 Materialize gate spec](../specifications/v4.7.4-materialize-gate-concept-validation-rules.md)
- [v4.7.3 Proposal Fragment spec](../specifications/v4.7.3-proposal-fragment-concept-spec.md)
- [v4.7.2 Trip Proposal Envelope spec](../specifications/v4.7.2-trip-proposal-envelope-concept-spec.md)
