# Current Work

## Current phase

v4.7.10 planning — Proposal Envelope show / inspect (P-2 candidate)

## Latest completed

- v4.7.9 Proposal Envelope file validation (P-1) — **released**
- v4.7.8 Proposal implementation planning — **released**
- v4.7.7 Public schema post-review — **released**
- v4.7.0〜v4.7.6 public direction / examples — **released**
- v4.6.x stream — **完了**

## Repository state

- Cargo version: `4.7.9`
- Latest release: **v4.7.9** — [v4.7.9-notes.md](releases/v4.7.9-notes.md)
- **v4.7.9 spec:** [v4.7.9-proposal-envelope-file-validation.md](specifications/v4.7.9-proposal-envelope-file-validation.md)
- **Proposal CLI:** `proposal validate <file>` · `proposal validate <file> --json`

## v4.7.x Proposal 実装

```text
P-0  planning — v4.7.8 完了
P-1  Envelope file validation — v4.7.9 完了
P-2  Envelope show / inspect — 候補
P-3  Fragment file validation — 後続
P-4  storage strategy — 後続
P-5  materialize / apply planning — 後続
P-6  materialize / apply implementation — 後続
```

## Next action

**v4.7.10 — テーマ未確定。** 候補のみ（Fragment / materialize / apply には入らない）:

```text
P-2 Proposal Envelope show / inspect
P-3 Proposal Fragment file validation
public examples CI validate-export check
export-schema / itinerary-model doc polish
migration runner / FK hardening
```

次マイルストーンは相談のうえ決定。v4.7.10 では **`proposal validate` の結果を人間が確認しやすくする show / inspect** を優先検討。

## Defer

- materialize / apply（P-6）
- Fragment validation 実装（P-3 — P-2 の後でも可）
- proposal import / list（P-4+）
- DB migration for proposals
- GUI 実装

Canonical defer list: [long-term-version-strategy.md](long-term-version-strategy.md)
