# Schema overview

Travel Ledger の **公開 Trip データ契約** の入口です。フィールド定義の正本は [export-schema.md](../specifications/export-schema.md) です。

---

## Terminology

| Term | Meaning |
|---|---|
| **schema v8** | **Current canonical** Travel Ledger Trip export schema. Output of `trip export` today. |
| **schema v3+** | **Historical generation** where the nested Trip / Day / Itinerary model became established. Not the document to implement against for new work. |
| **Travel Ledger schema** | The public Trip data contract. **Currently represented by schema v8.** |
| **`schema_version`** | Integer in export JSON (currently `8`). Independent of CLI package version. |
| **`generator_version`** | CLI release version from `Cargo.toml` at export time. |

```text
schema v8:
  Current canonical Travel Ledger Trip export schema.

schema v3+:
  Historical generation where the nested Trip / Day / Itinerary model
  became established.

Travel Ledger schema:
  Public Trip data contract. Currently represented by schema v8.
```

---

## Why two version numbers?

**Export schema version** evolves when the **JSON contract** changes (new top-level sections, nested shapes, validation rules).

**CLI version** evolves with every release (features, docs, bug fixes) and may not change the export contract.

Example: CLI v4.7.1 exporting `schema_version: 8` is expected and correct.

---

## schema v3+ (historical)

Before v3, export formats differed in how expenses and related entities were attached. From **schema v3 onward**, the core shape stabilized:

```text
trip
days[]
  itineraries[]
    expenses[] / reservations[] / …
notes[], participants[], checklist_items[], …
```

Later versions added entities and fields without replacing that nesting idea:

| Version | Notable additions (summary) |
|---|---|
| v3 | Nested expenses under itineraries |
| v4 | `participants[]` |
| v5+ | Estimates, receipts, metadata refinements |
| **v8** | Receipt inbox / `trashed_at`, current canonical |

When reading old release notes or specs, **“schema v3”** often means “the nested trip model era,” not “you should export v3 today.”

---

## schema v8 (canonical today)

**Use schema v8** for:

- New integrations reading `trip export` JSON
- `trip validate-export` conformance checks
- Public examples and documentation

Export includes metadata:

```text
schema_version
generator
generator_version
exported_at
```

Import accepts older schema versions with migration logic in the CLI — that is an **implementation concern**, not the public contract for new consumers.

---

## Validation and tools

| Tool | Role |
|---|---|
| `trip export` | Canonical serialization |
| `trip import` | Backward-compatible ingestion |
| `trip validate-export` | **Public conformance gate** |
| `trip diff` | Compare two export files |

---

## Examples (planned)

Canonical narrative sample: Okinawa fixture in `samples/okinawa_sesoko_2026/`.

Dedicated `docs/public/examples/*.json` may be added in a later v4.7.x release. Until then, use export output from the sample trip or [export-schema.md](../specifications/export-schema.md).

---

## Related

- [Public README](README.md)
- [Travel Ledger](travel-ledger.md)
- [Proposals outline](proposals.md)
- [v4.7.0 concept review](../specifications/v4.7.0-schema-publication-travel-ledger-public-direction-concept-review.md)
