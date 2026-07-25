# Travel Ledger Desktop

Developer preview of a Tauri app for Travel Ledger SQLite databases.

## Status

- Source-only (`bundle.active = false`) — not distributed with GitHub Release assets
- Latest workflow: **v4.13.0** — create Trips, add and edit activities, and reorder the selected Day
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
- Create a Trip with its Days, append activities, edit Activity details, and move activities up or down within the selected Day
- Activity is the UI term; the internal data model remains Itinerary
- Forget Database clears the remembered path only — it does **not** delete the file
- Trip editing, cross-Day Activity move/delete, drag-and-drop, Undo, Checklist, and desktop bundle distribution are out of scope
