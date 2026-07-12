# Proposals — public contract

AI、旅行業者、ブログ、手入力などから来る **まだ採用していない旅行案** の公開契約です。概念の背景や実装上の詳細は [v4.7.2](../specifications/v4.7.2-trip-proposal-envelope-concept-spec.md)（Envelope）、[v4.7.3](../specifications/v4.7.3-proposal-fragment-concept-spec.md)（Fragment）、[v4.7.4](../specifications/v4.7.4-materialize-gate-concept-validation-rules.md)（gate）を参照してください。

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

## Adoption gate

**Adoption gate** は、Proposal / Fragment を正式データにしてよいか人間が判断する **門番** です。

```text
Input   → Trip Proposal Envelope または Proposal Fragment
Gate    → human review / required decisions / validation / warnings
Output  → new Trip | updated Trip | reject | defer
```

| ルート | 操作 | 結果 |
|---|---|---|
| Trip Proposal Envelope | **materialize** | 新しい schema v8 Trip |
| Proposal Fragment | **apply** | 既存 schema v8 Trip を更新 |

### Validation（概要）

```text
blocking:     実日付未確定、title 欠如、target Trip 不在 など → 採用を止める
non-blocking: 期限切れ、古い情報、time_overlap の可能性 など → 人間が判断
```

期限切れ・古さは **warning のみ**。自動破棄や import 禁止にはしない。

---

## Trip Proposal Envelope

```text
Trip Proposal Envelope
  ├─ metadata
  ├─ proposal        — title, destination, date_policy, 候補行程
  └─ materialize hints
```

Gate で確定すること（例）: `start_date` / `end_date`、title、Day 構成、Itinerary 採否。

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

## CLI commands

コマンド名は `travel-ledger-cli` です。

```bash
travel-ledger-cli proposal validate <envelope.json>
travel-ledger-cli proposal validate <envelope.json> --json
travel-ledger-cli proposal show <envelope.json>
travel-ledger-cli proposal inspect <envelope.json>
travel-ledger-cli proposal materialize <envelope.json> --dry-run [--output trip.json] [--start YYYY-MM-DD] [--end YYYY-MM-DD]
travel-ledger-cli proposal materialize <envelope.json> --confirm [--start YYYY-MM-DD] [--end YYYY-MM-DD]
travel-ledger-cli fragment validate <fragment.json>
travel-ledger-cli fragment validate <fragment.json> --json
travel-ledger-cli fragment apply <fragment.json> --dry-run --trip <id> [--output preview.json]
travel-ledger-cli fragment apply <fragment.json> --confirm --trip <id>
```

### dry-run と confirm

| コマンド | 意味 | DB への影響 |
|---|---|---|
| `proposal materialize --dry-run` | Trip JSON 候補の preview | なし |
| `proposal materialize --confirm` | 新規 Trip を DB に保存 | あり |
| `fragment apply --dry-run` | 既存 Trip 更新の preview（read-only DB access） | なし |
| `fragment apply --confirm` | gate 通過後に既存 Trip を更新 | あり |

`--dry-run` と `--confirm` は併用できません。

`fragment apply --confirm` で反映できる intent には、itinerary への add / update / delete、note・expense・estimate・reservation の追加などがあります。対応範囲の詳細は [command-reference.md](../command-reference.md) を参照してください。

### validate の責務

| 対象 | コマンド | 備考 |
|---|---|---|
| schema v8 Trip | `trip validate-export` | 採用済み Trip JSON の検証 |
| Trip Proposal Envelope | `proposal validate` | `trip validate-export` とは別責務 |
| Proposal Fragment | `fragment validate` | Envelope / Trip export とも別責務 |

---

## Authoring and examples

- [examples/](examples/) — schema v8 Trip JSON（normative）
- [examples-non-normative/](examples-non-normative/) — Envelope / Fragment 概念例
- [examples.md](examples.md) — narrative と validate-export の読み方
- [ai-json-generation-guide.md](ai-json-generation-guide.md) — 生成 AI 向け作法
- [../ai.md](../ai.md) — AI 連携の概念と責務分担

---

## Not in public contract yet

以下は現時点の公開契約の外です。将来追加の候補であり、現行 CLI にあるとは限りません。

- `fragment show` / `fragment inspect`
- `proposal import` / `fragment import` / 一覧コマンド
- Proposal / Fragment の確定 JSON Schema 公開
- GUI による proposal review

実装メモ・バージョン別の詳細仕様は [specifications/](../specifications/) と [releases/](../releases/) を参照してください。

---

## Related

- [Public README](README.md)
- [Travel Ledger](travel-ledger.md)
- [Schema overview](schema.md)
- [AI integration guide](../ai.md)
- [v4.7.2 Trip Proposal Envelope spec](../specifications/v4.7.2-trip-proposal-envelope-concept-spec.md)
- [v4.7.3 Proposal Fragment spec](../specifications/v4.7.3-proposal-fragment-concept-spec.md)
- [v4.8.10 structured errors post-release series review](../specifications/v4.8.10-fragment-apply-structured-errors-post-release-review.md)
- [v4.8.9 confirm transaction structured errors follow-up](../specifications/v4.8.9-fragment-apply-confirm-transaction-structured-errors-follow-up.md)
- [v4.8.8 structured errors limited wiring expansion](../specifications/v4.8.8-fragment-apply-structured-errors-limited-wiring-expansion.md)
- [v4.8.7 structured errors public contract review](../specifications/v4.8.7-fragment-apply-structured-errors-public-contract-review.md)
- [v4.8.6 JSON structured_errors exposure](../specifications/v4.8.6-fragment-apply-json-structured-errors-exposure.md)
