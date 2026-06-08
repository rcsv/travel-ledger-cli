use anyhow::{Context, Result};

use crate::models::{ItineraryCategory, ItineraryItem};
use rusqlite::{params, Connection};

pub(crate) const ITINERARY_ITEM_SELECT_SQL: &str = "
    SELECT i.id, i.trip_id, d.day_number, i.title, i.note, i.start_time, i.sort_order,
           i.duration_minutes, i.travel_minutes, i.location, i.category, i.created_at, i.updated_at
    FROM itinerary_items i
    INNER JOIN days d ON i.day_id = d.id";

/// 新しい日程を追加する
#[allow(clippy::too_many_arguments)]
pub(crate) fn add_itinerary_item(
    conn: &Connection,
    trip_id: i64,
    day: i64,
    title: &str,
    note: Option<&str>,
    start_time: Option<&str>,
    sort_order: Option<i64>,
    duration_minutes: Option<i64>,
    travel_minutes: Option<i64>,
    location: Option<&str>,
    category: Option<ItineraryCategory>,
) -> Result<i64> {
    crate::trip::get_trip(conn, trip_id)?;
    if let Some(t) = start_time {
        parse_time_hhmm(t)?;
    }
    let day_id = crate::day::find_day_id_by_trip_and_day_number(conn, trip_id, day)?;
    let now = crate::db::now_string();
    let sort_order = sort_order.unwrap_or(0);
    let category = category.map(|c| c.as_str().to_string());
    conn.execute(
        "INSERT INTO itinerary_items
         (trip_id, day_id, day, title, note, start_time, sort_order, duration_minutes, travel_minutes,
          location, category, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
        params![
            trip_id,
            day_id,
            day,
            title,
            note,
            start_time,
            sort_order,
            duration_minutes,
            travel_minutes,
            location,
            category,
            &now,
            &now
        ],
    )
    .context("日程の追加に失敗しました")?;
    Ok(conn.last_insert_rowid())
}

/// 旅行に紐づく日程一覧を取得する
pub(crate) fn list_itinerary_items(conn: &Connection, trip_id: i64) -> Result<Vec<ItineraryItem>> {
    crate::trip::get_trip(conn, trip_id)?;
    let mut stmt = conn
        .prepare(&format!(
            "{ITINERARY_ITEM_SELECT_SQL}
             WHERE i.trip_id = ?1
             ORDER BY d.day_number, i.start_time IS NULL, i.start_time, i.sort_order, i.id"
        ))
        .context("日程一覧取得の準備に失敗しました")?;

    let items = stmt
        .query_map(params![trip_id], row_to_itinerary_item)
        .context("日程一覧取得に失敗しました")?
        .collect::<std::result::Result<Vec<_>, _>>()
        .context("日程一覧の読み込みに失敗しました")?;

    Ok(items)
}

/// 指定 Day に属する日程一覧を取得する（timeline と同じ並び順）
pub(crate) fn list_itinerary_items_for_day(
    conn: &Connection,
    trip_id: i64,
    day_number: i64,
) -> Result<Vec<ItineraryItem>> {
    crate::trip::get_trip(conn, trip_id)?;
    let _day = crate::day::find_day_by_trip_and_day_number(conn, trip_id, day_number)?;
    let mut stmt = conn
        .prepare(&format!(
            "{ITINERARY_ITEM_SELECT_SQL}
             WHERE i.trip_id = ?1 AND d.day_number = ?2
             ORDER BY d.day_number, i.start_time IS NULL, i.start_time, i.sort_order, i.id"
        ))
        .context("日程一覧取得の準備に失敗しました")?;

    let items = stmt
        .query_map(params![trip_id, day_number], row_to_itinerary_item)
        .context("日程一覧取得に失敗しました")?
        .collect::<std::result::Result<Vec<_>, _>>()
        .context("日程一覧の読み込みに失敗しました")?;

    Ok(items)
}

/// ID を指定して1件の日程を取得する
pub(crate) fn get_itinerary_item(conn: &Connection, id: i64) -> Result<ItineraryItem> {
    crate::db::map_query_row(
        conn.query_row(
            &format!("{ITINERARY_ITEM_SELECT_SQL} WHERE i.id = ?1"),
            params![id],
            row_to_itinerary_item,
        ),
        || anyhow::anyhow!("Itinerary not found: {id}"),
    )
}

/// 日程を更新する（指定されたフィールドのみ上書き）
#[allow(clippy::too_many_arguments)]
pub(crate) fn update_itinerary_item(
    conn: &Connection,
    id: i64,
    day: Option<i64>,
    title: Option<&str>,
    note: Option<Option<&str>>,
    start_time: Option<Option<&str>>,
    sort_order: Option<i64>,
    duration_minutes: Option<i64>,
    travel_minutes: Option<i64>,
    location: Option<Option<&str>>,
    category: Option<Option<ItineraryCategory>>,
) -> Result<()> {
    if day.is_none()
        && title.is_none()
        && note.is_none()
        && start_time.is_none()
        && sort_order.is_none()
        && duration_minutes.is_none()
        && travel_minutes.is_none()
        && location.is_none()
        && category.is_none()
    {
        anyhow::bail!(
            "更新する項目を1つ以上指定してください \
             (--day, --title, --note, --time, --order, --duration, --travel, --location, --category)"
        );
    }

    let mut item = get_itinerary_item(conn, id)?;
    let mut day_id = crate::day::find_day_id_by_trip_and_day_number(conn, item.trip_id, item.day)?;
    if let Some(d) = day {
        day_id = crate::day::find_day_id_by_trip_and_day_number(conn, item.trip_id, d)?;
        item.day = d;
    }
    if let Some(t) = title {
        item.title = t.to_string();
    }
    if let Some(n) = note {
        item.note = n.map(str::to_string);
    }
    if let Some(t) = start_time {
        if let Some(time_str) = t {
            parse_time_hhmm(time_str)?;
        }
        item.start_time = t.map(str::to_string);
    }
    if let Some(o) = sort_order {
        item.sort_order = o;
    }
    if let Some(d) = duration_minutes {
        item.duration_minutes = Some(d);
    }
    if let Some(t) = travel_minutes {
        item.travel_minutes = Some(t);
    }
    if let Some(l) = location {
        item.location = l.map(str::to_string);
    }
    if let Some(c) = category {
        item.category = c;
    }

    let now = crate::db::now_string();
    let category_db = item.category.map(|c| c.as_str().to_string());
    conn.execute(
        "UPDATE itinerary_items
         SET day_id = ?1, day = ?2, title = ?3, note = ?4, start_time = ?5, sort_order = ?6,
             duration_minutes = ?7, travel_minutes = ?8, location = ?9, category = ?10,
             updated_at = ?11
         WHERE id = ?12",
        params![
            day_id,
            item.day,
            item.title,
            item.note,
            item.start_time,
            item.sort_order,
            item.duration_minutes,
            item.travel_minutes,
            item.location,
            category_db,
            &now,
            id
        ],
    )
    .context("日程の更新に失敗しました")?;
    Ok(())
}

/// 日程を削除する
pub(crate) fn delete_itinerary_item(conn: &Connection, id: i64) -> Result<()> {
    get_itinerary_item(conn, id)?;
    crate::db::with_transaction(conn, "itinerary delete", |tx| {
        crate::note::delete_notes_for_itinerary(tx, id)?;
        tx.execute("DELETE FROM itinerary_items WHERE id = ?1", params![id])
            .context("日程の削除に失敗しました")?;
        Ok(())
    })
}

/// rusqlite の行データを ItineraryItem 構造体に変換する
pub(crate) fn row_to_itinerary_item(row: &rusqlite::Row) -> rusqlite::Result<ItineraryItem> {
    let category_raw: Option<String> = row.get(10)?;
    let category = match &category_raw {
        None => None,
        Some(value) => Some(crate::models::parse_itinerary_category(value).map_err(|_| {
            rusqlite::Error::InvalidColumnType(10, value.clone(), rusqlite::types::Type::Text)
        })?),
    };
    Ok(ItineraryItem {
        id: row.get(0)?,
        trip_id: row.get(1)?,
        day: row.get(2)?,
        title: row.get(3)?,
        note: row.get(4)?,
        start_time: row.get(5)?,
        sort_order: row.get(6)?,
        duration_minutes: row.get(7)?,
        travel_minutes: row.get(8)?,
        location: row.get(9)?,
        category,
        created_at: row.get(11)?,
        updated_at: row.get(12)?,
    })
}
/// テキストを表示用に整形する（未設定なら "-"）
pub(crate) fn fmt_text(text: &Option<String>) -> &str {
    text.as_deref().unwrap_or("-")
}

/// 分数を表示用に整形する（未設定なら "-"）
pub(crate) fn fmt_minutes(minutes: Option<i64>) -> String {
    match minutes {
        Some(m) => format!("{m}分"),
        None => "-".to_string(),
    }
}

/// HH:MM 形式を検証し、(時, 分) を返す
pub(crate) fn parse_time_hhmm(time: &str) -> Result<(i32, i32)> {
    let parts: Vec<&str> = time.split(':').collect();
    if parts.len() != 2 || parts[0].len() != 2 || parts[1].len() != 2 {
        anyhow::bail!("時刻は HH:MM 形式で指定してください: {time}");
    }
    let hour: i32 = parts[0]
        .parse()
        .with_context(|| format!("不正な時刻です: {time}"))?;
    let minute: i32 = parts[1]
        .parse()
        .with_context(|| format!("不正な時刻です: {time}"))?;
    if !(0..=23).contains(&hour) || !(0..=59).contains(&minute) {
        anyhow::bail!("不正な時刻です: {time}");
    }
    Ok((hour, minute))
}

/// HH:MM に分数を加算した時刻を返す（日をまたぐ計算はしない）
pub(crate) fn add_minutes_to_time(time: &str, minutes: i64) -> Result<String> {
    let (hour, minute) = parse_time_hhmm(time)?;
    let total = hour * 60 + minute + minutes as i32;
    if total < 0 {
        anyhow::bail!("時刻の計算結果が不正です");
    }
    let new_hour = total / 60;
    let new_minute = total % 60;
    if new_hour >= 24 {
        anyhow::bail!("終了予定時刻が24時を超えました（日跨ぎには未対応です）");
    }
    Ok(format!("{new_hour:02}:{new_minute:02}"))
}
/// 日程一覧を表形式で表示する
pub(crate) fn print_itinerary_list(items: &[ItineraryItem]) {
    if items.is_empty() {
        println!("日程はまだ登録されていません。");
        return;
    }

    println!(
        "{:<6} {:<6} {:<8} {:<14} {:<20} {:<8} {:<8} {:<12}",
        "ID", "日目", "時刻", "タイトル", "場所", "所要", "移動", "メモ"
    );
    println!("{}", "-".repeat(90));
    for item in items {
        println!(
            "{:<6} {:<6} {:<8} {:<14} {:<20} {:<8} {:<8} {:<12}",
            item.id,
            item.day,
            fmt_text(&item.start_time),
            item.title,
            fmt_text(&item.location),
            fmt_minutes(item.duration_minutes),
            fmt_minutes(item.travel_minutes),
            fmt_text(&item.note),
        );
    }
    println!();
    println!("合計: {} 件", items.len());
}

/// 日程の詳細を表示する
pub(crate) fn print_itinerary_detail(item: &ItineraryItem) {
    println!("ID        : {}", item.id);
    println!("旅行 ID   : {}", item.trip_id);
    println!("日目      : {}", item.day);
    println!("時刻      : {}", fmt_text(&item.start_time));
    println!("並び順    : {}", item.sort_order);
    println!("所要時間  : {}", fmt_minutes(item.duration_minutes));
    println!("移動時間  : {}", fmt_minutes(item.travel_minutes));
    println!("タイトル  : {}", item.title);
    if let Some(category) = item.category {
        println!("Category  : {}", category.as_str());
    }
    println!("場所      : {}", fmt_text(&item.location));
    println!("メモ      : {}", fmt_text(&item.note));
    println!("作成日時  : {}", item.created_at);
    println!("更新日時  : {}", item.updated_at);
}

/// 旅行のタイムラインを表示する
pub(crate) fn print_itinerary_timeline(items: &[ItineraryItem]) {
    if items.is_empty() {
        println!("日程はまだ登録されていません。");
        return;
    }

    let mut current_day: Option<i64> = None;
    for (index, item) in items.iter().enumerate() {
        if current_day != Some(item.day) {
            if current_day.is_some() {
                println!();
            }
            println!("Day {}", item.day);
            println!();
            current_day = Some(item.day);
        }

        match &item.start_time {
            Some(time) => {
                println!("{time} {}", item.title);
                if let Some(loc) = &item.location {
                    println!("  場所: {loc}");
                }
                if let Some(duration) = item.duration_minutes {
                    println!("  所要時間: {duration}分");
                    if let Ok(end_time) = add_minutes_to_time(time, duration) {
                        println!("  終了予定: {end_time}");
                    }
                }
            }
            None => {
                println!("時刻: 未定");
                println!("{}", item.title);
                if let Some(loc) = &item.location {
                    println!("  場所: {loc}");
                }
                if let Some(duration) = item.duration_minutes {
                    println!("  所要時間: {duration}分");
                }
            }
        }

        // 次の予定への移動時間を表示（同じ日の次の予定がある場合）
        if let Some(travel) = item.travel_minutes {
            let has_next_same_day = items
                .get(index + 1)
                .is_some_and(|next| next.day == item.day);
            if has_next_same_day {
                println!();
                println!("  ↓ 移動 {travel}分");
                println!();
            }
        } else if items
            .get(index + 1)
            .is_some_and(|next| next.day == item.day)
        {
            println!();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::open_db_at;
    use crate::models::ItineraryCategory;
    use crate::trip::add_test_trip;
    use rusqlite::Connection;

    fn test_db() -> Connection {
        open_db_at(":memory:").expect("インメモリ DB の作成に失敗")
    }

    fn itinerary_category_line(item: &crate::models::ItineraryItem) -> Option<String> {
        item.category.map(|c| format!("Category: {}", c.as_str()))
    }

    #[test]
    fn test_add_itinerary_item() {
        let conn = test_db();
        let trip_id = add_test_trip(&conn, "沖縄旅行").unwrap();

        let id = add_itinerary_item(
            &conn,
            trip_id,
            1,
            "首里城",
            Some("午前"),
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .unwrap();
        assert_eq!(id, 1);

        let item = get_itinerary_item(&conn, id).unwrap();
        assert_eq!(item.trip_id, trip_id);
        assert_eq!(item.day, 1);
        assert_eq!(item.title, "首里城");
        assert_eq!(item.note.as_deref(), Some("午前"));
        assert_eq!(item.sort_order, 0);

        let day_id: i64 = conn
            .query_row(
                "SELECT day_id FROM itinerary_items WHERE id = ?1",
                rusqlite::params![id],
                |row| row.get(0),
            )
            .unwrap();
        let expected_day_id =
            crate::day::find_day_id_by_trip_and_day_number(&conn, trip_id, 1).unwrap();
        assert_eq!(day_id, expected_day_id);
    }

    #[test]
    fn test_add_itinerary_item_rejects_day_outside_trip_range() {
        let conn = test_db();
        let trip_id = add_test_trip(&conn, "Short Trip").unwrap();
        assert!(add_itinerary_item(
            &conn,
            trip_id,
            99,
            "Out of range",
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .is_err());
    }

    #[test]
    fn test_add_itinerary_item_with_duration_and_travel() {
        let conn = test_db();
        let trip_id = add_test_trip(&conn, "沖縄旅行").unwrap();

        let id = add_itinerary_item(
            &conn,
            trip_id,
            1,
            "首里城",
            None,
            Some("09:00"),
            None,
            Some(90),
            Some(20),
            None,
            None,
        )
        .unwrap();

        let item = get_itinerary_item(&conn, id).unwrap();
        assert_eq!(item.duration_minutes, Some(90));
        assert_eq!(item.travel_minutes, Some(20));
    }

    #[test]
    fn test_add_itinerary_item_with_location() {
        let conn = test_db();
        let trip_id = add_test_trip(&conn, "沖縄旅行").unwrap();

        let id = add_itinerary_item(
            &conn,
            trip_id,
            1,
            "首里城",
            None,
            Some("09:00"),
            None,
            None,
            None,
            Some("沖縄県那覇市首里金城町1-2"),
            None,
        )
        .unwrap();

        let item = get_itinerary_item(&conn, id).unwrap();
        assert_eq!(item.location.as_deref(), Some("沖縄県那覇市首里金城町1-2"));
    }

    #[test]
    fn test_add_itinerary_item_with_start_time() {
        let conn = test_db();
        let trip_id = add_test_trip(&conn, "沖縄旅行").unwrap();

        let id = add_itinerary_item(
            &conn,
            trip_id,
            1,
            "首里城",
            None,
            Some("09:00"),
            None,
            None,
            None,
            None,
            None,
        )
        .unwrap();

        let item = get_itinerary_item(&conn, id).unwrap();
        assert_eq!(item.start_time.as_deref(), Some("09:00"));
    }

    #[test]
    fn test_add_itinerary_item_without_start_time() {
        let conn = test_db();
        let trip_id = add_test_trip(&conn, "沖縄旅行").unwrap();

        let id = add_itinerary_item(
            &conn,
            trip_id,
            1,
            "ホテルチェックイン",
            None,
            None,
            Some(99),
            None,
            None,
            None,
            None,
        )
        .unwrap();

        let item = get_itinerary_item(&conn, id).unwrap();
        assert!(item.start_time.is_none());
        assert_eq!(item.sort_order, 99);
    }

    #[test]
    fn test_add_minutes_to_time() {
        assert_eq!(add_minutes_to_time("09:00", 90).unwrap(), "10:30");
        assert_eq!(add_minutes_to_time("12:30", 30).unwrap(), "13:00");
        assert!(parse_time_hhmm("25:00").is_err());
        assert!(parse_time_hhmm("9:00").is_err());
        assert!(add_minutes_to_time("23:00", 120).is_err());
    }

    #[test]
    fn test_clear_itinerary_category() {
        let conn = test_db();
        let trip_id = add_test_trip(&conn, "沖縄旅行").unwrap();
        let id = add_itinerary_item(
            &conn,
            trip_id,
            1,
            "ホテル",
            None,
            None,
            None,
            None,
            None,
            None,
            Some(ItineraryCategory::Hotel),
        )
        .unwrap();

        update_itinerary_item(
            &conn,
            id,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            Some(None),
        )
        .unwrap();

        let item = get_itinerary_item(&conn, id).unwrap();
        assert!(item.category.is_none());
    }

    #[test]
    fn test_delete_itinerary_item() {
        let conn = test_db();
        let trip_id = add_test_trip(&conn, "沖縄旅行").unwrap();
        let id = add_itinerary_item(
            &conn,
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

        delete_itinerary_item(&conn, id).unwrap();

        assert!(list_itinerary_items(&conn, trip_id).unwrap().is_empty());
        assert!(get_itinerary_item(&conn, id).is_err());
    }

    #[test]
    fn test_get_itinerary_item() {
        let conn = test_db();
        let trip_id = add_test_trip(&conn, "沖縄旅行").unwrap();
        let id = add_itinerary_item(
            &conn,
            trip_id,
            1,
            "首里城",
            Some("午前"),
            Some("09:00"),
            None,
            None,
            None,
            None,
            None,
        )
        .unwrap();

        let item = get_itinerary_item(&conn, id).unwrap();
        assert_eq!(item.id, id);
        assert_eq!(item.day, 1);
        assert_eq!(item.title, "首里城");
    }

    #[test]
    fn test_itinerary_show_displays_category() {
        let conn = test_db();
        let trip_id = add_test_trip(&conn, "沖縄旅行").unwrap();
        let id = add_itinerary_item(
            &conn,
            trip_id,
            1,
            "ホテル",
            None,
            None,
            None,
            None,
            None,
            None,
            Some(ItineraryCategory::Hotel),
        )
        .unwrap();

        let item = get_itinerary_item(&conn, id).unwrap();
        assert_eq!(
            itinerary_category_line(&item).as_deref(),
            Some("Category: hotel")
        );
    }

    #[test]
    fn test_itinerary_show_omits_category_when_unset() {
        let conn = test_db();
        let trip_id = add_test_trip(&conn, "沖縄旅行").unwrap();
        let id = add_itinerary_item(
            &conn,
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

        let item = get_itinerary_item(&conn, id).unwrap();
        assert!(itinerary_category_line(&item).is_none());
    }

    #[test]
    fn test_list_itinerary_items() {
        let conn = test_db();
        let trip_id = add_test_trip(&conn, "沖縄旅行").unwrap();
        add_itinerary_item(
            &conn,
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
        add_itinerary_item(
            &conn,
            trip_id,
            2,
            "美ら海水族館",
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .unwrap();

        let items = list_itinerary_items(&conn, trip_id).unwrap();
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].title, "首里城");
        assert_eq!(items[1].title, "美ら海水族館");
    }

    #[test]
    fn test_list_itinerary_items_sorted_by_day_and_time() {
        let conn = test_db();
        let trip_id = add_test_trip(&conn, "沖縄旅行").unwrap();

        // 登録順をバラバラにしても、一覧は day → 時刻順になること
        add_itinerary_item(
            &conn,
            trip_id,
            1,
            "昼食",
            None,
            Some("12:30"),
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
            1,
            "首里城",
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
            1,
            "ホテル",
            None,
            None,
            Some(99),
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
            "2日目",
            None,
            Some("10:00"),
            None,
            None,
            None,
            None,
            None,
        )
        .unwrap();

        let items = list_itinerary_items(&conn, trip_id).unwrap();
        assert_eq!(items.len(), 4);
        assert_eq!(items[0].title, "首里城");
        assert_eq!(items[1].title, "昼食");
        assert_eq!(items[2].title, "ホテル");
        assert_eq!(items[3].title, "2日目");
    }

    #[test]
    fn test_timeline_items_sorted_by_day_and_time() {
        let conn = test_db();
        let trip_id = add_test_trip(&conn, "沖縄旅行").unwrap();

        add_itinerary_item(
            &conn,
            trip_id,
            1,
            "国際通り",
            None,
            Some("10:50"),
            None,
            Some(60),
            None,
            None,
            None,
        )
        .unwrap();
        add_itinerary_item(
            &conn,
            trip_id,
            1,
            "首里城",
            None,
            Some("09:00"),
            None,
            Some(90),
            Some(20),
            None,
            None,
        )
        .unwrap();
        add_itinerary_item(
            &conn,
            trip_id,
            2,
            "2日目",
            None,
            Some("10:00"),
            None,
            None,
            None,
            None,
            None,
        )
        .unwrap();

        let items = list_itinerary_items(&conn, trip_id).unwrap();
        assert_eq!(items.len(), 3);
        assert_eq!(items[0].title, "首里城");
        assert_eq!(items[1].title, "国際通り");
        assert_eq!(items[2].title, "2日目");
        assert_eq!(items[0].day, 1);
        assert_eq!(items[0].start_time.as_deref(), Some("09:00"));
    }

    #[test]
    fn test_update_itinerary_item() {
        let conn = test_db();
        let trip_id = add_test_trip(&conn, "沖縄旅行").unwrap();
        let id = add_itinerary_item(
            &conn,
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

        update_itinerary_item(
            &conn,
            id,
            Some(2),
            Some("美ら海水族館"),
            Some(Some("終日")),
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .unwrap();

        let item = get_itinerary_item(&conn, id).unwrap();
        assert_eq!(item.day, 2);
        assert_eq!(item.title, "美ら海水族館");
        assert_eq!(item.note.as_deref(), Some("終日"));

        let day_id: i64 = conn
            .query_row(
                "SELECT day_id FROM itinerary_items WHERE id = ?1",
                rusqlite::params![id],
                |row| row.get(0),
            )
            .unwrap();
        let expected_day_id =
            crate::day::find_day_id_by_trip_and_day_number(&conn, trip_id, 2).unwrap();
        assert_eq!(day_id, expected_day_id);
    }

    #[test]
    fn test_update_itinerary_item_category() {
        let conn = test_db();
        let trip_id = add_test_trip(&conn, "沖縄旅行").unwrap();
        let id = add_itinerary_item(
            &conn,
            trip_id,
            1,
            "Hilton Hawaiian Village",
            None,
            None,
            None,
            None,
            None,
            Some("Waikiki"),
            None,
        )
        .unwrap();

        update_itinerary_item(
            &conn,
            id,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            Some(Some(ItineraryCategory::Hotel)),
        )
        .unwrap();

        let item = get_itinerary_item(&conn, id).unwrap();
        assert_eq!(item.category, Some(ItineraryCategory::Hotel));
    }

    #[test]
    fn test_update_itinerary_item_duration_and_travel() {
        let conn = test_db();
        let trip_id = add_test_trip(&conn, "沖縄旅行").unwrap();
        let id = add_itinerary_item(
            &conn,
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

        update_itinerary_item(
            &conn,
            id,
            None,
            None,
            None,
            None,
            None,
            Some(90),
            Some(20),
            None,
            None,
        )
        .unwrap();

        let item = get_itinerary_item(&conn, id).unwrap();
        assert_eq!(item.duration_minutes, Some(90));
        assert_eq!(item.travel_minutes, Some(20));
    }

    #[test]
    fn test_update_itinerary_item_location() {
        let conn = test_db();
        let trip_id = add_test_trip(&conn, "沖縄旅行").unwrap();
        let id = add_itinerary_item(
            &conn,
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

        update_itinerary_item(
            &conn,
            id,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            Some(Some("沖縄県那覇市首里金城町1-2")),
            None,
        )
        .unwrap();

        let item = get_itinerary_item(&conn, id).unwrap();
        assert_eq!(item.location.as_deref(), Some("沖縄県那覇市首里金城町1-2"));
    }

    #[test]
    fn test_update_itinerary_item_start_time_and_sort_order() {
        let conn = test_db();
        let trip_id = add_test_trip(&conn, "沖縄旅行").unwrap();
        let id = add_itinerary_item(
            &conn,
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

        update_itinerary_item(
            &conn,
            id,
            None,
            None,
            None,
            Some(Some("09:30")),
            Some(5),
            None,
            None,
            None,
            None,
        )
        .unwrap();

        let item = get_itinerary_item(&conn, id).unwrap();
        assert_eq!(item.start_time.as_deref(), Some("09:30"));
        assert_eq!(item.sort_order, 5);
    }

    #[test]
    fn test_itinerary_list_json_empty() {
        let conn = test_db();
        let trip_id = add_test_trip(&conn, "沖縄旅行").unwrap();

        let items = list_itinerary_items(&conn, trip_id).unwrap();
        let json = serde_json::to_string_pretty(&items).unwrap();

        assert_eq!(json, "[]");
    }

    #[test]
    fn test_itinerary_list_json() {
        let conn = test_db();
        let trip_id = add_test_trip(&conn, "沖縄旅行").unwrap();
        add_itinerary_item(
            &conn,
            trip_id,
            1,
            "首里城",
            None,
            Some("09:00"),
            None,
            Some(90),
            Some(20),
            Some("那覇市"),
            Some(ItineraryCategory::Activity),
        )
        .unwrap();
        add_itinerary_item(
            &conn,
            trip_id,
            2,
            "美ら海",
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .unwrap();

        let items = list_itinerary_items(&conn, trip_id).unwrap();
        let json = serde_json::to_string_pretty(&items).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        assert!(parsed.is_array());
        assert_eq!(parsed.as_array().unwrap().len(), 2);
        assert_eq!(parsed[0]["title"], "首里城");
        assert_eq!(parsed[0]["category"], "activity");
        assert_eq!(parsed[1]["title"], "美ら海");
        assert!(parsed[1].get("category").is_none());
    }

    #[test]
    fn test_itinerary_show_json() {
        let conn = test_db();
        let trip_id = add_test_trip(&conn, "沖縄旅行").unwrap();
        let id = add_itinerary_item(
            &conn,
            trip_id,
            1,
            "首里城",
            Some("見学"),
            Some("09:00"),
            Some(1),
            Some(90),
            Some(20),
            Some("那覇市"),
            Some(ItineraryCategory::Museum),
        )
        .unwrap();

        let item = get_itinerary_item(&conn, id).unwrap();
        let json = serde_json::to_string_pretty(&item).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed["id"], id);
        assert_eq!(parsed["trip_id"], trip_id);
        assert_eq!(parsed["day"], 1);
        assert_eq!(parsed["title"], "首里城");
        assert_eq!(parsed["note"], "見学");
        assert_eq!(parsed["start_time"], "09:00");
        assert_eq!(parsed["sort_order"], 1);
        assert_eq!(parsed["duration_minutes"], 90);
        assert_eq!(parsed["travel_minutes"], 20);
        assert_eq!(parsed["location"], "那覇市");
        assert_eq!(parsed["category"], "museum");
    }

    #[test]
    fn test_get_itinerary_item_not_found() {
        let conn = test_db();
        let err = get_itinerary_item(&conn, 9999)
            .err()
            .expect("expected error");
        assert_eq!(err.to_string(), "Itinerary not found: 9999");
        assert!(!format!("{err:#}").contains("Query returned no rows"));
    }
}
