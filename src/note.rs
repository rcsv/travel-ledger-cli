use anyhow::{Context, Result};
use rusqlite::{params, Connection};
use serde::Serialize;
use std::collections::HashMap;

use crate::models::{
    parse_note_owner_type, ExportNote, ItineraryItem, ItineraryNoteKey, Note, NoteOwnerType,
};

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

/// 2 Day 間で Day-level Note の owner_id を入れ替える（`day swap` 用）
pub(crate) fn swap_day_note_owners(
    conn: &Connection,
    day_a_id: i64,
    day_b_id: i64,
    now: &str,
) -> Result<usize> {
    let sentinel = -day_a_id;
    let staged = conn
        .execute(
            "UPDATE notes SET owner_id = ?1, updated_at = ?2
         WHERE owner_type = 'day' AND owner_id = ?3",
            params![sentinel, now, day_a_id],
        )
        .context("Day Note owner の退避に失敗しました")?;
    let moved_b_to_a = conn
        .execute(
            "UPDATE notes SET owner_id = ?1, updated_at = ?2
             WHERE owner_type = 'day' AND owner_id = ?3",
            params![day_a_id, now, day_b_id],
        )
        .context("Day B Note owner の移動に失敗しました")?;
    let moved_a_to_b = conn
        .execute(
            "UPDATE notes SET owner_id = ?1, updated_at = ?2
             WHERE owner_type = 'day' AND owner_id = ?3",
            params![day_b_id, now, sentinel],
        )
        .context("Day A Note owner の移動に失敗しました")?;
    Ok(staged + moved_b_to_a + moved_a_to_b)
}

pub(crate) fn delete_notes_for_itinerary(conn: &Connection, itinerary_id: i64) -> Result<()> {
    conn.execute(
        "DELETE FROM notes WHERE owner_type = 'itinerary' AND owner_id = ?1",
        params![itinerary_id],
    )
    .context("Itinerary Note の削除に失敗しました")?;
    Ok(())
}

/// Trip 配下のすべての Note を取得する（export 用）
pub(crate) fn list_all_notes_for_trip(conn: &Connection, trip_id: i64) -> Result<Vec<Note>> {
    let mut stmt = conn
        .prepare(&format!(
            "{NOTE_SELECT_SQL}
             WHERE (owner_type = 'trip' AND owner_id = ?1)
                OR (owner_type = 'day' AND owner_id IN (
                      SELECT id FROM days WHERE trip_id = ?1
                    ))
                OR (owner_type = 'itinerary' AND owner_id IN (
                      SELECT id FROM itinerary_items WHERE trip_id = ?1
                    ))
             ORDER BY owner_type, owner_id, sort_order ASC, id ASC"
        ))
        .context("Trip 配下 Note 一覧取得の準備に失敗しました")?;

    let notes = stmt
        .query_map(params![trip_id], row_to_note)
        .context("Trip 配下 Note 一覧取得に失敗しました")?
        .collect::<std::result::Result<Vec<_>, _>>()
        .context("Trip 配下 Note 一覧の読み込みに失敗しました")?;

    Ok(notes)
}

