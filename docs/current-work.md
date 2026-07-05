# Current Work

## Current phase

v4.7.8 planning — Proposal implementation planning candidate

## Latest completed

- v4.7.7 Public schema post-review — **released**
- v4.7.6 Public JSON examples / concept stream post-review — **released**
- v4.7.5 Public examples / AI JSON generation guide — **released**
- v4.7.4 Materialize gate concept / validation rules — **released**
- v4.7.0〜v4.7.3 public direction / Envelope / Fragment — **released**
- v4.6.x stream — **完了**

## Repository state

- Cargo version: `4.7.7`
- Latest release: **v4.7.7** — [v4.7.7-notes.md](releases/v4.7.7-notes.md)
- **v4.7.7 spec:** [v4.7.7-public-schema-post-review.md](specifications/v4.7.7-public-schema-post-review.md)
- **Public JSON:** [public/examples/](public/examples/) · [public/examples-non-normative/](public/examples-non-normative/)

## v4.7.x 新章

```text
v4.7.0  public direction concept review — 完了
v4.7.1  public README / schema docs outline — 完了
v4.7.2  Trip Proposal Envelope concept spec — 完了
v4.7.3  Proposal Fragment concept spec — 完了
v4.7.4  materialize gate / validation rules — 完了
v4.7.5  public examples / AI JSON generation guide — 完了
v4.7.6  public JSON examples / concept stream post-review — 完了
v4.7.7  public schema post-review — 完了
```

v4.7.7 で public schema / export-schema / examples / AI guide の整合性 post-review 完了。外向き入口は v4.7.0〜v4.7.7 で **揃った**。

## Next action

**v4.7.8 — テーマ未確定。** 候補のみ（確定実装には入らない）:

```text
Proposal implementation planning
export-schema / itinerary-model doc polish
public examples CI validate-export check
migration runner / FK hardening
```

次マイルストーンは相談のうえ決定。v4.7.8 ではいきなり Proposal / Fragment / materialize の実装には入らず、まず **planning** から。

## Defer

- materialize / apply command 実装
- Proposal / Fragment import 実装
- repository split
- GUI 実装

Canonical defer list: [long-term-version-strategy.md](long-term-version-strategy.md)
