# Current Work

## Current phase

v4.7.10 release preparation — Proposal Envelope show / inspect (P-2)

## Latest completed

- v4.7.9 Proposal Envelope file validation (P-1) — **released**
- v4.7.8 Proposal implementation planning — **released**
- v4.7.7 Public schema post-review — **released**
- v4.7.0〜v4.7.6 public direction / examples — **released**

## Repository state

- Cargo version: `4.7.10`
- Latest release (published): **v4.7.9** — [v4.7.9-notes.md](releases/v4.7.9-notes.md)
- **v4.7.10 spec:** [v4.7.10-proposal-envelope-show-inspect.md](specifications/v4.7.10-proposal-envelope-show-inspect.md)
- **Proposal CLI:** `proposal validate` · `proposal show` · `proposal inspect`

## v4.7.x Proposal 実装

```text
P-0  planning — v4.7.8 完了
P-1  Envelope file validation — v4.7.9 完了
P-2  Envelope show / inspect — v4.7.10 リリース準備中
P-3  Fragment file validation — 後続
```

## Next action

**v4.7.10 — Proposal Envelope show / inspect**（P-2 実装、未タグ）

次マイルストーン候補（P-3 — 未確定）:

```text
Proposal Fragment file validation
public examples CI validate-export check
```

v4.7.10 では Fragment / materialize / apply には入らない。

## Defer

- materialize / apply（P-6）
- Fragment validation（P-3 — 次候補）
- proposal import / list（P-4+）
- DB migration for proposals

Canonical defer list: [long-term-version-strategy.md](long-term-version-strategy.md)
