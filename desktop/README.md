# Travel Ledger Desktop

Developer preview of a Tauri app for Travel Ledger SQLite databases.

## Status

- Source-only (`bundle.active = false`) — not distributed with GitHub Release assets
- Latest workflow: **v4.12.0** — create Trips, add activities, and edit selected Activity details
- Persistence: last successfully opened DB path only (v4.10.1)
- Profile / user preferences are not implemented yet (future candidate)

## Run from source

```sh
cd desktop
npm install
npm run tauri dev
```

Or: `tools/desktop/dev.sh`

## Notes

- Open an existing `.db` / `.sqlite` / `.sqlite3` Travel Ledger database
- Create a Trip with its Days, append activities, then edit a selected Activity's title, start time, location, and note
- Activity is the UI term; the internal data model remains Itinerary
- Forget Database clears the remembered path only — it does **not** delete the file
- Trip editing, Activity reorder/move/delete, Checklist, and desktop bundle distribution are out of scope
