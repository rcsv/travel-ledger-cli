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

/// issue メッセージから Day N を抽出する（例: `Day 1 has no restaurant`）
pub(crate) fn extract_day_from_message(message: &str) -> Option<i64> {
    let rest = message.strip_prefix("Day ")?;
    let day_str: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
    if day_str.is_empty() {
        return None;
    }
    day_str.parse().ok()
}

fn day_for_issue(issue: &DoctorIssue) -> i64 {
    issue
        .target_day()
        .or_else(|| extract_day_from_message(&issue.warning_message()))
        .filter(|&d| d > 0)
        .unwrap_or(1)
}

/// 1件の issue に対する CLI コマンド例を生成する
pub(crate) fn generate_command_hints(issue: &DoctorIssue, trip_id: i64) -> Vec<String> {
    match issue.code {
        DoctorIssueCode::EmptyItinerary => vec![format!(
            r#"cargo run -- itinerary add {trip_id} --day 1 --time 09:00 --duration 60 "First activity""#
        )],
        DoctorIssueCode::NoRestaurant => {
            let day = day_for_issue(issue);
            vec![
                format!(
                    r#"cargo run -- itinerary add {trip_id} --day {day} --time 12:00 --duration 60 "Lunch""#
                ),
                "cargo run -- itinerary update <itinerary_id> --category restaurant".to_string(),
            ]
        }
        DoctorIssueCode::HighTravelTime | DoctorIssueCode::OverloadedDay => vec![
            format!("cargo run -- itinerary timeline {trip_id}"),
            format!("cargo run -- itinerary list {trip_id}"),
        ],
        DoctorIssueCode::MissingDuration => {
            if let Some(itinerary_id) = issue.target_itinerary_id() {
                vec![format!(
                    "cargo run -- itinerary update {itinerary_id} --duration 60"
                )]
            } else {
                vec![format!("cargo run -- itinerary list {trip_id}")]
            }
        }
    }
}

fn format_try_section(issue: &DoctorIssue, trip_id: i64) -> String {
    let hints = generate_command_hints(issue, trip_id);
    if hints.is_empty() {
        return String::new();
    }

    let mut section = String::from("\nTry\n---\n");
    for hint in hints {
        section.push_str(&hint);
        section.push('\n');
    }
    section
}

