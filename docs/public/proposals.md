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
v4.7.6  public JSON examples / concept stream post-review — 完了
v4.7.7  public schema post-review — 完了
v4.7.8  Proposal implementation planning — 完了
v4.7.9  Proposal Envelope file validation — 完了
v4.7.10 Proposal Envelope show / inspect — 完了
v4.7.11 Proposal Fragment file validation — 完了
v4.7.12 Public examples validation guard — 完了
v4.7.13 Proposal storage strategy planning — 完了
v4.7.14 Public examples guard CI isolation hotfix — 完了
v4.7.15 Materialize / apply planning — 完了
v4.7.16 Proposal materialize dry-run (P-6a) — 完了
v4.7.17 Proposal materialize --confirm (P-6b) — 完了
v4.7.18 Fragment apply dry-run (P-6c) — 完了
v4.7.19 Fragment apply --confirm (P-6d) — 完了
v4.7.20 P-6 post-implementation review — 実装済み
v4.7.21 Fragment apply add_itinerary field expansion (P-6e) — 実装済み
v4.7.22 Fragment apply add_note dry-run (P-6f) — 実装済み
v4.7.23 Fragment apply add_note --confirm (P-6f) — 実装済み
v4.7.24 Fragment apply add_expense dry-run (P-6g) — 実装済み
v4.7.25 Fragment apply add_expense --confirm (P-6g) — 実装済み
v4.7.26 Fragment apply add_reservation dry-run (P-6h) — 実装済み
v4.7.27 Fragment apply add_reservation --confirm (P-6h) — 実装済み
v4.7.28 Fragment apply update_itinerary dry-run (P-6i) — 実装済み
v4.7.29 Fragment apply update_itinerary --confirm (P-6i) — 実装済み
v4.7.30 P-6j destructive / structural apply policy — planning 済み
v4.7.31 Fragment apply delete_itinerary dry-run (P-6j) — 実装済み
v4.7.32 Fragment apply delete_itinerary --confirm (P-6j) — 実装済み
v4.7.33 P-6j safety / UX hardening for delete_itinerary — 実装済み
v4.7.34 P-6k reorder_itinerary planning — documentation-only（planning 済み）
v4.7.35 P-6k reorder_itinerary dry-run（same-day）— 実装済み
v4.7.36 P-6k reorder_itinerary --confirm（same-day）— 実装済み
v4.7.37 P-6l cross-day move planning — documentation-only（planning 済み）
v4.7.38 P-6l move_itinerary dry-run（cross-day）— 実装済み
v4.7.39 P-6l move_itinerary --confirm（cross-day）— 実装済み
```

Implementation plan: [v4.7.8 spec](../specifications/v4.7.8-proposal-implementation-planning.md) · P-6j confirm: v4.7.32 · P-6j delete dry-run: [v4.7.31 spec](../specifications/v4.7.31-p6j-delete-itinerary-dry-run.md) · P-6j policy: [v4.7.30 spec](../specifications/v4.7.30-p6j-destructive-structural-apply-policy.md) · P-6i confirm: [v4.7.29 spec](../specifications/v4.7.29-fragment-apply-update-itinerary-confirm.md) · P-6i dry-run: [v4.7.28 spec](../specifications/v4.7.28-fragment-apply-update-itinerary-dry-run.md) · P-6h confirm: [v4.7.27 spec](../specifications/v4.7.27-fragment-apply-add-reservation-confirm.md) · P-6h dry-run: [v4.7.26 spec](../specifications/v4.7.26-fragment-apply-add-reservation-dry-run.md) · P-6g confirm: [v4.7.25 spec](../specifications/v4.7.25-fragment-apply-add-expense-confirm.md) · P-6g dry-run: [v4.7.24 spec](../specifications/v4.7.24-fragment-apply-add-expense-dry-run.md) · P-6f confirm: [v4.7.23 spec](../specifications/v4.7.23-fragment-apply-add-note-confirm.md) · P-6f dry-run: [v4.7.22 spec](../specifications/v4.7.22-fragment-apply-add-note-dry-run.md) · P-6e: [v4.7.21 spec](../specifications/v4.7.21-fragment-apply-add-itinerary-field-expansion.md) · P-6 review: [v4.7.20 spec](../specifications/v4.7.20-p6-post-implementation-review.md) · P-6d: [v4.7.19 spec](../specifications/v4.7.19-fragment-apply-confirm.md) · P-6c: [v4.7.18 spec](../specifications/v4.7.18-fragment-apply-dry-run.md) · P-6b: [v4.7.17 spec](../specifications/v4.7.17-proposal-materialize-confirm.md) · P-6a: [v4.7.16 spec](../specifications/v4.7.16-proposal-materialize-dry-run.md) · P-5: [v4.7.15 spec](../specifications/v4.7.15-materialize-apply-planning-spec.md)

### CLI（v4.7.9+）

```bash
caglla proposal validate <envelope.json>
caglla proposal validate <envelope.json> --json
caglla proposal show <envelope.json>      # v4.7.10+
caglla proposal inspect <envelope.json>     # v4.7.10+
caglla proposal materialize <envelope.json> --dry-run [--output trip.json] [--start YYYY-MM-DD] [--end YYYY-MM-DD]  # v4.7.16+
caglla proposal materialize <envelope.json> --confirm [--start YYYY-MM-DD] [--end YYYY-MM-DD]  # v4.7.17+
caglla fragment validate <fragment.json>    # v4.7.11+
caglla fragment validate <fragment.json> --json
caglla fragment apply <fragment.json> --dry-run --trip <id> [--output preview.json]  # v4.7.18+
caglla fragment apply <fragment.json> --confirm --trip <id>  # v4.7.19+
```

`fragment apply --dry-run`: **apply preview / apply simulation** — **read-only DB access** で既存 Trip を読み取り、Trip / Day / Itinerary / Expense / Reservation は変更しない。v4.7.26+ で `add_reservation`（itinerary target）preview もサポート。v4.7.31+ で `delete_itinerary`（itinerary target、childless のみ preview 成功）もサポート。preview Trip JSON を `trip diff` 等で扱う場合は **`--output`** を使う。`--json` は gate report のみ。`fragment validate` とは異なり file-only ではない。

`fragment apply --confirm`: gate 通過後に **add_itinerary**（day target）、**add_note**（trip/day/itinerary）、**add_expense**（itinerary target）、**add_reservation**（itinerary target）、**update_itinerary**（itinerary target）、**delete_itinerary**（itinerary target、row-only delete）を DB へ反映。`--dry-run` と `--confirm` は併用不可（dry-run means no Trip domain data side effects）。

`proposal materialize --dry-run` / `--confirm`: Trip JSON 候補または DB 保存。`--dry-run` と `--confirm` は併用不可（dry-run means no side effects）。

Trip Proposal Envelope file の validation / 概要 / 詳細確認。**`trip validate-export` とは別責務** — schema v8 Trip には使わない。

Proposal Fragment file の validation。**Envelope とも Trip export とも別責務** — 既存 Trip への部分提案の入口。

Authoring 例: [examples/](examples/) · [examples-non-normative/](examples-non-normative/) · [examples.md](examples.md) · [ai-json-generation-guide.md](ai-json-generation-guide.md)

---

## Out of scope (still)

```text
fragment show / inspect
materialize / apply commands（P-6 以降）
proposal / fragment import / list（P-4+）
JSON schemas 確定
GUI for proposal review
```

---

## Related

- [Public JSON examples](examples/) — schema v8 Trip files
- [Non-normative examples](examples-non-normative/) — Envelope / Fragment
- [Examples](examples.md) — narrative と validate-export
- [AI JSON generation guide](ai-json-generation-guide.md) — 生成 AI 向け作法
- [Public README](README.md)
- [Travel Ledger](travel-ledger.md)
- [Schema overview](schema.md)
- [v4.7.8 Implementation planning spec](../specifications/v4.7.8-proposal-implementation-planning.md)
- [v4.7.4 Materialize gate spec](../specifications/v4.7.4-materialize-gate-concept-validation-rules.md)
- [v4.7.3 Proposal Fragment spec](../specifications/v4.7.3-proposal-fragment-concept-spec.md)
- [v4.7.2 Trip Proposal Envelope spec](../specifications/v4.7.2-trip-proposal-envelope-concept-spec.md)