/// Trip 配下の Note を export 形式に変換する
pub(crate) fn build_export_notes(conn: &Connection, trip_id: i64) -> Result<Vec<ExportNote>> {
    let notes = list_all_notes_for_trip(conn, trip_id)?;
    let days = crate::day::list_days(conn, trip_id)?;
    let day_number_by_id: HashMap<i64, i64> =
        days.iter().map(|day| (day.id, day.day_number)).collect();
    let itineraries = crate::itinerary::list_itinerary_items(conn, trip_id)?;
    let itinerary_by_id: HashMap<i64, &ItineraryItem> =
        itineraries.iter().map(|item| (item.id, item)).collect();

    notes
        .into_iter()
        .map(|note| match note.owner_type {
            NoteOwnerType::Trip => Ok(ExportNote::Trip {
                title: note.title,
                body: note.body,
            }),
            NoteOwnerType::Day => {
                let day_number =
                    day_number_by_id
                        .get(&note.owner_id)
                        .copied()
                        .with_context(|| {
                            format!("Day Note の owner_id {} が見つかりません", note.owner_id)
                        })?;
                Ok(ExportNote::Day {
                    day_number,
                    title: note.title,
                    body: note.body,
                })
            }
            NoteOwnerType::Itinerary => {
                let item = itinerary_by_id.get(&note.owner_id).with_context(|| {
                    format!(
                        "Itinerary Note の owner_id {} が見つかりません",
                        note.owner_id
                    )
                })?;
                Ok(ExportNote::Itinerary {
                    itinerary_key: ItineraryNoteKey {
                        day_number: item.day,
                        sort_order: item.sort_order,
                        start_time: item.start_time.clone(),
                        title: item.title.clone(),
                    },
                    title: note.title,
                    body: note.body,
                })
            }
        })
        .collect()
}

/// export 内 itinerary_items から itinerary_key を解決する
pub(crate) fn resolve_itinerary_id_from_export_items(
    items: &[ItineraryItem],
    key: &ItineraryNoteKey,
) -> Result<i64> {
    let by_day_sort: Vec<&ItineraryItem> = items
        .iter()
        .filter(|item| item.day == key.day_number && item.sort_order == key.sort_order)
        .collect();

    match by_day_sort.len() {
        1 => Ok(by_day_sort[0].id),
        0 => resolve_itinerary_id_fallback(items, key),
        _ => resolve_itinerary_id_among_candidates(&by_day_sort, key),
    }
}

fn resolve_itinerary_id_among_candidates(
    candidates: &[&ItineraryItem],
    key: &ItineraryNoteKey,
) -> Result<i64> {
    let by_title_time: Vec<&ItineraryItem> = candidates
        .iter()
        .copied()
        .filter(|item| item.title == key.title && item.start_time == key.start_time)
        .collect();
    if by_title_time.len() == 1 {
        return Ok(by_title_time[0].id);
    }

    let by_title: Vec<&ItineraryItem> = candidates
        .iter()
        .copied()
        .filter(|item| item.title == key.title)
        .collect();
    match by_title.len() {
        1 => Ok(by_title[0].id),
        0 => anyhow::bail!(
            "itinerary_key (day={}, sort_order={}, title={}) を itinerary_items から解決できません",
            key.day_number,
            key.sort_order,
            key.title
        ),
        _ => anyhow::bail!(
            "itinerary_key が複数の itinerary_items に一致します（day={}, sort_order={}, title={}）",
            key.day_number,
            key.sort_order,
            key.title
        ),
    }
}

fn resolve_itinerary_id_fallback(items: &[ItineraryItem], key: &ItineraryNoteKey) -> Result<i64> {
    if let Some(id) = try_resolve_itinerary_by(items, |item| {
        item.day == key.day_number && item.start_time == key.start_time && item.title == key.title
    })? {
        return Ok(id);
    }
    if let Some(id) = try_resolve_itinerary_by(items, |item| {
        item.day == key.day_number && item.title == key.title
    })? {
        return Ok(id);
    }

    anyhow::bail!(
        "itinerary_key (day={}, sort_order={}, title={}) を itinerary_items から解決できません",
        key.day_number,
        key.sort_order,
        key.title
    )
}

fn try_resolve_itinerary_by<F>(items: &[ItineraryItem], predicate: F) -> Result<Option<i64>>
where
    F: Fn(&ItineraryItem) -> bool,
{
    let matches: Vec<_> = items.iter().filter(|item| predicate(item)).collect();
    match matches.len() {
        0 => Ok(None),
        1 => Ok(Some(matches[0].id)),
        _ => anyhow::bail!(
            "itinerary_key が複数の itinerary_items に一致します（day={}, sort_order={}, title={}）",
            matches[0].day,
            matches[0].sort_order,
            matches[0].title
        ),
    }
}

