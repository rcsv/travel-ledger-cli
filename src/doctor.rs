use std::collections::HashMap;

use anyhow::Result;
use rusqlite::Connection;

use crate::models::{
    DoctorIssue, DoctorIssueCode, DoctorIssueTarget, ItineraryCategory, ItineraryItem,
};

const MAX_ITINERARIES_PER_DAY: usize = 7;
const MAX_TRAVEL_MINUTES_PER_DAY: i64 = 180;

/// 旅行計画の点検結果
pub(crate) struct DoctorReport {
    pub warnings: Vec<String>,
    pub suggestions: Vec<String>,
    pub info: Vec<String>,
}

/// 旅行計画の問題一覧を構造化して返す
pub(crate) fn analyze_trip_issues(conn: &Connection, trip_id: i64) -> Result<Vec<DoctorIssue>> {
    crate::trip::get_trip(conn, trip_id)?;
    let items = crate::itinerary::list_itinerary_items(conn, trip_id)?;
    Ok(collect_trip_issues(&items))
}

fn collect_trip_issues(items: &[ItineraryItem]) -> Vec<DoctorIssue> {
    if items.is_empty() {
        return vec![DoctorIssue {
            code: DoctorIssueCode::EmptyItinerary,
            target: DoctorIssueTarget::Trip,
            day: None,
            itinerary_count: None,
            travel_minutes: None,
        }];
    }

    let mut by_day: HashMap<i64, Vec<&ItineraryItem>> = HashMap::new();
    for item in items {
        by_day.entry(item.day).or_default().push(item);
    }

    let mut days: Vec<i64> = by_day.keys().copied().collect();
    days.sort_unstable();

    let mut issues = Vec::new();

    for day in days {
        let day_items = &by_day[&day];
        let count = day_items.len();

        if count >= MAX_ITINERARIES_PER_DAY {
            issues.push(DoctorIssue {
                code: DoctorIssueCode::OverloadedDay,
                target: DoctorIssueTarget::Day(day),
                day: Some(day),
                itinerary_count: Some(count),
                travel_minutes: None,
            });
        }

        let has_restaurant = day_items
            .iter()
            .any(|item| item.category == Some(ItineraryCategory::Restaurant));
        if !has_restaurant {
            issues.push(DoctorIssue {
                code: DoctorIssueCode::NoRestaurant,
                target: DoctorIssueTarget::Day(day),
                day: Some(day),
                itinerary_count: None,
                travel_minutes: None,
            });
        }

        let travel_total: i64 = day_items
            .iter()
            .filter_map(|item| item.travel_minutes)
            .sum();
        if travel_total >= MAX_TRAVEL_MINUTES_PER_DAY {
            issues.push(DoctorIssue {
                code: DoctorIssueCode::HighTravelTime,
                target: DoctorIssueTarget::Day(day),
                day: Some(day),
                itinerary_count: None,
                travel_minutes: Some(travel_total),
            });
        }
    }

    for item in items {
        if item.duration_minutes.is_none() {
            issues.push(DoctorIssue {
                code: DoctorIssueCode::MissingDuration,
                target: DoctorIssueTarget::Itinerary(item.id),
                day: None,
                itinerary_count: None,
                travel_minutes: None,
            });
        }
    }

    issues
}

fn missing_duration_warning(count: usize) -> String {
    if count == 1 {
        "1 itinerary has no duration estimate".to_string()
    } else {
        format!("{count} itineraries have no duration estimate")
    }
}

fn issues_to_doctor_report(issues: &[DoctorIssue]) -> DoctorReport {
    let mut warnings = Vec::new();
    let mut suggestions = Vec::new();
    let mut info = Vec::new();

    let missing_duration_count = issues
        .iter()
        .filter(|issue| issue.code == DoctorIssueCode::MissingDuration)
        .count();

    for issue in issues {
        match issue.code {
            DoctorIssueCode::EmptyItinerary => {
                info.push(issue.warning_message());
            }
            DoctorIssueCode::NoRestaurant => {
                warnings.push(issue.warning_message());
                if let Some(day) = issue.target_day() {
                    suggestions.push(format!(
                        "Consider adding a lunch or dinner plan to Day {day}"
                    ));
                }
            }
            DoctorIssueCode::HighTravelTime => {
                warnings.push(issue.warning_message());
                if let Some(day) = issue.target_day() {
                    suggestions.push(format!("Consider reducing travel time on Day {day}"));
                }
            }
            DoctorIssueCode::OverloadedDay => {
                warnings.push(issue.warning_message());
            }
            DoctorIssueCode::MissingDuration => {}
        }
    }

    if missing_duration_count > 0 {
        warnings.push(missing_duration_warning(missing_duration_count));
    }

    DoctorReport {
        warnings,
        suggestions,
        info,
    }
}

