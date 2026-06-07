use std::collections::HashMap;

use anyhow::Result;
use rusqlite::Connection;
use serde::Serialize;

use crate::models::ItineraryCategory;

/// 旅行統計の集計結果
#[derive(Serialize)]
pub(crate) struct TripStats {
    pub trip_name: String,
    pub days: i64,
    pub itinerary_count: usize,
    pub checklist_total: usize,
    pub checklist_completed: usize,
    pub category_counts: HashMap<String, i64>,
    pub stay_minutes: i64,
    pub travel_minutes: i64,
    pub total_minutes: i64,
}

impl TripStats {
    pub fn total_minutes(&self) -> i64 {
        self.total_minutes
    }
}

/// 分単位の時間を表示用に整形する（例: 3h20m, 45m, 12h05m）
pub(crate) fn format_minutes_duration(total_minutes: i64) -> String {
    if total_minutes <= 0 {
        return "0m".to_string();
    }
    let hours = total_minutes / 60;
    let minutes = total_minutes % 60;
    if hours == 0 {
        format!("{minutes}m")
    } else {
        format!("{hours}h{minutes:02}m")
    }
}

/// 旅行統計を集計する
pub(crate) fn compute_trip_stats(conn: &Connection, trip_id: i64) -> Result<TripStats> {
    let trip = crate::trip::get_trip(conn, trip_id)?;
    let itinerary_items = crate::itinerary::list_itinerary_items(conn, trip_id)?;
    let checklist_items = crate::checklist::list_checklist_items(conn, trip_id)?;

    let days = itinerary_items
        .iter()
        .map(|item| item.day)
        .max()
        .unwrap_or(0);

    let mut category_counts: HashMap<String, i64> = HashMap::new();
    let mut uncategorized = 0i64;
    for item in &itinerary_items {
        match item.category {
            Some(category) => {
                *category_counts
                    .entry(category.as_str().to_string())
                    .or_insert(0) += 1;
            }
            None => uncategorized += 1,
        }
    }
    if uncategorized > 0 {
        category_counts.insert("uncategorized".to_string(), uncategorized);
    }

    let stay_minutes: i64 = itinerary_items
        .iter()
        .filter_map(|item| item.duration_minutes)
        .sum();
    let travel_minutes: i64 = itinerary_items
        .iter()
        .filter_map(|item| item.travel_minutes)
        .sum();

    let checklist_completed = checklist_items.iter().filter(|item| item.is_done).count();

    Ok(TripStats {
        trip_name: trip.name,
        days,
        itinerary_count: itinerary_items.len(),
        checklist_total: checklist_items.len(),
        checklist_completed,
        category_counts,
        stay_minutes,
        travel_minutes,
        total_minutes: stay_minutes + travel_minutes,
    })
}

