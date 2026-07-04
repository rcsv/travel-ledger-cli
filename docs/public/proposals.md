# Proposals — outline

AI、旅行業者、ブログ、手入力などから来る **まだ採用していない旅行案** の扱いです。

v4.7.2 で **Trip Proposal Envelope** の概念を整理しました。詳細は [v4.7.2 concept spec](../specifications/v4.7.2-trip-proposal-envelope-concept-spec.md) を参照してください。

---

## Core rule

```text
schema v8 Trip = 採用済みの正式データ（実日付を持つ）
Proposal       = 候補案（日付未定可）— Trip の外側
Materialize    = 人間が採用したときだけ正式 Trip へ変換する gate
```

日付未定の提案を schema v8 Trip に無理に入れない。

---

## Trip Proposal Envelope

**Trip Proposal Envelope** は、正式 Trip に変換される前の **未採用案を包む概念コンテナ** です（schema v8 の外側）。

```text
Trip Proposal Envelope
  ├─ metadata        — いつ・誰が・いつまで有効か
  ├─ proposal        — タイトル・目的地・日程方針・候補行程
  └─ materialize hints — 採用前に人が決めること・警告
```

| 観点 | Trip Proposal Envelope | schema v8 Trip |
|---|---|---|
| 状態 | 未採用 | 採用済み |
| 日付 | 未定 / 候補可 | 実日付必須 |
| trip export | 対象外 | 正本 |

**なぜ Trip ではないか:** 採用前の案は古くなりうるし、複数案を並べて比較する必要がある。正式 Trip と混ぜると export / validate-export の意味が曖昧になる。

---

## date_policy（日付方針）

Proposal は日付未定を許容します。

```text
fixed_dates     — 日付は書いてあるが、まだ未採用
flexible_dates  — 候補はあるが確定していない
undated         — 日付未定の旅行案
```

materialize して schema v8 Trip にする時点では、人間が **実日付を確定** する前提です（詳細は v4.7.4）。

---

## Concepts

| Concept | One-line description |
|---|---|
| **Trip Proposal** | Whole-trip suggestion before adoption |
| **Trip Proposal Envelope** | Container for one unadopted whole-trip proposal |
| **Proposal Fragment** | Partial suggestion for an existing Trip / Day (v4.7.3) |
| **Materialize** | Adoption gate: Proposal → schema v8 Trip with confirmed dates (v4.7.4) |

---

## Flow (target)

```text
AI / provider / manual draft
  → Trip Proposal Envelope (outside schema v8)
  → human review / compare
  → materialize (explicit adoption)
  → schema v8 Trip in local ledger
  → export / Travel Book / future Calendar / GUI
```

---

## Expiry

```text
valid_until present     → use as soft expiry
valid_until absent      → default treat as created_at + 1 year (warning)
no_expiration: true     → no default expiry
expired                 → warning, not hard import block
```

期限切れは「古い可能性があります」という warning のみ。Proposal を自動破棄したり import 禁止にはしません。

---

## Specification roadmap

```text
v4.7.2  Trip Proposal Envelope concept spec — 完了
v4.7.3  Proposal Fragment concept spec
v4.7.4  materialize gate concept / validation rules
```

---

## What is out of scope (still)

```text
Proposal Envelope JSON schema
proposal import / list / show commands
materialize command
GUI for proposal review
```

---

## Broader roadmap

Longer-term ideas live in [future-roadmap-planning-memo.md](../future-roadmap-planning-memo.md). That memo is **planning only**, not a commitment.

---

## Related

- [Public README](README.md)
- [Travel Ledger](travel-ledger.md)
- [Schema overview](schema.md)
- [v4.7.2 Trip Proposal Envelope spec](../specifications/v4.7.2-trip-proposal-envelope-concept-spec.md)
- [v4.7.0 concept review](../specifications/v4.7.0-schema-publication-travel-ledger-public-direction-concept-review.md)
