# Current Work

## Current phase

v4.7.11 planning — Proposal Fragment file validation (P-3 candidate)

## Latest completed

- v4.7.10 Proposal Envelope show / inspect (P-2) — **released**
- v4.7.9 Proposal Envelope file validation (P-1) — **released**
- v4.7.8 Proposal implementation planning — **released**
- v4.7.7 Public schema post-review — **released**
- v4.7.0〜v4.7.6 public direction / examples — **released**

## Repository state

- Cargo version: `4.7.10`
- Latest release: **v4.7.10** — [v4.7.10-notes.md](releases/v4.7.10-notes.md)
- **Proposal CLI:** `proposal validate` · `proposal show` · `proposal inspect`

## v4.7.x Proposal 実装

```text
P-0  planning — v4.7.8 完了
P-1  Envelope file validation — v4.7.9 完了
P-2  Envelope show / inspect — v4.7.10 完了
P-3  Fragment file validation — 候補
P-4  storage strategy — 後続
P-5  materialize / apply planning — 後続
P-6  materialize / apply implementation — 後続
```

## Next action

**v4.7.11 — テーマ未確定。** 候補のみ（materialize / apply / DB storage には入らない）:

```text
P-3 Proposal Fragment file validation
public examples CI validate-export check
export-schema / itinerary-model doc polish
migration runner / FK hardening
```

次マイルストーンは相談のうえ決定。v4.7.11 では **Fragment file validation の境界固め** を優先検討。

## Defer

- materialize / apply（P-6）
- proposal import / list / DB storage（P-4+）
- GUI 実装

Canonical defer list: [long-term-version-strategy.md](long-term-version-strategy.md)
