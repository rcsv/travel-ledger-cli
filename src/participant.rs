use anyhow::{Context, Result};
use rusqlite::{params, Connection};
use serde::Serialize;

use crate::models::{ExportParticipantV4, Participant, ParticipantCounts};

pub(crate) const MAX_PARTICIPANT_NAME_LEN: usize = 200;

const PARTICIPANT_SELECT_SQL: &str = "
    SELECT id, trip_id, name, sort_order, is_self, created_at, updated_at
    FROM participants";

pub(crate) fn migrate_participants(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS participants (
            id          INTEGER PRIMARY KEY AUTOINCREMENT,
            trip_id     INTEGER NOT NULL,
            name        TEXT NOT NULL,
            sort_order  INTEGER NOT NULL DEFAULT 0,
            is_self     INTEGER NOT NULL DEFAULT 0,
            created_at  TEXT NOT NULL,
            updated_at  TEXT NOT NULL
        )",
        [],
    )
    .context("participants テーブルの作成に失敗しました")?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_participants_trip ON participants(trip_id)",
        [],
    )
    .context("idx_participants_trip の作成に失敗しました")?;
    conn.execute(
        "CREATE UNIQUE INDEX IF NOT EXISTS idx_participants_one_self_per_trip
         ON participants(trip_id) WHERE is_self = 1",
        [],
    )
    .context("idx_participants_one_self_per_trip の作成に失敗しました")?;
    Ok(())
}

pub(crate) fn validate_participant_name(name: &str) -> Result<String> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        anyhow::bail!("name must not be empty");
    }
    if trimmed.chars().count() > MAX_PARTICIPANT_NAME_LEN {
        anyhow::bail!("name is too long (max {MAX_PARTICIPANT_NAME_LEN} characters)");
    }
    Ok(trimmed.to_string())
}

pub(crate) fn validate_sort_order(sort_order: i64) -> Result<()> {
    if sort_order < 0 {
        anyhow::bail!("sort_order must be non-negative");
    }
    Ok(())
}

fn sqlite_bool(value: bool) -> i64 {
    i64::from(value)
}

fn row_to_participant(row: &rusqlite::Row) -> rusqlite::Result<Participant> {
    Ok(Participant {
        id: row.get(0)?,
        trip_id: row.get(1)?,
        name: row.get(2)?,
        sort_order: row.get(3)?,
        is_self: row.get::<_, i64>(4)? != 0,
        created_at: row.get(5)?,
        updated_at: row.get(6)?,
    })
}

pub(crate) fn count_self_participants_for_trip(conn: &Connection, trip_id: i64) -> Result<i64> {
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM participants WHERE trip_id = ?1 AND is_self = 1",
        params![trip_id],
        |row| row.get(0),
    )?;
    Ok(count)
}

fn next_sort_order_for_trip(conn: &Connection, trip_id: i64) -> Result<i64> {
    let max: Option<i64> = conn
        .query_row(
            "SELECT MAX(sort_order) FROM participants WHERE trip_id = ?1",
            params![trip_id],
            |row| row.get(0),
        )
        .ok();
    Ok(max.map(|v| v + 1).unwrap_or(0))
}

pub(crate) fn compute_participant_counts(participants: &[Participant]) -> ParticipantCounts {
    let registered_count = participants.len();
    let participants_recorded = registered_count > 0;
    let self_count = participants.iter().filter(|p| p.is_self).count();

    if self_count == 1 {
        ParticipantCounts {
            registered_count,
            participant_count: Some(registered_count),
            companion_count: Some(registered_count.saturating_sub(1)),
            self_known: true,
            participants_recorded,
        }
    } else {
        ParticipantCounts {
            registered_count,
            participant_count: None,
            companion_count: None,
            self_known: false,
            participants_recorded,
        }
    }
}

pub(crate) fn compute_participant_counts_for_trip(
    conn: &Connection,
    trip_id: i64,
) -> Result<ParticipantCounts> {
    let participants = list_participants_by_trip(conn, trip_id)?;
    Ok(compute_participant_counts(&participants))
}

#[derive(Serialize)]
pub(crate) struct ParticipantListJson {
    pub schema_version: i32,
    pub trip_id: i64,
    pub participants: Vec<Participant>,
    pub counts: ParticipantCounts,
}

