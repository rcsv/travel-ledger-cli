# Current Work

## Current phase

v4.7.14 released — v4.7.15 planning

## Latest completed

- v4.7.14 Public examples guard CI isolation hotfix — **released**
- v4.7.13 Proposal storage strategy planning (P-4) — **released**
- v4.7.12 Public examples validation guard — **released**
- v4.7.11 Proposal Fragment file validation (P-3) — **released**
- v4.7.10 Proposal Envelope show / inspect (P-2) — **released**

## Repository state

- Cargo version: `4.7.14`
- Latest release: **v4.7.14** — [v4.7.14-notes.md](releases/v4.7.14-notes.md)
- **Examples guard:** `tests/public_examples_validation_guard.rs`（CI isolation 適用済み）

## v4.7.x Proposal 実装

```text
P-0  planning — v4.7.8 完了
P-1  Envelope file validation — v4.7.9 完了
P-2  Envelope show / inspect — v4.7.10 完了
P-3  Fragment file validation — v4.7.11 完了
guard Public examples validation guard — v4.7.12 完了
P-4  storage strategy planning — v4.7.13 完了
hotfix guard CI isolation — v4.7.14 完了
P-5  materialize / apply planning — 後続
P-6  materialize / apply implementation — 後続
```

## Next action

**v4.7.15 — テーマ未確定。** 候補:

```text
P-5 materialize / apply planning spec
P-3 残 fragment show / inspect（file-only）
```

次マイルストーンは相談のうえ決定。

## Defer

- DB storage / import / list 実装（P-5 後に再評価）
- materialize / apply（P-6）
- GUI 実装

Canonical defer list: [long-term-version-strategy.md](long-term-version-strategy.md)
