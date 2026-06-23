use std::collections::HashMap;

use anyhow::{Context, Result};
use rusqlite::Connection;

use crate::analysis::statistics::{
    compute_difference_totals, format_minutes_duration, sum_estimate_totals_by_currency,
    sum_expense_totals_by_currency, TripStats,
};
use crate::domain::models::{
    ChecklistItem, Day, Estimate, Expense, ItineraryItem, Participant, Trip,
};
use crate::reservation::ReservationWithContext;

/// Markdown 出力用に日程一覧を取得する（`list_itinerary_items` と同一順序）
pub(crate) fn list_itinerary_items_for_markdown(
    conn: &Connection,
    trip_id: i64,
) -> Result<Vec<ItineraryItem>> {
    crate::itinerary::list_itinerary_items(conn, trip_id)
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
pub(crate) fn format_itinerary_item_markdown(
    conn: &Connection,
    item: &ItineraryItem,
    estimates: &[Estimate],
    expenses: &[Expense],
) -> Result<String> {
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

    if let Some(estimates_md) = crate::estimate::format_estimates_markdown_section(estimates) {
        lines.push(estimates_md);
    }

    if !expenses.is_empty() {
        lines.push(String::new());
        lines.push("Expenses:".to_string());
        for expense in expenses {
            lines.push(crate::expense::format_expense_markdown_line(conn, expense)?);
        }
    }

    append_itinerary_planned_actual_difference(&mut lines, estimates, expenses);

    Ok(lines.join("\n"))
}

fn append_itinerary_planned_actual_difference(
    lines: &mut Vec<String>,
    estimates: &[Estimate],
    expenses: &[Expense],
) {
    let estimate_totals = sum_estimate_totals_by_currency(estimates);
    let expense_totals = sum_expense_totals_by_currency(expenses);
    let Some(difference_totals) = compute_difference_totals(
        estimates.len(),
        expenses.len(),
        &estimate_totals,
        &expense_totals,
    ) else {
        return;
    };

    lines.push(String::new());
    lines.push("Planned total:".to_string());
    for (currency, total) in &estimate_totals {
        lines.push(format!(
            "  - {} {}",
            currency,
            crate::money::format_amount_value(*total, currency)
        ));
    }
    lines.push("Actual total:".to_string());
    for (currency, total) in &expense_totals {
        lines.push(format!(
            "  - {} {}",
            currency,
            crate::money::format_amount_value(*total, currency)
        ));
    }
    lines.push("Difference:".to_string());
    for (currency, total) in &difference_totals {
        lines.push(format!(
            "  - {} {}",
            currency,
            crate::money::format_amount_value(*total, currency)
        ));
    }
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

/// Trip 全体の Reservation セクションを Markdown 形式に整形する（0 件なら None）
pub(crate) fn format_reservations_markdown(
    reservations: &[ReservationWithContext],
) -> Option<String> {
    if reservations.is_empty() {
        return None;
    }

    let mut lines = vec!["## Reservations".to_string(), String::new()];
    let mut current_type: Option<String> = None;
    for row in reservations {
        let res = &row.reservation;
        if current_type.as_deref() != Some(res.reservation_type.as_str()) {
            if current_type.is_some() {
                lines.push(String::new());
            }
            lines.push(format!(
                "### {}",
                crate::reservation::format_reservation_type_display(&res.reservation_type)
            ));
            lines.push(String::new());
            current_type = Some(res.reservation_type.clone());
        }

        lines.push(format!(
            "**Day {} / {}** — {}",
            row.day_number, row.itinerary_title, res.provider_name
        ));
        lines.push(format!("Provider: {}", res.provider_name));
        if let Some(code) = &res.confirmation_code {
            lines.push(format!("Confirmation: {code}"));
        }
        if let Some(url) = &res.reservation_site_url {
            lines.push(format!("URL: {url}"));
        }
        if let Some(remark) = &res.remark {
            lines.push(format!("Remark: {remark}"));
        }
        let period = crate::reservation::format_period(&res.start_at, &res.end_at);
        if period != "-" {
            lines.push(format!("Period: {period}"));
        }
        lines.push(String::new());
    }

    Some(format!("\n\n{}\n", lines.join("\n").trim_end()))
}

fn append_participants_section(output: &mut String, participants: &[Participant]) {
    if participants.is_empty() {
        return;
    }
    output.push_str("\n## Participants\n\n");
    output.push_str("| # | Name | Self |\n");
    output.push_str("|---|------|------|\n");
    for (index, participant) in participants.iter().enumerate() {
        let self_mark = if participant.is_self { "yes" } else { "no" };
        output.push_str(&format!(
            "| {} | {} | {} |\n",
            index + 1,
            participant.name,
            self_mark
        ));
    }
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
    if stats.participants_recorded {
        if stats.self_known {
            let travelers = stats
                .traveler_count
                .or(stats.participant_count)
                .unwrap_or(stats.registered_participant_count);
            let companions = stats.companion_count.unwrap_or(0);
            output.push_str(&format!(
                "- Travelers: {travelers} (companions: {companions})\n"
            ));
        } else {
            output.push_str(&format!(
                "- Participants: {} recorded (traveler count unknown)\n",
                stats.registered_participant_count
            ));
        }
    }
    if stats.estimate_count > 0 {
        output.push_str(&format!("- Estimates: {}\n", stats.estimate_count));
        output.push_str("- Planned total:\n");
        for (currency, total) in &stats.estimate_totals {
            output.push_str(&format!(
                "  - {} {}\n",
                currency,
                crate::money::format_amount_value(*total, currency)
            ));
        }
    }
    if stats.expense_count > 0 {
        output.push_str(&format!("- Expenses: {}\n", stats.expense_count));
        output.push_str("- Actual total:\n");
        for (currency, total) in &stats.expense_totals {
            output.push_str(&format!(
                "  - {} {}\n",
                currency,
                crate::money::format_amount_value(*total, currency)
            ));
        }
    }
    if let Some(difference_totals) = &stats.difference_totals {
        output.push_str("- Difference:\n");
        for (currency, total) in difference_totals {
            output.push_str(&format!(
                "  - {} {}\n",
                currency,
                crate::money::format_amount_value(*total, currency)
            ));
        }
    }
}

/// 旅行と日程一覧から Markdown 文字列を組み立てる
#[allow(clippy::too_many_arguments)]
pub(crate) fn format_trip_markdown(
    conn: &Connection,
    trip: &Trip,
    days: &[Day],
    items: &[ItineraryItem],
    checklist: &[ChecklistItem],
    participants: &[Participant],
    stats: &TripStats,
    estimates_by_itinerary: &HashMap<i64, Vec<Estimate>>,
    expenses_by_itinerary: &HashMap<i64, Vec<Expense>>,
    reservations: &[ReservationWithContext],
) -> Result<String> {
    let day_summaries: HashMap<i64, Option<String>> = days
        .iter()
        .map(|d| (d.day_number, d.summary.clone()))
        .collect();

    let mut output = format!("# {}\n", trip.name);
    if let Some(summary) = &trip.summary {
        output.push('\n');
        output.push_str(summary);
        output.push('\n');
    }
    if let Some(dates) = format_trip_date_range(trip) {
        output.push('\n');
        output.push_str(&dates);
        output.push('\n');
    }

    append_overview_section(&mut output, stats);
    append_participants_section(&mut output, participants);

    if let Some(reservations_md) = format_reservations_markdown(reservations) {
        output.push_str(&reservations_md);
    }

    let mut current_day: Option<i64> = None;
    for item in items {
        if current_day != Some(item.day) {
            if current_day.is_some() {
                output.push_str("\n\n");
            } else {
                output.push('\n');
            }
            output.push_str(&format!("## Day {}\n\n", item.day));
            if let Some(day_summary) = day_summaries.get(&item.day).and_then(|s| s.as_ref()) {
                output.push_str(day_summary);
                output.push_str("\n\n");
            }
            current_day = Some(item.day);
        } else {
            output.push_str("\n\n");
        }
        let estimates = estimates_by_itinerary
            .get(&item.id)
            .map(|list| list.as_slice())
            .unwrap_or(&[]);
        let expenses = expenses_by_itinerary
            .get(&item.id)
            .map(|list| list.as_slice())
            .unwrap_or(&[]);
        output.push_str(&format_itinerary_item_markdown(
            conn, item, estimates, expenses,
        )?);
    }

    if let Some(checklist_md) = format_checklist_markdown(checklist) {
        output.push_str(&checklist_md);
    }

    Ok(output)
}

/// 旅行しおりを Markdown 文字列として組み立てる
pub(crate) fn generate_trip_markdown(conn: &Connection, trip_id: i64) -> Result<String> {
    let trip = crate::trip::get_trip(conn, trip_id)?;
    let days = crate::day::list_days(conn, trip_id)?;
    let items = list_itinerary_items_for_markdown(conn, trip_id)?;
    let checklist = crate::checklist::list_checklist_items(conn, trip_id)?;
    let stats = crate::analysis::statistics::compute_trip_stats(conn, trip_id)?;
    let mut estimates_by_itinerary: HashMap<i64, Vec<Estimate>> = HashMap::new();
    for estimate in crate::estimate::list_estimates_for_trip(conn, trip_id)? {
        estimates_by_itinerary
            .entry(estimate.itinerary_id)
            .or_default()
            .push(estimate);
    }
    let mut expenses_by_itinerary: HashMap<i64, Vec<Expense>> = HashMap::new();
    for expense in crate::expense::list_expenses_for_trip(conn, trip_id)? {
        expenses_by_itinerary
            .entry(expense.itinerary_id)
            .or_default()
            .push(expense);
    }
    let reservations = crate::reservation::list_reservations_for_trip(conn, trip_id)?;
    let participants = crate::participant::list_participants_by_trip(conn, trip_id)?;
    format_trip_markdown(
        conn,
        &trip,
        &days,
        &items,
        &checklist,
        &participants,
        &stats,
        &estimates_by_itinerary,
        &expenses_by_itinerary,
        &reservations,
    )
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
    use crate::domain::models::ItineraryCategory;
    use crate::itinerary::add_itinerary_item;
    use crate::storage::db::open_db_at;
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
        let trip_id = add_trip(&conn, "沖縄旅行", "2026-04-26", "2026-04-29", None).unwrap();
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
        let trip_id = add_trip(&conn, "沖縄旅行", "2026-04-26", "2026-04-29", None).unwrap();
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

    #[test]
    fn test_export_md_includes_reservations() {
        let conn = test_db();
        let trip_id = add_test_trip(&conn, "Reservation MD Trip").unwrap();
        let itinerary_id = add_itinerary_item(
            &conn,
            trip_id,
            1,
            "Check-in",
            None,
            Some("16:40"),
            Some(0),
            None,
            None,
            Some("Hilton Sesoko"),
            None,
        )
        .unwrap();
        crate::reservation::add_reservation(
            &conn,
            itinerary_id,
            "hotel",
            "Hilton Sesoko Resort",
            Some("ABC123"),
            None,
            Some("Twin room"),
            Some("2026-04-26T16:40"),
            Some("2026-04-29T10:00"),
        )
        .unwrap();

        let md = generate_trip_markdown(&conn, trip_id).unwrap();
        assert!(md.contains("## Reservations"));
        assert!(md.contains("### Hotel"));
        assert!(md.contains("Day 1 / Check-in"));
        assert!(md.contains("Confirmation: ABC123"));
        assert!(md.contains("Period: 2026-04-26T16:40 — 2026-04-29T10:00"));

        let reservations_pos = md.find("## Reservations").unwrap();
        let day1_pos = md.find("## Day 1").unwrap();
        assert!(reservations_pos < day1_pos);
    }

    #[test]
    fn test_export_md_omits_reservations_when_none() {
        let conn = test_db();
        let trip_id = add_test_trip(&conn, "No Reservation Trip").unwrap();
        let md = generate_trip_markdown(&conn, trip_id).unwrap();
        assert!(!md.contains("## Reservations"));
    }

    #[test]
    fn test_export_md_includes_expenses() {
        let conn = test_db();
        let trip_id = add_test_trip(&conn, "Expense MD Trip").unwrap();
        let itinerary_id = add_itinerary_item(
            &conn,
            trip_id,
            1,
            "Aquarium",
            None,
            Some("09:00"),
            Some(0),
            None,
            None,
            None,
            None,
        )
        .unwrap();
        crate::expense::add_expense(
            &conn,
            itinerary_id,
            "2500",
            "JPY",
            Some("入館料"),
            None,
            None,
            None,
            &crate::expense::ExpenseSharedOptions::default(),
        )
        .unwrap();
        crate::expense::add_expense(
            &conn,
            itinerary_id,
            "500",
            "JPY",
            Some("駐車場"),
            None,
            None,
            None,
            &crate::expense::ExpenseSharedOptions::default(),
        )
        .unwrap();

        let md = generate_trip_markdown(&conn, trip_id).unwrap();
        assert!(md.contains("Expenses:"));
        assert!(md.contains("- 入館料: 2,500 JPY"));
        assert!(md.contains("- 駐車場: 500 JPY"));
        let aquarium_pos = md.find("### 09:00 Aquarium").unwrap();
        let expense_line_pos = md.find("- 入館料: 2,500 JPY").unwrap();
        assert!(aquarium_pos < expense_line_pos);
    }

    #[test]
    fn test_export_md_includes_shared_expense_payer_and_beneficiaries() {
        let conn = test_db();
        let trip_id = add_test_trip(&conn, "Shared Expense MD Trip").unwrap();
        let payer =
            crate::participant::create_participant(&conn, trip_id, "Alice", None, true).unwrap();
        let beneficiary =
            crate::participant::create_participant(&conn, trip_id, "Bob", None, false).unwrap();
        let itinerary_id = add_itinerary_item(
            &conn,
            trip_id,
            1,
            "Dinner",
            None,
            Some("18:00"),
            Some(0),
            None,
            None,
            None,
            None,
        )
        .unwrap();
        crate::expense::add_expense(
            &conn,
            itinerary_id,
            "4000",
            "JPY",
            Some("Restaurant"),
            None,
            None,
            None,
            &crate::expense::ExpenseSharedOptions {
                paid_by_participant_id: Some(payer),
                beneficiary_participant_ids: Some(vec![payer, beneficiary]),
                ..crate::expense::ExpenseSharedOptions::default()
            },
        )
        .unwrap();

        let md = generate_trip_markdown(&conn, trip_id).unwrap();
        assert!(md.contains("Paid by: Alice"));
        assert!(md.contains("Shared: Alice, Bob"));
    }

    #[test]
    fn test_export_md_omits_expenses_when_none() {
        let conn = test_db();
        let trip_id = add_test_trip(&conn, "No Expense Trip").unwrap();
        add_itinerary_item(
            &conn,
            trip_id,
            1,
            "Walk",
            None,
            None,
            Some(0),
            None,
            None,
            None,
            None,
        )
        .unwrap();

        let md = generate_trip_markdown(&conn, trip_id).unwrap();
        assert!(!md.contains("Expenses:"));
    }

    #[test]
    fn test_export_md_overview_includes_difference() {
        let conn = test_db();
        let trip_id = add_test_trip(&conn, "Difference MD Trip").unwrap();
        let itinerary_id = add_itinerary_item(
            &conn,
            trip_id,
            1,
            "Aquarium",
            None,
            None,
            Some(0),
            None,
            None,
            None,
            None,
        )
        .unwrap();
        crate::estimate::add_estimate(
            &conn,
            itinerary_id,
            "180000",
            "JPY",
            Some("予算"),
            None,
            None,
        )
        .unwrap();
        crate::expense::add_expense(
            &conn,
            itinerary_id,
            "172500",
            "JPY",
            Some("実績"),
            None,
            None,
            None,
            &crate::expense::ExpenseSharedOptions::default(),
        )
        .unwrap();

        let md = generate_trip_markdown(&conn, trip_id).unwrap();
        assert!(md.contains("- Planned total:"));
        assert!(md.contains("- Actual total:"));
        assert!(md.contains("- Difference:"));
        assert!(md.contains("JPY -7,500"));
    }

    #[test]
    fn test_export_md_overview_omits_difference_without_both() {
        let conn = test_db();
        let trip_id = add_test_trip(&conn, "Estimate Only MD Trip").unwrap();
        let itinerary_id = add_itinerary_item(
            &conn,
            trip_id,
            1,
            "Aquarium",
            None,
            None,
            Some(0),
            None,
            None,
            None,
            None,
        )
        .unwrap();
        crate::estimate::add_estimate(&conn, itinerary_id, "2180", "JPY", None, None, None)
            .unwrap();

        let md = generate_trip_markdown(&conn, trip_id).unwrap();
        assert!(md.contains("- Planned total:"));
        assert!(!md.contains("- Difference:"));
    }

    #[test]
    fn test_export_md_itinerary_includes_planned_actual_difference() {
        let conn = test_db();
        let trip_id = add_test_trip(&conn, "Itinerary Difference Trip").unwrap();
        let itinerary_id = add_itinerary_item(
            &conn,
            trip_id,
            1,
            "Breakfast",
            None,
            None,
            Some(0),
            None,
            None,
            None,
            None,
        )
        .unwrap();
        crate::estimate::add_estimate(
            &conn,
            itinerary_id,
            "14000",
            "JPY",
            Some("朝食"),
            None,
            None,
        )
        .unwrap();
        crate::expense::add_expense(
            &conn,
            itinerary_id,
            "13750",
            "JPY",
            Some("朝食"),
            None,
            None,
            None,
            &crate::expense::ExpenseSharedOptions::default(),
        )
        .unwrap();

        let md = generate_trip_markdown(&conn, trip_id).unwrap();
        let section_start = md.find("### Breakfast").unwrap();
        let section = &md[section_start..];
        assert!(section.contains("Planned total:"));
        assert!(section.contains("Actual total:"));
        assert!(section.contains("Difference:"));
        assert!(section.contains("JPY 14,000"));
        assert!(section.contains("JPY 13,750"));
        assert!(section.contains("JPY -250"));
    }

    #[test]
    fn test_export_md_itinerary_omits_difference_with_estimate_only() {
        let conn = test_db();
        let trip_id = add_test_trip(&conn, "Itinerary Estimate Only Trip").unwrap();
        let itinerary_id = add_itinerary_item(
            &conn,
            trip_id,
            1,
            "Aquarium",
            None,
            None,
            Some(0),
            None,
            None,
            None,
            None,
        )
        .unwrap();
        crate::estimate::add_estimate(&conn, itinerary_id, "2180", "JPY", None, None, None)
            .unwrap();

        let md = generate_trip_markdown(&conn, trip_id).unwrap();
        let section = &md[md.find("### Aquarium").unwrap()..];
        assert!(section.contains("予定費用:"));
        assert!(!section.contains("Planned total:"));
        assert!(!section.contains("Difference:"));
    }

    #[test]
    fn test_export_md_itinerary_omits_difference_with_expense_only() {
        let conn = test_db();
        let trip_id = add_test_trip(&conn, "Itinerary Expense Only Trip").unwrap();
        let itinerary_id = add_itinerary_item(
            &conn,
            trip_id,
            1,
            "Lunch",
            None,
            None,
            Some(0),
            None,
            None,
            None,
            None,
        )
        .unwrap();
        crate::expense::add_expense(
            &conn,
            itinerary_id,
            "1200",
            "JPY",
            Some("昼食"),
            None,
            None,
            None,
            &crate::expense::ExpenseSharedOptions::default(),
        )
        .unwrap();

        let md = generate_trip_markdown(&conn, trip_id).unwrap();
        let section = &md[md.find("### Lunch").unwrap()..];
        assert!(section.contains("Expenses:"));
        assert!(!section.contains("Planned total:"));
        assert!(!section.contains("Difference:"));
    }

    #[test]
    fn test_export_md_itinerary_multi_currency_difference() {
        let conn = test_db();
        let trip_id = add_test_trip(&conn, "Itinerary Multi Currency Trip").unwrap();
        let itinerary_id = add_itinerary_item(
            &conn,
            trip_id,
            1,
            "Shopping",
            None,
            None,
            Some(0),
            None,
            None,
            None,
            None,
        )
        .unwrap();
        crate::estimate::add_estimate(&conn, itinerary_id, "10000", "JPY", None, None, None)
            .unwrap();
        crate::estimate::add_estimate(&conn, itinerary_id, "50000", "KRW", None, None, None)
            .unwrap();
        crate::expense::add_expense(
            &conn,
            itinerary_id,
            "9500",
            "JPY",
            None,
            None,
            None,
            None,
            &crate::expense::ExpenseSharedOptions::default(),
        )
        .unwrap();

        let md = generate_trip_markdown(&conn, trip_id).unwrap();
        let section = &md[md.find("### Shopping").unwrap()..];
        assert!(section.contains("JPY -500"));
        assert!(section.contains("KRW -50,000"));
    }

    #[test]
    fn test_export_md_overview_difference_unaffected_by_itinerary_difference() {
        let conn = test_db();
        let trip_id = add_test_trip(&conn, "Overview And Itinerary Difference Trip").unwrap();
        let itinerary_id = add_itinerary_item(
            &conn,
            trip_id,
            1,
            "Aquarium",
            None,
            None,
            Some(0),
            None,
            None,
            None,
            None,
        )
        .unwrap();
        crate::estimate::add_estimate(&conn, itinerary_id, "180000", "JPY", None, None, None)
            .unwrap();
        crate::expense::add_expense(
            &conn,
            itinerary_id,
            "172500",
            "JPY",
            None,
            None,
            None,
            None,
            &crate::expense::ExpenseSharedOptions::default(),
        )
        .unwrap();

        let md = generate_trip_markdown(&conn, trip_id).unwrap();
        let overview = &md[..md.find("## Day 1").unwrap()];
        assert!(overview.contains("- Difference:"));
        assert!(overview.contains("JPY -7,500"));

        let itinerary = &md[md.find("### Aquarium").unwrap()..];
        assert!(itinerary.contains("Difference:"));
        assert!(itinerary.contains("JPY -7,500"));
    }
}
