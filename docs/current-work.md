# Current Work

## Current phase

v4.6.4 planning — read-only service boundary pilot

## Latest completed

- v4.6.3 Command handler split Phase 1 — **released**
- v4.6.2 SQLite migration strategy review — **released**
- v4.6.1 SQLite FK / orphan data hardening review — **released**

## Repository state

- Cargo version: `4.6.3`
- Latest release: **v4.6.3** — [v4.6.3-notes.md](releases/v4.6.3-notes.md)
- **v4.6.3 review:** [v4.6.3-command-handler-split-phase-1.md](specifications/v4.6.3-command-handler-split-phase-1.md)

## Next action

**v4.6.4 — read-only service boundary pilot**（optional implementation）

候補 command:

```text
trip list / show
day list
itinerary list / timeline
trip stats
```

- service が structured result を返し、CLI は既存 human/JSON 出力を維持
- CLI behavior / golden 不変を gate とする

**Parallel track（v4.6.x、独立）:**

- migration runner / orphan detection（[v4.6.2 review](specifications/v4.6.2-sqlite-migration-strategy-review.md) 順序）

## Defer

- Tauri / GUI 実装
- `main.rs` 一括 `commands/` 移動
- write command の service 化（Tier 3+）
- `trip delete` / `import` / `duplicate` / `receipt assign`

Canonical defer list: [long-term-version-strategy.md](long-term-version-strategy.md)
