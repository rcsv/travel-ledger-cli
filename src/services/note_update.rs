use anyhow::Result;
use rusqlite::Connection;

use crate::domain::models::Note;

/// CLI mirror of `NoteAction::Update` fields (not a wire DTO).
pub struct NoteUpdateParams {
    pub id: i64,
    pub title: Option<String>,
    pub body: Option<String>,
}

/// Write `note update` use case result (CLI / future GUI).
pub struct NoteUpdateServiceResult {
    pub note: Note,
}

/// Updates a note and returns the persisted row, without terminal I/O.
pub fn update_note(conn: &Connection, params: NoteUpdateParams) -> Result<NoteUpdateServiceResult> {
    crate::note::update_note(
        conn,
        params.id,
        params.title.as_deref(),
        params.body.as_deref(),
    )?;
    let note = crate::note::get_note(conn, params.id)?;
    Ok(NoteUpdateServiceResult { note })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::db::open_db_at;
    use rusqlite::Connection;

    fn test_db() -> Connection {
        open_db_at(":memory:").expect("インメモリ DB の作成に失敗")
    }

    fn seed_trip_note(conn: &Connection) -> i64 {
        let trip_id =
            crate::trip::add_trip(conn, "Note Trip", "2026-06-01", "2026-06-02", None).unwrap();
        crate::note::add_note(
            conn,
            crate::note::resolve_note_owner_for_list(&conn, Some(trip_id), None, None).unwrap(),
            Some("title"),
            "before",
        )
        .unwrap()
    }

    #[test]
    fn service_update_returns_note() {
        let conn = test_db();
        let id = seed_trip_note(&conn);

        let result = update_note(
            &conn,
            NoteUpdateParams {
                id,
                title: None,
                body: Some("after".to_string()),
            },
        )
        .unwrap();

        assert_eq!(result.note.id, id);
        assert_eq!(result.note.body, "after");
    }

    #[test]
    fn service_update_not_found() {
        let conn = test_db();
        let err = update_note(
            &conn,
            NoteUpdateParams {
                id: 9999,
                title: None,
                body: Some("x".to_string()),
            },
        )
        .err()
        .expect("expected error");
        assert_eq!(err.to_string(), "Note not found: 9999");
    }

    #[test]
    fn service_update_matches_show_after_update() {
        let conn = test_db();
        let id = seed_trip_note(&conn);

        let update_result = update_note(
            &conn,
            NoteUpdateParams {
                id,
                title: Some("updated".to_string()),
                body: Some("after".to_string()),
            },
        )
        .unwrap();

        let show_result = crate::services::note_show::show_note(&conn, id).unwrap();
        assert_eq!(update_result.note, show_result.note);
    }
}
