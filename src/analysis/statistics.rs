use std::collections::{BTreeMap, HashMap};

use anyhow::Result;
use rusqlite::Connection;
use serde::Serialize;

use crate::domain::models::{Estimate, Expense, ItineraryCategory};

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
    pub expense_count: usize,
    pub expense_totals: BTreeMap<String, i64>,
    pub estimate_count: usize,
    pub estimate_totals: BTreeMap<String, i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub difference_totals: Option<BTreeMap<String, i64>>,
    pub registered_participant_count: usize,
    pub participants_recorded: bool,
    pub self_known: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub participant_count: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub traveler_count: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub companion_count: Option<usize>,
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

/// Planned と Actual の両方があるときのみ、通貨別 Difference（Actual − Planned）を返す。
pub(crate) fn compute_difference_totals(
    estimate_count: usize,
    expense_count: usize,
    estimate_totals: &BTreeMap<String, i64>,
    expense_totals: &BTreeMap<String, i64>,
) -> Option<BTreeMap<String, i64>> {
    if estimate_count == 0 || expense_count == 0 {
        return None;
    }

    let mut currencies = BTreeMap::new();
    for currency in estimate_totals.keys().chain(expense_totals.keys()) {
        currencies.insert(currency.clone(), ());
    }

    let mut difference_totals = BTreeMap::new();
    for currency in currencies.keys() {
        let planned = estimate_totals.get(currency).copied().unwrap_or(0);
        let actual = expense_totals.get(currency).copied().unwrap_or(0);
        difference_totals.insert(currency.clone(), actual - planned);
    }

    Some(difference_totals)
}

/// Itinerary 配下 Estimate の通貨別合計。
pub(crate) fn sum_estimate_totals_by_currency(estimates: &[Estimate]) -> BTreeMap<String, i64> {
    let mut totals = BTreeMap::new();
    for estimate in estimates {
        *totals.entry(estimate.currency.clone()).or_insert(0) += estimate.amount;
    }
    totals
}

