# Current Work

## Current phase

v4.6.9 — itinerary timeline service boundary（implementation complete, release pending）

## Latest completed

- v4.6.8 Itinerary list service boundary — **released**
- v4.6.7 Day list service boundary — **released**
- v4.6.6 Trip show service boundary — **released**
- v4.6.5 Read-only service boundary expansion — **released**
- v4.6.4 Read-only service boundary pilot — **released**
- v4.6.3 Command handler split Phase 1 — **released**
- v4.6.2 SQLite migration strategy review — **released**
- v4.6.1 SQLite FK / orphan data hardening review — **released**

## Repository state

- Cargo version: `4.6.9`
- Latest release: **v4.6.8** — [v4.6.8-notes.md](releases/v4.6.8-notes.md)
- **v4.6.9 spec:** [v4.6.9-itinerary-timeline-service-boundary.md](specifications/v4.6.9-itinerary-timeline-service-boundary.md)

## Next action

**v4.6.9 release**（optional formal release）

- `make check` PASS
- Okinawa golden: Colophon `Version: 4.6.9` のみ

**v4.6.10 候補:**

- `itinerary show` service boundary

**Parallel track（v4.6.x、独立）:**

- migration runner / orphan detection（[v4.6.2 review](specifications/v4.6.2-sqlite-migration-strategy-review.md) 順序）

## Defer

- Tauri / GUI 実装
- `main.rs` 一括 `commands/` 移動
- write command の service 化（Tier 3+）
- `trip delete` / `import` / `duplicate` / `receipt assign`

Canonical defer list: [long-term-version-strategy.md](long-term-version-strategy.md)
