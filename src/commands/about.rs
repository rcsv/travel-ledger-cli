//! English overview for the `--about` toy flag.

const ABOUT: &str = r#"Caglla.Travel CLI
====================

A local-first travel planning CLI for managing trips, itineraries, checklists,
expenses, and Markdown/JSON exports. Data is stored in a local SQLite database
(caglla.db). Web sync and cloud features are not supported.

Features
--------
  - Trip CRUD, duplicate, stats, doctor, and advisor
  - Day list/show/update and day swap
  - Itinerary CRUD and timeline view
  - Notes, expenses, estimates, reservations, and participants
  - Checklist management and auto-generation (checklist-generate)
  - JSON export/import, validate-export, diff, and Markdown export (export-md)
  - Trip / Day summaries (--summary)

Data model
----------
  Trip
   └─ Day
        └─ Itinerary
             ├─ Expense
             └─ Note

Itinerary is not a venue — it is the smallest unit of travel activity. A title
and --day are enough to register one; location is optional. Highways, fuel
stops, check-in, and return-home legs are all valid itineraries.

Quick start
-----------
  caglla db reset
  caglla trip add "Okinawa" --start 2026-04-26 --end 2026-04-29
  caglla itinerary add 1 --day 1 --time 09:00 --duration 90 --travel 20 "Shuri Castle"
  caglla itinerary timeline 1

Main commands
-------------
  Trip        trip add/list/show/update/delete/duplicate/stats/...
  Day         day list/show/update/swap
  Itinerary   itinerary add/list/show/update/delete/timeline
  Checklist   checklist ... / trip checklist-generate
  Export      trip export/import/validate-export/diff/export-md
  Diagnostics trip doctor / trip advisor

Documentation: https://github.com/rcsv/travel-ledger-cli
License: MIT
"#;

/// Print the English overview to stdout.
pub fn print() {
    print!("{ABOUT}");
}
