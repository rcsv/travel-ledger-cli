use anyhow::{Context, Result};
use rusqlite::{params, Connection};
use serde::Serialize;

use crate::models::{parse_note_owner_type, Note, NoteOwnerType};

const NOTE_SELECT_SQL: &str = "
    SELECT id, owner_type, owner_id, title, body, sort_order, created_at, updated_at
    FROM notes";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ResolvedNoteOwner {
    Trip(i64),
    Day(i64),
    Itinerary(i64),
}

impl ResolvedNoteOwner {
    pub(crate) fn owner_type(self) -> NoteOwnerType {
        match self {
            Self::Trip(_) => NoteOwnerType::Trip,
            Self::Day(_) => NoteOwnerType::Day,
            Self::Itinerary(_) => NoteOwnerType::Itinerary,
        }
    }

    pub(crate) fn owner_id(self) -> i64 {
        match self {
            Self::Trip(id) | Self::Day(id) | Self::Itinerary(id) => id,
        }
    }
}

#[derive(Serialize)]
pub(crate) struct NoteListJson {
    pub owner_type: NoteOwnerType,
    pub owner_id: i64,
    pub notes: Vec<Note>,
}

pub(crate) fn resolve_note_owner_for_add(
    conn: &Connection,
    trip: Option<i64>,
    day: Option<i64>,
    itinerary: Option<i64>,
) -> Result<ResolvedNoteOwner> {
    match (trip, day, itinerary) {
        (Some(trip_id), None, None) => {
            crate::trip::get_trip(conn, trip_id)?;
            Ok(ResolvedNoteOwner::Trip(trip_id))
        }
        (Some(trip_id), Some(day_number), None) => {
            let day_id = crate::day::find_day_id_by_trip_and_day_number(conn, trip_id, day_number)?;
            Ok(ResolvedNoteOwner::Day(day_id))
        }
        (None, None, Some(itinerary_id)) => {
            crate::itinerary::get_itinerary_item(conn, itinerary_id)?;
            Ok(ResolvedNoteOwner::Itinerary(itinerary_id))
        }
        (None, Some(_), None) => {
            anyhow::bail!("--day を指定する場合は --trip も必要です");
        }
        (Some(_), None, Some(_)) | (Some(_), Some(_), Some(_)) | (None, Some(_), Some(_)) => {
            anyhow::bail!(
                "owner は --trip、--trip + --day、--itinerary のいずれか1つだけ指定してください"
            );
        }
        (None, None, None) => {
            anyhow::bail!("owner を指定してください (--trip、--trip + --day、または --itinerary)");
        }
    }
}

pub(crate) fn resolve_note_owner_for_list(
    conn: &Connection,
    trip: Option<i64>,
    day: Option<i64>,
    itinerary: Option<i64>,
) -> Result<ResolvedNoteOwner> {
    resolve_note_owner_for_add(conn, trip, day, itinerary)
}

fn validate_body(body: &str) -> Result<()> {
    if body.is_empty() {
        anyhow::bail!("body は必須です");
    }
    Ok(())
}

pub(crate) fn add_note(
    conn: &Connection,
    owner: ResolvedNoteOwner,
    title: Option<&str>,
    body: &str,
) -> Result<i64> {
    validate_body(body)?;
    match owner {
        ResolvedNoteOwner::Trip(trip_id) => {
            crate::trip::get_trip(conn, trip_id)?;
        }
        ResolvedNoteOwner::Day(day_id) => {
            find_day_by_id(conn, day_id)?;
        }
        ResolvedNoteOwner::Itinerary(itinerary_id) => {
            crate::itinerary::get_itinerary_item(conn, itinerary_id)?;
        }
    }

    let owner_type = owner.owner_type();
    let owner_id = owner.owner_id();
    let now = crate::db::now_string();
    conn.execute(
        "INSERT INTO notes
         (owner_type, owner_id, title, body, sort_order, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, 0, ?5, ?6)",
        params![owner_type.as_str(), owner_id, title, body, &now, &now],
    )
    .context("Note の追加に失敗しました")?;
    Ok(conn.last_insert_rowid())
}

