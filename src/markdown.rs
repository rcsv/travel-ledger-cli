use anyhow::{Context, Result};
use rusqlite::{params, Connection};

use crate::models::{ChecklistItem, ItineraryItem, Trip};
use crate::stats::{format_minutes_duration, TripStats};

/// Markdown 出力用に日程一覧を取得する（day → sort_order → id 順）
pub(crate) fn list_itinerary_items_for_markdown(
    conn: &Connection,
    trip_id: i64,
) -> Result<Vec<ItineraryItem>> {
    crate::trip::get_trip(conn, trip_id)?;
    let mut stmt = conn
        .prepare(
            "SELECT id, trip_id, day, title, note, start_time, sort_order,
                    duration_minutes, travel_minutes, location, category, created_at, updated_at
             FROM itinerary_items
             WHERE trip_id = ?1
             ORDER BY day, sort_order, id",
        )
        .context("日程一覧取得の準備に失敗しました")?;

    let items = stmt
        .query_map(params![trip_id], crate::itinerary::row_to_itinerary_item)
        .context("日程一覧取得に失敗しました")?
        .collect::<std::result::Result<Vec<_>, _>>()
        .context("日程一覧の読み込みに失敗しました")?;

    Ok(items)
}

/// 旅行の日付範囲を Markdown 用の1行テキストに整形する
pub(crate) fn format_trip_date_range(trip: &Trip) -> Option<String> {
    match (&trip.start_date, &trip.end_date) {
        (Some(start), Some(end)) => Some(format!("{start} 〜 {end}")),
        (Some(start), None) => Some(start.clone()),
        (None, Some(end)) => Some(end.clone()),
        (None, None) => None,
    }
}

/// 1件の日程を Markdown 形式に整形する
pub(crate) fn format_itinerary_item_markdown(item: &ItineraryItem) -> String {
    let mut lines = Vec::new();
    let heading = match &item.start_time {
        Some(time) => format!("### {time} {}", item.title),
        None => format!("### {}", item.title),
    };
    lines.push(heading);

    let mut detail_lines = Vec::new();
    if let Some(category) = item.category {
        detail_lines.push(format!("- Category: {}", category.as_str()));
    }
    if let Some(location) = &item.location {
        detail_lines.push(format!("- 場所: {location}"));
    }
    if let Some(duration) = item.duration_minutes {
        detail_lines.push(format!("- 所要時間: {duration}分"));
    }
    if let Some(travel) = item.travel_minutes {
        detail_lines.push(format!("- 移動時間: {travel}分"));
    }
    if let Some(note) = &item.note {
        detail_lines.push(format!("- メモ: {note}"));
    }

    if !detail_lines.is_empty() {
        lines.push(String::new());
        lines.extend(detail_lines);
    }

    lines.join("\n")
}

/// チェックリスト一覧を Markdown 形式に整形する（項目がなければ None）
pub(crate) fn format_checklist_markdown(items: &[ChecklistItem]) -> Option<String> {
    if items.is_empty() {
        return None;
    }

    let mut lines = vec!["## Checklist".to_string(), String::new()];
    for item in items {
        let mark = if item.is_done { 'x' } else { ' ' };
        lines.push(format!("- [{mark}] {}", item.title));
    }
    Some(format!("\n\n{}\n", lines.join("\n")))
}

/// Overview セクション（旅行統計サマリー）を Markdown に追記する
fn append_overview_section(output: &mut String, stats: &TripStats) {
    output.push_str("\n## Overview\n\n");
    output.push_str(&format!("- Days: {}\n", stats.days));
    output.push_str(&format!("- Itineraries: {}\n", stats.itinerary_count));
    output.push_str(&format!(
        "- Checklist: {} / {} completed\n",
        stats.checklist_completed, stats.checklist_total
    ));
    output.push_str(&format!(
        "- Stay Time: {}\n",
        format_minutes_duration(stats.stay_minutes)
    ));
    output.push_str(&format!(
        "- Travel Time: {}\n",
        format_minutes_duration(stats.travel_minutes)
    ));
    output.push_str(&format!(
        "- Total Time: {}\n",
        format_minutes_duration(stats.total_minutes())
    ));
}

