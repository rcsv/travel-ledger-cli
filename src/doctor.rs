use std::collections::HashMap;

use anyhow::Result;
use rusqlite::Connection;

use crate::models::{
    DoctorIssue, DoctorIssueCode, DoctorIssueTarget, DoctorReportJson, ItineraryCategory,
    ItineraryItem,
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

/// 旅行計画の問題一覧を JSON envelope として返す
pub(crate) fn trip_doctor_report_json(conn: &Connection, trip_id: i64) -> Result<DoctorReportJson> {
    let issues = analyze_trip_issues(conn, trip_id)?;
    let json_issues = issues
        .iter()
        .map(|issue| issue.to_issue_json(trip_id))
        .collect();
    Ok(DoctorReportJson::new(trip_id, json_issues))
}

/// 旅行計画の点検結果を表示する
pub(crate) fn run_trip_doctor(conn: &Connection, trip_id: i64, json: bool) -> Result<()> {
    if json {
        let report = trip_doctor_report_json(conn, trip_id)?;
        crate::trip::print_json(&report)?;
        return Ok(());
    }

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
    use crate::trip::add_test_trip;
    use rusqlite::Connection;

    fn test_db() -> Connection {
        open_db_at(":memory:").expect("インメモリ DB の作成に失敗")
    }

    #[test]
    fn test_empty_itinerary_has_trip_target() {
        let conn = test_db();
        let trip_id = add_test_trip(&conn, "空の旅行").unwrap();

        let issues = analyze_trip_issues(&conn, trip_id).unwrap();
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].code, DoctorIssueCode::EmptyItinerary);
        assert_eq!(issues[0].target, DoctorIssueTarget::Trip);
    }

    #[test]
    fn test_day_issues_have_day_target() {
        let conn = test_db();
        let trip_id = add_test_trip(&conn, "食事なし旅行").unwrap();
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
        let trip_id = add_test_trip(&conn, "時間未設定旅行").unwrap();
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
        let trip_id = add_test_trip(&conn, "詰め込み旅行").unwrap();

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
        let trip_id = add_test_trip(&conn, "食事なし旅行").unwrap();
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
        let trip_id = add_test_trip(&conn, "移動多め旅行").unwrap();
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
        let trip_id = add_test_trip(&conn, "時間未設定旅行").unwrap();
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
        let trip_id = add_test_trip(&conn, "時間未設定旅行").unwrap();
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
        let trip_id = add_test_trip(&conn, "問題なし旅行").unwrap();
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
        let trip_id = add_test_trip(&conn, "空の旅行").unwrap();

        let report = analyze_trip(&conn, trip_id).unwrap();
        assert!(report.warnings.is_empty());
        assert!(report.suggestions.is_empty());
        assert_eq!(report.info, vec!["No itinerary found.".to_string()]);

        let issues = analyze_trip_issues(&conn, trip_id).unwrap();
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].code, DoctorIssueCode::EmptyItinerary);
        run_trip_doctor(&conn, trip_id, false).unwrap();
    }

    #[test]
    fn test_trip_doctor_json_clean() {
        let conn = test_db();
        let trip_id = add_test_trip(&conn, "問題なし旅行").unwrap();
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

        let report = trip_doctor_report_json(&conn, trip_id).unwrap();
        let json = serde_json::to_string_pretty(&report).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        assert_eq!(report.issues.len(), 0);
        assert_eq!(parsed["schema_version"], 1);
        assert_eq!(parsed["trip_id"], trip_id);
        assert_eq!(parsed["issues"], serde_json::json!([]));
    }

    #[test]
    fn test_trip_doctor_json_missing_duration() {
        let conn = test_db();
        let trip_id = add_test_trip(&conn, "時間未設定旅行").unwrap();
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

        let report = trip_doctor_report_json(&conn, trip_id).unwrap();
        let json = serde_json::to_string_pretty(&report).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        let missing = report
            .issues
            .iter()
            .find(|issue| issue.code == DoctorIssueCode::MissingDuration)
            .expect("missing duration json issue");
        assert_eq!(
            missing.severity,
            crate::models::DoctorIssueSeverity::Warning
        );
        assert_eq!(
            missing.target.target_type,
            crate::models::IssueTargetType::Itinerary
        );
        assert_eq!(missing.details.itinerary_id, Some(missing.target.id));

        assert_eq!(parsed["schema_version"], 1);
        assert!(parsed["issues"]
            .as_array()
            .unwrap()
            .iter()
            .any(|issue| issue["code"] == "missing_duration"));
        assert!(parsed["issues"]
            .as_array()
            .unwrap()
            .iter()
            .any(|issue| issue["target"]["type"] == "itinerary"));
    }

    #[test]
    fn test_trip_doctor_json_combined_issues() {
        let conn = test_db();
        let trip_id = add_test_trip(&conn, "複合問題旅行").unwrap();

        for i in 0..8 {
            add_itinerary_item(
                &conn,
                trip_id,
                1,
                &format!("Activity {i}"),
                None,
                None,
                Some(i),
                Some(30),
                Some(25),
                None,
                Some(ItineraryCategory::Activity),
            )
            .unwrap();
        }
        add_itinerary_item(
            &conn,
            trip_id,
            2,
            "Free time",
            None,
            None,
            Some(1),
            None,
            None,
            None,
            None,
        )
        .unwrap();

        let report = trip_doctor_report_json(&conn, trip_id).unwrap();
        let parsed: serde_json::Value =
            serde_json::from_str(&serde_json::to_string_pretty(&report).unwrap()).unwrap();

        assert!(report.issues.len() >= 3);
        assert!(report
            .issues
            .iter()
            .any(|issue| issue.code == DoctorIssueCode::OverloadedDay));
        assert!(report
            .issues
            .iter()
            .any(|issue| issue.code == DoctorIssueCode::NoRestaurant));
        assert!(report
            .issues
            .iter()
            .any(|issue| issue.code == DoctorIssueCode::HighTravelTime));
        assert!(report
            .issues
            .iter()
            .any(|issue| issue.code == DoctorIssueCode::MissingDuration));
        assert_eq!(parsed["schema_version"], 1);
        assert!(parsed["issues"].as_array().unwrap().len() >= 3);
    }
}
