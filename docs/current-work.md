# Current Work

## Current phase

v4.7.16 released — P-6b planning

## Latest completed

- v4.7.16 Proposal materialize dry-run (P-6a) — **released**
- v4.7.15 Materialize / apply planning (P-5) — **released**
- v4.7.14 Public examples guard CI isolation hotfix — **released**
- v4.7.13 Proposal storage strategy planning (P-4) — **released**

## Repository state

- Cargo version: `4.7.16`
- Latest release: **v4.7.16** — [v4.7.16-notes.md](releases/v4.7.16-notes.md)
- **Proposal CLI:** `proposal validate` · `show` · `inspect` · `materialize --dry-run` · `fragment validate`

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
P-6a Envelope materialize --dry-run — v4.7.16 完了
P-6  materialize / apply implementation — 継続（P-6b 以降）
```

## Next action

**P-6b candidate** — `proposal materialize --confirm` → DB insert — 相談

並行候補: `fragment show` / `inspect`（file-only）

## Defer

- P-6c+ Fragment apply dry-run / confirm
- doctor / advisor finding schema / AI Fragment generation
- DB proposal storage / import / list
- GUI 実装

Canonical defer list: [long-term-version-strategy.md](long-term-version-strategy.md)
