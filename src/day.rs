use anyhow::{Context, Result};
use chrono::NaiveDate;
use rusqlite::{params, Connection};
use serde::Serialize;

use crate::domain::models::{Day, ItineraryItem, Trip};
use crate::storage::db::now_string;

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
            "INSERT OR IGNORE INTO days (trip_id, day_number, title, summary, created_at, updated_at)
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
            let day = find_day_by_trip_and_day_number(conn, trip_id, day_number)?;
            if day_has_itinerary(conn, trip_id, day_number)? {
                anyhow::bail!("Day {day_number} に日程があるため、旅行期間を短縮できません");
            }
            if !day.title.is_empty()
                || day
                    .summary
                    .as_ref()
                    .is_some_and(|summary| !summary.is_empty())
            {
                anyhow::bail!(
                    "Day {day_number} にタイトルまたは概要があるため、旅行期間を短縮できません"
                );
            }
            // Day 削除前に Day Note と Receipt day 参照をクリア
            crate::note::delete_notes_for_day(conn, day.id)?;
            crate::receipt::nullify_receipts_for_day(conn, day.id)?;
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

/// trip_id + day_number から Day を取得する
pub(crate) fn find_day_by_trip_and_day_number(
    conn: &Connection,
    trip_id: i64,
    day_number: i64,
) -> Result<Day> {
    crate::storage::db::map_query_row(
        conn.query_row(
            "SELECT id, trip_id, day_number, title, summary, created_at, updated_at
             FROM days WHERE trip_id = ?1 AND day_number = ?2",
            params![trip_id, day_number],
            row_to_day,
        ),
        || anyhow::anyhow!("Day not found: trip {trip_id} day {day_number}"),
    )
}

/// trip_id + day_number から Day ID を取得する（Trip 期間外はエラー）
pub(crate) fn find_day_id_by_trip_and_day_number(
    conn: &Connection,
    trip_id: i64,
    day_number: i64,
) -> Result<i64> {
    Ok(find_day_by_trip_and_day_number(conn, trip_id, day_number)?.id)
}

/// Trip 開始日と day_number からカレンダー日付 (YYYY-MM-DD) を導出する
pub(crate) fn day_date_for_trip(trip: &Trip, day_number: i64) -> Result<String> {
    let start = trip
        .start_date
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("Trip {} has no start_date", trip.id))?;
    let date = parse_trip_date(start)? + chrono::Duration::days(day_number - 1);
    Ok(date.format("%Y-%m-%d").to_string())
}

#[derive(Serialize)]
struct DayListEntryJson {
    id: i64,
    day_number: i64,
    date: String,
    title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    summary: Option<String>,
}

#[derive(Serialize)]
struct DayListJson {
    trip_id: i64,
    trip_name: String,
    days: Vec<DayListEntryJson>,
}

#[derive(Serialize)]
struct DayShowJson {
    trip_id: i64,
    trip_name: String,
    day_number: i64,
    date: String,
    day_id: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    summary: Option<String>,
    itineraries: Vec<ItineraryItem>,
}

/// Day 一覧を表示する
pub(crate) fn run_day_list(conn: &Connection, trip_id: i64, json: bool) -> Result<()> {
    let trip = crate::trip::get_trip(conn, trip_id)?;
    let days = list_days(conn, trip_id)?;
    if json {
        let entries = days
            .iter()
            .map(|day| {
                Ok(DayListEntryJson {
                    id: day.id,
                    day_number: day.day_number,
                    date: day_date_for_trip(&trip, day.day_number)?,
                    title: day.title.clone(),
                    summary: day.summary.clone(),
                })
            })
            .collect::<Result<Vec<_>>>()?;
        crate::output::json::print_json(&DayListJson {
            trip_id,
            trip_name: trip.name,
            days: entries,
        })?;
    } else {
        print_day_list(&trip, &days)?;
    }
    Ok(())
}

