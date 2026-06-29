use std::collections::HashMap;

use anyhow::{Context, Result};
use chrono::Utc;
use rusqlite::Connection;

use crate::analysis::statistics::{format_minutes_duration, TripStats};
use crate::domain::models::{
    ChecklistItem, Day, Estimate, ExportNote, ItineraryItem, Participant, Trip,
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

/// Daily schedule 章向けに 1 件の Itinerary を Markdown 形式に整形する
pub(crate) fn format_itinerary_item_markdown(item: &ItineraryItem) -> String {
    let heading = match &item.start_time {
        Some(time) => format!("#### {time} {}", item.title),
        None => format!("#### {}", item.title),
    };
    let mut lines = vec![heading];

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

fn append_cover(output: &mut String, trip: &Trip, stats: &TripStats) {
    output.push_str(&format!("# {}\n", trip.name));
    if let Some(dates) = format_trip_date_range(trip) {
        output.push('\n');
        output.push_str(&dates);
        output.push('\n');
    }
    if stats.participants_recorded {
        if stats.self_known {
            let travelers = stats
                .traveler_count
                .or(stats.participant_count)
                .unwrap_or(stats.registered_participant_count);
            output.push_str(&format!("\nTravelers: {travelers}\n"));
        } else {
            output.push_str(&format!(
                "\nParticipants: {} recorded\n",
                stats.registered_participant_count
            ));
        }
    }
}

fn trip_overview_worth_showing(
    trip: &Trip,
    participants: &[Participant],
    stats: &TripStats,
) -> bool {
    trip.summary.as_ref().is_some_and(|s| !s.trim().is_empty())
        || !participants.is_empty()
        || stats.itinerary_count > 0
        || stats.checklist_total > 0
        || stats.stay_minutes > 0
        || stats.travel_minutes > 0
        || stats.participants_recorded
        || stats.days > 0
        || trip.start_date.is_some()
        || trip.end_date.is_some()
}

fn append_trip_overview_stats(output: &mut String, stats: &TripStats) {
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
}

fn append_trip_overview(
    output: &mut String,
    trip: &Trip,
    participants: &[Participant],
    stats: &TripStats,
) {
    output.push_str("\n## Trip overview\n\n");
    if let Some(summary) = &trip.summary {
        if !summary.trim().is_empty() {
            output.push_str(summary);
            output.push_str("\n\n");
        }
    }
    if !participants.is_empty() {
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
        output.push('\n');
    }
    append_trip_overview_stats(output, stats);
}

fn format_day_heading(trip: &Trip, day_number: i64) -> String {
    match crate::day::day_date_for_trip(trip, day_number) {
        Ok(date) => format!("### Day {day_number} — {date}"),
        Err(_) => format!("### Day {day_number}"),
    }
}

fn append_daily_schedule(output: &mut String, trip: &Trip, days: &[Day], items: &[ItineraryItem]) {
    output.push_str("\n## Daily schedule\n\n");
    if days.is_empty() && items.is_empty() {
        output.push_str("_No itineraries scheduled._\n");
        return;
    }

    let day_numbers: Vec<i64> = if days.is_empty() {
        let mut numbers: Vec<i64> = items.iter().map(|item| item.day).collect();
        numbers.sort_unstable();
        numbers.dedup();
        numbers
    } else {
        days.iter().map(|day| day.day_number).collect()
    };

    let summaries: HashMap<i64, Option<String>> = days
        .iter()
        .map(|day| (day.day_number, day.summary.clone()))
        .collect();

    for (index, day_number) in day_numbers.iter().enumerate() {
        if index > 0 {
            output.push_str("\n\n");
        }
        output.push_str(&format_day_heading(trip, *day_number));
        output.push_str("\n\n");
        if let Some(day_summary) = summaries.get(day_number).and_then(|s| s.as_ref()) {
            output.push_str(day_summary);
            output.push_str("\n\n");
        }
        let day_items: Vec<&ItineraryItem> = items
            .iter()
            .filter(|item| item.day == *day_number)
            .collect();
        if day_items.is_empty() {
            output.push_str("_No itineraries scheduled._\n");
            continue;
        }
        for (item_index, item) in day_items.iter().enumerate() {
            if item_index > 0 {
                output.push_str("\n\n");
            }
            output.push_str(&format_itinerary_item_markdown(item));
        }
    }
}

fn format_planned_cost_chapter(
    items: &[ItineraryItem],
    trip_estimates: &[Estimate],
    stats: &TripStats,
) -> Option<String> {
    if trip_estimates.is_empty() {
        return None;
    }

    let itinerary_by_id: HashMap<i64, &ItineraryItem> =
        items.iter().map(|item| (item.id, item)).collect();

    let mut lines = vec!["## Planned cost".to_string(), String::new()];
    lines.push(format!("- Estimates: {}", stats.estimate_count));
    if !stats.estimate_totals.is_empty() {
        lines.push("- Planned total:".to_string());
        for (currency, total) in &stats.estimate_totals {
            lines.push(format!(
                "  - {} {}",
                currency,
                crate::money::format_amount_value(*total, currency)
            ));
        }
    }
    lines.push(String::new());

    let mut current_itinerary_id: Option<i64> = None;
    for estimate in trip_estimates {
        if current_itinerary_id != Some(estimate.itinerary_id) {
            if current_itinerary_id.is_some() {
                lines.push(String::new());
            }
            current_itinerary_id = Some(estimate.itinerary_id);
            if let Some(item) = itinerary_by_id.get(&estimate.itinerary_id) {
                let context = match &item.start_time {
                    Some(time) => format!("Day {} / {time} {}", item.day, item.title),
                    None => format!("Day {} / {}", item.day, item.title),
                };
                lines.push(format!("### {context}"));
                lines.push(String::new());
                lines.push("| Item | Amount | Note |".to_string());
                lines.push("|---|---:|---|".to_string());
            }
        }
        let title = estimate
            .title
            .as_deref()
            .filter(|value| !value.trim().is_empty())
            .unwrap_or("-");
        let amount =
            crate::estimate::format_estimate_amount_markdown(estimate.amount, &estimate.currency);
        let note = estimate.note.as_deref().unwrap_or("");
        lines.push(format!("| {title} | {amount} | {note} |"));
    }

    Some(format!("\n\n{}\n", lines.join("\n").trim_end()))
}

fn format_note_heading(note: &ExportNote) -> String {
    match note {
        ExportNote::Trip { title, .. } => {
            let label = title
                .as_deref()
                .filter(|value| !value.trim().is_empty())
                .unwrap_or("Trip note");
            format!("### Trip — {label}")
        }
        ExportNote::Day {
            day_number, title, ..
        } => {
            let label = title
                .as_deref()
                .filter(|value| !value.trim().is_empty())
                .unwrap_or("Day note");
            format!("### Day {day_number} — {label}")
        }
        ExportNote::Itinerary {
            itinerary_key,
            title,
            ..
        } => {
            let label = title
                .as_deref()
                .filter(|value| !value.trim().is_empty())
                .unwrap_or("Itinerary note");
            let context = match &itinerary_key.start_time {
                Some(time) => format!(
                    "Day {} / {time} {}",
                    itinerary_key.day_number, itinerary_key.title
                ),
                None => format!("Day {} / {}", itinerary_key.day_number, itinerary_key.title),
            };
            format!("### {context} — {label}")
        }
    }
}

fn note_body(note: &ExportNote) -> &str {
    match note {
        ExportNote::Trip { body, .. }
        | ExportNote::Day { body, .. }
        | ExportNote::Itinerary { body, .. } => body,
    }
}

/// Trip / Day / Itinerary スコープの Note entity を Markdown 章に整形する（0 件なら None）
pub(crate) fn format_notes_chapter(export_notes: &[ExportNote]) -> Option<String> {
    if export_notes.is_empty() {
        return None;
    }

    let mut lines = vec!["## Notes".to_string(), String::new()];
    for (index, note) in export_notes.iter().enumerate() {
        if index > 0 {
            lines.push(String::new());
        }
        lines.push(format_note_heading(note));
        lines.push(String::new());
        lines.push(note_body(note).to_string());
    }

    Some(format!("\n\n{}\n", lines.join("\n").trim_end()))
}

fn append_colophon(output: &mut String, trip: &Trip) {
    output.push_str("\n## Colophon\n\n");
    output.push_str("Generated by Caglla.Travel CLI\n");
    output.push_str("Travel Book Generator v0\n");
    output.push_str(&format!("Version: {}\n", env!("CARGO_PKG_VERSION")));
    output.push_str(&format!("Generated at: {}\n", Utc::now().to_rfc3339()));
    output.push_str(&format!("Trip: {}\n", trip.name));
    if let Some(dates) = format_trip_date_range(trip) {
        output.push_str(&format!("Dates: {dates}\n"));
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

/// 旅行と日程一覧から Travel Book Markdown 文字列を組み立てる
#[allow(clippy::too_many_arguments)]
pub(crate) fn format_trip_markdown(
    trip: &Trip,
    days: &[Day],
    items: &[ItineraryItem],
    checklist: &[ChecklistItem],
    participants: &[Participant],
    stats: &TripStats,
    trip_estimates: &[Estimate],
    reservations: &[ReservationWithContext],
    export_notes: &[ExportNote],
) -> String {
    let mut output = String::new();

    append_cover(&mut output, trip, stats);

    if trip_overview_worth_showing(trip, participants, stats) {
        append_trip_overview(&mut output, trip, participants, stats);
    }

    append_daily_schedule(&mut output, trip, days, items);

    if let Some(reservations_md) = format_reservations_markdown(reservations) {
        output.push_str(&reservations_md);
    }

    if let Some(checklist_md) = format_checklist_markdown(checklist) {
        output.push_str(&checklist_md);
    }

    if let Some(planned_cost_md) = format_planned_cost_chapter(items, trip_estimates, stats) {
        output.push_str(&planned_cost_md);
    }

    if let Some(notes_md) = format_notes_chapter(export_notes) {
        output.push_str(&notes_md);
    }

    append_colophon(&mut output, trip);

    output.trim_end().to_string()
}

/// 旅行しおりを Markdown 文字列として組み立てる
pub(crate) fn generate_trip_markdown(conn: &Connection, trip_id: i64) -> Result<String> {
    let trip = crate::trip::get_trip(conn, trip_id)?;
    let days = crate::day::list_days(conn, trip_id)?;
    let items = list_itinerary_items_for_markdown(conn, trip_id)?;
    let checklist = crate::checklist::list_checklist_items(conn, trip_id)?;
    let stats = crate::analysis::statistics::compute_trip_stats(conn, trip_id)?;
    let trip_estimates = crate::estimate::list_estimates_for_trip(conn, trip_id)?;
    let reservations = crate::reservation::list_reservations_for_trip(conn, trip_id)?;
    let participants = crate::participant::list_participants_by_trip(conn, trip_id)?;
    let export_notes = crate::note::build_export_notes(conn, trip_id)?;
    Ok(format_trip_markdown(
        &trip,
        &days,
        &items,
        &checklist,
        &participants,
        &stats,
        &trip_estimates,
        &reservations,
        &export_notes,
    ))
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
        let day1_pos = md.find("### Day 1 —").unwrap();
        let day2_pos = md.find("### Day 2 —").unwrap();
        let first_item_pos = md.find("#### 10:00 1日目").unwrap();
        let second_day_first_pos = md.find("#### 09:00 2日目・先").unwrap();
        let second_day_second_pos = md.find("#### 14:00 2日目・後").unwrap();

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
        assert!(md.contains("#### Hilton Hawaiian Village"));
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
        assert!(md.contains("#### 散歩"));
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
        assert!(md.contains("#### 08:00 朝食"));
        assert!(md.contains("#### 自由時間"));
        assert!(!md.contains("#### 自由時間 自由時間"));
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
        assert!(md.contains("## Daily schedule"));
        assert!(md.contains("### Day 1 — 2026-04-26"));
        assert!(md.contains("#### 09:00 那覇空港"));
        assert!(md.contains("- 場所: 那覇空港"));
        assert!(md.contains("- 所要時間: 60分"));
        assert!(md.contains("- 移動時間: 30分"));
        assert!(md.contains("- メモ: レンタカー受け取り"));
        assert!(md.contains("## Colophon"));
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
        assert!(md.contains("## Trip overview"));
        assert!(md.contains("- Days: 2"));
        assert!(md.contains("- Itineraries: 2"));
        assert!(md.contains("- Checklist: 1 / 2 completed"));
        assert!(md.contains("- Stay Time: 2h30m"));
        assert!(md.contains("- Travel Time: 50m"));
        assert!(md.contains("- Total Time: 3h20m"));
        assert!(!md.contains("- Planned total:"));
        assert!(!md.contains("- Actual total:"));
        assert!(!md.contains("- Difference:"));
        assert!(!md.contains("Category Breakdown"));
        assert!(!md.contains("uncategorized"));

        let overview_pos = md.find("## Trip overview").unwrap();
        let daily_pos = md.find("## Daily schedule").unwrap();
        assert!(overview_pos < daily_pos);
    }

    #[test]
    fn test_export_md_overview_checklist_zero() {
        let conn = test_db();
        let trip_id = add_test_trip(&conn, "沖縄旅行").unwrap();

        let md = generate_trip_markdown(&conn, trip_id).unwrap();
        assert!(md.contains("## Trip overview"));
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
        assert!(content.contains("#### 09:00 首里城"));

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
        let daily_pos = md.find("## Daily schedule").unwrap();
        assert!(daily_pos < reservations_pos);
    }

    #[test]
    fn test_export_md_omits_reservations_when_none() {
        let conn = test_db();
        let trip_id = add_test_trip(&conn, "No Reservation Trip").unwrap();
        let md = generate_trip_markdown(&conn, trip_id).unwrap();
        assert!(!md.contains("## Reservations"));
    }

    #[test]
    fn test_export_md_omits_expenses_in_travel_book() {
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

        let md = generate_trip_markdown(&conn, trip_id).unwrap();
        assert!(!md.contains("Expenses:"));
        assert!(!md.contains("- 入館料: 2,500 JPY"));
        assert!(md.contains("#### 09:00 Aquarium"));
    }

    #[test]
    fn test_export_md_omits_shared_expense_details() {
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
        assert!(!md.contains("Paid by: Alice"));
        assert!(!md.contains("Shared: Alice, Bob"));
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
    fn test_export_md_planned_cost_chapter() {
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
        assert!(md.contains("## Planned cost"));
        assert!(md.contains("- Planned total:"));
        assert!(md.contains("JPY 180,000"));
        assert!(!md.contains("- Actual total:"));
        assert!(!md.contains("- Difference:"));
        let daily = &md[..md.find("## Planned cost").unwrap()];
        assert!(!daily.contains("予定費用:"));
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
        assert!(md.contains("## Planned cost"));
        assert!(md.contains("- Planned total:"));
        assert!(!md.contains("- Difference:"));
    }

    #[test]
    fn test_export_md_daily_schedule_omits_estimates() {
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
        let daily_end = md.find("## Planned cost").unwrap_or(md.len());
        let daily = &md[..daily_end];
        assert!(daily.contains("#### Aquarium"));
        assert!(!daily.contains("予定費用:"));
        assert!(!daily.contains("Planned total:"));
        assert!(md.contains("| - | JPY 2,180 |  |"));
    }

    #[test]
    fn test_export_md_daily_schedule_omits_expenses() {
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
        assert!(md.contains("#### Lunch"));
        assert!(!md.contains("Expenses:"));
        assert!(!md.contains("Planned total:"));
        assert!(!md.contains("Difference:"));
    }

    #[test]
    fn test_export_md_planned_cost_multi_currency() {
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

        let md = generate_trip_markdown(&conn, trip_id).unwrap();
        let planned = &md[md.find("## Planned cost").unwrap()..];
        assert!(planned.contains("JPY 10,000"));
        assert!(planned.contains("KRW 50,000"));
    }

    #[test]
    fn test_export_md_includes_notes_chapter() {
        let conn = test_db();
        let trip_id = add_test_trip(&conn, "Notes MD Trip").unwrap();
        crate::note::add_note(
            &conn,
            crate::note::ResolvedNoteOwner::Trip(trip_id),
            Some("Overview memo"),
            "Trip-level note body",
        )
        .unwrap();

        let md = generate_trip_markdown(&conn, trip_id).unwrap();
        assert!(md.contains("## Notes"));
        assert!(md.contains("### Trip — Overview memo"));
        assert!(md.contains("Trip-level note body"));
        let notes_pos = md.find("## Notes").unwrap();
        let colophon_pos = md.find("## Colophon").unwrap();
        assert!(notes_pos < colophon_pos);
    }

    #[test]
    fn test_export_md_trip_summary_in_overview_not_cover() {
        let conn = test_db();
        let trip_id = add_trip(
            &conn,
            "Summary Trip",
            "2026-04-26",
            "2026-04-29",
            Some("Short trip summary"),
        )
        .unwrap();

        let md = generate_trip_markdown(&conn, trip_id).unwrap();
        let cover_end = md.find("## Trip overview").unwrap();
        let cover = &md[..cover_end];
        assert!(!cover.contains("Short trip summary"));
        assert!(md.contains("## Trip overview"));
        assert!(md.contains("Short trip summary"));
    }
}
