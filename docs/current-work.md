# Current Work

## Current phase

v4.7.8 release preparation — Proposal implementation planning

## Latest completed

- v4.7.7 Public schema post-review — **released**
- v4.7.6 Public JSON examples / concept stream post-review — **released**
- v4.7.5 Public examples / AI JSON generation guide — **released**
- v4.7.4 Materialize gate concept / validation rules — **released**
- v4.7.0〜v4.7.3 public direction / Envelope / Fragment — **released**
- v4.6.x stream — **完了**

## Repository state

- Cargo version: `4.7.8`
- Latest release (published): **v4.7.7** — [v4.7.7-notes.md](releases/v4.7.7-notes.md)
- **v4.7.8 spec:** [v4.7.8-proposal-implementation-planning.md](specifications/v4.7.8-proposal-implementation-planning.md)
- **v4.7.8 notes:** [v4.7.8-notes.md](releases/v4.7.8-notes.md)
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
v4.7.8  Proposal implementation planning — リリース準備中
```

v4.7.8 で Proposal 実装フェーズ（P-0〜P-6）・file-based 優先・command 候補を整理。実装は未着手。

## Next action

**v4.7.8 — Proposal implementation planning**（documentation-only、未タグ）

主成果物: [v4.7.8-proposal-implementation-planning.md](specifications/v4.7.8-proposal-implementation-planning.md)

次マイルストーン候補（P-1 実装 — 未確定）:

```text
Proposal Envelope file validation（proposal validate 候補）
```

v4.7.8 では Proposal / Fragment / materialize の **実装には入らない**。

## Defer

- materialize / apply command 実装（P-6）
- Proposal / Fragment import / list（P-4+）
- DB migration for proposals
- repository split
- GUI 実装

Canonical defer list: [long-term-version-strategy.md](long-term-version-strategy.md)