pub(crate) fn list_notes_for_owner(
    conn: &Connection,
    owner_type: NoteOwnerType,
    owner_id: i64,
) -> Result<Vec<Note>> {
    let mut stmt = conn
        .prepare(&format!(
            "{NOTE_SELECT_SQL}
             WHERE owner_type = ?1 AND owner_id = ?2
             ORDER BY sort_order ASC, id ASC"
        ))
        .context("Note 一覧取得の準備に失敗しました")?;

    let notes = stmt
        .query_map(params![owner_type.as_str(), owner_id], row_to_note)
        .context("Note 一覧取得に失敗しました")?
        .collect::<std::result::Result<Vec<_>, _>>()
        .context("Note 一覧の読み込みに失敗しました")?;

    Ok(notes)
}

pub(crate) fn get_note(conn: &Connection, id: i64) -> Result<Note> {
    crate::db::map_query_row(
        conn.query_row(
            &format!("{NOTE_SELECT_SQL} WHERE id = ?1"),
            params![id],
            row_to_note,
        ),
        || anyhow::anyhow!("Note not found: {id}"),
    )
}

pub(crate) fn update_note(
    conn: &Connection,
    id: i64,
    title: Option<&str>,
    body: Option<&str>,
) -> Result<()> {
    if title.is_none() && body.is_none() {
        anyhow::bail!("更新する項目を1つ以上指定してください (--title, --body)");
    }

    let mut note = get_note(conn, id)?;
    if let Some(value) = title {
        note.title = Some(value.to_string());
    }
    if let Some(value) = body {
        validate_body(value)?;
        note.body = value.to_string();
    }

    let now = crate::db::now_string();
    conn.execute(
        "UPDATE notes SET title = ?1, body = ?2, updated_at = ?3 WHERE id = ?4",
        params![note.title, note.body, &now, id],
    )
    .context("Note の更新に失敗しました")?;
    Ok(())
}

pub(crate) fn delete_note(conn: &Connection, id: i64) -> Result<()> {
    get_note(conn, id)?;
    conn.execute("DELETE FROM notes WHERE id = ?1", params![id])
        .context("Note の削除に失敗しました")?;
    Ok(())
}

/// Trip 配下のすべての Note を削除する（Trip / Day / Itinerary 由来）。
///
/// FK は張らないため、`trip delete` では親行削除の前に呼ぶ（`days` / `itinerary_items` が
/// 残っている間に subquery で owner_id を解決する）。
pub(crate) fn delete_notes_for_trip(conn: &Connection, trip_id: i64) -> Result<()> {
    conn.execute(
        "DELETE FROM notes
         WHERE (owner_type = 'trip' AND owner_id = ?1)
            OR (owner_type = 'day' AND owner_id IN (
                  SELECT id FROM days WHERE trip_id = ?1
                ))
            OR (owner_type = 'itinerary' AND owner_id IN (
                  SELECT id FROM itinerary_items WHERE trip_id = ?1
                ))",
        params![trip_id],
    )
    .context("Trip 配下 Note の削除に失敗しました")?;
    Ok(())
}

pub(crate) fn delete_notes_for_day(conn: &Connection, day_id: i64) -> Result<()> {
    conn.execute(
        "DELETE FROM notes WHERE owner_type = 'day' AND owner_id = ?1",
        params![day_id],
    )
    .context("Day Note の削除に失敗しました")?;
    Ok(())
}

pub(crate) fn delete_notes_for_itinerary(conn: &Connection, itinerary_id: i64) -> Result<()> {
    conn.execute(
        "DELETE FROM notes WHERE owner_type = 'itinerary' AND owner_id = ?1",
        params![itinerary_id],
    )
    .context("Itinerary Note の削除に失敗しました")?;
    Ok(())
}

#[cfg(test)]
fn count_notes(conn: &Connection) -> Result<i64> {
    Ok(conn.query_row("SELECT COUNT(*) FROM notes", [], |row| row.get(0))?)
}

fn find_day_by_id(conn: &Connection, day_id: i64) -> Result<crate::models::Day> {
    crate::db::map_query_row(
        conn.query_row(
            "SELECT id, trip_id, day_number, title, description, created_at, updated_at
             FROM days WHERE id = ?1",
            params![day_id],
            crate::day::row_to_day,
        ),
        || anyhow::anyhow!("Day not found: {day_id}"),
    )
}