/// 旅行統計を表示する
pub(crate) fn print_trip_stats(conn: &Connection, trip_id: i64) -> Result<()> {
    let stats = compute_trip_stats(conn, trip_id)?;

    println!("Trip Stats");
    println!("==========");
    println!();
    println!("Trip: {}", stats.trip_name);
    println!();
    println!("Days: {}", stats.days);
    println!();
    println!("Itineraries: {}", stats.itinerary_count);
    println!();
    println!("Checklist");
    println!("---------");
    println!(
        "Completed: {} / {}",
        stats.checklist_completed, stats.checklist_total
    );
    println!();

    let has_categories = ItineraryCategory::all()
        .iter()
        .any(|c| stats.category_counts.get(c.as_str()).copied().unwrap_or(0) > 0)
        || stats
            .category_counts
            .get("uncategorized")
            .copied()
            .unwrap_or(0)
            > 0;

    if has_categories {
        println!("Category Breakdown");
        println!("------------------");
        for category in ItineraryCategory::all() {
            let count = stats
                .category_counts
                .get(category.as_str())
                .copied()
                .unwrap_or(0);
            if count > 0 {
                println!("{:<12} {}", category.as_str(), count);
            }
        }
        if let Some(count) = stats.category_counts.get("uncategorized") {
            if *count > 0 {
                println!("{:<12} {}", "uncategorized", count);
            }
        }
        println!();
    }

    println!("Time Summary");
    println!("------------");
    println!(
        "Stay Time:   {}",
        format_minutes_duration(stats.stay_minutes)
    );
    println!(
        "Travel Time: {}",
        format_minutes_duration(stats.travel_minutes)
    );
    println!(
        "Total Time:  {}",
        format_minutes_duration(stats.total_minutes())
    );

    Ok(())
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

    #[test]
    fn test_stats_itinerary_count() {
        let conn = test_db();
        let trip_id = add_test_trip(&conn, "沖縄旅行").unwrap();
        crate::itinerary::add_itinerary_item(
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
        crate::itinerary::add_itinerary_item(
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

        let stats = compute_trip_stats(&conn, trip_id).unwrap();
        assert_eq!(stats.itinerary_count, 2);
        assert_eq!(stats.days, 2);
    }

    #[test]
    fn test_stats_category_breakdown() {
        let conn = test_db();
        let trip_id = add_test_trip(&conn, "沖縄旅行").unwrap();
        crate::itinerary::add_itinerary_item(
            &conn,
            trip_id,
            1,
            "フライト",
            None,
            None,
            None,
            None,
            None,
            None,
            Some(ItineraryCategory::Flight),
        )
        .unwrap();
        crate::itinerary::add_itinerary_item(
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
        crate::itinerary::add_itinerary_item(
            &conn, trip_id, 1, "散歩", None, None, None, None, None, None, None,
        )
        .unwrap();

        let stats = compute_trip_stats(&conn, trip_id).unwrap();
        assert_eq!(stats.category_counts.get("flight"), Some(&1));
        assert_eq!(stats.category_counts.get("hotel"), Some(&1));
        assert_eq!(stats.category_counts.get("uncategorized"), Some(&1));
    }

    #[test]
    fn test_stats_checklist_completion() {
        let conn = test_db();
        let trip_id = add_test_trip(&conn, "沖縄旅行").unwrap();
        let id1 = crate::checklist::add_checklist_item(&conn, trip_id, "パスポート").unwrap();
        crate::checklist::add_checklist_item(&conn, trip_id, "充電器").unwrap();
        crate::checklist::add_checklist_item(&conn, trip_id, "タオル").unwrap();
        crate::checklist::set_checklist_done(&conn, id1, true).unwrap();

        let stats = compute_trip_stats(&conn, trip_id).unwrap();
        assert_eq!(stats.checklist_total, 3);
        assert_eq!(stats.checklist_completed, 1);
    }

    #[test]
    fn test_stats_duration_total() {
        let conn = test_db();
        let trip_id = add_test_trip(&conn, "沖縄旅行").unwrap();
        crate::itinerary::add_itinerary_item(
            &conn,
            trip_id,
            1,
            "首里城",
            None,
            None,
            None,
            Some(90),
            None,
            None,
            None,
        )
        .unwrap();
        crate::itinerary::add_itinerary_item(
            &conn,
            trip_id,
            1,
            "昼食",
            None,
            None,
            None,
            Some(60),
            None,
            None,
            None,
        )
        .unwrap();

        let stats = compute_trip_stats(&conn, trip_id).unwrap();
        assert_eq!(stats.stay_minutes, 150);
        assert_eq!(format_minutes_duration(stats.stay_minutes), "2h30m");
    }

    #[test]
    fn test_stats_travel_time_total() {
        let conn = test_db();
        let trip_id = add_test_trip(&conn, "沖縄旅行").unwrap();
        crate::itinerary::add_itinerary_item(
            &conn,
            trip_id,
            1,
            "首里城",
            None,
            None,
            None,
            None,
            Some(20),
            None,
            None,
        )
        .unwrap();
        crate::itinerary::add_itinerary_item(
            &conn,
            trip_id,
            1,
            "国際通り",
            None,
            None,
            None,
            None,
            Some(45),
            None,
            None,
        )
        .unwrap();

        let stats = compute_trip_stats(&conn, trip_id).unwrap();
        assert_eq!(stats.travel_minutes, 65);
        assert_eq!(format_minutes_duration(stats.travel_minutes), "1h05m");
        assert_eq!(stats.total_minutes, 65);
        assert_eq!(stats.total_minutes(), 65);
    }

    #[test]
    fn test_stats_to_json() {
        let conn = test_db();
        let trip_id = add_test_trip(&conn, "沖縄旅行").unwrap();
        crate::itinerary::add_itinerary_item(
            &conn,
            trip_id,
            1,
            "首里城",
            None,
            None,
            None,
            Some(90),
            Some(20),
            None,
            Some(ItineraryCategory::Activity),
        )
        .unwrap();
        let checklist_id =
            crate::checklist::add_checklist_item(&conn, trip_id, "パスポート").unwrap();
        crate::checklist::set_checklist_done(&conn, checklist_id, true).unwrap();

        let stats = compute_trip_stats(&conn, trip_id).unwrap();
        let json = serde_json::to_string_pretty(&stats).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed["trip_name"], "沖縄旅行");
        assert_eq!(parsed["days"], 1);
        assert_eq!(parsed["itinerary_count"], 1);
        assert_eq!(parsed["checklist_total"], 1);
        assert_eq!(parsed["checklist_completed"], 1);
        assert_eq!(parsed["category_counts"]["activity"], 1);
        assert_eq!(parsed["stay_minutes"], 90);
        assert_eq!(parsed["travel_minutes"], 20);
        assert_eq!(parsed["total_minutes"], 110);
    }

    #[test]
    fn test_stats_empty_itinerary_succeeds() {
        let conn = test_db();
        let trip_id = add_test_trip(&conn, "空の旅行").unwrap();

        let stats = compute_trip_stats(&conn, trip_id).unwrap();
        assert_eq!(stats.itinerary_count, 0);
        assert_eq!(stats.days, 0);
        assert_eq!(stats.stay_minutes, 0);
        assert_eq!(stats.travel_minutes, 0);
        assert_eq!(stats.total_minutes, 0);
        assert_eq!(stats.checklist_total, 0);
        assert!(stats.category_counts.is_empty());

        print_trip_stats(&conn, trip_id).unwrap();
    }
}