/// 旅行計画の改善提案を表示する
pub(crate) fn run_trip_advisor(conn: &Connection, trip_id: i64, with_commands: bool) -> Result<()> {
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
        print_issue_followup(issue, trip_id, with_commands);
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
        print_issue_followup(issue, trip_id, with_commands);
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

fn print_issue_followup(issue: &DoctorIssue, trip_id: i64, with_commands: bool) {
    print_advice_block(issue);
    if with_commands {
        let try_section = format_try_section(issue, trip_id);
        if !try_section.is_empty() {
            print!("{try_section}");
        }
    }
}

#[cfg(test)]
mod advisor_tests {
    use super::*;
    use crate::db::open_db_at;
    use crate::itinerary::add_itinerary_item;
    use crate::models::{DoctorIssueTarget, ItineraryCategory};
    use crate::trip::add_trip;
    use rusqlite::Connection;

    fn test_db() -> Connection {
        open_db_at(":memory:").expect("インメモリ DB の作成に失敗")
    }

    #[test]
    fn test_generate_advice_for_each_issue_code() {
        let empty = DoctorIssue {
            code: DoctorIssueCode::EmptyItinerary,
            target: DoctorIssueTarget::Trip,
            day: None,
            itinerary_count: None,
            travel_minutes: None,
        };
        assert_eq!(
            generate_advice(&empty),
            vec!["Start by adding at least one itinerary.".to_string()]
        );

        let restaurant = DoctorIssue {
            code: DoctorIssueCode::NoRestaurant,
            target: DoctorIssueTarget::Day(1),
            day: Some(1),
            itinerary_count: None,
            travel_minutes: None,
        };
        assert_eq!(
            generate_advice(&restaurant),
            vec!["Consider adding a lunch or dinner plan.".to_string()]
        );

        let travel = DoctorIssue {
            code: DoctorIssueCode::HighTravelTime,
            target: DoctorIssueTarget::Day(1),
            day: Some(1),
            itinerary_count: None,
            travel_minutes: Some(200),
        };
        assert_eq!(generate_advice(&travel).len(), 2);

        let duration = DoctorIssue {
            code: DoctorIssueCode::MissingDuration,
            target: DoctorIssueTarget::Itinerary(3),
            day: None,
            itinerary_count: None,
            travel_minutes: None,
        };
        assert_eq!(generate_advice(&duration).len(), 2);

        let overloaded = DoctorIssue {
            code: DoctorIssueCode::OverloadedDay,
            target: DoctorIssueTarget::Day(1),
            day: Some(1),
            itinerary_count: Some(8),
            travel_minutes: None,
        };
        assert_eq!(generate_advice(&overloaded).len(), 2);
    }

    #[test]
    fn test_extract_day_from_message() {
        assert_eq!(extract_day_from_message("Day 1 has no restaurant"), Some(1));
        assert_eq!(
            extract_day_from_message("Day 3 has high travel time (3h25m)"),
            Some(3)
        );
        assert_eq!(
            extract_day_from_message("Day 2 has many itineraries (8)"),
            Some(2)
        );
        assert_eq!(
            extract_day_from_message("Itinerary 3 has no duration estimate"),
            None
        );
        assert_eq!(extract_day_from_message("No itinerary found."), None);
    }

    #[test]
    fn test_extract_day_from_message_does_not_panic_on_unexpected_input() {
        assert_eq!(extract_day_from_message(""), None);
        assert_eq!(extract_day_from_message("Day has no restaurant"), None);
        assert_eq!(extract_day_from_message("Unexpected warning"), None);
    }

    #[test]
    fn test_generate_command_hints_for_each_issue_code() {
        let trip_id = 1;

        let empty = DoctorIssue {
            code: DoctorIssueCode::EmptyItinerary,
            target: DoctorIssueTarget::Trip,
            day: None,
            itinerary_count: None,
            travel_minutes: None,
        };
        assert_eq!(
            generate_command_hints(&empty, trip_id),
            vec![r#"cargo run -- itinerary add 1 --day 1 --time 09:00 --duration 60 "First activity""#
                .to_string()]
        );

        let restaurant = DoctorIssue {
            code: DoctorIssueCode::NoRestaurant,
            target: DoctorIssueTarget::Day(2),
            day: Some(2),
            itinerary_count: None,
            travel_minutes: None,
        };
        assert_eq!(
            generate_command_hints(&restaurant, trip_id),
            vec![
                r#"cargo run -- itinerary add 1 --day 2 --time 12:00 --duration 60 "Lunch""#
                    .to_string(),
                "cargo run -- itinerary update <itinerary_id> --category restaurant".to_string(),
            ]
        );

        let travel = DoctorIssue {
            code: DoctorIssueCode::HighTravelTime,
            target: DoctorIssueTarget::Day(1),
            day: Some(1),
            itinerary_count: None,
            travel_minutes: Some(205),
        };
        assert_eq!(
            generate_command_hints(&travel, trip_id),
            vec![
                "cargo run -- itinerary timeline 1".to_string(),
                "cargo run -- itinerary list 1".to_string(),
            ]
        );

        let duration = DoctorIssue {
            code: DoctorIssueCode::MissingDuration,
            target: DoctorIssueTarget::Itinerary(3),
            day: None,
            itinerary_count: None,
            travel_minutes: None,
        };
        assert_eq!(
            generate_command_hints(&duration, trip_id),
            vec!["cargo run -- itinerary update 3 --duration 60".to_string()]
        );

        let overloaded = DoctorIssue {
            code: DoctorIssueCode::OverloadedDay,
            target: DoctorIssueTarget::Day(1),
            day: Some(1),
            itinerary_count: Some(8),
            travel_minutes: None,
        };
        assert_eq!(
            generate_command_hints(&overloaded, trip_id),
            vec![
                "cargo run -- itinerary timeline 1".to_string(),
                "cargo run -- itinerary list 1".to_string(),
            ]
        );
    }

    #[test]
    fn test_no_restaurant_uses_target_day() {
        let issue = DoctorIssue {
            code: DoctorIssueCode::NoRestaurant,
            target: DoctorIssueTarget::Day(3),
            day: None,
            itinerary_count: None,
            travel_minutes: None,
        };
        let hints = generate_command_hints(&issue, 5);
        assert_eq!(hints.len(), 2);
        assert!(hints[0].contains("--day 3"));
        assert!(hints[1].contains("--category restaurant"));
    }

    #[test]
    fn test_no_restaurant_falls_back_to_message_parse_without_target_day() {
        let issue = DoctorIssue {
            code: DoctorIssueCode::NoRestaurant,
            target: DoctorIssueTarget::Trip,
            day: None,
            itinerary_count: None,
            travel_minutes: None,
        };
        let hints = generate_command_hints(&issue, 5);
        assert_eq!(hints.len(), 2);
        assert!(hints[0].contains("--day 1"));
    }

    #[test]
    fn test_missing_duration_uses_itinerary_update_command() {
        let issue = DoctorIssue {
            code: DoctorIssueCode::MissingDuration,
            target: DoctorIssueTarget::Itinerary(7),
            day: None,
            itinerary_count: None,
            travel_minutes: None,
        };
        assert_eq!(
            generate_command_hints(&issue, 1),
            vec!["cargo run -- itinerary update 7 --duration 60".to_string()]
        );
    }

    #[test]
    fn test_missing_duration_without_itinerary_target_falls_back_to_list() {
        let issue = DoctorIssue {
            code: DoctorIssueCode::MissingDuration,
            target: DoctorIssueTarget::Trip,
            day: None,
            itinerary_count: None,
            travel_minutes: None,
        };
        assert_eq!(
            generate_command_hints(&issue, 1),
            vec!["cargo run -- itinerary list 1".to_string()]
        );
    }

    #[test]
    fn test_format_try_section_only_when_with_commands() {
        let issue = DoctorIssue {
            code: DoctorIssueCode::NoRestaurant,
            target: DoctorIssueTarget::Day(1),
            day: Some(1),
            itinerary_count: None,
            travel_minutes: None,
        };

        let advice_only = {
            let mut lines = vec!["Advice".to_string(), "------".to_string()];
            for advice in generate_advice(&issue) {
                lines.push(format!("- {advice}"));
            }
            lines.join("\n")
        };
        let try_section = format_try_section(&issue, 1);

        assert!(!advice_only.contains("Try"));
        assert!(try_section.contains("Try"));
        assert!(try_section.contains("itinerary add 1 --day 1"));
        assert!(try_section.contains("update <itinerary_id> --category restaurant"));
    }

    #[test]
    fn test_clean_trip_has_no_issues_and_no_try_sections() {
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
        run_trip_advisor(&conn, trip_id, false).unwrap();
    }

    #[test]
    fn test_advisor_empty_itinerary_with_commands_includes_try() {
        let issue = DoctorIssue {
            code: DoctorIssueCode::EmptyItinerary,
            target: DoctorIssueTarget::Trip,
            day: None,
            itinerary_count: None,
            travel_minutes: None,
        };
        let try_section = format_try_section(&issue, 1);
        assert!(try_section.contains("itinerary add 1"));
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