fn row_to_note(row: &rusqlite::Row) -> rusqlite::Result<Note> {
    let owner_type_raw: String = row.get(1)?;
    let owner_type = parse_note_owner_type(&owner_type_raw).map_err(|_| {
        rusqlite::Error::InvalidColumnType(1, owner_type_raw, rusqlite::types::Type::Text)
    })?;
    Ok(Note {
        id: row.get(0)?,
        owner_type,
        owner_id: row.get(2)?,
        title: row.get(3)?,
        body: row.get(4)?,
        sort_order: row.get(5)?,
        created_at: row.get(6)?,
        updated_at: row.get(7)?,
    })
}

pub(crate) fn print_note_list(owner_type: NoteOwnerType, owner_id: i64, notes: &[Note]) {
    let label = match owner_type {
        NoteOwnerType::Trip => format!("Trip {owner_id}"),
        NoteOwnerType::Day => format!("Day {owner_id}"),
        NoteOwnerType::Itinerary => format!("Itinerary {owner_id}"),
    };
    println!("{label} の Note ({} 件):", notes.len());
    if notes.is_empty() {
        return;
    }
    println!();
    println!("{:<6} {:<14} Body (先頭)", "ID", "Title");
    println!("{}", "-".repeat(60));
    for note in notes {
        let title = note.title.as_deref().unwrap_or("(なし)");
        let body_preview = note.body.lines().next().unwrap_or("");
        let body_preview = if body_preview.chars().count() > 30 {
            format!("{}...", body_preview.chars().take(30).collect::<String>())
        } else {
            body_preview.to_string()
        };
        println!("{:<6} {:<14} {body_preview}", note.id, title);
    }
}

