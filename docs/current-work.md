# Current Work

## Current phase

v4.7.6 release preparation — public JSON examples / concept stream post-review

## Latest completed

- v4.7.5 Public examples / AI JSON generation guide — **released**
- v4.7.4 Materialize gate concept / validation rules — **released**
- v4.7.2〜v4.7.3 Trip Proposal Envelope / Proposal Fragment — **released**
- v4.7.0〜v4.7.1 public direction / docs outline — **released**
- v4.6.x stream — **完了**

## Repository state

- Cargo version: `4.7.6`
- Latest release: **v4.7.5** — [v4.7.5-notes.md](releases/v4.7.5-notes.md)
- **v4.7.6 spec:** [v4.7.6-public-json-examples-concept-stream-post-review.md](specifications/v4.7.6-public-json-examples-concept-stream-post-review.md)
- **Public JSON:** [public/examples/](public/examples/) · [public/examples-non-normative/](public/examples-non-normative/)

## v4.7.x 新章

```text
v4.7.0  public direction concept review — 完了
v4.7.1  public README / schema docs outline — 完了
v4.7.2  Trip Proposal Envelope concept spec — 完了
v4.7.3  Proposal Fragment concept spec — 完了
v4.7.4  materialize gate / validation rules — 完了
v4.7.5  public examples / AI JSON generation guide — 完了
v4.7.6  public JSON examples / concept stream post-review — 準備完了（リリース待ち）
```

v4.7.x concept + authoring + **見える JSON** が v4.7.6 で揃った。

## Next action

**v4.7.6 正式リリース** — release commit → tag → workflow → follow-up commit。

次マイルストーン（v4.7.7 以降）は **未確定**。候補のみ:

```text
Proposal implementation planning
export-schema v8 doc alignment
public examples CI validate-export check
migration runner / FK hardening
```

v4.7.7 では Proposal / Fragment / materialize の実装には入らない。

## Defer

- materialize / apply command 実装
- Proposal / Fragment import 実装
- repository split
- GUI 実装

Canonical defer list: [long-term-version-strategy.md](long-term-version-strategy.md)
