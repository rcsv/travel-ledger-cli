# Travel Ledger

## What it is

**Travel Ledger** is a structured format for trip data that travelers can **own locally** — not locked inside a single app or service.

It models adopted plans and records: days, itineraries, expenses, notes, reservations, participants, and related metadata. The canonical interchange format today is **export schema v8** (see [schema.md](schema.md)).

Caglla.Travel is **not** a booking site, map app, or social network. It is a **local-first travel data ledger** with tools to validate, convert, and inspect that data.

---

## Problem it addresses

Trip planning data tends to scatter across:

- booking and OTA sites
- maps and navigation apps
- notes and chat
- email confirmations
- spreadsheets and AI-generated itineraries

Services come and go; history is hard to share with travel companions; AI suggestions are easy to accept without review.

Travel Ledger keeps **adopted** trip data in one structured place. Proposals from AI or providers stay **outside** the canonical Trip until a human adopts them (see [proposals.md](proposals.md)).

---

## Layers

```text
Travel Ledger schema     — public Trip data contract (currently schema v8)

travel-ledger-cli        — reference implementation in this repository
  export / import / validate-export / diff
  SQLite persistence, CLI commands, doctor / advisor

future GUI               — schema consumer (not implemented)
  reads/writes via service layer; separate presentation

Proposal layer (future)  — outside schema v8 Trip
  Trip Proposal, Proposal Fragment, materialize gate
```

---

## Reference implementation (`travel-ledger-cli`)

| Role | CLI surface (examples) |
|---|---|
| Serialize | `trip export` |
| Deserialize | `trip import` |
| Validate | `trip validate-export` |
| Compare | `trip diff` |
| Quality hints | `trip doctor`, `trip advisor` |
| Human-readable output | `trip export-md` (Travel Book) |

Third parties can treat `trip validate-export` as a **conformance check** against the published contract.

Internal JSON from `--json` on list/show commands is **not** the public contract — it may change between CLI versions.

---

## Future GUI

A future desktop or mobile app (`travel-ledger-app` — name TBD) would be a **schema consumer**:

- Same domain model and export contract as the CLI
- Mutations through application services (pattern established in v4.6.x)
- No requirement to duplicate business rules in the UI layer

GUI work is deferred; v4.7.x focuses on public documentation and Proposal concepts.

---

## Related reading

- [Public README](README.md)
- [Schema overview](schema.md)
- [Proposals (outline)](proposals.md)
- [v4.7.0 concept review](../specifications/v4.7.0-schema-publication-travel-ledger-public-direction-concept-review.md)
- [Future roadmap memo](../future-roadmap-planning-memo.md)