/// 旅行計画を分析し、警告と提案を返す
pub(crate) fn analyze_trip(conn: &Connection, trip_id: i64) -> Result<DoctorReport> {
    let issues = analyze_trip_issues(conn, trip_id)?;
    Ok(issues_to_doctor_report(&issues))
}

/// 旅行計画の点検結果を表示する
pub(crate) fn run_trip_doctor(conn: &Connection, trip_id: i64) -> Result<()> {
    let trip = crate::trip::get_trip(conn, trip_id)?;
    let report = analyze_trip(conn, trip_id)?;

    println!("Trip Doctor");
    println!("===========");
    println!();
    println!("Trip: {}", trip.name);
    println!();

    if report.warnings.is_empty() && report.suggestions.is_empty() && report.info.is_empty() {
        println!("No major issues found.");
        return Ok(());
    }

    if !report.warnings.is_empty() {
        println!("Warnings");
        println!("--------");
        for warning in &report.warnings {
            println!("- {warning}");
        }
        println!();
    }

    if !report.suggestions.is_empty() {
        println!("Suggestions");
        println!("-----------");
        for suggestion in &report.suggestions {
            println!("- {suggestion}");
        }
        if !report.info.is_empty() {
            println!();
        }
    }

    if !report.info.is_empty() {
        println!("Info");
        println!("----");
        for message in &report.info {
            println!("- {message}");
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::open_db_at;
    use crate::itinerary::add_itinerary_item;
    use crate::models::ItineraryCategory;
    use crate::trip::add_trip;
    use rusqlite::Connection;

    fn test_db() -> Connection {
        open_db_at(":memory:").expect("インメモリ DB の作成に失敗")
    }

    #[test]
    fn test_empty_itinerary_has_trip_target() {
        let conn = test_db();
        let trip_id = add_trip(&conn, "空の旅行", None, None).unwrap();

        let issues = analyze_trip_issues(&conn, trip_id).unwrap();
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].code, DoctorIssueCode::EmptyItinerary);
        assert_eq!(issues[0].target, DoctorIssueTarget::Trip);
    }

    #[test]
    fn test_day_issues_have_day_target() {
        let conn = test_db();
        let trip_id = add_trip(&conn, "食事なし旅行", None, None).unwrap();
        add_itinerary_item(
            &conn,
            trip_id,
            3,
            "観光",
            None,
            None,
            Some(1),
            Some(90),
            None,
            None,
            Some(ItineraryCategory::Activity),
        )
        .unwrap();

        let issues = analyze_trip_issues(&conn, trip_id).unwrap();
        let restaurant = issues
            .iter()
            .find(|issue| issue.code == DoctorIssueCode::NoRestaurant)
            .expect("restaurant issue");
        assert_eq!(restaurant.target, DoctorIssueTarget::Day(3));
    }

    #[test]
    fn test_missing_duration_issues_have_itinerary_target() {
        let conn = test_db();
        let trip_id = add_trip(&conn, "時間未設定旅行", None, None).unwrap();
        for i in 1..=3 {
            add_itinerary_item(
                &conn,
                trip_id,
                1,
                &format!("予定{i}"),
                None,
                None,
                Some(i),
                None,
                None,
                None,
                None,
            )
            .unwrap();
        }

        let issues = analyze_trip_issues(&conn, trip_id).unwrap();
        let missing: Vec<_> = issues
            .iter()
            .filter(|issue| issue.code == DoctorIssueCode::MissingDuration)
            .collect();
        assert_eq!(missing.len(), 3);
        for issue in missing {
            assert!(matches!(issue.target, DoctorIssueTarget::Itinerary(_)));
        }
    }

    #[test]
    fn test_doctor_detects_many_itineraries_per_day() {
        let conn = test_db();
        let trip_id = add_trip(&conn, "詰め込み旅行", None, None).unwrap();

        for i in 0..8 {
            add_itinerary_item(
                &conn,
                trip_id,
                2,
                &format!("予定{i}"),
                None,
                None,
                Some(i),
                Some(60),
                None,
                None,
                Some(ItineraryCategory::Activity),
            )
            .unwrap();
        }

        let report = analyze_trip(&conn, trip_id).unwrap();
        assert!(report
            .warnings
            .iter()
            .any(|w| w == "Day 2 has many itineraries (8)"));

        let issues = analyze_trip_issues(&conn, trip_id).unwrap();
        let overloaded = issues
            .iter()
            .find(|issue| issue.code == DoctorIssueCode::OverloadedDay)
            .expect("overloaded issue");
        assert_eq!(overloaded.target, DoctorIssueTarget::Day(2));
    }

    #[test]
    fn test_doctor_detects_missing_restaurant() {
        let conn = test_db();
        let trip_id = add_trip(&conn, "食事なし旅行", None, None).unwrap();
        add_itinerary_item(
            &conn,
            trip_id,
            3,
            "観光",
            None,
            None,
            Some(1),
            Some(90),
            None,
            None,
            Some(ItineraryCategory::Activity),
        )
        .unwrap();

        let report = analyze_trip(&conn, trip_id).unwrap();
        assert!(report
            .warnings
            .iter()
            .any(|w| w == "Day 3 has no restaurant"));
        assert!(report
            .suggestions
            .iter()
            .any(|s| { s == "Consider adding a lunch or dinner plan to Day 3" }));
    }

    #[test]
    fn test_doctor_detects_high_travel_time() {
        let conn = test_db();
        let trip_id = add_trip(&conn, "移動多め旅行", None, None).unwrap();
        add_itinerary_item(
            &conn,
            trip_id,
            4,
            "移動1",
            None,
            None,
            Some(1),
            Some(60),
            Some(100),
            None,
            Some(ItineraryCategory::Transport),
        )
        .unwrap();
        add_itinerary_item(
            &conn,
            trip_id,
            4,
            "移動2",
            None,
            None,
            Some(2),
            Some(60),
            Some(90),
            None,
            Some(ItineraryCategory::Transport),
        )
        .unwrap();

        let report = analyze_trip(&conn, trip_id).unwrap();
        assert!(report
            .warnings
            .iter()
            .any(|w| w == "Day 4 has high travel time (3h10m)"));
        assert!(report
            .suggestions
            .iter()
            .any(|s| s == "Consider reducing travel time on Day 4"));

        let issues = analyze_trip_issues(&conn, trip_id).unwrap();
        let travel = issues
            .iter()
            .find(|issue| issue.code == DoctorIssueCode::HighTravelTime)
            .expect("travel issue");
        assert_eq!(travel.target, DoctorIssueTarget::Day(4));
    }

    #[test]
    fn test_doctor_detects_missing_duration_singular() {
        let conn = test_db();
        let trip_id = add_trip(&conn, "時間未設定旅行", None, None).unwrap();
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

        let report = analyze_trip(&conn, trip_id).unwrap();
        assert!(report
            .warnings
            .iter()
            .any(|w| w == "1 itinerary has no duration estimate"));

        let issues = analyze_trip_issues(&conn, trip_id).unwrap();
        let missing = issues
            .iter()
            .find(|issue| issue.code == DoctorIssueCode::MissingDuration)
            .expect("missing duration issue");
        assert!(matches!(missing.target, DoctorIssueTarget::Itinerary(_)));
    }

    #[test]
    fn test_doctor_detects_missing_duration_plural() {
        let conn = test_db();
        let trip_id = add_trip(&conn, "時間未設定旅行", None, None).unwrap();
        for i in 1..=3 {
            add_itinerary_item(
                &conn,
                trip_id,
                1,
                &format!("予定{i}"),
                None,
                None,
                Some(i),
                None,
                None,
                None,
                None,
            )
            .unwrap();
        }

        let report = analyze_trip(&conn, trip_id).unwrap();
        assert!(report
            .warnings
            .iter()
            .any(|w| w == "3 itineraries have no duration estimate"));
    }

    #[test]
    fn test_doctor_clean_trip_has_no_issues() {
        let conn = test_db();
        let trip_id = add_trip(&conn, "問題なし旅行", None, None).unwrap();
        add_itinerary_item(
            &conn,
            trip_id,
            1,
            "昼食",
            None,
            None,
            Some(1),
            Some(60),
            Some(30),
            None,
            Some(ItineraryCategory::Restaurant),
        )
        .unwrap();

        let report = analyze_trip(&conn, trip_id).unwrap();
        assert!(report.warnings.is_empty());
        assert!(report.suggestions.is_empty());
        assert!(report.info.is_empty());
        assert!(analyze_trip_issues(&conn, trip_id).unwrap().is_empty());
    }

    #[test]
    fn test_doctor_empty_itinerary_reports_info() {
        let conn = test_db();
        let trip_id = add_trip(&conn, "空の旅行", None, None).unwrap();

        let report = analyze_trip(&conn, trip_id).unwrap();
        assert!(report.warnings.is_empty());
        assert!(report.suggestions.is_empty());
        assert_eq!(report.info, vec!["No itinerary found.".to_string()]);

        let issues = analyze_trip_issues(&conn, trip_id).unwrap();
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].code, DoctorIssueCode::EmptyItinerary);
        run_trip_doctor(&conn, trip_id).unwrap();
    }
}
