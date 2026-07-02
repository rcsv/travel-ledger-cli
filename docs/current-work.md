# Current Work

## Current phase

v4.6.0 planning — Core architecture hardening review

## Latest completed

- v4.5.1 doctor / advisor Receipt utilization — **released**
- v4.5.0 Receipt Inbox responsibilities review — **released**
- v4.4.8 Travel Book presentation helper cleanup — **released**

## Repository state

- Cargo version: `4.5.1`
- Latest release: **v4.5.1** — [v4.5.1-notes.md](releases/v4.5.1-notes.md)
- **v4.5.1 plan:** [v4.5.1-doctor-advisor-receipt-utilization-implementation-plan.md](specifications/v4.5.1-doctor-advisor-receipt-utilization-implementation-plan.md)
- **v4.5.0 review:** [v4.5.0-receipt-inbox-responsibilities-review.md](specifications/v4.5.0-receipt-inbox-responsibilities-review.md)

## Next action

**v4.6.0 — Core architecture hardening review**（documentation-only）

- TripStats.days 意味、SQLite FK / orphan data、migration strategy、Receipt state、main.rs、domain/models.rs 等を棚卸し
- v4.6.1+ は review で確定した優先順位に従って小さく実装

**Defer:**

- `TravelBookDocument` prototype（UI / Venue 要件まで）
- Evidence / Attachment / Travel Journal 実装
- trip stats への Receipt 反映、Potential Actual 表示

## Do not start yet

- Receipt 専用 `image_path` 先行実装
- trip stats / Planned vs Actual への Receipt・Pending 反映
- Balance / Settlement
- `TravelBookDocument` full abstraction（UI/Venue requirements）

Canonical defer list: [long-term-version-strategy.md](long-term-version-strategy.md)