fn resolve_export_note_owner(
    conn: &Connection,
    trip_id: i64,
    export_note: &ExportNote,
    day_count: i64,
    itinerary_items: &[ItineraryItem],
) -> Result<ResolvedNoteOwner> {
    match export_note {
        ExportNote::Trip { body, .. } => {
            validate_body(body)?;
            Ok(ResolvedNoteOwner::Trip(trip_id))
        }
        ExportNote::Day {
            day_number, body, ..
        } => {
            validate_body(body)?;
            if *day_number < 1 || *day_number > day_count {
                anyhow::bail!(
                    "day_number ({day_number}) は旅行期間 (1..={day_count}) の範囲外です"
                );
            }
            let day_id =
                crate::day::find_day_id_by_trip_and_day_number(conn, trip_id, *day_number)?;
            Ok(ResolvedNoteOwner::Day(day_id))
        }
        ExportNote::Itinerary {
            itinerary_key,
            body,
            ..
        } => {
            validate_body(body)?;
            let itinerary_id =
                resolve_itinerary_id_from_export_items(itinerary_items, itinerary_key)?;
            Ok(ResolvedNoteOwner::Itinerary(itinerary_id))
        }
    }
}

/// export JSON の Note を import する
pub(crate) fn import_export_notes(
    conn: &Connection,
    trip_id: i64,
    notes: &[ExportNote],
    day_count: i64,
) -> Result<usize> {
    let itinerary_items = crate::itinerary::list_itinerary_items(conn, trip_id)?;
    let mut count = 0;
    for (index, export_note) in notes.iter().enumerate() {
        let owner =
            resolve_export_note_owner(conn, trip_id, export_note, day_count, &itinerary_items)
                .with_context(|| format!("notes[{index}]"))?;
        let (title, body) = match export_note {
            ExportNote::Trip { title, body } => (title.as_deref(), body.as_str()),
            ExportNote::Day { title, body, .. } => (title.as_deref(), body.as_str()),
            ExportNote::Itinerary { title, body, .. } => (title.as_deref(), body.as_str()),
        };
        add_note(conn, owner, title, body)?;
        count += 1;
    }
    Ok(count)
}

/// export JSON の Note を検証する（エラー文言の一覧）
pub(crate) fn collect_export_note_validation_errors(
    export: &crate::models::TripExport,
) -> Vec<String> {
    let day_count = match (
        export.trip.start_date.as_deref(),
        export.trip.end_date.as_deref(),
    ) {
        (Some(start), Some(end)) => match crate::day::validate_trip_date_range(start, end) {
            Ok(count) => count,
            Err(_) => return Vec::new(),
        },
        _ => return Vec::new(),
    };

    let mut errors = Vec::new();

    for (index, note) in export.notes().iter().enumerate() {
        let prefix = format!("notes[{index}]");
        match note {
            ExportNote::Trip { body, .. } => {
                if body.is_empty() {
                    errors.push(format!("{prefix}: body は必須です"));
                }
            }
            ExportNote::Day {
                day_number, body, ..
            } => {
                if body.is_empty() {
                    errors.push(format!("{prefix}: body は必須です"));
                }
                if *day_number < 1 || *day_number > day_count {
                    errors.push(format!(
                        "{prefix}: day_number ({day_number}) は旅行期間 (1..={day_count}) の範囲外です"
                    ));
                }
            }
            ExportNote::Itinerary {
                itinerary_key,
                body,
                ..
            } => {
                if body.is_empty() {
                    errors.push(format!("{prefix}: body は必須です"));
                }
                if itinerary_key.title.trim().is_empty() {
                    errors.push(format!("{prefix}: itinerary_key.title は必須です"));
                }
                if let Err(error) =
                    resolve_itinerary_id_from_export_items(&export.itinerary_items, itinerary_key)
                {
                    errors.push(format!("{prefix}: {error}"));
                }
            }
        }
    }
    errors
}

