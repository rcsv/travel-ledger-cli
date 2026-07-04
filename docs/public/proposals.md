# Proposals — outline

AI、旅行業者、ブログ、手入力などから来る **まだ採用していない旅行案** の扱いについて、public docs から参照できる **概要のみ** を記載します。

**詳細仕様は v4.7.2 以降** で扱います。本ページは v4.7.1 の入口です。

---

## Core rule

```text
schema v8 Trip = 採用済みの正式データ（実日付を持つ）
Proposal       = 候補案（日付未定可）— Trip の外側
Materialize    = 人間が採用したときだけ正式 Trip へ変換する gate
```

日付未定の提案を schema v8 Trip に無理に入れない。

---

## Concepts (names only)

| Concept | One-line description |
|---|---|
| **Trip Proposal** | Whole-trip suggestion before adoption |
| **Proposal Fragment** | Partial suggestion to merge into an existing Trip / Day |
| **Materialize** | Adoption gate: Proposal → schema v8 Trip with confirmed dates |

Planned specification releases:

```text
v4.7.2  Trip Proposal Envelope concept spec
v4.7.3  Proposal Fragment concept spec
v4.7.4  materialize gate concept / validation rules
```

---

## Flow (target)

```text
AI / provider / manual draft
  → Proposal Envelope (outside schema v8)
  → human review / compare
  → materialize (explicit adoption)
  → schema v8 Trip in local ledger
  → export / Travel Book / future Calendar / GUI
```

---

## Expiry (direction from v4.7.0)

Not implemented in v4.7.1. Documented direction for future specs:

```text
valid_until present     → use as soft expiry
valid_until absent      → default treat as created_at + 1 year (warning)
no_expiration: true     → no default expiry
expired                 → warning, not hard import block
```

---

## What is out of scope in v4.7.1

```text
Proposal Envelope JSON schema
proposal import / list / show commands
materialize command
GUI for proposal review
```

---

## Broader roadmap

Longer-term ideas (Calendar ICS, Travel Pack, Evidence, Route Segment, etc.) live in [future-roadmap-planning-memo.md](../future-roadmap-planning-memo.md). That memo is **planning only**, not a commitment.

Public docs and the memo cross-reference each other; v4.7.x **schema-publication** is the active documentation track.

---

## Related

- [Public README](README.md)
- [Travel Ledger](travel-ledger.md)
- [Schema overview](schema.md)
- [v4.7.0 concept review](../specifications/v4.7.0-schema-publication-travel-ledger-public-direction-concept-review.md)
