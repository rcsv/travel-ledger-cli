//! Travel Book 向け presentation ルール（renderer 非依存）。
//!
//! Markdown / GUI / Web など複数 renderer が共有する表示判断を置く。
//! 構文（見出し級・表・箇条書き）は各 renderer 側に残す。

use chrono::{NaiveDate, NaiveDateTime, Timelike};

use crate::analysis::statistics::TripStats;
use crate::domain::models::{
    Day, Estimate, ExportNote, ItineraryCategory, ItineraryItem, ItineraryNoteKey, Participant,
    Trip,
};

/// Trip の日付範囲ラベル（`start 〜 end`、片方のみの場合はその日付）
pub(crate) fn travel_book_trip_date_range(trip: &Trip) -> Option<String> {
    match (&trip.start_date, &trip.end_date) {
        (Some(start), Some(end)) => Some(format!("{start} 〜 {end}")),
        (Some(start), None) => Some(start.clone()),
        (None, Some(end)) => Some(end.clone()),
        (None, None) => None,
    }
}

/// Trip overview 章を出す価値があるか
pub(crate) fn trip_overview_worth_showing(
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

/// Stay / Travel / Total 時間メトリクス行を出すか
pub(crate) fn trip_overview_time_metrics_worth_showing(stats: &TripStats) -> bool {
    stats.stay_minutes > 0 || stats.travel_minutes > 0 || stats.total_minutes() > 0
}

/// Days overview 用エントリ（非空 Day summary のみ）
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct DaysOverviewEntry {
    pub day_number: i64,
    pub summary: String,
}

/// Day summary がある日だけ抽出し、`day_number` 昇順で返す
pub(crate) fn collect_days_overview_entries(days: &[Day]) -> Vec<DaysOverviewEntry> {
    let mut entries: Vec<DaysOverviewEntry> = days
        .iter()
        .filter_map(|day| {
            day.summary
                .as_deref()
                .map(str::trim)
                .filter(|summary| !summary.is_empty())
                .map(|summary| DaysOverviewEntry {
                    day_number: day.day_number,
                    summary: summary.to_string(),
                })
        })
        .collect();
    entries.sort_by_key(|entry| entry.day_number);
    entries
}

/// Days overview 一覧ラベル（`Day N — date`）
pub(crate) fn travel_book_day_overview_label(trip: &Trip, day_number: i64) -> String {
    match crate::day::day_date_for_trip(trip, day_number) {
        Ok(date) => format!("Day {day_number} — {date}"),
        Err(_) => format!("Day {day_number}"),
    }
}

/// Planned cost 表で Note 列を出すか（いずれかの note が非空なら true）
pub(crate) fn planned_cost_note_column_visible(estimates: &[&Estimate]) -> bool {
    estimates.iter().any(|estimate| {
        estimate
            .note
            .as_deref()
            .map(str::trim)
            .is_some_and(|note| !note.is_empty())
    })
}

/// Planned cost 行タイトル（空・空白のみなら `-`）
pub(crate) fn planned_cost_estimate_display_title(title: Option<&str>) -> &str {
    title
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("-")
}

/// Itinerary コンテキストラベル（`Day N / time title` または `Day N / title`）
fn travel_book_itinerary_context_label(
    day_number: i64,
    start_time: Option<&str>,
    title: &str,
) -> String {
    match start_time {
        Some(time) => format!("Day {day_number} / {time} {title}"),
        None => format!("Day {day_number} / {title}"),
    }
}

/// Note（Itinerary スコープ）の itinerary コンテキスト
pub(crate) fn travel_book_note_itinerary_context(key: &ItineraryNoteKey) -> String {
    travel_book_itinerary_context_label(key.day_number, key.start_time.as_deref(), &key.title)
}

/// Planned cost itinerary グループ見出し用コンテキスト（`Day N / …`）
pub(crate) fn planned_cost_itinerary_group_label(item: &ItineraryItem) -> String {
    travel_book_itinerary_context_label(item.day, item.start_time.as_deref(), &item.title)
}

/// Note 見出しラベル（`Trip — …` / `Day N — …` / `Day N / … — …`）。Markdown `###` は含まない
pub(crate) fn travel_book_note_heading_label(note: &ExportNote) -> String {
    match note {
        ExportNote::Trip { title, .. } => {
            let label = title
                .as_deref()
                .filter(|value| !value.trim().is_empty())
                .unwrap_or("Trip note");
            format!("Trip — {label}")
        }
        ExportNote::Day {
            day_number, title, ..
        } => {
            let label = title
                .as_deref()
                .filter(|value| !value.trim().is_empty())
                .unwrap_or("Day note");
            format!("Day {day_number} — {label}")
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
            let context = travel_book_note_itinerary_context(itinerary_key);
            format!("{context} — {label}")
        }
    }
}

/// Daily schedule 向け itinerary カテゴリ詳細の label / value（Markdown bullet は含まない）
pub(crate) fn travel_book_category_detail_label(
    category: ItineraryCategory,
) -> (&'static str, &'static str) {
    ("種別", category.definition().display_name)
}

/// Note entity の Travel Book 出力順（Trip → Day → Itinerary）
pub(crate) fn travel_book_note_sort_key(note: &ExportNote) -> (i32, i64, i64, String) {
    match note {
        ExportNote::Trip { title, body, .. } => {
            let label = title.as_deref().unwrap_or("").to_string();
            (0, 0, 0, label + body)
        }
        ExportNote::Day {
            day_number,
            title,
            body,
            ..
        } => {
            let label = title.as_deref().unwrap_or("").to_string();
            (1, *day_number, 0, label + body)
        }
        ExportNote::Itinerary {
            itinerary_key,
            title,
            body,
            ..
        } => {
            let label = title.as_deref().unwrap_or("").to_string();
            (
                2,
                itinerary_key.day_number,
                itinerary_key.sort_order,
                label + body,
            )
        }
    }
}

/// Note entity を Travel Book 順に並べ替える
pub(crate) fn sort_export_notes_for_travel_book(export_notes: &mut [ExportNote]) {
    export_notes.sort_by(|left, right| {
        travel_book_note_sort_key(left).cmp(&travel_book_note_sort_key(right))
    });
}

/// Provider が itinerary 見出しと冗長か（同一または相互包含なら省略）
pub(crate) fn reservation_provider_line_redundant(
    provider_name: &str,
    itinerary_title: &str,
) -> bool {
    let provider = provider_name.trim();
    let title = itinerary_title.trim();
    if provider.is_empty() {
        return true;
    }
    if provider == title {
        return true;
    }
    if title.contains(provider) || provider.contains(title) {
        return true;
    }
    false
}

fn parse_reservation_datetime(value: &str) -> Option<NaiveDateTime> {
    let trimmed = value.trim();
    NaiveDateTime::parse_from_str(trimmed, "%Y-%m-%dT%H:%M")
        .or_else(|_| NaiveDateTime::parse_from_str(trimmed, "%Y-%m-%dT%H:%M:%S"))
        .or_else(|_| NaiveDateTime::parse_from_str(trimmed, "%Y-%m-%d %H:%M:%S"))
        .ok()
}

fn parse_reservation_date(value: &str) -> Option<NaiveDate> {
    let trimmed = value.trim();
    NaiveDate::parse_from_str(trimmed, "%Y-%m-%d").ok()
}

fn format_reservation_datetime_endpoint(value: &str) -> String {
    if let Some(dt) = parse_reservation_datetime(value) {
        return format!("{} {:02}:{:02}", dt.date(), dt.hour(), dt.minute());
    }
    if let Some(date) = parse_reservation_date(value) {
        return date.to_string();
    }
    value.trim().to_string()
}

/// Reservation の start/end を人間可読な期間文字列に整形する
pub(crate) fn format_travel_book_reservation_period(
    start_at: &Option<String>,
    end_at: &Option<String>,
) -> Option<String> {
    let start_raw = start_at
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty());
    let end_raw = end_at
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty());

    match (start_raw, end_raw) {
        (Some(start), Some(end)) => {
            if let (Some(start_dt), Some(end_dt)) = (
                parse_reservation_datetime(start),
                parse_reservation_datetime(end),
            ) {
                let start_text = format!(
                    "{} {:02}:{:02}",
                    start_dt.date(),
                    start_dt.hour(),
                    start_dt.minute()
                );
                let end_text = if start_dt.date() == end_dt.date() {
                    format!("{:02}:{:02}", end_dt.hour(), end_dt.minute())
                } else {
                    format!(
                        "{} {:02}:{:02}",
                        end_dt.date(),
                        end_dt.hour(),
                        end_dt.minute()
                    )
                };
                return Some(format!("{start_text} 〜 {end_text}"));
            }
            Some(format!(
                "{} 〜 {}",
                format_reservation_datetime_endpoint(start),
                format_reservation_datetime_endpoint(end)
            ))
        }
        (Some(start), None) => Some(format_reservation_datetime_endpoint(start)),
        (None, Some(end)) => Some(format_reservation_datetime_endpoint(end)),
        (None, None) => None,
    }
}

