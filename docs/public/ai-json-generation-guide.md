# AI JSON generation guide

生成 AI や外部 provider が Travel Ledger 関連 JSON を **安全に** 生成するための作法です。詳細仕様は [v4.7.5 spec](../specifications/v4.7.5-public-examples-ai-json-generation-guide.md)。例は [examples.md](examples.md)。

**最重要:** きれいな JSON を作ることより、**未確定情報を正式データに混ぜない** こと。

---

## 出力してよい JSON の種類

| 種類 | いつ使う |
|---|---|
| **schema v8 Trip** | 採用済み・実日付確定・事実確認済み |
| **Trip Proposal Envelope** | 旅行全体の未採用案 |
| **Proposal Fragment** | 既存 Trip への部分提案 |

---

## 使い分け（早見表）

### schema v8 Trip を直接出してよい

```text
✓ 実日付（start_date / end_date）が確定
✓ title / destination が明確
✓ Day 構成が確定
✓ Itinerary が採用済み
✓ 予約・費用・メモが事実として確認済み
```

### Trip Proposal Envelope にすべき

```text
✓ 旅行全体の未採用案
✓ 日付未定 or 日付候補のみ
✓ 複数案の比較段階
✓ AI / provider / manual draft 由来でまだ正式 Trip にしていない
```

### Proposal Fragment にすべき

```text
✓ 既存 Trip への部分提案
✓ 特定 Day / Itinerary への追加・補足・代替
✓ warning / 注意喚起
✓ target が未確定の差し込み候補
```

### schema v8 Trip にしてはいけない

```text
✗ 日付未定
✗ 複数候補の比較段階
✗ AI が勝手に作った旅行案（未採用）
✗ 予約・料金・営業時間が未確認
✗ 既存 Trip へ差し込むだけの部分提案
```

---

## 捏造禁止

以下を **確認なく invent しない**:

```text
予約番号（confirmation_code）
価格・料金（amount）
営業時間・空席状況
フライト便名・時刻（未確認の場合）
ホテル予約の確定状態
```

旅行計画でなく **ファンタジー小説** になる。不明は不明のまま。

---

## 不明値の扱い

| 状況 | やること |
|---|---|
| 値が不明 | フィールドを **省略** または `null` — 推測で埋めない |
| 不足が重要 | `missing_fields` に列挙（Envelope / Fragment） |
| 前提・推測 | `assumptions` に明示（「仮定していること」） |
| リスク・古さ | `warnings` に列挙 |

```text
不明 → missing_fields / assumptions / warnings
確定 → schema v8 の該当フィールド（採用済みのみ）
```

---

## Materialize gate との関係

```text
AI 出力（Envelope / Fragment）
  → 人間が review
  → adoption gate（materialize / apply）
  → schema v8 Trip

AI が schema v8 Trip を出力しても、
  「採用済み」扱いにしない — 人間の gate が必要
```

AI には **gate を bypass する出力をさせない**。`"status": "adopted"` のような独自フラグで正式化しない。

---

## validate-export 境界

```text
schema v8 Trip          → trip validate-export 対象
Trip Proposal Envelope  → proposal validate / show / inspect 対象（v4.7.9+）— trip validate-export ではない
Proposal Fragment       → 対象外（将来 fragment validate）
gate 後の Trip          → trip validate-export 対象
```

Proposal を `trip validate-export` に通すよう指示しない。Envelope は `proposal validate` を使う。

---

## プロンプト例（英語）

### System / role

```text
You are generating Travel Ledger-compatible planning data.

Rules:
1. Do not mix adopted facts with unadopted suggestions.
2. Do not invent confirmed reservations, prices, opening hours, or booking references.
3. If a value is unknown, leave it unknown and list it in missing_fields.
4. If trip dates are not confirmed, output a Trip Proposal Envelope, NOT a schema v8 Trip.
5. If the suggestion modifies an existing trip, output a Proposal Fragment, NOT a schema v8 Trip.
6. All unadopted suggestions remain outside schema v8 until a human completes materialize/apply gate.
7. schema v8 Trip requires start_date, end_date, and adopted day/itinerary structure.
8. Use warnings for stale or unverified information; use assumptions for explicit guesses.
```

### User context（例）

```text
Context:
- Existing adopted trip: "Okinawa 3-day" 2026-04-26..28 (schema v8, already in ledger)
- Task: Suggest a lunch spot for day 2 after the aquarium

Output: Proposal Fragment only. Do not output schema v8 Trip.
Include missing_fields for unverified opening hours.
```

### Whole-trip draft（例）

```text
Context:
- User wants a 4-day Okinawa trip "sometime in autumn 2026" — dates not fixed
- Source: AI brainstorm

Output: Trip Proposal Envelope with date_policy flexible_dates or undated.
Do NOT set start_date/end_date on a schema v8 Trip.
List required_decisions for materialize gate.
```

---

## プロンプト例（日本語補足）

```text
Travel Ledger 用の JSON を生成してください。

- 採用済みの事実と、まだ採用していない候補案を混ぜないでください。
- 日付が確定していない旅行案は schema v8 Trip にせず、Trip Proposal Envelope としてください。
- 既存 Trip への追加・差し替え案は Proposal Fragment としてください。
- 予約番号・料金・営業時間は確認できない場合、捏造せず missing_fields / warnings に書いてください。
- 正式 Trip 化は人間の materialize / apply gate 後のみです。
```

---

## AI に渡す入力情報（推奨）

| 入力 | 用途 |
|---|---|
| 既存 Trip export（schema v8） | Fragment の target 文脈 |
| ユーザー意図（自然言語） | 何を提案するか |
| 確認済み事実のリスト | schema v8 に載せてよい境界 |
| 未確認事項のリスト | missing_fields の種 |
| v4.7.2〜v4.7.4 概念 spec への参照 | Envelope / Fragment / gate |
| [examples/](examples/) schema v8 JSON | 採用済み Trip の参照実例 |
| [examples-non-normative/](examples-non-normative/) | Envelope / Fragment 概念例 |

---

## 関連

- [Public examples](examples.md)
- [schema v8 JSON files](examples/)
- [Non-normative examples](examples-non-normative/)
- [Proposals outline](proposals.md)
- [Schema overview](schema.md)
- [v4.7.15 spec](../specifications/v4.7.15-materialize-apply-planning-spec.md)
- [v4.7.13 spec](../specifications/v4.7.13-proposal-storage-strategy-planning.md)
- [v4.7.12 spec](../specifications/v4.7.12-public-examples-validation-guard.md)
- [v4.7.11 spec](../specifications/v4.7.11-proposal-fragment-file-validation.md)
- [v4.7.10 spec](../specifications/v4.7.10-proposal-envelope-show-inspect.md)
- [v4.7.9 spec](../specifications/v4.7.9-proposal-envelope-file-validation.md)
- [v4.7.8 spec](../specifications/v4.7.8-proposal-implementation-planning.md)
- [v4.7.7 spec](../specifications/v4.7.7-public-schema-post-review.md)
- [v4.7.6 spec](../specifications/v4.7.6-public-json-examples-concept-stream-post-review.md)