pub(crate) fn create_participant(
    conn: &Connection,
    trip_id: i64,
    name: &str,
    sort_order: Option<i64>,
    is_self: bool,
) -> Result<i64> {
    crate::trip::get_trip(conn, trip_id)?;
    let name = validate_participant_name(name)?;
    let sort_order = match sort_order {
        Some(value) => {
            validate_sort_order(value)?;
            value
        }
        None => next_sort_order_for_trip(conn, trip_id)?,
    };

    if is_self && count_self_participants_for_trip(conn, trip_id)? > 0 {
        anyhow::bail!("trip already has a self participant");
    }

    let now = crate::db::now_string();
    conn.execute(
        "INSERT INTO participants
         (trip_id, name, sort_order, is_self, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![trip_id, name, sort_order, sqlite_bool(is_self), &now, &now],
    )
    .context("Participant の追加に失敗しました")?;
    Ok(conn.last_insert_rowid())
}

pub(crate) fn list_participants_by_trip(
    conn: &Connection,
    trip_id: i64,
) -> Result<Vec<Participant>> {
    crate::trip::get_trip(conn, trip_id)?;
    let mut stmt = conn
        .prepare(&format!(
            "{PARTICIPANT_SELECT_SQL}
             WHERE trip_id = ?1
             ORDER BY sort_order ASC, id ASC"
        ))
        .context("Participant 一覧取得の準備に失敗しました")?;

    let participants = stmt
        .query_map(params![trip_id], row_to_participant)
        .context("Participant 一覧取得に失敗しました")?
        .collect::<std::result::Result<Vec<_>, _>>()
        .context("Participant 一覧の読み込みに失敗しました")?;
    Ok(participants)
}

pub(crate) fn get_participant(conn: &Connection, id: i64) -> Result<Participant> {
    crate::db::map_query_row(
        conn.query_row(
            &format!("{PARTICIPANT_SELECT_SQL} WHERE id = ?1"),
            params![id],
            row_to_participant,
        ),
        || anyhow::anyhow!("participant not found: {id}"),
    )
}

pub(crate) fn update_participant(
    conn: &Connection,
    id: i64,
    name: Option<&str>,
    sort_order: Option<i64>,
    set_self: Option<bool>,
) -> Result<()> {
    if name.is_none() && sort_order.is_none() && set_self.is_none() {
        anyhow::bail!("at least one of --name, --sort-order, --self, --not-self is required");
    }

    let mut participant = get_participant(conn, id)?;
    if let Some(value) = name {
        participant.name = validate_participant_name(value)?;
    }
    if let Some(value) = sort_order {
        validate_sort_order(value)?;
        participant.sort_order = value;
    }

    crate::db::with_transaction(conn, "participant update", |tx| {
        if set_self == Some(true) {
            tx.execute(
                "UPDATE participants SET is_self = 0, updated_at = ?1
                 WHERE trip_id = ?2 AND id != ?3",
                params![crate::db::now_string(), participant.trip_id, id],
            )
            .context("self participant の付け替えに失敗しました")?;
            participant.is_self = true;
        } else if set_self == Some(false) {
            participant.is_self = false;
        }

        let now = crate::db::now_string();
        tx.execute(
            "UPDATE participants
             SET name = ?1, sort_order = ?2, is_self = ?3, updated_at = ?4
             WHERE id = ?5",
            params![
                participant.name,
                participant.sort_order,
                sqlite_bool(participant.is_self),
                &now,
                id
            ],
        )
        .context("Participant の更新に失敗しました")?;
        Ok(())
    })
}

pub(crate) fn delete_participant(conn: &Connection, id: i64) -> Result<()> {
    get_participant(conn, id)?;
    conn.execute("DELETE FROM participants WHERE id = ?1", params![id])
        .context("Participant の削除に失敗しました")?;
    Ok(())
}

pub(crate) fn delete_participants_for_trip(conn: &Connection, trip_id: i64) -> Result<()> {
    conn.execute(
        "DELETE FROM participants WHERE trip_id = ?1",
        params![trip_id],
    )
    .context("Trip 配下 Participant の削除に失敗しました")?;
    Ok(())
}

#[allow(dead_code)] // trip duplicate は export/import 経由で複製; 単体 API は unit test と将来用
pub(crate) fn duplicate_participants_for_trip(
    conn: &Connection,
    src_trip_id: i64,
    dst_trip_id: i64,
) -> Result<()> {
    for participant in list_participants_by_trip(conn, src_trip_id)? {
        create_participant(
            conn,
            dst_trip_id,
            &participant.name,
            Some(participant.sort_order),
            participant.is_self,
        )?;
    }
    Ok(())
}

pub(crate) fn build_export_participants(
    conn: &Connection,
    trip_id: i64,
) -> Result<Vec<ExportParticipantV4>> {
    Ok(list_participants_by_trip(conn, trip_id)?
        .into_iter()
        .map(|p| ExportParticipantV4 {
            name: p.name,
            sort_order: p.sort_order,
            is_self: p.is_self,
        })
        .collect())
}

pub(crate) fn validate_export_participant_v4(participant: &ExportParticipantV4) -> Result<()> {
    validate_participant_name(&participant.name)?;
    validate_sort_order(participant.sort_order)?;
    Ok(())
}

pub(crate) fn collect_export_participant_validation_errors(
    participants: &[ExportParticipantV4],
) -> Vec<String> {
    let mut errors = Vec::new();
    for (index, participant) in participants.iter().enumerate() {
        if let Err(error) = validate_export_participant_v4(participant) {
            errors.push(format!("participants[{index}]: {error}"));
        }
    }
    let self_count = participants.iter().filter(|p| p.is_self).count();
    if self_count > 1 {
        errors.push(format!(
            "participants: only one is_self=true allowed per trip (found {self_count})"
        ));
    }
    errors
}

pub(crate) fn import_export_participants(
    conn: &Connection,
    trip_id: i64,
    participants: &[ExportParticipantV4],
) -> Result<()> {
    for participant in participants {
        validate_export_participant_v4(participant)?;
        create_participant(
            conn,
            trip_id,
            &participant.name,
            Some(participant.sort_order),
            participant.is_self,
        )?;
    }
    Ok(())
}

pub(crate) fn print_participant_list_human(
    participants: &[Participant],
    counts: &ParticipantCounts,
) {
    if participants.is_empty() {
        println!("Participant は登録されていません。");
        print_participant_counts_footer(counts);
        return;
    }

    println!("{:<6} {:<24} {:<6} {:<5}", "ID", "NAME", "SORT", "SELF");
    println!("{}", "-".repeat(45));
    for participant in participants {
        let self_mark = if participant.is_self { "yes" } else { "no" };
        println!(
            "{:<6} {:<24} {:<6} {}",
            participant.id, participant.name, participant.sort_order, self_mark
        );
    }
    println!();
    print_participant_counts_footer(counts);
}

fn print_participant_counts_footer(counts: &ParticipantCounts) {
    if !counts.participants_recorded {
        println!("Participants: not recorded");
        return;
    }
    if counts.self_known {
        let n = counts.participant_count.unwrap_or(counts.registered_count);
        let c = counts.companion_count.unwrap_or(0);
        println!("Participants: {n} (companions: {c})");
    } else {
        println!(
            "Participants: {} recorded (traveler count unknown)",
            counts.registered_count
        );
    }
}

pub(crate) fn print_participant_detail(participant: &Participant) {
    println!("ID        : {}", participant.id);
    println!("Trip ID   : {}", participant.trip_id);
    println!("Name      : {}", participant.name);
    println!("Sort order: {}", participant.sort_order);
    println!(
        "Self      : {}",
        if participant.is_self { "yes" } else { "no" }
    );
    println!("Created   : {}", participant.created_at);
    println!("Updated   : {}", participant.updated_at);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{open_db_at, reset_db};

    fn setup_conn() -> rusqlite::Connection {
        let conn = open_db_at(":memory:").unwrap();
        conn
    }

    fn add_sample_trip(conn: &rusqlite::Connection) -> i64 {
        crate::trip::add_trip(conn, "Test Trip", "2026-06-01", "2026-06-03", None).unwrap()
    }

    #[test]
    fn test_migrate_participants_idempotent() {
        let conn = setup_conn();
        migrate_participants(&conn).unwrap();
        migrate_participants(&conn).unwrap();
    }

    #[test]
    fn test_participant_crud_and_counts() {
        let conn = setup_conn();
        let trip_id = add_sample_trip(&conn);

        let id = create_participant(&conn, trip_id, "  Alice  ", None, true).unwrap();
        let participant = get_participant(&conn, id).unwrap();
        assert_eq!(participant.name, "Alice");
        assert!(participant.is_self);

        let counts = compute_participant_counts_for_trip(&conn, trip_id).unwrap();
        assert_eq!(counts.registered_count, 1);
        assert_eq!(counts.participant_count, Some(1));
        assert_eq!(counts.companion_count, Some(0));
        assert!(counts.self_known);

        create_participant(&conn, trip_id, "Bob", None, false).unwrap();
        let counts = compute_participant_counts_for_trip(&conn, trip_id).unwrap();
        assert_eq!(counts.participant_count, Some(2));
        assert_eq!(counts.companion_count, Some(1));

        update_participant(&conn, id, Some("Alicia"), None, None).unwrap();
        assert_eq!(get_participant(&conn, id).unwrap().name, "Alicia");

        delete_participant(&conn, id).unwrap();
        let remaining = list_participants_by_trip(&conn, trip_id).unwrap();
        assert_eq!(remaining.len(), 1);
        assert!(!remaining[0].is_self);
        let counts = compute_participant_counts(&remaining);
        assert!(!counts.self_known);
        assert!(counts.companion_count.is_none());
    }

    #[test]
    fn test_self_max_one_on_add() {
        let conn = setup_conn();
        let trip_id = add_sample_trip(&conn);
        create_participant(&conn, trip_id, "Me", None, true).unwrap();
        let err =
            create_participant(&conn, trip_id, "Other", None, true).expect_err("expected error");
        assert!(err.to_string().contains("self participant"));
    }

    #[test]
    fn test_update_self_transfer() {
        let conn = setup_conn();
        let trip_id = add_sample_trip(&conn);
        let self_id = create_participant(&conn, trip_id, "Me", None, true).unwrap();
        let other_id = create_participant(&conn, trip_id, "Other", None, false).unwrap();

        update_participant(&conn, other_id, None, None, Some(true)).unwrap();
        assert!(!get_participant(&conn, self_id).unwrap().is_self);
        assert!(get_participant(&conn, other_id).unwrap().is_self);
    }

    #[test]
    fn test_delete_participants_for_trip_cascade() {
        let conn = setup_conn();
        let trip_id = add_sample_trip(&conn);
        create_participant(&conn, trip_id, "A", None, false).unwrap();
        delete_participants_for_trip(&conn, trip_id).unwrap();
        assert!(list_participants_by_trip(&conn, trip_id)
            .unwrap()
            .is_empty());
    }

    #[test]
    fn test_duplicate_participants_for_trip() {
        let conn = setup_conn();
        let src = add_sample_trip(&conn);
        let dst = add_sample_trip(&conn);
        create_participant(&conn, src, "Me", Some(0), true).unwrap();
        create_participant(&conn, src, "Partner", Some(1), false).unwrap();
        duplicate_participants_for_trip(&conn, src, dst).unwrap();
        let copied = list_participants_by_trip(&conn, dst).unwrap();
        assert_eq!(copied.len(), 2);
        assert!(copied.iter().any(|p| p.is_self && p.name == "Me"));
    }

    #[test]
    fn test_validate_export_multiple_self() {
        let participants = vec![
            ExportParticipantV4 {
                name: "A".to_string(),
                sort_order: 0,
                is_self: true,
            },
            ExportParticipantV4 {
                name: "B".to_string(),
                sort_order: 1,
                is_self: true,
            },
        ];
        let errors = collect_export_participant_validation_errors(&participants);
        assert!(!errors.is_empty());
    }

    #[test]
    fn test_empty_trip_counts_unknown() {
        let counts = compute_participant_counts(&[]);
        assert!(!counts.participants_recorded);
        assert!(counts.participant_count.is_none());
        assert!(counts.companion_count.is_none());
    }

    #[test]
    fn test_reset_db_clears_participants() {
        let path = std::env::temp_dir().join(format!(
            "caglla_participant_reset_{}.db",
            std::process::id()
        ));
        let _ = std::fs::remove_file(&path);
        let conn = open_db_at(path.to_str().unwrap()).unwrap();
        let trip_id = add_sample_trip(&conn);
        create_participant(&conn, trip_id, "A", None, false).unwrap();
        reset_db(&conn).unwrap();
        assert!(list_participants_by_trip(&conn, trip_id).is_err());
        let _ = std::fs::remove_file(path);
    }
}