pub(crate) fn print_note_detail(note: &Note) {
    println!("ID         : {}", note.id);
    println!("Owner type : {}", note.owner_type.as_str());
    println!("Owner ID   : {}", note.owner_id);
    println!("Title      : {}", note.title.as_deref().unwrap_or("-"));
    println!("Body       : {}", note.body);
    println!("Sort order : {}", note.sort_order);
    println!("作成日時   : {}", note.created_at);
    println!("更新日時   : {}", note.updated_at);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::open_db_at;
    use crate::itinerary::add_itinerary_item;
    use crate::trip::{add_trip, delete_trip, update_trip};

    fn test_db() -> Connection {
        open_db_at(":memory:").expect("インメモリ DB の作成に失敗")
    }

    #[test]
    fn test_add_trip_day_and_itinerary_notes() {
        let conn = test_db();
        let trip_id = add_trip(&conn, "Note Trip", "2026-04-26", "2026-04-29").unwrap();
        let itinerary_id = add_itinerary_item(
            &conn, trip_id, 2, "Museum", None, None, None, None, None, None, None,
        )
        .unwrap();

        let trip_note = add_note(
            &conn,
            ResolvedNoteOwner::Trip(trip_id),
            Some("全体"),
            "trip body",
        )
        .unwrap();
        let day = crate::day::find_day_by_trip_and_day_number(&conn, trip_id, 2).unwrap();
        let day_note = add_note(&conn, ResolvedNoteOwner::Day(day.id), None, "day body").unwrap();
        let itinerary_note = add_note(
            &conn,
            ResolvedNoteOwner::Itinerary(itinerary_id),
            Some("駐車場"),
            "itinerary body",
        )
        .unwrap();

        let trip_notes = list_notes_for_owner(&conn, NoteOwnerType::Trip, trip_id).unwrap();
        assert_eq!(trip_notes.len(), 1);
        assert_eq!(trip_notes[0].id, trip_note);

        let day_notes = list_notes_for_owner(&conn, NoteOwnerType::Day, day.id).unwrap();
        assert_eq!(day_notes.len(), 1);
        assert_eq!(day_notes[0].id, day_note);

        let itinerary_notes =
            list_notes_for_owner(&conn, NoteOwnerType::Itinerary, itinerary_id).unwrap();
        assert_eq!(itinerary_notes.len(), 1);
        assert_eq!(itinerary_notes[0].id, itinerary_note);
    }

    #[test]
    fn test_add_note_rejects_invalid_owner() {
        let conn = test_db();
        assert!(add_note(&conn, ResolvedNoteOwner::Trip(999), None, "body",).is_err());
        let trip_id = add_trip(&conn, "Trip", "2026-04-26", "2026-04-29").unwrap();
        assert!(add_note(&conn, ResolvedNoteOwner::Day(999), None, "body",).is_err());
        assert!(add_note(&conn, ResolvedNoteOwner::Itinerary(999), None, "body",).is_err());
        let day = crate::day::find_day_by_trip_and_day_number(&conn, trip_id, 1).unwrap();
        assert!(add_note(&conn, ResolvedNoteOwner::Day(day.id), None, "").is_err());
    }

    #[test]
    fn test_parse_note_owner_type_rejects_invalid() {
        assert!(parse_note_owner_type("invalid").is_err());
    }

    #[test]
    fn test_get_update_delete_note() {
        let conn = test_db();
        let trip_id = add_trip(&conn, "Trip", "2026-04-26", "2026-04-29").unwrap();
        let id = add_note(
            &conn,
            ResolvedNoteOwner::Trip(trip_id),
            Some("title"),
            "body",
        )
        .unwrap();

        let note = get_note(&conn, id).unwrap();
        assert_eq!(note.title.as_deref(), Some("title"));
        assert_eq!(note.body, "body");

        update_note(&conn, id, Some("new title"), Some("new body")).unwrap();
        let updated = get_note(&conn, id).unwrap();
        assert_eq!(updated.title.as_deref(), Some("new title"));
        assert_eq!(updated.body, "new body");

        delete_note(&conn, id).unwrap();
        assert!(get_note(&conn, id).is_err());
    }

    #[test]
    fn test_delete_notes_for_trip_cascade() {
        let conn = test_db();
        let trip_id = add_trip(&conn, "Trip", "2026-04-26", "2026-04-29").unwrap();
        let itinerary_id = add_itinerary_item(
            &conn, trip_id, 2, "Plan", None, None, None, None, None, None, None,
        )
        .unwrap();
        let day = crate::day::find_day_by_trip_and_day_number(&conn, trip_id, 2).unwrap();
        add_note(&conn, ResolvedNoteOwner::Trip(trip_id), None, "trip").unwrap();
        add_note(&conn, ResolvedNoteOwner::Day(day.id), None, "day").unwrap();
        add_note(
            &conn,
            ResolvedNoteOwner::Itinerary(itinerary_id),
            None,
            "itinerary",
        )
        .unwrap();
        assert_eq!(count_notes(&conn).unwrap(), 3);

        delete_notes_for_trip(&conn, trip_id).unwrap();
        assert_eq!(count_notes(&conn).unwrap(), 0);
    }

    #[test]
    fn test_delete_notes_for_day_on_trip_shrink() {
        let conn = test_db();
        let trip_id = add_trip(&conn, "Trip", "2026-04-26", "2026-04-29").unwrap();
        let day = crate::day::find_day_by_trip_and_day_number(&conn, trip_id, 4).unwrap();
        add_note(&conn, ResolvedNoteOwner::Day(day.id), None, "day4").unwrap();
        delete_notes_for_day(&conn, day.id).unwrap();
        assert_eq!(count_notes(&conn).unwrap(), 0);
    }

    #[test]
    fn test_delete_notes_for_itinerary() {
        let conn = test_db();
        let trip_id = add_trip(&conn, "Trip", "2026-04-26", "2026-04-29").unwrap();
        let itinerary_id = add_itinerary_item(
            &conn, trip_id, 1, "Plan", None, None, None, None, None, None, None,
        )
        .unwrap();
        add_note(
            &conn,
            ResolvedNoteOwner::Itinerary(itinerary_id),
            None,
            "memo",
        )
        .unwrap();
        delete_notes_for_itinerary(&conn, itinerary_id).unwrap();
        assert_eq!(count_notes(&conn).unwrap(), 0);
    }

    #[test]
    fn test_trip_delete_removes_all_owner_notes() {
        let conn = test_db();
        let trip_id = add_trip(&conn, "Trip", "2026-04-26", "2026-04-29").unwrap();
        let itinerary_id = add_itinerary_item(
            &conn, trip_id, 2, "Plan", None, None, None, None, None, None, None,
        )
        .unwrap();
        let day = crate::day::find_day_by_trip_and_day_number(&conn, trip_id, 2).unwrap();
        add_note(&conn, ResolvedNoteOwner::Trip(trip_id), None, "trip").unwrap();
        add_note(&conn, ResolvedNoteOwner::Day(day.id), None, "day").unwrap();
        add_note(
            &conn,
            ResolvedNoteOwner::Itinerary(itinerary_id),
            None,
            "itinerary",
        )
        .unwrap();
        assert_eq!(count_notes(&conn).unwrap(), 3);

        delete_trip(&conn, trip_id).unwrap();
        assert_eq!(count_notes(&conn).unwrap(), 0);
    }

    #[test]
    fn test_trip_update_shrink_deletes_only_removed_day_notes() {
        let conn = test_db();
        let trip_id = add_trip(&conn, "Trip", "2026-04-26", "2026-04-29").unwrap();
        let day1 = crate::day::find_day_by_trip_and_day_number(&conn, trip_id, 1).unwrap();
        let day4 = crate::day::find_day_by_trip_and_day_number(&conn, trip_id, 4).unwrap();
        add_note(&conn, ResolvedNoteOwner::Day(day1.id), None, "keep").unwrap();
        add_note(&conn, ResolvedNoteOwner::Day(day4.id), None, "remove").unwrap();

        update_trip(&conn, trip_id, None, None, Some("2026-04-28")).unwrap();

        assert_eq!(count_notes(&conn).unwrap(), 1);
        let remaining = list_notes_for_owner(&conn, NoteOwnerType::Day, day1.id).unwrap();
        assert_eq!(remaining[0].body, "keep");
    }

    #[test]
    fn test_trip_update_shrink_rejects_day_with_itinerary_before_deleting_notes() {
        let conn = test_db();
        let trip_id = add_trip(&conn, "Trip", "2026-04-26", "2026-04-29").unwrap();
        let itinerary_id = add_itinerary_item(
            &conn,
            trip_id,
            4,
            "Late Plan",
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .unwrap();
        let day4 = crate::day::find_day_by_trip_and_day_number(&conn, trip_id, 4).unwrap();
        add_note(&conn, ResolvedNoteOwner::Day(day4.id), None, "day4").unwrap();
        add_note(
            &conn,
            ResolvedNoteOwner::Itinerary(itinerary_id),
            None,
            "itinerary on day4",
        )
        .unwrap();

        assert!(update_trip(&conn, trip_id, None, None, Some("2026-04-28")).is_err());
        assert_eq!(count_notes(&conn).unwrap(), 2);
    }

    #[test]
    fn test_update_trip_shrink_rolls_back_when_middle_day_blocks_deletion() {
        let conn = test_db();
        let trip_id = add_trip(&conn, "Trip", "2026-04-26", "2026-04-29").unwrap();
        let day4 = crate::day::find_day_by_trip_and_day_number(&conn, trip_id, 4).unwrap();
        add_note(&conn, ResolvedNoteOwner::Day(day4.id), None, "day4 note").unwrap();
        add_itinerary_item(
            &conn,
            trip_id,
            3,
            "Busy Day 3",
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .unwrap();
        let trip_before = crate::trip::get_trip(&conn, trip_id).unwrap();

        assert!(update_trip(&conn, trip_id, None, None, Some("2026-04-27")).is_err());

        let trip_after = crate::trip::get_trip(&conn, trip_id).unwrap();
        assert_eq!(trip_before.end_date, trip_after.end_date);
        assert_eq!(
            list_notes_for_owner(&conn, NoteOwnerType::Day, day4.id)
                .unwrap()
                .len(),
            1
        );
        assert_eq!(crate::day::list_days(&conn, trip_id).unwrap().len(), 4);
    }

    #[test]
    fn test_itinerary_delete_leaves_trip_and_day_notes() {
        let conn = test_db();
        let trip_id = add_trip(&conn, "Trip", "2026-04-26", "2026-04-29").unwrap();
        let day = crate::day::find_day_by_trip_and_day_number(&conn, trip_id, 1).unwrap();
        let keep_id = add_itinerary_item(
            &conn, trip_id, 1, "Keep", None, None, None, None, None, None, None,
        )
        .unwrap();
        let delete_id = add_itinerary_item(
            &conn, trip_id, 2, "Remove", None, None, None, None, None, None, None,
        )
        .unwrap();
        add_note(&conn, ResolvedNoteOwner::Trip(trip_id), None, "trip").unwrap();
        add_note(&conn, ResolvedNoteOwner::Day(day.id), None, "day").unwrap();
        add_note(
            &conn,
            ResolvedNoteOwner::Itinerary(keep_id),
            None,
            "keep itinerary",
        )
        .unwrap();
        add_note(
            &conn,
            ResolvedNoteOwner::Itinerary(delete_id),
            None,
            "delete itinerary",
        )
        .unwrap();

        crate::itinerary::delete_itinerary_item(&conn, delete_id).unwrap();

        assert_eq!(count_notes(&conn).unwrap(), 3);
        assert_eq!(
            list_notes_for_owner(&conn, NoteOwnerType::Trip, trip_id)
                .unwrap()
                .len(),
            1
        );
        assert_eq!(
            list_notes_for_owner(&conn, NoteOwnerType::Day, day.id)
                .unwrap()
                .len(),
            1
        );
        let kept_itinerary_notes =
            list_notes_for_owner(&conn, NoteOwnerType::Itinerary, keep_id).unwrap();
        assert_eq!(kept_itinerary_notes.len(), 1);
        assert_eq!(kept_itinerary_notes[0].body, "keep itinerary");
        assert!(
            list_notes_for_owner(&conn, NoteOwnerType::Itinerary, delete_id)
                .unwrap()
                .is_empty()
        );
    }

    #[test]
    fn test_itinerary_delete_removes_only_target_itinerary_notes() {
        let conn = test_db();
        let trip_id = add_trip(&conn, "Trip", "2026-04-26", "2026-04-29").unwrap();
        let itinerary_id = add_itinerary_item(
            &conn, trip_id, 1, "Plan", None, None, None, None, None, None, None,
        )
        .unwrap();
        add_note(
            &conn,
            ResolvedNoteOwner::Itinerary(itinerary_id),
            None,
            "memo",
        )
        .unwrap();
        crate::itinerary::delete_itinerary_item(&conn, itinerary_id).unwrap();
        assert_eq!(count_notes(&conn).unwrap(), 0);
    }

    /// day swap では Note を更新しない。
    /// Day Note は days.id に残り、Itinerary Note は itinerary_items.id に残る（予定だけが移動）。
    #[test]
    fn test_day_swap_leaves_day_notes_on_days_id_and_itinerary_notes_on_itinerary_id() {
        let conn = test_db();
        let trip_id = add_trip(&conn, "Trip", "2026-04-26", "2026-04-29").unwrap();
        let day2 = crate::day::find_day_by_trip_and_day_number(&conn, trip_id, 2).unwrap();
        let day3 = crate::day::find_day_by_trip_and_day_number(&conn, trip_id, 3).unwrap();
        add_note(&conn, ResolvedNoteOwner::Day(day2.id), None, "day2 note").unwrap();
        let itinerary_id = add_itinerary_item(
            &conn, trip_id, 2, "Plan", None, None, None, None, None, None, None,
        )
        .unwrap();
        add_note(
            &conn,
            ResolvedNoteOwner::Itinerary(itinerary_id),
            None,
            "itinerary note",
        )
        .unwrap();

        crate::day::swap_day_itineraries(&conn, trip_id, 2, 3).unwrap();

        let day2_notes = list_notes_for_owner(&conn, NoteOwnerType::Day, day2.id).unwrap();
        let day3_notes = list_notes_for_owner(&conn, NoteOwnerType::Day, day3.id).unwrap();
        assert_eq!(day2_notes.len(), 1);
        assert_eq!(day2_notes[0].body, "day2 note");
        assert_eq!(day3_notes.len(), 0);

        let item = crate::itinerary::get_itinerary_item(&conn, itinerary_id).unwrap();
        assert_eq!(item.day, 3);
        let itinerary_notes =
            list_notes_for_owner(&conn, NoteOwnerType::Itinerary, itinerary_id).unwrap();
        assert_eq!(itinerary_notes.len(), 1);
        assert_eq!(itinerary_notes[0].body, "itinerary note");
    }

    #[test]
    fn test_resolve_note_owner_for_add_day_requires_trip() {
        let conn = test_db();
        assert!(resolve_note_owner_for_add(&conn, None, Some(2), None).is_err());
    }
}