#[cfg(test)]
fn count_notes(conn: &Connection) -> Result<i64> {
    Ok(conn.query_row("SELECT COUNT(*) FROM notes", [], |row| row.get(0))?)
}

fn find_day_by_id(conn: &Connection, day_id: i64) -> Result<crate::models::Day> {
    crate::db::map_query_row(
        conn.query_row(
            "SELECT id, trip_id, day_number, title, summary, created_at, updated_at
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
        let trip_id = add_trip(&conn, "Note Trip", "2026-04-26", "2026-04-29", None).unwrap();
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
        let trip_id = add_trip(&conn, "Trip", "2026-04-26", "2026-04-29", None).unwrap();
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
        let trip_id = add_trip(&conn, "Trip", "2026-04-26", "2026-04-29", None).unwrap();
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
        let trip_id = add_trip(&conn, "Trip", "2026-04-26", "2026-04-29", None).unwrap();
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
        let trip_id = add_trip(&conn, "Trip", "2026-04-26", "2026-04-29", None).unwrap();
        let day = crate::day::find_day_by_trip_and_day_number(&conn, trip_id, 4).unwrap();
        add_note(&conn, ResolvedNoteOwner::Day(day.id), None, "day4").unwrap();
        delete_notes_for_day(&conn, day.id).unwrap();
        assert_eq!(count_notes(&conn).unwrap(), 0);
    }

    #[test]
    fn test_delete_notes_for_itinerary() {
        let conn = test_db();
        let trip_id = add_trip(&conn, "Trip", "2026-04-26", "2026-04-29", None).unwrap();
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
        let trip_id = add_trip(&conn, "Trip", "2026-04-26", "2026-04-29", None).unwrap();
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
        let trip_id = add_trip(&conn, "Trip", "2026-04-26", "2026-04-29", None).unwrap();
        let day1 = crate::day::find_day_by_trip_and_day_number(&conn, trip_id, 1).unwrap();
        let day4 = crate::day::find_day_by_trip_and_day_number(&conn, trip_id, 4).unwrap();
        add_note(&conn, ResolvedNoteOwner::Day(day1.id), None, "keep").unwrap();
        add_note(&conn, ResolvedNoteOwner::Day(day4.id), None, "remove").unwrap();

        update_trip(&conn, trip_id, None, None, Some("2026-04-28"), None, false).unwrap();

        assert_eq!(count_notes(&conn).unwrap(), 1);
        let remaining = list_notes_for_owner(&conn, NoteOwnerType::Day, day1.id).unwrap();
        assert_eq!(remaining[0].body, "keep");
    }

    #[test]
    fn test_trip_update_shrink_rejects_day_with_itinerary_before_deleting_notes() {
        let conn = test_db();
        let trip_id = add_trip(&conn, "Trip", "2026-04-26", "2026-04-29", None).unwrap();
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

        assert!(update_trip(&conn, trip_id, None, None, Some("2026-04-28"), None, false).is_err());
        assert_eq!(count_notes(&conn).unwrap(), 2);
    }

    #[test]
    fn test_update_trip_shrink_rolls_back_when_middle_day_blocks_deletion() {
        let conn = test_db();
        let trip_id = add_trip(&conn, "Trip", "2026-04-26", "2026-04-29", None).unwrap();
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

        assert!(update_trip(&conn, trip_id, None, None, Some("2026-04-27"), None, false).is_err());

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
        let trip_id = add_trip(&conn, "Trip", "2026-04-26", "2026-04-29", None).unwrap();
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
        let trip_id = add_trip(&conn, "Trip", "2026-04-26", "2026-04-29", None).unwrap();
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

    /// day swap では Day-level Note の owner を入れ替え、Itinerary Note は Itinerary と一緒に移動する。
    #[test]
    fn test_day_swap_exchanges_day_notes_and_moves_itinerary_notes() {
        let conn = test_db();
        let trip_id = add_trip(&conn, "Trip", "2026-04-26", "2026-04-29", None).unwrap();
        let day2 = crate::day::find_day_by_trip_and_day_number(&conn, trip_id, 2).unwrap();
        let day3 = crate::day::find_day_by_trip_and_day_number(&conn, trip_id, 3).unwrap();
        add_note(&conn, ResolvedNoteOwner::Day(day2.id), None, "day2 note").unwrap();
        add_note(&conn, ResolvedNoteOwner::Day(day3.id), None, "day3 note").unwrap();
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

        crate::day::swap_day_plan_payload(&conn, trip_id, 2, 3).unwrap();

        let day2_notes = list_notes_for_owner(&conn, NoteOwnerType::Day, day2.id).unwrap();
        let day3_notes = list_notes_for_owner(&conn, NoteOwnerType::Day, day3.id).unwrap();
        assert_eq!(day2_notes.len(), 1);
        assert_eq!(day2_notes[0].body, "day3 note");
        assert_eq!(day3_notes.len(), 1);
        assert_eq!(day3_notes[0].body, "day2 note");

        let item = crate::itinerary::get_itinerary_item(&conn, itinerary_id).unwrap();
        assert_eq!(item.day, 3);
        let itinerary_notes =
            list_notes_for_owner(&conn, NoteOwnerType::Itinerary, itinerary_id).unwrap();
        assert_eq!(itinerary_notes.len(), 1);
        assert_eq!(itinerary_notes[0].body, "itinerary note");
    }

    #[test]
    fn test_day_swap_leaves_trip_notes_unchanged() {
        let conn = test_db();
        let trip_id = add_trip(&conn, "Trip", "2026-04-26", "2026-04-29", None).unwrap();
        add_note(&conn, ResolvedNoteOwner::Trip(trip_id), None, "trip note").unwrap();
        add_itinerary_item(
            &conn, trip_id, 2, "A", None, None, None, None, None, None, None,
        )
        .unwrap();
        add_itinerary_item(
            &conn, trip_id, 3, "B", None, None, None, None, None, None, None,
        )
        .unwrap();

        crate::day::swap_day_plan_payload(&conn, trip_id, 2, 3).unwrap();

        let trip_notes = list_notes_for_owner(&conn, NoteOwnerType::Trip, trip_id).unwrap();
        assert_eq!(trip_notes.len(), 1);
        assert_eq!(trip_notes[0].body, "trip note");
    }

    #[test]
    fn test_resolve_note_owner_for_add_day_requires_trip() {
        let conn = test_db();
        assert!(resolve_note_owner_for_add(&conn, None, Some(2), None).is_err());
    }

    fn test_itinerary_item(
        id: i64,
        day: i64,
        title: &str,
        start_time: Option<&str>,
        sort_order: i64,
    ) -> ItineraryItem {
        ItineraryItem {
            id,
            trip_id: 1,
            day,
            title: title.to_string(),
            note: None,
            start_time: start_time.map(str::to_string),
            sort_order,
            duration_minutes: None,
            travel_minutes: None,
            location: None,
            category: None,
            created_at: String::new(),
            updated_at: String::new(),
        }
    }

    fn itinerary_note_key(
        day: i64,
        sort_order: i64,
        title: &str,
        start_time: Option<&str>,
    ) -> ItineraryNoteKey {
        ItineraryNoteKey {
            day_number: day,
            sort_order,
            start_time: start_time.map(str::to_string),
            title: title.to_string(),
        }
    }

    #[test]
    fn test_resolve_itinerary_id_unique_by_day_and_sort_order() {
        let items = vec![test_itinerary_item(10, 3, "Breakfast", Some("07:00"), 1000)];
        let key = itinerary_note_key(3, 1000, "Breakfast", Some("07:00"));
        assert_eq!(
            resolve_itinerary_id_from_export_items(&items, &key).unwrap(),
            10
        );
    }

    #[test]
    fn test_resolve_itinerary_id_narrows_by_title_and_start_time_when_sort_order_collides() {
        let items = vec![
            test_itinerary_item(10, 3, "Existing breakfast", Some("08:00"), 1000),
            test_itinerary_item(11, 3, "Hotel breakfast", Some("07:00"), 1000),
        ];
        let key = itinerary_note_key(3, 1000, "Hotel breakfast", Some("07:00"));
        assert_eq!(
            resolve_itinerary_id_from_export_items(&items, &key).unwrap(),
            11
        );
    }

    #[test]
    fn test_resolve_itinerary_id_ambiguous_when_day_sort_title_start_time_all_match() {
        let items = vec![
            test_itinerary_item(10, 3, "Hotel breakfast", Some("07:00"), 1000),
            test_itinerary_item(11, 3, "Hotel breakfast", Some("07:00"), 1000),
        ];
        let key = itinerary_note_key(3, 1000, "Hotel breakfast", Some("07:00"));
        let err = resolve_itinerary_id_from_export_items(&items, &key).unwrap_err();
        assert!(err.to_string().contains("複数"));
    }

    #[test]
    fn test_replicate_duplicate_sort_order_note_import_resolves_correct_itinerary() {
        use crate::itinerary::{get_itinerary_item, replicate_itinerary_items};
        use crate::trip::{export_trip_to_json, import_trip_from_json};

        let conn = test_db();
        let trip_id = add_trip(&conn, "Trip", "2026-01-01", "2026-01-05", None).unwrap();

        let existing_id = add_itinerary_item(
            &conn,
            trip_id,
            3,
            "Existing breakfast",
            None,
            Some("08:00"),
            Some(1000),
            None,
            None,
            None,
            None,
        )
        .unwrap();
        let source_id = add_itinerary_item(
            &conn,
            trip_id,
            2,
            "Hotel breakfast",
            None,
            Some("07:00"),
            Some(1000),
            None,
            None,
            None,
            None,
        )
        .unwrap();
        add_note(
            &conn,
            ResolvedNoteOwner::Itinerary(source_id),
            Some("allergy"),
            "replicated note body",
        )
        .unwrap();

        let result = replicate_itinerary_items(&conn, &[source_id], &[3], true, false).unwrap();
        let copied_id = result.by_day[0].created_ids[0];

        assert_eq!(
            get_itinerary_item(&conn, existing_id).unwrap().sort_order,
            1000
        );
        assert_eq!(
            get_itinerary_item(&conn, copied_id).unwrap().sort_order,
            1000
        );

        let json = export_trip_to_json(&conn, trip_id).unwrap();
        let imported_trip_id = import_trip_from_json(&conn, &json).unwrap();
        let imported_items =
            crate::itinerary::list_itinerary_items(&conn, imported_trip_id).unwrap();

        let replicated = imported_items
            .iter()
            .find(|item| item.day == 3 && item.title == "Hotel breakfast")
            .expect("replicated itinerary");
        let existing = imported_items
            .iter()
            .find(|item| item.day == 3 && item.title == "Existing breakfast")
            .expect("existing itinerary");

        let replicated_notes =
            list_notes_for_owner(&conn, NoteOwnerType::Itinerary, replicated.id).unwrap();
        let existing_notes =
            list_notes_for_owner(&conn, NoteOwnerType::Itinerary, existing.id).unwrap();

        assert_eq!(replicated_notes.len(), 1);
        assert_eq!(replicated_notes[0].body, "replicated note body");
        assert_eq!(replicated_notes[0].title.as_deref(), Some("allergy"));
        assert!(existing_notes.is_empty());
    }
}
