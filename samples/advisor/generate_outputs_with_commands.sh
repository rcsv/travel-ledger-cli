#!/usr/bin/env bash
# trip advisor --with-commands 全パターンの検証出力を生成する
#
# 使い方（リポジトリルートから）:
#   bash samples/advisor/generate_outputs_with_commands.sh
#
# 出力先:
#   samples/advisor/outputs_with_commands/*.txt

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
OUT_DIR="$ROOT/samples/advisor/outputs_with_commands"
cd "$ROOT"

mkdir -p "$OUT_DIR"

run() {
  cargo run --quiet -- "$@" 2>&1
}

write_header() {
  local file="$1"
  local title="$2"
  local description="$3"
  {
    echo "# trip advisor --with-commands validation: $title"
    echo "# Generated: $(date -u +"%Y-%m-%dT%H:%M:%SZ")"
    echo "#"
    echo "# $description"
    echo "#"
    echo "# Command: cargo run -- trip advisor 1 --with-commands"
    echo ""
  } >"$file"
}

append_advisor_output() {
  local file="$1"
  run trip advisor 1 --with-commands >>"$file"
}

echo "==> 01-clean-trip"
run db reset >/dev/null
run trip add "Clean Trip" --start 2026-05-01 --end 2026-05-01 >/dev/null
run itinerary add 1 --day 1 --time 12:00 --duration 60 --travel 20 \
  --location "Cafe Example" "Lunch" >/dev/null
run itinerary update 1 --category restaurant >/dev/null
write_header "$OUT_DIR/01-clean-trip.txt" "clean trip" \
  "問題なし → No major issues found.（Try なし）"
append_advisor_output "$OUT_DIR/01-clean-trip.txt"

echo "==> 02-empty-itinerary"
run db reset >/dev/null
run trip add "Empty Trip" >/dev/null
write_header "$OUT_DIR/02-empty-itinerary.txt" "empty itinerary" \
  "Info + EmptyItinerary advice + itinerary add Try"
append_advisor_output "$OUT_DIR/02-empty-itinerary.txt"

echo "==> 03-overloaded-day"
run db reset >/dev/null
run trip add "Overloaded Trip" --start 2026-05-02 --end 2026-05-02 >/dev/null
for i in $(seq 1 8); do
  run itinerary add 1 --day 1 --time "$(printf "%02d:00" $((8 + i)))" \
    --duration 30 --travel 5 --location "Spot $i" "Activity $i" >/dev/null
  run itinerary update "$i" --category restaurant >/dev/null
done
write_header "$OUT_DIR/03-overloaded-day.txt" "overloaded day" \
  "OverloadedDay warning + advice + timeline/list Try"
append_advisor_output "$OUT_DIR/03-overloaded-day.txt"

echo "==> 04-no-restaurant"
run db reset >/dev/null
run trip add "No Restaurant Trip" --start 2026-05-03 --end 2026-05-03 >/dev/null
run itinerary add 1 --day 1 --time 10:00 --duration 120 --travel 30 \
  --location "Shurijo Castle" "Castle visit" >/dev/null
run itinerary update 1 --category activity >/dev/null
write_header "$OUT_DIR/04-no-restaurant.txt" "no restaurant" \
  "NoRestaurant warning + advice + restaurant add Try"
append_advisor_output "$OUT_DIR/04-no-restaurant.txt"

echo "==> 05-high-travel-time"
run db reset >/dev/null
run trip add "High Travel Trip" --start 2026-05-04 --end 2026-05-04 >/dev/null
run itinerary add 1 --day 1 --time 09:00 --duration 60 --travel 100 \
  --location "North" "Drive north leg 1" >/dev/null
run itinerary update 1 --category transport >/dev/null
run itinerary add 1 --day 1 --time 12:00 --duration 60 --travel 90 \
  --location "South" "Drive south leg 2" >/dev/null
run itinerary update 2 --category transport >/dev/null
run itinerary add 1 --day 1 --time 18:00 --duration 90 --travel 15 \
  --location "Naha" "Dinner" >/dev/null
run itinerary update 3 --category restaurant >/dev/null
write_header "$OUT_DIR/05-high-travel-time.txt" "high travel time" \
  "HighTravelTime warning + advice + timeline/list Try"
append_advisor_output "$OUT_DIR/05-high-travel-time.txt"

echo "==> 06-missing-duration"
run db reset >/dev/null
run trip add "Missing Duration Trip" --start 2026-05-05 --end 2026-05-05 >/dev/null
run itinerary add 1 --day 1 --time 14:00 --travel 10 \
  --location "Unknown stop" "Unplanned stop" >/dev/null
run itinerary add 1 --day 1 --time 19:00 --duration 60 --travel 10 \
  --location "Restaurant" "Dinner" >/dev/null
run itinerary update 2 --category restaurant >/dev/null
write_header "$OUT_DIR/06-missing-duration.txt" "missing duration" \
  "MissingDuration warning + advice + itinerary list Try"
append_advisor_output "$OUT_DIR/06-missing-duration.txt"

echo "==> 07-combined-issues"
run db reset >/dev/null
run trip add "Combined Issues Trip" --start 2026-05-06 --end 2026-05-08 >/dev/null
for i in $(seq 1 8); do
  run itinerary add 1 --day 1 --time "$(printf "%02d:00" $((7 + i)))" \
    --duration 20 --travel 25 --location "Leg $i" "Transit $i" >/dev/null
  run itinerary update "$i" --category transport >/dev/null
done
run itinerary add 1 --day 2 --time 10:00 --duration 90 --travel 20 \
  --location "Museum" "Museum visit" >/dev/null
run itinerary update 9 --category museum >/dev/null
run itinerary add 1 --day 3 --time 11:00 --travel 30 --location "Walk" "Free time" >/dev/null
write_header "$OUT_DIR/07-combined-issues.txt" "combined issues" \
  "複数 issue ごとに Warning + Advice + Try"
append_advisor_output "$OUT_DIR/07-combined-issues.txt"

echo ""
echo "==> 検証出力を生成しました: $OUT_DIR"
ls -1 "$OUT_DIR"
