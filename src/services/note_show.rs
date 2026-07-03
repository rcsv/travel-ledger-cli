use anyhow::Result;
use rusqlite::Connection;

use crate::domain::models::Note;

/// Read-only `note show` use case result (CLI / future GUI).
pub struct NoteShowServiceResult {
    pub note: Note,
}

/// Loads a note without terminal I/O.
pub fn show_note(conn: &Connection, id: i64) -> Result<NoteShowServiceResult> {
    let note = crate::note::get_note(conn, id)?;
    Ok(NoteShowServiceResult { note })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::models::NoteOwnerType;
    use crate::storage::db::open_db_at;
    use rusqlite::Connection;

    fn test_db() -> Connection {
        open_db_at(":memory:").expect("インメモリ DB の作成に失敗")
    }

    fn seed_trip_with_day_and_itinerary(conn: &Connection) -> (i64, i64, i64) {
        let trip_id =
            crate::trip::add_trip(conn, "Note Trip", "2026-06-01", "2026-06-02", None).unwrap();
        let day = crate::day::find_day_id_by_trip_and_day_number(conn, trip_id, 1).unwrap();
        let itinerary_id = crate::itinerary::add_itinerary_item(
            conn,
            trip_id,
            1,
            "首里城",
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .unwrap();
        (trip_id, day, itinerary_id)
    }

    #[test]
    fn service_returns_existing_note() {
        let conn = test_db();
        let (trip_id, _, _) = seed_trip_with_day_and_itinerary(&conn);
        let id = crate::note::add_note(
            &conn,
            crate::note::resolve_note_owner_for_list(&conn, Some(trip_id), None, None).unwrap(),
            Some("title"),
            "show body",
        )
        .unwrap();

        let result = show_note(&conn, id).unwrap();
        assert_eq!(result.note.id, id);
        assert_eq!(result.note.title.as_deref(), Some("title"));
        assert_eq!(result.note.body, "show body");
    }

    #[test]
    fn service_preserves_owner_fields() {
        let conn = test_db();
        let (_, day_id, _) = seed_trip_with_day_and_itinerary(&conn);
        let id = crate::note::add_note(
            &conn,
            crate::note::ResolvedNoteOwner::Day(day_id),
            None,
            "day note body",
        )
        .unwrap();

        let result = show_note(&conn, id).unwrap();
        assert_eq!(result.note.owner_type, NoteOwnerType::Day);
        assert_eq!(result.note.owner_id, day_id);
    }

    #[test]
    fn service_preserves_not_found_error_message() {
        let conn = test_db();
        let err = show_note(&conn, 9999).err().expect("expected error");
        assert_eq!(err.to_string(), "Note not found: 9999");
    }
}