/// 旅行と日程一覧から Markdown 文字列を組み立てる
pub(crate) fn format_trip_markdown(
    trip: &Trip,
    items: &[ItineraryItem],
    checklist: &[ChecklistItem],
    stats: &TripStats,
) -> String {
    let mut output = format!("# {}\n", trip.name);
    if let Some(dates) = format_trip_date_range(trip) {
        output.push('\n');
        output.push_str(&dates);
        output.push('\n');
    }

    append_overview_section(&mut output, stats);

    let mut current_day: Option<i64> = None;
    for item in items {
        if current_day != Some(item.day) {
            if current_day.is_some() {
                output.push_str("\n\n");
            } else {
                output.push('\n');
            }
            output.push_str(&format!("## Day {}\n\n", item.day));
            current_day = Some(item.day);
        } else {
            output.push_str("\n\n");
        }
        output.push_str(&format_itinerary_item_markdown(item));
    }

    if let Some(checklist_md) = format_checklist_markdown(checklist) {
        output.push_str(&checklist_md);
    }

    output
}

/// 旅行しおりを Markdown 文字列として組み立てる
pub(crate) fn generate_trip_markdown(conn: &Connection, trip_id: i64) -> Result<String> {
    let trip = crate::trip::get_trip(conn, trip_id)?;
    let items = list_itinerary_items_for_markdown(conn, trip_id)?;
    let checklist = crate::checklist::list_checklist_items(conn, trip_id)?;
    let stats = crate::stats::compute_trip_stats(conn, trip_id)?;
    Ok(format_trip_markdown(&trip, &items, &checklist, &stats))
}

/// Markdown を標準出力に出力する
pub(crate) fn print_markdown_to_stdout(markdown: &str) {
    println!("{markdown}");
}

/// Markdown をファイルに書き込む（既存ファイルは上書き）
pub(crate) fn write_markdown_to_file(path: &str, markdown: &str) -> Result<()> {
    std::fs::write(path, markdown)
        .with_context(|| format!("ファイル '{path}' への書き込みに失敗しました"))?;
    println!("Markdown exported: {path}");
    Ok(())
}

