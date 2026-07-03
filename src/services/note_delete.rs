use anyhow::Result;
use rusqlite::Connection;

/// Write `note delete` use case result — pre-delete snapshot (no enrich).
pub struct NoteDeleteServiceResult {
    pub id: i64,
    pub title: Option<String>,
}

/// CLI mirror of `NoteAction::Delete` fields (not a wire DTO).
pub struct NoteDeleteParams {
    pub id: i64,
}

/// Deletes a note and returns a pre-delete snapshot, without terminal I/O.
pub fn delete_note(conn: &Connection, params: NoteDeleteParams) -> Result<NoteDeleteServiceResult> {
    let note = crate::note::get_note(conn, params.id)?;
    let result = NoteDeleteServiceResult {
        id: note.id,
        title: note.title.clone(),
    };
    crate::note::delete_note(conn, params.id)?;
    Ok(result)
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
            Some("delete me"),
            "body",
        )
        .unwrap()
    }

    #[test]
    fn service_delete_returns_snapshot() {
        let conn = test_db();
        let id = seed_trip_note(&conn);

        let result = delete_note(&conn, NoteDeleteParams { id }).unwrap();
        assert_eq!(result.id, id);
        assert_eq!(result.title.as_deref(), Some("delete me"));

        let err = crate::note::get_note(&conn, id)
            .err()
            .expect("expected error");
        assert_eq!(err.to_string(), format!("Note not found: {id}"));
    }

    #[test]
    fn service_delete_not_found() {
        let conn = test_db();
        let err = delete_note(&conn, NoteDeleteParams { id: 9999 })
            .err()
            .expect("expected error");
        assert_eq!(err.to_string(), "Note not found: 9999");
    }
}
