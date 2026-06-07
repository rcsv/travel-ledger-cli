use anyhow::{Context, Result};
use chrono::NaiveDate;
use rusqlite::{params, Connection};

use crate::db::now_string;
use crate::models::Day;

/// YYYY-MM-DD 形式の日付文字列をパースする
pub(crate) fn parse_trip_date(date: &str) -> Result<NaiveDate> {
    NaiveDate::parse_from_str(date, "%Y-%m-%d")
        .with_context(|| format!("日付の形式が不正です (YYYY-MM-DD): {date}"))
}

/// 旅行期間の日数（開始日・終了日を含む）を返す
pub(crate) fn trip_day_count(start: &str, end: &str) -> Result<i64> {
    let start = parse_trip_date(start)?;
    let end = parse_trip_date(end)?;
    if end < start {
        anyhow::bail!("終了日は開始日以降である必要があります");
    }
    Ok((end - start).num_days() + 1)
}

/// 旅行期間の日付整合性を検証し、日数を返す
pub(crate) fn validate_trip_date_range(start: &str, end: &str) -> Result<i64> {
    trip_day_count(start, end)
}

/// Trip 作成時に Day 1..=day_count を生成する
pub(crate) fn create_days_for_trip(conn: &Connection, trip_id: i64, day_count: i64) -> Result<()> {
    ensure_days_range(conn, trip_id, 1, day_count)
}

fn ensure_days_range(conn: &Connection, trip_id: i64, from: i64, to: i64) -> Result<()> {
    if from > to {
        return Ok(());
    }
    let now = now_string();
    for day_number in from..=to {
        conn.execute(
            "INSERT OR IGNORE INTO days (trip_id, day_number, title, description, created_at, updated_at)
             VALUES (?1, ?2, '', NULL, ?3, ?3)",
            params![trip_id, day_number, &now],
        )
        .with_context(|| format!("Day {day_number} の作成に失敗しました"))?;
    }
    Ok(())
}

/// 旅行期間変更後に Day 行数を調整する
pub(crate) fn sync_days_to_trip_duration(
    conn: &Connection,
    trip_id: i64,
    new_count: i64,
) -> Result<()> {
    let current_max = max_day_number(conn, trip_id)?;
    if new_count > current_max {
        ensure_days_range(conn, trip_id, current_max + 1, new_count)?;
        return Ok(());
    }
    if new_count < current_max {
        for day_number in (new_count + 1..=current_max).rev() {
            let day = get_day_by_number(conn, trip_id, day_number)?;
            if day_has_itinerary(conn, trip_id, day_number)? {
                anyhow::bail!("Day {day_number} に日程があるため、旅行期間を短縮できません");
            }
            if !day.title.is_empty()
                || day
                    .description
                    .as_ref()
                    .is_some_and(|description| !description.is_empty())
            {
                anyhow::bail!(
                    "Day {day_number} にタイトルまたは説明があるため、旅行期間を短縮できません"
                );
            }
            conn.execute("DELETE FROM days WHERE id = ?1", params![day.id])
                .context("Day の削除に失敗しました")?;
        }
    }
    Ok(())
}

fn max_day_number(conn: &Connection, trip_id: i64) -> Result<i64> {
    Ok(list_days(conn, trip_id)?
        .into_iter()
        .map(|day| day.day_number)
        .max()
        .unwrap_or(0))
}

/// day_number を指定して Day を取得する
pub(crate) fn get_day_by_number(conn: &Connection, trip_id: i64, day_number: i64) -> Result<Day> {
    crate::db::map_query_row(
        conn.query_row(
            "SELECT id, trip_id, day_number, title, description, created_at, updated_at
             FROM days WHERE trip_id = ?1 AND day_number = ?2",
            params![trip_id, day_number],
            row_to_day,
        ),
        || anyhow::anyhow!("Day not found: trip {trip_id} day {day_number}"),
    )
}

/// 旅行に紐づく Day 一覧を取得する
pub(crate) fn list_days(conn: &Connection, trip_id: i64) -> Result<Vec<Day>> {
    let mut stmt = conn
        .prepare(
            "SELECT id, trip_id, day_number, title, description, created_at, updated_at
             FROM days WHERE trip_id = ?1 ORDER BY day_number",
        )
        .context("Day 一覧取得の準備に失敗しました")?;

    let days = stmt
        .query_map(params![trip_id], row_to_day)
        .context("Day 一覧取得に失敗しました")?
        .collect::<std::result::Result<Vec<_>, _>>()
        .context("Day 一覧の読み込みに失敗しました")?;

    Ok(days)
}