/// Reservation 見出しの presentation 部分（Markdown bold は含まない）
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct TravelBookReservationHeading {
    pub main_label: String,
    pub provider_suffix: Option<String>,
}

/// Reservation 見出し（provider 冗長時は `provider_suffix` なし）
pub(crate) fn travel_book_reservation_heading(
    day_number: i64,
    itinerary_title: &str,
    provider_name: &str,
) -> TravelBookReservationHeading {
    let main_label = format!("Day {day_number} / {itinerary_title}");
    let provider_suffix = if reservation_provider_line_redundant(provider_name, itinerary_title) {
        None
    } else {
        Some(provider_name.to_string())
    };
    TravelBookReservationHeading {
        main_label,
        provider_suffix,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::models::ItineraryCategory;

    #[test]
    fn test_trip_overview_time_metrics_worth_showing() {
        use std::collections::{BTreeMap, HashMap};

        let mut stats = TripStats {
            trip_name: String::new(),
            days: 1,
            itinerary_count: 0,
            checklist_completed: 0,
            checklist_total: 0,
            category_counts: HashMap::new(),
            stay_minutes: 0,
            travel_minutes: 0,
            total_minutes: 0,
            expense_count: 0,
            expense_totals: BTreeMap::new(),
            estimate_count: 0,
            estimate_totals: BTreeMap::new(),
            difference_totals: None,
            registered_participant_count: 0,
            participants_recorded: false,
            self_known: false,
            participant_count: None,
            traveler_count: None,
            companion_count: None,
        };
        assert!(!trip_overview_time_metrics_worth_showing(&stats));
        stats.stay_minutes = 1;
        assert!(trip_overview_time_metrics_worth_showing(&stats));
    }

    #[test]
    fn test_collect_days_overview_entries() {
        let days = vec![
            Day {
                id: 1,
                trip_id: 1,
                day_number: 2,
                title: String::new(),
                summary: Some("  Day 2 summary  ".to_string()),
                created_at: String::new(),
                updated_at: String::new(),
            },
            Day {
                id: 2,
                trip_id: 1,
                day_number: 1,
                title: String::new(),
                summary: Some("Day 1 summary".to_string()),
                created_at: String::new(),
                updated_at: String::new(),
            },
            Day {
                id: 3,
                trip_id: 1,
                day_number: 3,
                title: String::new(),
                summary: Some("   ".to_string()),
                created_at: String::new(),
                updated_at: String::new(),
            },
        ];
        let entries = collect_days_overview_entries(&days);
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].day_number, 1);
        assert_eq!(entries[0].summary, "Day 1 summary");
        assert_eq!(entries[1].day_number, 2);
        assert_eq!(entries[1].summary, "Day 2 summary");
    }

    #[test]
    fn test_planned_cost_note_column_visible() {
        let with_note = Estimate {
            id: 1,
            itinerary_id: 1,
            title: None,
            amount: 100,
            currency: "JPY".to_string(),
            note: Some("memo".to_string()),
            sort_order: 0,
            created_at: String::new(),
            updated_at: String::new(),
        };
        let empty_note = Estimate {
            note: Some("  ".to_string()),
            ..with_note.clone()
        };
        let no_note = Estimate {
            note: None,
            ..with_note.clone()
        };
        assert!(planned_cost_note_column_visible(&[&with_note]));
        assert!(!planned_cost_note_column_visible(&[&empty_note, &no_note]));
    }

    #[test]
    fn test_planned_cost_estimate_display_title() {
        assert_eq!(planned_cost_estimate_display_title(None), "-");
        assert_eq!(planned_cost_estimate_display_title(Some("  ")), "-");
        assert_eq!(planned_cost_estimate_display_title(Some("Lunch")), "Lunch");
    }

    #[test]
    fn test_travel_book_category_detail_label_uses_definition_display_name() {
        assert_eq!(
            travel_book_category_detail_label(ItineraryCategory::Transport),
            ("種別", "移動")
        );
        assert_eq!(
            travel_book_category_detail_label(ItineraryCategory::Flight),
            ("種別", "フライト")
        );
        for category in ItineraryCategory::all() {
            let (label, value) = travel_book_category_detail_label(category);
            assert_eq!(label, "種別");
            assert!(!value.is_empty());
            assert!(!value.contains(category.as_str()));
        }
    }

    #[test]
    fn test_travel_book_reservation_heading() {
        assert_eq!(
            travel_book_reservation_heading(1, "NU045 NGO ⇒ OKA (11:00着)", "NU045 NGO ⇒ OKA",),
            TravelBookReservationHeading {
                main_label: "Day 1 / NU045 NGO ⇒ OKA (11:00着)".to_string(),
                provider_suffix: None,
            }
        );
        assert_eq!(
            travel_book_reservation_heading(1, "チェックイン", "ヒルトン瀬底"),
            TravelBookReservationHeading {
                main_label: "Day 1 / チェックイン".to_string(),
                provider_suffix: Some("ヒルトン瀬底".to_string()),
            }
        );
    }

    #[test]
    fn test_reservation_provider_line_redundant() {
        assert!(reservation_provider_line_redundant(
            "NU045 NGO ⇒ OKA",
            "NU045 NGO ⇒ OKA (11:00着)"
        ));
        assert!(reservation_provider_line_redundant(
            "セントレア P1 G Parking",
            "P1 G Parking"
        ));
        assert!(!reservation_provider_line_redundant(
            "ヒルトン瀬底",
            "チェックイン"
        ));
        assert!(!reservation_provider_line_redundant(
            "Ks Rent A Car",
            "Toyota Alphard 又は同等車種"
        ));
    }

    #[test]
    fn test_travel_book_trip_date_range() {
        let both = Trip {
            id: 1,
            name: "t".to_string(),
            start_date: Some("2026-04-26".to_string()),
            end_date: Some("2026-04-29".to_string()),
            summary: None,
            main_destination: None,
            main_destination_country_code: None,
            default_currency: None,
            created_at: String::new(),
            updated_at: String::new(),
        };
        assert_eq!(
            travel_book_trip_date_range(&both),
            Some("2026-04-26 〜 2026-04-29".to_string())
        );
        let start_only = Trip {
            end_date: None,
            ..both.clone()
        };
        assert_eq!(
            travel_book_trip_date_range(&start_only),
            Some("2026-04-26".to_string())
        );
        let neither = Trip {
            start_date: None,
            end_date: None,
            ..both
        };
        assert_eq!(travel_book_trip_date_range(&neither), None);
    }

    #[test]
    fn test_travel_book_note_itinerary_context() {
        let with_time = ItineraryNoteKey {
            day_number: 2,
            sort_order: 4,
            start_time: Some("08:30".to_string()),
            title: "水族館に入館".to_string(),
        };
        assert_eq!(
            travel_book_note_itinerary_context(&with_time),
            "Day 2 / 08:30 水族館に入館"
        );
        let no_time = ItineraryNoteKey {
            start_time: None,
            ..with_time
        };
        assert_eq!(
            travel_book_note_itinerary_context(&no_time),
            "Day 2 / 水族館に入館"
        );
    }

    #[test]
    fn test_travel_book_note_heading_label() {
        assert_eq!(
            travel_book_note_heading_label(&ExportNote::Trip {
                title: None,
                body: String::new(),
            }),
            "Trip — Trip note"
        );
        assert_eq!(
            travel_book_note_heading_label(&ExportNote::Day {
                day_number: 1,
                title: Some("memo".to_string()),
                body: String::new(),
            }),
            "Day 1 — memo"
        );
        assert_eq!(
            travel_book_note_heading_label(&ExportNote::Itinerary {
                itinerary_key: ItineraryNoteKey {
                    day_number: 1,
                    sort_order: 6,
                    start_time: Some("08:30".to_string()),
                    title: "NU045 NGO ⇒ OKA (11:00着)".to_string(),
                },
                title: None,
                body: String::new(),
            }),
            "Day 1 / 08:30 NU045 NGO ⇒ OKA (11:00着) — Itinerary note"
        );
    }

    #[test]
    fn test_format_travel_book_reservation_period_human_readable() {
        assert_eq!(
            format_travel_book_reservation_period(
                &Some("2026-04-26T16:40".to_string()),
                &Some("2026-04-29T10:00".to_string()),
            ),
            Some("2026-04-26 16:40 〜 2026-04-29 10:00".to_string())
        );
        assert_eq!(
            format_travel_book_reservation_period(
                &Some("2026-04-26T16:40".to_string()),
                &Some("2026-04-26T18:00".to_string()),
            ),
            Some("2026-04-26 16:40 〜 18:00".to_string())
        );
        assert_eq!(format_travel_book_reservation_period(&None, &None), None);
    }
}