/// Day 詳細（配下 Itinerary 含む）を表示する
pub(crate) fn run_day_show(
    conn: &Connection,
    trip_id: i64,
    day_number: i64,
    json: bool,
) -> Result<()> {
    let trip = crate::trip::get_trip(conn, trip_id)?;
    let day = find_day_by_trip_and_day_number(conn, trip_id, day_number)?;
    let date = day_date_for_trip(&trip, day_number)?;
    let items = crate::itinerary::list_itinerary_items_for_day(conn, trip_id, day_number)?;
    if json {
        crate::output::json::print_json(&DayShowJson {
            trip_id,
            trip_name: trip.name,
            day_number,
            date,
            day_id: day.id,
            summary: day.summary.clone(),
            itineraries: items,
        })?;
    } else {
        print_day_show(&trip, &day, day_number, &date, &items);
    }
    Ok(())
}

/// Day の summary を DB に設定する（import 用）
pub(crate) fn set_day_summary(
    conn: &Connection,
    trip_id: i64,
    day_number: i64,
    summary: Option<String>,
) -> Result<()> {
    let day = find_day_by_trip_and_day_number(conn, trip_id, day_number)?;
    let now = now_string();
    conn.execute(
        "UPDATE days SET summary = ?1, updated_at = ?2 WHERE id = ?3",
        params![summary, &now, day.id],
    )
    .context("Day summary の設定に失敗しました")?;
    Ok(())
}

/// Day の summary を更新する
pub(crate) fn update_day_summary(
    conn: &Connection,
    trip_id: i64,
    day_number: i64,
    summary: Option<&str>,
    clear_summary: bool,
) -> Result<()> {
    if !clear_summary && summary.is_none() {
        anyhow::bail!("更新する項目を指定してください (--summary または --clear-summary)");
    }
    let _trip = crate::trip::get_trip(conn, trip_id)?;
    let new_summary = if clear_summary {
        None
    } else {
        crate::summary::normalize_day_summary(summary)?
    };
    set_day_summary(conn, trip_id, day_number, new_summary)
}

/// Day 更新 CLI
pub(crate) fn run_day_update(
    conn: &Connection,
    trip_id: i64,
    day_number: i64,
    summary: Option<&str>,
    clear_summary: bool,
) -> Result<()> {
    update_day_summary(conn, trip_id, day_number, summary, clear_summary)?;
    let trip = crate::trip::get_trip(conn, trip_id)?;
    let day = find_day_by_trip_and_day_number(conn, trip_id, day_number)?;
    let date = day_date_for_trip(&trip, day_number)?;
    let items = crate::itinerary::list_itinerary_items_for_day(conn, trip_id, day_number)?;
    println!("Day {day_number} を更新しました");
    print_day_show(&trip, &day, day_number, &date, &items);
    Ok(())
}