fn day_has_itinerary(conn: &Connection, trip_id: i64, day_number: i64) -> Result<bool> {
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM itinerary_items WHERE trip_id = ?1 AND day = ?2",
        params![trip_id, day_number],
        |row| row.get(0),
    )?;
    Ok(count > 0)
}

pub(crate) fn row_to_day(row: &rusqlite::Row) -> rusqlite::Result<Day> {
    Ok(Day {
        id: row.get(0)?,
        trip_id: row.get(1)?,
        day_number: row.get(2)?,
        title: row.get(3)?,
        description: row.get(4)?,
        created_at: row.get(5)?,
        updated_at: row.get(6)?,
    })
}

/// 既存 DB 向け: Trip ごとに不足している Day 行を backfill する
pub(crate) fn migrate_days(conn: &Connection) -> Result<()> {
    let mut stmt = conn
        .prepare("SELECT id, start_date, end_date FROM trips ORDER BY id")
        .context("Trip 一覧取得の準備に失敗しました")?;
    let trips = stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, Option<String>>(1)?,
                row.get::<_, Option<String>>(2)?,
            ))
        })
        .context("Trip 一覧取得に失敗しました")?
        .collect::<std::result::Result<Vec<_>, _>>()
        .context("Trip 一覧の読み込みに失敗しました")?;

    for (trip_id, start_date, end_date) in trips {
        backfill_days_for_trip(conn, trip_id, start_date.as_deref(), end_date.as_deref())?;
    }
    Ok(())
}

fn backfill_days_for_trip(
    conn: &Connection,
    trip_id: i64,
    start_date: Option<&str>,
    end_date: Option<&str>,
) -> Result<()> {
    if let (Some(start), Some(end)) = (start_date, end_date) {
        let count = trip_day_count(start, end)?;
        create_days_for_trip(conn, trip_id, count)?;
        return Ok(());
    }

    let max_day: i64 = conn.query_row(
        "SELECT COALESCE(MAX(day), 0) FROM itinerary_items WHERE trip_id = ?1",
        params![trip_id],
        |row| row.get(0),
    )?;
    if max_day > 0 {
        create_days_for_trip(conn, trip_id, max_day)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::open_db_at;
    use crate::itinerary::add_itinerary_item;
    use crate::trip::add_trip;

    fn test_db() -> Connection {
        open_db_at(":memory:").expect("インメモリ DB の作成に失敗")
    }

    #[test]
    fn test_trip_day_count_inclusive() {
        assert_eq!(trip_day_count("2026-12-01", "2026-12-04").unwrap(), 4);
        assert_eq!(trip_day_count("2026-12-01", "2026-12-01").unwrap(), 1);
    }

    #[test]
    fn test_trip_day_count_rejects_invalid_range() {
        assert!(trip_day_count("2026-12-04", "2026-12-01").is_err());
    }

    #[test]
    fn test_create_days_for_trip() {
        let conn = test_db();
        let trip_id = add_trip(&conn, "Day Trip", "2026-12-01", "2026-12-03").unwrap();
        let days = list_days(&conn, trip_id).unwrap();
        assert_eq!(days.len(), 3);
        assert_eq!(days[0].day_number, 1);
        assert_eq!(days[2].day_number, 3);
        assert!(days[0].title.is_empty());
    }

    #[test]
    fn test_sync_days_extends_on_end_date_change() {
        let conn = test_db();
        let trip_id = add_trip(&conn, "Extend Trip", "2026-12-01", "2026-12-02").unwrap();
        sync_days_to_trip_duration(&conn, trip_id, 4).unwrap();
        assert_eq!(list_days(&conn, trip_id).unwrap().len(), 4);
    }

    #[test]
    fn test_sync_days_deletes_empty_extra_days() {
        let conn = test_db();
        let trip_id = add_trip(&conn, "Shrink Trip", "2026-12-01", "2026-12-04").unwrap();
        sync_days_to_trip_duration(&conn, trip_id, 2).unwrap();
        assert_eq!(list_days(&conn, trip_id).unwrap().len(), 2);
    }

    #[test]
    fn test_sync_days_rejects_shrink_when_itinerary_exists() {
        let conn = test_db();
        let trip_id = add_trip(&conn, "Busy Trip", "2026-12-01", "2026-12-03").unwrap();
        add_itinerary_item(
            &conn, trip_id, 3, "Activity", None, None, None, None, None, None, None,
        )
        .unwrap();
        assert!(sync_days_to_trip_duration(&conn, trip_id, 2).is_err());
    }
}
