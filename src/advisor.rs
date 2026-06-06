use anyhow::Result;
use rusqlite::Connection;

use crate::models::{DoctorIssue, DoctorIssueCode};

/// 1件の issue に対する改善提案を生成する
pub(crate) fn generate_advice(issue: &DoctorIssue) -> Vec<String> {
    match issue.code {
        DoctorIssueCode::EmptyItinerary => {
            vec!["Start by adding at least one itinerary.".to_string()]
        }
        DoctorIssueCode::NoRestaurant => {
            vec!["Consider adding a lunch or dinner plan.".to_string()]
        }
        DoctorIssueCode::HighTravelTime => vec![
            "Consider reducing travel time.".to_string(),
            "Group nearby attractions together.".to_string(),
        ],
        DoctorIssueCode::MissingDuration => vec![
            "Add an estimated duration.".to_string(),
            "Even a rough estimate improves planning quality.".to_string(),
        ],
        DoctorIssueCode::OverloadedDay => vec![
            "Consider moving some activities to another day.".to_string(),
            "Leave buffer time for delays and rest.".to_string(),
        ],
    }
}

/// 旅行計画の改善提案を表示する
pub(crate) fn run_trip_advisor(conn: &Connection, trip_id: i64) -> Result<()> {
    let trip = crate::trip::get_trip(conn, trip_id)?;
    let issues = crate::doctor::analyze_trip_issues(conn, trip_id)?;

    println!("Trip Advisor");
    println!("============");
    println!();
    println!("Trip: {}", trip.name);
    println!();

    if issues.is_empty() {
        println!("No major issues found.");
        return Ok(());
    }

    let only_empty = issues.len() == 1 && issues[0].code == DoctorIssueCode::EmptyItinerary;
    if only_empty {
        let issue = &issues[0];
        println!("Info");
        println!("----");
        println!("- {}", issue.warning_message());
        println!();
        print_advice_block(issue);
        return Ok(());
    }

    for issue in &issues {
        if issue.code == DoctorIssueCode::EmptyItinerary {
            continue;
        }
        println!("Warning");
        println!("-------");
        println!("- {}", issue.warning_message());
        println!();
        print_advice_block(issue);
        println!();
    }

    Ok(())
}

fn print_advice_block(issue: &DoctorIssue) {
    println!("Advice");
    println!("------");
    for advice in generate_advice(issue) {
        println!("- {advice}");
    }
}

#[cfg(test)]
mod advisor_tests {
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
    fn test_generate_advice_for_each_issue_code() {
        let empty = DoctorIssue {
            code: DoctorIssueCode::EmptyItinerary,
            day: None,
            itinerary_count: None,
            missing_duration_count: None,
            travel_minutes: None,
        };
        assert_eq!(
            generate_advice(&empty),
            vec!["Start by adding at least one itinerary.".to_string()]
        );

        let restaurant = DoctorIssue {
            code: DoctorIssueCode::NoRestaurant,
            day: Some(1),
            itinerary_count: None,
            missing_duration_count: None,
            travel_minutes: None,
        };
        assert_eq!(
            generate_advice(&restaurant),
            vec!["Consider adding a lunch or dinner plan.".to_string()]
        );

        let travel = DoctorIssue {
            code: DoctorIssueCode::HighTravelTime,
            day: Some(1),
            itinerary_count: None,
            missing_duration_count: None,
            travel_minutes: Some(200),
        };
        assert_eq!(generate_advice(&travel).len(), 2);

        let duration = DoctorIssue {
            code: DoctorIssueCode::MissingDuration,
            day: None,
            itinerary_count: None,
            missing_duration_count: Some(2),
            travel_minutes: None,
        };
        assert_eq!(generate_advice(&duration).len(), 2);

        let overloaded = DoctorIssue {
            code: DoctorIssueCode::OverloadedDay,
            day: Some(1),
            itinerary_count: Some(8),
            missing_duration_count: None,
            travel_minutes: None,
        };
        assert_eq!(generate_advice(&overloaded).len(), 2);
    }

    #[test]
    fn test_advisor_clean_trip_has_no_issues_message() {
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
            Some(20),
            None,
            Some(ItineraryCategory::Restaurant),
        )
        .unwrap();

        let issues = crate::doctor::analyze_trip_issues(&conn, trip_id).unwrap();
        assert!(issues.is_empty());
    }

    #[test]
    fn test_advisor_empty_itinerary_generates_advice() {
        let conn = test_db();
        let trip_id = add_trip(&conn, "空の旅行", None, None).unwrap();

        let issues = crate::doctor::analyze_trip_issues(&conn, trip_id).unwrap();
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].code, DoctorIssueCode::EmptyItinerary);
        assert_eq!(generate_advice(&issues[0]).len(), 1);
        run_trip_advisor(&conn, trip_id).unwrap();
    }

    #[test]
    fn test_advisor_no_restaurant_issue_has_advice() {
        let conn = test_db();
        let trip_id = add_trip(&conn, "食事なし旅行", None, None).unwrap();
        add_itinerary_item(
            &conn,
            trip_id,
            1,
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

        let issues = crate::doctor::analyze_trip_issues(&conn, trip_id).unwrap();
        let restaurant_issue = issues
            .iter()
            .find(|issue| issue.code == DoctorIssueCode::NoRestaurant)
            .expect("restaurant issue");
        assert!(!generate_advice(restaurant_issue).is_empty());
    }

    #[test]
    fn test_doctor_report_unchanged_for_existing_behavior() {
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

        let report = crate::doctor::analyze_trip(&conn, trip_id).unwrap();
        assert!(report
            .warnings
            .iter()
            .any(|w| w == "Day 3 has no restaurant"));
        assert!(report
            .suggestions
            .iter()
            .any(|s| s == "Consider adding a lunch or dinner plan to Day 3"));
    }
}