/// 2 つの Day の plan payload（Itinerary、title、summary、Day-level notes）を入れ替える。
/// `day_number` / カレンダー日付 / `days.id` / `created_at` は変更しない。
pub(crate) fn swap_day_plan_payload(
    conn: &Connection,
    trip_id: i64,
    day_a: i64,
    day_b: i64,
) -> Result<usize> {
    if day_a == day_b {
        anyhow::bail!("同じ Day を指定しています: Day {day_a}");
    }
    crate::trip::get_trip(conn, trip_id)?;
    let day_a_row = find_day_by_trip_and_day_number(conn, trip_id, day_a)?;
    let day_b_row = find_day_by_trip_and_day_number(conn, trip_id, day_b)?;
    let title_a = day_a_row.title.clone();
    let title_b = day_b_row.title.clone();
    let summary_a = day_a_row.summary.clone();
    let summary_b = day_b_row.summary.clone();
    let now = now_string();
    let tx = conn
        .unchecked_transaction()
        .context("Day swap トランザクションの開始に失敗しました")?;
    let itinerary_updated = tx
        .execute(
            "UPDATE itinerary_items SET
               day_id = CASE
                 WHEN day_id = ?1 THEN ?2
                 WHEN day_id = ?3 THEN ?4
               END,
               day = CASE
                 WHEN day_id = ?1 THEN ?5
                 WHEN day_id = ?3 THEN ?6
               END,
               updated_at = ?7
             WHERE day_id IN (?1, ?3)",
            params![
                day_a_row.id,
                day_b_row.id,
                day_b_row.id,
                day_a_row.id,
                day_b,
                day_a,
                &now,
            ],
        )
        .context("Day swap の Itinerary 更新に失敗しました")?;
    tx.execute(
        "UPDATE days SET title = ?1, summary = ?2, updated_at = ?3 WHERE id = ?4",
        params![title_b, summary_b, &now, day_a_row.id],
    )
    .context("Day swap の title/summary 更新に失敗しました")?;
    tx.execute(
        "UPDATE days SET title = ?1, summary = ?2, updated_at = ?3 WHERE id = ?4",
        params![title_a, summary_a, &now, day_b_row.id],
    )
    .context("Day swap の title/summary 更新に失敗しました")?;
    crate::note::swap_day_note_owners(&tx, day_a_row.id, day_b_row.id, &now)
        .context("Day swap の Day Note 更新に失敗しました")?;
    tx.commit()
        .context("Day swap トランザクションの確定に失敗しました")?;
    Ok(itinerary_updated)
}

fn print_day_list(trip: &Trip, days: &[Day]) -> Result<()> {
    println!("Trip: {}", trip.name);
    println!();
    for day in days {
        let date = day_date_for_trip(trip, day.day_number)?;
        println!("Day {}  {}", day.day_number, date);
    }
    Ok(())
}

fn print_day_show(trip: &Trip, day: &Day, day_number: i64, date: &str, items: &[ItineraryItem]) {
    println!("Trip: {}", trip.name);
    println!();
    println!("Day {day_number}");
    println!("Date: {date}");
    if let Some(summary) = &day.summary {
        println!();
        println!("概要:");
        for line in summary.lines() {
            println!("  {line}");
        }
    }
    println!();
    println!("Itineraries:");
    if items.is_empty() {
        println!("  （なし）");
        return;
    }
    for item in items {
        match &item.start_time {
            Some(time) => println!("- {time} {}", item.title),
            None => println!("- {}", item.title),
        }
    }
}

