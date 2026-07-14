# Travel Ledger Desktop

Developer preview of a **read-only** Tauri shell for Travel Ledger SQLite databases.

## Status

- Source-only (`bundle.active = false`) — not distributed with GitHub Release assets
- Latest polish: **v4.10.2** — Trip list / detail / timeline readability
- Persistence: last successfully opened DB path only (v4.10.1)

## Run from source

```sh
cd desktop
npm install
npm run tauri dev
```

Or: `tools/desktop/dev.sh`

## Notes

- Open an existing `.db` / `.sqlite` / `.sqlite3` Travel Ledger database
- Forget Database clears the remembered path only — it does **not** delete the file
- Write UI, Checklist, and itinerary completion are out of scope
