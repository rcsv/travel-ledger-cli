use anyhow::Result;
use rusqlite::Connection;

use crate::domain::models::Note;

/// CLI mirror of `NoteAction::Add` fields (not a wire DTO).
pub struct NoteAddParams {
    pub trip: Option<i64>,
    pub day: Option<i64>,
    pub itinerary: Option<i64>,
    pub title: Option<String>,
    pub body: String,
}

/// Write `note add` use case result (CLI / future GUI).
pub struct NoteAddServiceResult {
    pub id: i64,
    pub note: Note,
}

/// Adds a note and returns the persisted row, without terminal I/O.
pub fn add_note(conn: &Connection, params: NoteAddParams) -> Result<NoteAddServiceResult> {
    let owner =
        crate::note::resolve_note_owner_for_add(conn, params.trip, params.day, params.itinerary)?;
    let id = crate::note::add_note(conn, owner, params.title.as_deref(), &params.body)?;
    let note = crate::note::get_note(conn, id)?;
    Ok(NoteAddServiceResult { id, note })
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
        let day_id = crate::day::find_day_id_by_trip_and_day_number(conn, trip_id, 1).unwrap();
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
        (trip_id, day_id, itinerary_id)
    }

    #[test]
    fn service_add_trip_note_returns_note() {
        let conn = test_db();
        let (trip_id, _, _) = seed_trip_with_day_and_itinerary(&conn);

        let result = add_note(
            &conn,
            NoteAddParams {
                trip: Some(trip_id),
                day: None,
                itinerary: None,
                title: Some("全体メモ".to_string()),
                body: "旅の方針".to_string(),
            },
        )
        .unwrap();

        assert_eq!(result.id, result.note.id);
        assert_eq!(result.note.owner_type, NoteOwnerType::Trip);
        assert_eq!(result.note.owner_id, trip_id);
        assert_eq!(result.note.title.as_deref(), Some("全体メモ"));
        assert_eq!(result.note.body, "旅の方針");
    }

    #[test]
    fn service_add_day_note() {
        let conn = test_db();
        let (trip_id, day_id, _) = seed_trip_with_day_and_itinerary(&conn);

        let result = add_note(
            &conn,
            NoteAddParams {
                trip: Some(trip_id),
                day: Some(1),
                itinerary: None,
                title: None,
                body: "day note".to_string(),
            },
        )
        .unwrap();

        assert_eq!(result.note.owner_type, NoteOwnerType::Day);
        assert_eq!(result.note.owner_id, day_id);
    }

    #[test]
    fn service_add_itinerary_note() {
        let conn = test_db();
        let (_, _, itinerary_id) = seed_trip_with_day_and_itinerary(&conn);

        let result = add_note(
            &conn,
            NoteAddParams {
                trip: None,
                day: None,
                itinerary: Some(itinerary_id),
                title: Some("駐車場".to_string()),
                body: "北側P".to_string(),
            },
        )
        .unwrap();

        assert_eq!(result.note.owner_type, NoteOwnerType::Itinerary);
        assert_eq!(result.note.owner_id, itinerary_id);
    }

    #[test]
    fn service_add_rejects_empty_body() {
        let conn = test_db();
        let (trip_id, _, _) = seed_trip_with_day_and_itinerary(&conn);

        let err = add_note(
            &conn,
            NoteAddParams {
                trip: Some(trip_id),
                day: None,
                itinerary: None,
                title: None,
                body: String::new(),
            },
        )
        .err()
        .expect("expected error");
        assert!(err.to_string().contains("body は必須"));
    }

    #[test]
    fn service_add_rejects_conflicting_owner_flags() {
        let conn = test_db();
        let (trip_id, _, itinerary_id) = seed_trip_with_day_and_itinerary(&conn);

        let err = add_note(
            &conn,
            NoteAddParams {
                trip: Some(trip_id),
                day: None,
                itinerary: Some(itinerary_id),
                title: None,
                body: "x".to_string(),
            },
        )
        .err()
        .expect("expected error");
        assert!(err.to_string().contains("owner は"));
    }

    #[test]
    fn service_add_matches_show_after_insert() {
        let conn = test_db();
        let (trip_id, _, _) = seed_trip_with_day_and_itinerary(&conn);

        let add_result = add_note(
            &conn,
            NoteAddParams {
                trip: Some(trip_id),
                day: None,
                itinerary: None,
                title: None,
                body: "show me".to_string(),
            },
        )
        .unwrap();

        let show_result = crate::services::note_show::show_note(&conn, add_result.id).unwrap();
        assert_eq!(add_result.note, show_result.note);
    }
}
