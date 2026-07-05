# Current Work

## Current phase

v4.7.15 released — v4.7.16 planning

## Latest completed

- v4.7.15 Materialize / apply planning (P-5) — **released**
- v4.7.14 Public examples guard CI isolation hotfix — **released**
- v4.7.13 Proposal storage strategy planning (P-4) — **released**
- v4.7.12 Public examples validation guard — **released**
- v4.7.11 Proposal Fragment file validation (P-3) — **released**

## Repository state

- Cargo version: `4.7.15`
- Latest release: **v4.7.15** — [v4.7.15-notes.md](releases/v4.7.15-notes.md)
- **Proposal CLI:** `proposal validate` · `show` · `inspect` · `fragment validate`

## v4.7.x Proposal 実装

```text
P-0  planning — v4.7.8 完了
P-1  Envelope file validation — v4.7.9 完了
P-2  Envelope show / inspect — v4.7.10 完了
P-3  Fragment file validation — v4.7.11 完了
guard Public examples validation guard — v4.7.12 完了
P-4  storage strategy planning — v4.7.13 完了
hotfix guard CI isolation — v4.7.14 完了
P-5  materialize / apply planning — v4.7.15 完了
P-6  materialize / apply implementation — 後続
```

## Next action

**v4.7.16 — テーマ未確定。** 候補:

```text
P-6a proposal materialize --dry-run（Envelope → schema v8 JSON、DB なし）
fragment show / inspect（file-only、並行可）
```

次マイルストーンは相談のうえ決定。

## Defer

- P-6b+ DB commit / fragment apply
- doctor / advisor finding schema / AI Fragment generation
- DB proposal storage / import / list
- GUI 実装

Canonical defer list: [long-term-version-strategy.md](long-term-version-strategy.md)
