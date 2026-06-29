#!/usr/bin/env bash
# Regenerate samples/okinawa_sesoko_2026/expected-export-v3.json from seed.sh
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SAMPLE_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WORK="${CAGLLA_SAMPLE_WORKDIR:-$ROOT}"
EXPORT_PATH="$WORK/okinawa-export-regen.json"
GOLDEN="$SAMPLE_DIR/expected-export-v3.json"

cd "$ROOT"
bash "$SAMPLE_DIR/seed.sh"

cargo run --quiet --manifest-path "$ROOT/Cargo.toml" -- \
  trip export 1 --output "$EXPORT_PATH"

jq '{
  schema_version: .schema_version,
  trip: (.trip | {name, start_date, end_date, summary}),
  days: .days,
  checklist_items: [.checklist_items[] | {title, is_done, sort_order}],
  notes: .notes,
  participants: (.participants // []),
  receipts: [
    .receipts[]?
    | {
        day_ref,
        amount,
        currency,
        memo,
        status,
        trashed: (.trashed_at != null)
      }
  ] | sort_by(.memo)
}' "$EXPORT_PATH" > "$GOLDEN"

MD_EXPORT_PATH="$WORK/okinawa-export-md-regen.md"
MD_GOLDEN="$SAMPLE_DIR/expected-export-md.md"

cargo run --quiet --manifest-path "$ROOT/Cargo.toml" -- \
  trip export-md 1 --output "$MD_EXPORT_PATH"

sed -E 's/^Generated at: .*/Generated at: TIMESTAMP/' "$MD_EXPORT_PATH" > "$MD_GOLDEN"

echo "Wrote $GOLDEN"
echo "Wrote $MD_GOLDEN"
