# Proposals — outline

AI、旅行業者、ブログ、手入力などから来る **まだ採用していない旅行案** の扱いです。

- [v4.7.2](../specifications/v4.7.2-trip-proposal-envelope-concept-spec.md) — **Trip Proposal Envelope**（旅行全体の未採用案）
- [v4.7.3](../specifications/v4.7.3-proposal-fragment-concept-spec.md) — **Proposal Fragment**（既存 Trip への部分提案）

---

## Core rule

```text
schema v8 Trip = 採用済みの正式データ（実日付を持つ）
Proposal       = 候補案 — Trip の外側
Materialize    = 人間が採用したときだけ正式 Trip へ変換・反映する gate
```

採用前の提案を schema v8 Trip に無理に入れない。

---

## Two proposal types

| Type | Scope | 既存 Trip |
|---|---|---|
| **Trip Proposal Envelope** | 旅行全体の未採用案 | なくても成立 |
| **Proposal Fragment** | 既存 Trip / Day / Itinerary への部分提案 | **関係が本質** |

Fragment は「小さい Trip」ではない。既存計画に差し込む **候補パーツ** です。

---

## Trip Proposal Envelope

正式 Trip に変換される前の **旅行全体の未採用案**（schema v8 の外側）。

```text
Trip Proposal Envelope
  ├─ metadata
  ├─ proposal        — title, destination, date_policy, 候補行程
  └─ materialize hints
```

詳細: [v4.7.2 spec](../specifications/v4.7.2-trip-proposal-envelope-concept-spec.md)

---

## Proposal Fragment

**既存 Trip** に対する部分提案（schema v8 の外側、採用前は正式データではない）。

```text
Proposal Fragment
  ├─ metadata
  ├─ target          — trip / day / itinerary / unresolved
  ├─ fragment        — intent, candidate content, placement hints
  └─ adoption hints  — conflicts, warnings, required decisions
```

### target（差し込み先）

```text
trip-level       — Trip 全体への提案
day-level        — 特定 Day への提案
itinerary-level  — 特定 Itinerary への提案
unresolved       — どこに入れるか未確定（許容）
```

### intent（概念）

```text
add              — 新しい候補を追加
enrich           — 既存要素に情報を補足
replace_candidate — 代替候補を提示
reorder_hint     — 並び・時間帯の見直し
warning          — 既存計画への注意喚起
```

採用されると **既存** schema v8 Trip の正式要素へ反映（gate は v4.7.4）。

詳細: [v4.7.3 spec](../specifications/v4.7.3-proposal-fragment-concept-spec.md)

---

## Flow (target)

```text
Whole-trip idea:
  AI / provider → Trip Proposal Envelope → review → materialize → new schema v8 Trip

Partial idea:
  AI / provider → Proposal Fragment → review target/conflicts → adopt → merge into existing Trip
```

---

## Expiry

Trip Proposal も Fragment も同一方針:

```text
valid_until present     → soft expiry
valid_until absent      → default created_at + 1 year (warning)
no_expiration: true     → no default expiry
expired                 → warning only — no auto-delete or import block
```

---

## Specification roadmap

```text
v4.7.2  Trip Proposal Envelope — 完了
v4.7.3  Proposal Fragment — 完了
v4.7.4  materialize gate / validation rules
```

---

## Out of scope (still)

```text
JSON schemas for Envelope / Fragment
proposal / fragment import commands
materialize / apply commands
GUI for proposal review
```

---

## Related

- [Public README](README.md)
- [Travel Ledger](travel-ledger.md)
- [Schema overview](schema.md)
- [v4.7.3 Proposal Fragment spec](../specifications/v4.7.3-proposal-fragment-concept-spec.md)
- [v4.7.2 Trip Proposal Envelope spec](../specifications/v4.7.2-trip-proposal-envelope-concept-spec.md)
