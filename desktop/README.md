# Travel Ledger Desktop

Developer preview of a Tauri app for Travel Ledger SQLite databases.

## Status

- Source-only (`bundle.active = false`) — not distributed with GitHub Release assets
- Latest workflow: **v4.11.0** — create Trips and add activities to the selected Day
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
- Create a Trip with its Days, then append activities with a title and optional start time, location, and note
- Activity is the UI term; the internal data model remains Itinerary
- Forget Database clears the remembered path only — it does **not** delete the file
- Trip editing, activity editing, Checklist, and desktop bundle distribution are out of scope