/// 旅行しおりを Markdown で出力する（ファイルまたは標準出力）
pub(crate) fn write_trip_markdown(
    conn: &Connection,
    trip_id: i64,
    output: Option<&str>,
) -> Result<()> {
    let markdown = generate_trip_markdown(conn, trip_id)?;
    match output {
        Some(path) => write_markdown_to_file(path, &markdown),
        None => {
            print_markdown_to_stdout(&markdown);
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::checklist::{add_checklist_item, set_checklist_done};
    use crate::db::open_db_at;
    use crate::itinerary::add_itinerary_item;
    use crate::models::ItineraryCategory;
    use crate::trip::{add_test_trip, add_trip};
    use rusqlite::Connection;

    fn test_db() -> Connection {
        open_db_at(":memory:").expect("インメモリ DB の作成に失敗")
    }

    #[test]
    fn test_export_md_day_and_sort_order() {
        let conn = test_db();
        let trip_id = add_test_trip(&conn, "並び順テスト").unwrap();

        add_itinerary_item(
            &conn,
            trip_id,
            2,
            "2日目・後",
            None,
            Some("14:00"),
            Some(2),
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
            "2日目・先",
            None,
            Some("09:00"),
            Some(1),
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
            "1日目",
            None,
            Some("10:00"),
            Some(1),
            None,
            None,
            None,
            None,
        )
        .unwrap();

        let md = generate_trip_markdown(&conn, trip_id).unwrap();
        let day1_pos = md.find("## Day 1").unwrap();
        let day2_pos = md.find("## Day 2").unwrap();
        let first_item_pos = md.find("### 10:00 1日目").unwrap();
        let second_day_first_pos = md.find("### 09:00 2日目・先").unwrap();
        let second_day_second_pos = md.find("### 14:00 2日目・後").unwrap();

        assert!(day1_pos < day2_pos);
        assert!(day1_pos < first_item_pos);
        assert!(second_day_first_pos < second_day_second_pos);
    }

    #[test]
    fn test_export_md_includes_category() {
        let conn = test_db();
        let trip_id = add_test_trip(&conn, "ハワイ旅行").unwrap();
        add_itinerary_item(
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
            Some(ItineraryCategory::Hotel),
        )
        .unwrap();

        let md = generate_trip_markdown(&conn, trip_id).unwrap();
        assert!(md.contains("### Hilton Hawaiian Village"));
        assert!(md.contains("- Category: hotel"));
        assert!(md.contains("- 場所: Waikiki"));
        let category_pos = md.find("- Category: hotel").unwrap();
        let location_pos = md.find("- 場所: Waikiki").unwrap();
        assert!(category_pos < location_pos);
    }

    #[test]
    fn test_export_md_includes_checklist() {
        let conn = test_db();
        let trip_id = add_test_trip(&conn, "沖縄旅行").unwrap();
        add_checklist_item(&conn, trip_id, "パスポート").unwrap();
        let charger_id = add_checklist_item(&conn, trip_id, "充電器").unwrap();
        set_checklist_done(&conn, charger_id, true).unwrap();

        let md = generate_trip_markdown(&conn, trip_id).unwrap();
        assert!(md.contains("## Checklist"));
        assert!(md.contains("- [ ] パスポート"));
        assert!(md.contains("- [x] 充電器"));
        assert!(md.find("## Checklist").unwrap() > md.find("# 沖縄旅行").unwrap());

        // 一覧表示と同じ並び（未完了が先）
        let passport_pos = md.find("- [ ] パスポート").unwrap();
        let charger_pos = md.find("- [x] 充電器").unwrap();
        assert!(passport_pos < charger_pos);
    }

    #[test]
    fn test_export_md_no_checklist_section() {
        let conn = test_db();
        let trip_id = add_test_trip(&conn, "沖縄旅行").unwrap();

        let md = generate_trip_markdown(&conn, trip_id).unwrap();
        assert!(!md.contains("## Checklist"));
    }
    #[test]
    fn test_export_md_omits_category_when_unset() {
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

        let md = generate_trip_markdown(&conn, trip_id).unwrap();
        assert!(!md.contains("- Category:"));
    }

    #[test]
    fn test_export_md_optional_fields_omitted() {
        let conn = test_db();
        let trip_id = add_test_trip(&conn, "ミニマル旅行").unwrap();
        add_itinerary_item(
            &conn,
            trip_id,
            1,
            "散歩",
            None,
            None,
            Some(1),
            None,
            None,
            None,
            None,
        )
        .unwrap();

        let md = generate_trip_markdown(&conn, trip_id).unwrap();
        assert!(md.contains("### 散歩"));
        assert!(!md.contains("- 場所:"));
        assert!(!md.contains("- 所要時間:"));
        assert!(!md.contains("- 移動時間:"));
        assert!(!md.contains("- メモ:"));
    }

    #[test]
    fn test_export_md_start_time_with_and_without() {
        let conn = test_db();
        let trip_id = add_test_trip(&conn, "テスト旅行").unwrap();
        add_itinerary_item(
            &conn,
            trip_id,
            1,
            "朝食",
            None,
            Some("08:00"),
            Some(1),
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
            "自由時間",
            None,
            None,
            Some(2),
            None,
            None,
            None,
            None,
        )
        .unwrap();

        let md = generate_trip_markdown(&conn, trip_id).unwrap();
        assert!(md.contains("### 08:00 朝食"));
        assert!(md.contains("### 自由時間"));
        assert!(!md.contains("### 自由時間 自由時間"));
    }

    #[test]
    fn test_export_md_with_itinerary() {
        let conn = test_db();
        let trip_id = add_trip(&conn, "沖縄旅行", "2026-04-26", "2026-04-29").unwrap();
        add_itinerary_item(
            &conn,
            trip_id,
            1,
            "那覇空港",
            Some("レンタカー受け取り"),
            Some("09:00"),
            Some(1),
            Some(60),
            Some(30),
            Some("那覇空港"),
            None,
        )
        .unwrap();

        let md = generate_trip_markdown(&conn, trip_id).unwrap();
        assert!(md.contains("# 沖縄旅行"));
        assert!(md.contains("2026-04-26 〜 2026-04-29"));
        assert!(md.contains("## Day 1"));
        assert!(md.contains("### 09:00 那覇空港"));
        assert!(md.contains("- 場所: 那覇空港"));
        assert!(md.contains("- 所要時間: 60分"));
        assert!(md.contains("- 移動時間: 30分"));
        assert!(md.contains("- メモ: レンタカー受け取り"));
    }

    #[test]
    fn test_export_md_includes_overview() {
        let conn = test_db();
        let trip_id = add_trip(&conn, "沖縄旅行", "2026-04-26", "2026-04-29").unwrap();
        add_itinerary_item(
            &conn,
            trip_id,
            1,
            "那覇空港",
            None,
            Some("09:00"),
            Some(1),
            Some(60),
            Some(30),
            None,
            None,
        )
        .unwrap();
        add_itinerary_item(
            &conn,
            trip_id,
            2,
            "首里城",
            None,
            Some("10:00"),
            Some(1),
            Some(90),
            Some(20),
            None,
            None,
        )
        .unwrap();
        add_checklist_item(&conn, trip_id, "パスポート").unwrap();
        let charger_id = add_checklist_item(&conn, trip_id, "充電器").unwrap();
        set_checklist_done(&conn, charger_id, true).unwrap();

        let md = generate_trip_markdown(&conn, trip_id).unwrap();
        assert!(md.contains("## Overview"));
        assert!(md.contains("- Days: 2"));
        assert!(md.contains("- Itineraries: 2"));
        assert!(md.contains("- Checklist: 1 / 2 completed"));
        assert!(md.contains("- Stay Time: 2h30m"));
        assert!(md.contains("- Travel Time: 50m"));
        assert!(md.contains("- Total Time: 3h20m"));
        assert!(!md.contains("Category Breakdown"));
        assert!(!md.contains("uncategorized"));

        let overview_pos = md.find("## Overview").unwrap();
        let day1_pos = md.find("## Day 1").unwrap();
        assert!(overview_pos < day1_pos);
    }

    #[test]
    fn test_export_md_overview_checklist_zero() {
        let conn = test_db();
        let trip_id = add_test_trip(&conn, "沖縄旅行").unwrap();

        let md = generate_trip_markdown(&conn, trip_id).unwrap();
        assert!(md.contains("## Overview"));
        assert!(md.contains("- Checklist: 0 / 0 completed"));
    }

    #[test]
    fn test_write_trip_markdown_to_file() {
        let conn = test_db();
        let trip_id = add_test_trip(&conn, "ファイル出力テスト").unwrap();
        add_itinerary_item(
            &conn,
            trip_id,
            1,
            "首里城",
            None,
            Some("09:00"),
            Some(1),
            None,
            None,
            None,
            None,
        )
        .unwrap();

        let path =
            std::env::temp_dir().join(format!("caglla-export-md-test-{}.md", std::process::id()));
        let path_str = path.to_str().expect("一時ファイルパスが不正です");

        write_trip_markdown(&conn, trip_id, Some(path_str)).unwrap();

        let content =
            std::fs::read_to_string(path_str).expect("書き込んだファイルの読み込みに失敗");
        assert!(content.contains("# ファイル出力テスト"));
        assert!(content.contains("### 09:00 首里城"));

        std::fs::remove_file(path_str).ok();
    }

    #[test]
    fn test_write_trip_markdown_none_succeeds() {
        let conn = test_db();
        let trip_id = add_test_trip(&conn, "標準出力テスト").unwrap();
        write_trip_markdown(&conn, trip_id, None).unwrap();
    }
}