/// 旅行に紐づく Day 一覧を取得する
pub(crate) fn list_days(conn: &Connection, trip_id: i64) -> Result<Vec<Day>> {
    let mut stmt = conn
        .prepare(
            "SELECT id, trip_id, day_number, title, summary, created_at, updated_at
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
        "SELECT COUNT(*) FROM itinerary_items
         WHERE day_id = (
           SELECT id FROM days WHERE trip_id = ?1 AND day_number = ?2
         )",
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
        summary: row.get(4)?,
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
    use crate::itinerary::add_itinerary_item;
    use crate::storage::db::open_db_at;
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
        let trip_id = add_trip(&conn, "Day Trip", "2026-12-01", "2026-12-03", None).unwrap();
        let days = list_days(&conn, trip_id).unwrap();
        assert_eq!(days.len(), 3);
        assert_eq!(days[0].day_number, 1);
        assert_eq!(days[2].day_number, 3);
        assert!(days[0].title.is_empty());
    }

    #[test]
    fn test_sync_days_extends_on_end_date_change() {
        let conn = test_db();
        let trip_id = add_trip(&conn, "Extend Trip", "2026-12-01", "2026-12-02", None).unwrap();
        sync_days_to_trip_duration(&conn, trip_id, 4).unwrap();
        assert_eq!(list_days(&conn, trip_id).unwrap().len(), 4);
    }

    #[test]
    fn test_sync_days_deletes_empty_extra_days() {
        let conn = test_db();
        let trip_id = add_trip(&conn, "Shrink Trip", "2026-12-01", "2026-12-04", None).unwrap();
        sync_days_to_trip_duration(&conn, trip_id, 2).unwrap();
        assert_eq!(list_days(&conn, trip_id).unwrap().len(), 2);
    }

    #[test]
    fn test_sync_days_rejects_shrink_when_itinerary_exists() {
        let conn = test_db();
        let trip_id = add_trip(&conn, "Busy Trip", "2026-12-01", "2026-12-03", None).unwrap();
        add_itinerary_item(
            &conn, trip_id, 3, "Activity", None, None, None, None, None, None, None,
        )
        .unwrap();
        assert!(sync_days_to_trip_duration(&conn, trip_id, 2).is_err());
    }

    #[test]
    fn test_day_date_for_trip() {
        let conn = test_db();
        let trip_id = add_trip(&conn, "Okinawa", "2026-04-26", "2026-04-29", None).unwrap();
        let trip = crate::trip::get_trip(&conn, trip_id).unwrap();
        assert_eq!(day_date_for_trip(&trip, 1).unwrap(), "2026-04-26");
        assert_eq!(day_date_for_trip(&trip, 4).unwrap(), "2026-04-29");
    }

    #[test]
    fn test_day_list_includes_derived_dates() {
        let conn = test_db();
        let trip_id = add_trip(
            &conn,
            "Okinawa Family Trip",
            "2026-04-26",
            "2026-04-29",
            None,
        )
        .unwrap();
        let trip = crate::trip::get_trip(&conn, trip_id).unwrap();
        let days = list_days(&conn, trip_id).unwrap();
        assert_eq!(days.len(), 4);
        assert_eq!(day_date_for_trip(&trip, 1).unwrap(), "2026-04-26");
        assert_eq!(day_date_for_trip(&trip, 2).unwrap(), "2026-04-27");
    }

    #[test]
    fn test_run_day_show_empty_day() {
        let conn = test_db();
        let trip_id = add_trip(&conn, "Empty Day Trip", "2026-04-26", "2026-04-29", None).unwrap();
        let items = crate::itinerary::list_itinerary_items_for_day(&conn, trip_id, 2).unwrap();
        assert!(items.is_empty());
    }

    #[test]
    fn test_run_day_show_rejects_invalid_day_number() {
        let conn = test_db();
        let trip_id = add_trip(&conn, "Range Trip", "2026-04-26", "2026-04-29", None).unwrap();
        assert!(run_day_show(&conn, trip_id, 99, false).is_err());
    }

    fn set_day_metadata(
        conn: &Connection,
        trip_id: i64,
        day_number: i64,
        title: &str,
        summary: Option<&str>,
    ) {
        let day = find_day_by_trip_and_day_number(conn, trip_id, day_number).unwrap();
        let now = now_string();
        conn.execute(
            "UPDATE days SET title = ?1, summary = ?2, updated_at = ?3 WHERE id = ?4",
            params![title, summary, &now, day.id],
        )
        .unwrap();
    }

    #[test]
    fn test_swap_day_plan_payload_exchanges_plan_metadata() {
        use crate::domain::models::NoteOwnerType;
        use crate::note::{add_note, list_notes_for_owner, ResolvedNoteOwner};

        let conn = test_db();
        let trip_id = add_trip(&conn, "Swap Trip", "2026-04-26", "2026-04-29", None).unwrap();
        let trip = crate::trip::get_trip(&conn, trip_id).unwrap();
        let day2_before = find_day_by_trip_and_day_number(&conn, trip_id, 2).unwrap();
        let day3_before = find_day_by_trip_and_day_number(&conn, trip_id, 3).unwrap();
        let date2_before = day_date_for_trip(&trip, 2).unwrap();
        let date3_before = day_date_for_trip(&trip, 3).unwrap();

        set_day_metadata(
            &conn,
            trip_id,
            2,
            "水族館の日",
            Some("美ら海水族館を中心に回る"),
        );
        set_day_metadata(
            &conn,
            trip_id,
            3,
            "ビーチの日",
            Some("瀬底ビーチでゆっくりする"),
        );
        add_itinerary_item(
            &conn,
            trip_id,
            2,
            "美ら海水族館",
            None,
            Some("09:00"),
            None,
            None,
            None,
            None,
            None,
        )
        .unwrap();
        add_itinerary_item(
            &conn,
            trip_id,
            2,
            "海邦丸",
            None,
            Some("13:00"),
            None,
            None,
            None,
            None,
            None,
        )
        .unwrap();
        add_itinerary_item(
            &conn,
            trip_id,
            3,
            "瀬底ビーチ",
            None,
            Some("10:00"),
            None,
            None,
            None,
            None,
            None,
        )
        .unwrap();
        let day2 = find_day_by_trip_and_day_number(&conn, trip_id, 2).unwrap();
        let day3 = find_day_by_trip_and_day_number(&conn, trip_id, 3).unwrap();
        add_note(
            &conn,
            ResolvedNoteOwner::Day(day2.id),
            None,
            "午後は無理しない",
        )
        .unwrap();
        add_note(
            &conn,
            ResolvedNoteOwner::Day(day3.id),
            None,
            "天気が悪ければ室内案",
        )
        .unwrap();
        add_note(
            &conn,
            ResolvedNoteOwner::Trip(trip_id),
            None,
            "trip level note",
        )
        .unwrap();

        swap_day_plan_payload(&conn, trip_id, 2, 3).unwrap();

        let day2_after = find_day_by_trip_and_day_number(&conn, trip_id, 2).unwrap();
        let day3_after = find_day_by_trip_and_day_number(&conn, trip_id, 3).unwrap();
        assert_eq!(day2_after.id, day2_before.id);
        assert_eq!(day3_after.id, day3_before.id);
        assert_eq!(day2_after.day_number, 2);
        assert_eq!(day3_after.day_number, 3);
        assert_eq!(day_date_for_trip(&trip, 2).unwrap(), date2_before);
        assert_eq!(day_date_for_trip(&trip, 3).unwrap(), date3_before);

        assert_eq!(day2_after.title, "ビーチの日");
        assert_eq!(day3_after.title, "水族館の日");
        assert_eq!(
            day2_after.summary.as_deref(),
            Some("瀬底ビーチでゆっくりする")
        );
        assert_eq!(
            day3_after.summary.as_deref(),
            Some("美ら海水族館を中心に回る")
        );

        let day2_items = crate::itinerary::list_itinerary_items_for_day(&conn, trip_id, 2).unwrap();
        let day3_items = crate::itinerary::list_itinerary_items_for_day(&conn, trip_id, 3).unwrap();
        assert_eq!(day2_items.len(), 1);
        assert_eq!(day3_items.len(), 2);
        assert_eq!(day2_items[0].title, "瀬底ビーチ");
        assert_eq!(day3_items[0].title, "美ら海水族館");
        assert_eq!(day3_items[1].title, "海邦丸");

        let day2_notes = list_notes_for_owner(&conn, NoteOwnerType::Day, day2_after.id).unwrap();
        let day3_notes = list_notes_for_owner(&conn, NoteOwnerType::Day, day3_after.id).unwrap();
        assert_eq!(day2_notes[0].body, "天気が悪ければ室内案");
        assert_eq!(day3_notes[0].body, "午後は無理しない");

        let trip_notes = list_notes_for_owner(&conn, NoteOwnerType::Trip, trip_id).unwrap();
        assert_eq!(trip_notes.len(), 1);
        assert_eq!(trip_notes[0].body, "trip level note");
    }

    #[test]
    fn test_swap_day_plan_payload_exchanges_day2_and_day3() {
        let conn = test_db();
        let trip_id = add_trip(&conn, "Swap Trip", "2026-04-26", "2026-04-29", None).unwrap();
        add_itinerary_item(
            &conn,
            trip_id,
            2,
            "Aquarium",
            None,
            Some("09:00"),
            None,
            None,
            None,
            None,
            None,
        )
        .unwrap();
        add_itinerary_item(
            &conn,
            trip_id,
            2,
            "Beach",
            None,
            Some("13:00"),
            None,
            None,
            None,
            None,
            None,
        )
        .unwrap();
        add_itinerary_item(
            &conn,
            trip_id,
            3,
            "Castle",
            None,
            Some("10:00"),
            None,
            None,
            None,
            None,
            None,
        )
        .unwrap();

        let updated = swap_day_plan_payload(&conn, trip_id, 2, 3).unwrap();
        assert_eq!(updated, 3);

        let day2 = crate::itinerary::list_itinerary_items_for_day(&conn, trip_id, 2).unwrap();
        let day3 = crate::itinerary::list_itinerary_items_for_day(&conn, trip_id, 3).unwrap();
        assert_eq!(day2.len(), 1);
        assert_eq!(day3.len(), 2);
        assert_eq!(day2[0].title, "Castle");
        assert_eq!(day3[0].title, "Aquarium");
        assert_eq!(day3[1].title, "Beach");
    }

    #[test]
    fn test_swap_preserves_total_itinerary_count() {
        let conn = test_db();
        let trip_id = add_trip(&conn, "Count Trip", "2026-04-26", "2026-04-29", None).unwrap();
        add_itinerary_item(
            &conn, trip_id, 2, "A", None, None, None, None, None, None, None,
        )
        .unwrap();
        add_itinerary_item(
            &conn, trip_id, 3, "B", None, None, None, None, None, None, None,
        )
        .unwrap();
        add_itinerary_item(
            &conn, trip_id, 3, "C", None, None, None, None, None, None, None,
        )
        .unwrap();
        let before = crate::itinerary::list_itinerary_items(&conn, trip_id).unwrap();
        swap_day_plan_payload(&conn, trip_id, 2, 3).unwrap();
        let after = crate::itinerary::list_itinerary_items(&conn, trip_id).unwrap();
        assert_eq!(before.len(), after.len());
        assert_eq!(after.iter().filter(|item| item.day == 2).count(), 2);
        assert_eq!(after.iter().filter(|item| item.day == 3).count(), 1);
    }

    #[test]
    fn test_swap_rejects_same_day() {
        let conn = test_db();
        let trip_id = add_trip(&conn, "Same Day Trip", "2026-04-26", "2026-04-29", None).unwrap();
        add_itinerary_item(
            &conn, trip_id, 2, "Plan", None, None, None, None, None, None, None,
        )
        .unwrap();
        let before = crate::itinerary::list_itinerary_items_for_day(&conn, trip_id, 2).unwrap();
        let err = swap_day_plan_payload(&conn, trip_id, 2, 2).unwrap_err();
        assert!(err.to_string().contains("同じ Day"));
        let after = crate::itinerary::list_itinerary_items_for_day(&conn, trip_id, 2).unwrap();
        assert_eq!(before.len(), after.len());
        assert_eq!(before[0].title, after[0].title);
    }

    #[test]
    fn test_swap_leaves_data_unchanged_on_invalid_day() {
        let conn = test_db();
        let trip_id =
            add_trip(&conn, "Invalid Day Trip", "2026-04-26", "2026-04-29", None).unwrap();
        add_itinerary_item(
            &conn, trip_id, 2, "Plan", None, None, None, None, None, None, None,
        )
        .unwrap();
        let before = crate::itinerary::list_itinerary_items_for_day(&conn, trip_id, 2).unwrap();
        assert!(swap_day_plan_payload(&conn, trip_id, 2, 99).is_err());
        let after = crate::itinerary::list_itinerary_items_for_day(&conn, trip_id, 2).unwrap();
        assert_eq!(before.len(), after.len());
        assert_eq!(before[0].title, after[0].title);
    }

    #[test]
    fn test_swap_transaction_rollback_on_failed_commit() {
        let conn = test_db();
        let trip_id = add_trip(&conn, "Rollback Trip", "2026-04-26", "2026-04-29", None).unwrap();
        add_itinerary_item(
            &conn, trip_id, 2, "A", None, None, None, None, None, None, None,
        )
        .unwrap();
        add_itinerary_item(
            &conn, trip_id, 3, "B", None, None, None, None, None, None, None,
        )
        .unwrap();
        let day_a = find_day_by_trip_and_day_number(&conn, trip_id, 2).unwrap();
        let day_b = find_day_by_trip_and_day_number(&conn, trip_id, 3).unwrap();
        let before_day2 =
            crate::itinerary::list_itinerary_items_for_day(&conn, trip_id, 2).unwrap();
        let before_day3 =
            crate::itinerary::list_itinerary_items_for_day(&conn, trip_id, 3).unwrap();

        let tx = conn.unchecked_transaction().unwrap();
        tx.execute(
            "UPDATE itinerary_items SET day_id = ?1 WHERE day_id = ?2",
            params![day_b.id, day_a.id],
        )
        .unwrap();
        drop(tx);

        let after_day2 = crate::itinerary::list_itinerary_items_for_day(&conn, trip_id, 2).unwrap();
        let after_day3 = crate::itinerary::list_itinerary_items_for_day(&conn, trip_id, 3).unwrap();
        assert_eq!(before_day2[0].title, after_day2[0].title);
        assert_eq!(before_day3[0].title, after_day3[0].title);
    }

    #[test]
    fn test_day_list_json_payload() {
        let conn = test_db();
        let trip_id = add_trip(&conn, "JSON Trip", "2026-04-26", "2026-04-28", None).unwrap();
        let trip = crate::trip::get_trip(&conn, trip_id).unwrap();
        let days = list_days(&conn, trip_id).unwrap();
        let entries = days
            .iter()
            .map(|day| {
                Ok(DayListEntryJson {
                    id: day.id,
                    day_number: day.day_number,
                    date: day_date_for_trip(&trip, day.day_number)?,
                    title: day.title.clone(),
                    summary: day.summary.clone(),
                })
            })
            .collect::<Result<Vec<_>>>()
            .unwrap();
        let json = serde_json::to_string_pretty(&DayListJson {
            trip_id,
            trip_name: trip.name,
            days: entries,
        })
        .unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["trip_id"], trip_id);
        assert_eq!(parsed["days"].as_array().unwrap().len(), 3);
        assert_eq!(parsed["days"][0]["date"], "2026-04-26");
    }

    #[test]
    fn test_day_show_json_payload() {
        let conn = test_db();
        let trip_id = add_trip(&conn, "JSON Show Trip", "2026-04-26", "2026-04-28", None).unwrap();
        add_itinerary_item(
            &conn,
            trip_id,
            2,
            "Museum",
            None,
            Some("09:00"),
            None,
            None,
            None,
            None,
            None,
        )
        .unwrap();
        let trip = crate::trip::get_trip(&conn, trip_id).unwrap();
        let day = find_day_by_trip_and_day_number(&conn, trip_id, 2).unwrap();
        let items = crate::itinerary::list_itinerary_items_for_day(&conn, trip_id, 2).unwrap();
        let date = day_date_for_trip(&trip, 2).unwrap();
        let json = serde_json::to_string_pretty(&DayShowJson {
            trip_id,
            trip_name: trip.name,
            day_number: 2,
            date,
            day_id: day.id,
            summary: day.summary.clone(),
            itineraries: items,
        })
        .unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["day_number"], 2);
        assert_eq!(parsed["date"], "2026-04-27");
        assert_eq!(parsed["itineraries"].as_array().unwrap().len(), 1);
        assert_eq!(parsed["itineraries"][0]["title"], "Museum");
    }
}