/// Itinerary 配下 Expense の通貨別合計。
pub(crate) fn sum_expense_totals_by_currency(expenses: &[Expense]) -> BTreeMap<String, i64> {
    let mut totals = BTreeMap::new();
    for expense in expenses {
        *totals.entry(expense.currency.clone()).or_insert(0) += expense.amount;
    }
    totals
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

    let expenses = crate::expense::list_expenses_for_trip(conn, trip_id)?;
    let expense_count = expenses.len();
    let mut expense_totals: BTreeMap<String, i64> = BTreeMap::new();
    for expense in &expenses {
        *expense_totals.entry(expense.currency.clone()).or_insert(0) += expense.amount;
    }

    let estimates = crate::estimate::list_estimates_for_trip(conn, trip_id)?;
    let estimate_count = estimates.len();
    let mut estimate_totals: BTreeMap<String, i64> = BTreeMap::new();
    for estimate in &estimates {
        *estimate_totals
            .entry(estimate.currency.clone())
            .or_insert(0) += estimate.amount;
    }

    let participant_counts =
        crate::participant::compute_participant_counts_for_trip(conn, trip_id)?;

    let difference_totals = compute_difference_totals(
        estimate_count,
        expense_count,
        &estimate_totals,
        &expense_totals,
    );

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
        expense_count,
        expense_totals,
        estimate_count,
        estimate_totals,
        difference_totals,
        registered_participant_count: participant_counts.registered_count,
        participants_recorded: participant_counts.participants_recorded,
        self_known: participant_counts.self_known,
        participant_count: participant_counts.participant_count,
        traveler_count: participant_counts.participant_count,
        companion_count: participant_counts.companion_count,
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
    println!();
    if !stats.participants_recorded {
        println!("Participants: not recorded");
    } else if stats.self_known {
        let travelers = stats
            .traveler_count
            .or(stats.participant_count)
            .unwrap_or(stats.registered_participant_count);
        let companions = stats.companion_count.unwrap_or(0);
        println!("Participants: {travelers} (companions: {companions})");
    } else {
        println!(
            "Participants: {} recorded (traveler count unknown)",
            stats.registered_participant_count
        );
    }
    println!();
    if stats.estimate_count > 0 {
        println!("Estimates: {}", stats.estimate_count);
        println!("Planned total:");
        for (currency, total) in &stats.estimate_totals {
            println!(
                "  {} {}",
                currency,
                crate::money::format_amount_value(*total, currency)
            );
        }
        println!();
    }
    println!("Expenses: {}", stats.expense_count);
    if stats.expense_count > 0 {
        println!("Actual total:");
        for (currency, total) in &stats.expense_totals {
            println!(
                "  {} {}",
                currency,
                crate::money::format_amount_value(*total, currency)
            );
        }
    }
    if let Some(difference_totals) = &stats.difference_totals {
        println!();
        println!("Difference:");
        for (currency, total) in difference_totals {
            println!(
                "  {} {}",
                currency,
                crate::money::format_amount_value(*total, currency)
            );
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::models::{Estimate, Expense, ItineraryCategory};
    use crate::storage::db::open_db_at;
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
        assert_eq!(stats.expense_count, 0);
        assert!(stats.expense_totals.is_empty());
        assert_eq!(stats.estimate_count, 0);
        assert!(stats.estimate_totals.is_empty());
        assert!(stats.difference_totals.is_none());

        print_trip_stats(&conn, trip_id).unwrap();
    }

    #[test]
    fn test_stats_expense_count_and_totals() {
        let conn = test_db();
        let trip_id = add_test_trip(&conn, "Expense Stats Trip").unwrap();
        let itinerary_id = crate::itinerary::add_itinerary_item(
            &conn, trip_id, 1, "Lunch", None, None, None, None, None, None, None,
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
        crate::expense::add_expense(
            &conn,
            itinerary_id,
            "300",
            "JPY",
            None,
            None,
            None,
            None,
            &crate::expense::ExpenseSharedOptions::default(),
        )
        .unwrap();

        let stats = compute_trip_stats(&conn, trip_id).unwrap();
        assert_eq!(stats.expense_count, 2);
        assert_eq!(stats.expense_totals.get("JPY"), Some(&1500));
        assert!(stats.difference_totals.is_none());
    }

    #[test]
    fn test_stats_expense_multi_currency() {
        let conn = test_db();
        let trip_id = add_test_trip(&conn, "Multi Currency Trip").unwrap();
        let itinerary_id = crate::itinerary::add_itinerary_item(
            &conn, trip_id, 1, "Shopping", None, None, None, None, None, None, None,
        )
        .unwrap();
        crate::expense::add_expense(
            &conn,
            itinerary_id,
            "10000",
            "JPY",
            Some("お土産"),
            None,
            None,
            None,
            &crate::expense::ExpenseSharedOptions::default(),
        )
        .unwrap();
        crate::expense::add_expense(
            &conn,
            itinerary_id,
            "12.50",
            "USD",
            Some("Coffee"),
            None,
            None,
            None,
            &crate::expense::ExpenseSharedOptions::default(),
        )
        .unwrap();
        crate::expense::add_expense(
            &conn,
            itinerary_id,
            "5.00",
            "USD",
            None,
            None,
            None,
            None,
            &crate::expense::ExpenseSharedOptions::default(),
        )
        .unwrap();

        let stats = compute_trip_stats(&conn, trip_id).unwrap();
        assert_eq!(stats.expense_count, 3);
        assert_eq!(stats.expense_totals.get("JPY"), Some(&10000));
        assert_eq!(stats.expense_totals.get("USD"), Some(&1750));

        print_trip_stats(&conn, trip_id).unwrap();
    }

    #[test]
    fn test_stats_estimate_count_and_totals() {
        let conn = test_db();
        let trip_id = add_test_trip(&conn, "Estimate Stats Trip").unwrap();
        let itinerary_id = crate::itinerary::add_itinerary_item(
            &conn, trip_id, 1, "Aquarium", None, None, None, None, None, None, None,
        )
        .unwrap();
        crate::estimate::add_estimate(
            &conn,
            itinerary_id,
            "2180",
            "JPY",
            Some("入館料"),
            Some("大人5名想定"),
            None,
        )
        .unwrap();
        crate::estimate::add_estimate(
            &conn,
            itinerary_id,
            "5000",
            "JPY",
            Some("カフェ"),
            None,
            None,
        )
        .unwrap();

        let stats = compute_trip_stats(&conn, trip_id).unwrap();
        assert_eq!(stats.estimate_count, 2);
        assert_eq!(stats.estimate_totals.get("JPY"), Some(&7180));
    }

    #[test]
    fn test_stats_estimate_multi_currency() {
        let conn = test_db();
        let trip_id = add_test_trip(&conn, "Estimate Multi Currency Trip").unwrap();
        let itinerary_id = crate::itinerary::add_itinerary_item(
            &conn, trip_id, 1, "Shopping", None, None, None, None, None, None, None,
        )
        .unwrap();
        crate::estimate::add_estimate(
            &conn,
            itinerary_id,
            "10000",
            "JPY",
            Some("お土産"),
            None,
            None,
        )
        .unwrap();
        crate::estimate::add_estimate(
            &conn,
            itinerary_id,
            "12.50",
            "USD",
            Some("Coffee"),
            None,
            None,
        )
        .unwrap();
        crate::estimate::add_estimate(&conn, itinerary_id, "5.00", "USD", None, None, None)
            .unwrap();

        let stats = compute_trip_stats(&conn, trip_id).unwrap();
        assert_eq!(stats.estimate_count, 3);
        assert_eq!(stats.estimate_totals.get("JPY"), Some(&10000));
        assert_eq!(stats.estimate_totals.get("USD"), Some(&1750));

        let json = serde_json::to_string_pretty(&stats).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["estimate_count"], 3);
        assert_eq!(parsed["estimate_totals"]["JPY"], 10000);
        assert_eq!(parsed["estimate_totals"]["USD"], 1750);
        assert!(parsed.get("difference_totals").is_none());
    }

    #[test]
    fn test_compute_difference_totals_gate_requires_both() {
        let mut planned = BTreeMap::new();
        planned.insert("JPY".to_string(), 1000);
        let mut actual = BTreeMap::new();
        actual.insert("JPY".to_string(), 800);

        assert!(compute_difference_totals(0, 1, &planned, &actual).is_none());
        assert!(compute_difference_totals(1, 0, &planned, &actual).is_none());
    }

    #[test]
    fn test_compute_difference_totals_actual_minus_planned() {
        let mut planned = BTreeMap::new();
        planned.insert("JPY".to_string(), 180_000);
        let mut actual = BTreeMap::new();
        actual.insert("JPY".to_string(), 172_500);

        let diff = compute_difference_totals(1, 1, &planned, &actual).unwrap();
        assert_eq!(diff.get("JPY"), Some(&-7_500));
    }

    #[test]
    fn test_compute_difference_totals_currency_union() {
        let mut planned = BTreeMap::new();
        planned.insert("JPY".to_string(), 10_000);
        planned.insert("USD".to_string(), 50);
        let mut actual = BTreeMap::new();
        actual.insert("JPY".to_string(), 9_500);

        let diff = compute_difference_totals(2, 1, &planned, &actual).unwrap();
        assert_eq!(diff.get("JPY"), Some(&-500));
        assert_eq!(diff.get("USD"), Some(&-50));
        assert_eq!(diff.len(), 2);
    }

    #[test]
    fn test_compute_difference_totals_zero_when_equal() {
        let mut planned = BTreeMap::new();
        planned.insert("JPY".to_string(), 2500);
        let mut actual = BTreeMap::new();
        actual.insert("JPY".to_string(), 2500);

        let diff = compute_difference_totals(1, 1, &planned, &actual).unwrap();
        assert_eq!(diff.get("JPY"), Some(&0));
    }

    #[test]
    fn test_stats_difference_when_estimates_and_expenses_present() {
        let conn = test_db();
        let trip_id = add_test_trip(&conn, "Difference Stats Trip").unwrap();
        let itinerary_id = crate::itinerary::add_itinerary_item(
            &conn, trip_id, 1, "Aquarium", None, None, None, None, None, None, None,
        )
        .unwrap();
        crate::estimate::add_estimate(
            &conn,
            itinerary_id,
            "7180",
            "JPY",
            Some("入館料"),
            None,
            None,
        )
        .unwrap();
        crate::expense::add_expense(
            &conn,
            itinerary_id,
            "6500",
            "JPY",
            Some("入館料"),
            None,
            None,
            None,
            &crate::expense::ExpenseSharedOptions::default(),
        )
        .unwrap();

        let stats = compute_trip_stats(&conn, trip_id).unwrap();
        assert_eq!(
            stats.difference_totals.as_ref().unwrap().get("JPY"),
            Some(&-680)
        );

        let json = serde_json::to_string_pretty(&stats).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["difference_totals"]["JPY"], -680);
    }

    #[test]
    fn test_stats_estimate_only_has_no_difference_totals() {
        let conn = test_db();
        let trip_id = add_test_trip(&conn, "Estimate Only Trip").unwrap();
        let itinerary_id = crate::itinerary::add_itinerary_item(
            &conn, trip_id, 1, "Aquarium", None, None, None, None, None, None, None,
        )
        .unwrap();
        crate::estimate::add_estimate(&conn, itinerary_id, "2180", "JPY", None, None, None)
            .unwrap();

        let stats = compute_trip_stats(&conn, trip_id).unwrap();
        assert!(stats.difference_totals.is_none());

        let json = serde_json::to_string_pretty(&stats).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert!(parsed.get("difference_totals").is_none());
    }

    #[test]
    fn test_sum_estimate_totals_by_currency() {
        let estimates = vec![
            Estimate {
                id: 1,
                itinerary_id: 1,
                title: Some("入館料".to_string()),
                amount: 2180,
                currency: "JPY".to_string(),
                note: None,
                sort_order: 0,
                created_at: String::new(),
                updated_at: String::new(),
            },
            Estimate {
                id: 2,
                itinerary_id: 1,
                title: Some("カフェ".to_string()),
                amount: 5000,
                currency: "JPY".to_string(),
                note: None,
                sort_order: 1,
                created_at: String::new(),
                updated_at: String::new(),
            },
        ];

        let totals = sum_estimate_totals_by_currency(&estimates);
        assert_eq!(totals.get("JPY"), Some(&7180));
    }

    #[test]
    fn test_sum_expense_totals_by_currency_multi_currency() {
        let expenses = vec![
            Expense {
                id: 1,
                itinerary_id: 1,
                title: None,
                amount: 9500,
                currency: "JPY".to_string(),
                note: None,
                expense_date: None,
                paid_by_participant_id: None,
                paid_by_name: None,
                sort_order: 0,
                created_at: String::new(),
                updated_at: String::new(),
            },
            Expense {
                id: 2,
                itinerary_id: 1,
                title: None,
                amount: 1250,
                currency: "USD".to_string(),
                note: None,
                expense_date: None,
                paid_by_participant_id: None,
                paid_by_name: None,
                sort_order: 1,
                created_at: String::new(),
                updated_at: String::new(),
            },
        ];

        let totals = sum_expense_totals_by_currency(&expenses);
        assert_eq!(totals.get("JPY"), Some(&9500));
        assert_eq!(totals.get("USD"), Some(&1250));
    }

    #[test]
    fn test_itinerary_difference_actual_minus_planned() {
        let planned = sum_estimate_totals_by_currency(&[Estimate {
            id: 1,
            itinerary_id: 1,
            title: None,
            amount: 14_000,
            currency: "JPY".to_string(),
            note: None,
            sort_order: 0,
            created_at: String::new(),
            updated_at: String::new(),
        }]);
        assert_eq!(planned.get("JPY"), Some(&14_000));

        let actual = sum_expense_totals_by_currency(&[Expense {
            id: 1,
            itinerary_id: 1,
            title: None,
            amount: 13_750,
            currency: "JPY".to_string(),
            note: None,
            expense_date: None,
            paid_by_participant_id: None,
            paid_by_name: None,
            sort_order: 0,
            created_at: String::new(),
            updated_at: String::new(),
        }]);

        let diff = compute_difference_totals(1, 1, &planned, &actual).unwrap();
        assert_eq!(diff.get("JPY"), Some(&-250));

        let mut planned_multi = planned;
        planned_multi.insert("USD".to_string(), 50);
        let diff = compute_difference_totals(2, 1, &planned_multi, &actual).unwrap();
        assert_eq!(diff.get("JPY"), Some(&-250));
        assert_eq!(diff.get("USD"), Some(&-50));
    }
}
