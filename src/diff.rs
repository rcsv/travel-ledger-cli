use std::cmp::Ordering;
use std::collections::HashMap;

use anyhow::Result;

use crate::models::{ItineraryCategory, ItineraryItem, TripExport};

/// itinerary_items の比較キー（day + start_time + title）
#[derive(Clone, Eq, PartialEq, Hash)]
struct ItineraryKey {
    day: i64,
    start_time: Option<String>,
    title: String,
}

/// 1件の itinerary におけるフィールド変更
struct ItineraryFieldChange {
    day: i64,
    start_time: Option<String>,
    title: String,
    field: String,
    old_value: String,
    new_value: String,
}

/// trip diff の結果
pub(crate) struct TripDiff {
    trip_changes: Vec<(String, String, String)>,
    itinerary_added: Vec<ItineraryItem>,
    itinerary_removed: Vec<ItineraryItem>,
    itinerary_modified: Vec<ItineraryFieldChange>,
}

fn itinerary_key(item: &ItineraryItem) -> ItineraryKey {
    ItineraryKey {
        day: item.day,
        start_time: item.start_time.clone(),
        title: item.title.clone(),
    }
}

/// itinerary の表示用1行（例: Day1 09:00 首里城）
fn format_itinerary_line(item: &ItineraryItem) -> String {
    let time = item.start_time.as_deref().unwrap_or("-");
    format!("Day{} {time} {}", item.day, item.title)
}

/// Option 値を diff 表示用に整形する
fn fmt_diff_option_str(value: &Option<String>) -> String {
    value.as_deref().unwrap_or("-").to_string()
}

fn fmt_diff_option_i64(value: Option<i64>) -> String {
    value
        .map(|v| v.to_string())
        .unwrap_or_else(|| "-".to_string())
}

fn fmt_diff_option_category(value: Option<ItineraryCategory>) -> String {
    value
        .map(|c| c.as_str().to_string())
        .unwrap_or_else(|| "-".to_string())
}

/// itinerary_items の並び順（day → 時刻あり優先 → 時刻 → タイトル）
fn compare_itinerary_items(a: &ItineraryItem, b: &ItineraryItem) -> Ordering {
    match a.day.cmp(&b.day) {
        Ordering::Equal => match (a.start_time.is_none(), b.start_time.is_none()) {
            (false, true) => Ordering::Less,
            (true, false) => Ordering::Greater,
            _ => a
                .start_time
                .cmp(&b.start_time)
                .then_with(|| a.title.cmp(&b.title)),
        },
        other => other,
    }
}

/// 2つの export JSON の差分を計算する（厳密比較）
pub(crate) fn compute_trip_diff(old: &TripExport, new: &TripExport) -> TripDiff {
    let mut trip_changes = Vec::new();

    if old.trip.name != new.trip.name {
        trip_changes.push((
            "name".to_string(),
            old.trip.name.clone(),
            new.trip.name.clone(),
        ));
    }
    if old.trip.start_date != new.trip.start_date {
        trip_changes.push((
            "start_date".to_string(),
            fmt_diff_option_str(&old.trip.start_date),
            fmt_diff_option_str(&new.trip.start_date),
        ));
    }
    if old.trip.end_date != new.trip.end_date {
        trip_changes.push((
            "end_date".to_string(),
            fmt_diff_option_str(&old.trip.end_date),
            fmt_diff_option_str(&new.trip.end_date),
        ));
    }

    let old_map: HashMap<ItineraryKey, &ItineraryItem> = old
        .itinerary_items
        .iter()
        .map(|item| (itinerary_key(item), item))
        .collect();
    let new_map: HashMap<ItineraryKey, &ItineraryItem> = new
        .itinerary_items
        .iter()
        .map(|item| (itinerary_key(item), item))
        .collect();

    let mut itinerary_removed: Vec<ItineraryItem> = old
        .itinerary_items
        .iter()
        .filter(|item| !new_map.contains_key(&itinerary_key(item)))
        .cloned()
        .collect();
    let mut itinerary_added: Vec<ItineraryItem> = new
        .itinerary_items
        .iter()
        .filter(|item| !old_map.contains_key(&itinerary_key(item)))
        .cloned()
        .collect();

    itinerary_removed.sort_by(compare_itinerary_items);
    itinerary_added.sort_by(compare_itinerary_items);

    let mut itinerary_modified = Vec::new();
    for (key, old_item) in &old_map {
        let Some(new_item) = new_map.get(key) else {
            continue;
        };

        let fields = [
            (
                "note",
                fmt_diff_option_str(&old_item.note),
                fmt_diff_option_str(&new_item.note),
            ),
            (
                "location",
                fmt_diff_option_str(&old_item.location),
                fmt_diff_option_str(&new_item.location),
            ),
            (
                "duration_minutes",
                fmt_diff_option_i64(old_item.duration_minutes),
                fmt_diff_option_i64(new_item.duration_minutes),
            ),
            (
                "travel_minutes",
                fmt_diff_option_i64(old_item.travel_minutes),
                fmt_diff_option_i64(new_item.travel_minutes),
            ),
            (
                "category",
                fmt_diff_option_category(old_item.category),
                fmt_diff_option_category(new_item.category),
            ),
        ];

        for (field, old_value, new_value) in fields {
            if old_value != new_value {
                itinerary_modified.push(ItineraryFieldChange {
                    day: old_item.day,
                    start_time: old_item.start_time.clone(),
                    title: old_item.title.clone(),
                    field: field.to_string(),
                    old_value,
                    new_value,
                });
            }
        }
    }

    itinerary_modified.sort_by(|a, b| {
        compare_itinerary_items(
            &ItineraryItem {
                id: 0,
                trip_id: 0,
                day: a.day,
                title: a.title.clone(),
                note: None,
                start_time: a.start_time.clone(),
                sort_order: 0,
                duration_minutes: None,
                travel_minutes: None,
                location: None,
                category: None,
                created_at: String::new(),
                updated_at: String::new(),
            },
            &ItineraryItem {
                id: 0,
                trip_id: 0,
                day: b.day,
                title: b.title.clone(),
                note: None,
                start_time: b.start_time.clone(),
                sort_order: 0,
                duration_minutes: None,
                travel_minutes: None,
                location: None,
                category: None,
                created_at: String::new(),
                updated_at: String::new(),
            },
        )
        .then_with(|| a.field.cmp(&b.field))
    });

    TripDiff {
        trip_changes,
        itinerary_added,
        itinerary_removed,
        itinerary_modified,
    }
}

/// trip diff の結果を表示する
pub(crate) fn print_trip_diff(diff: &TripDiff) {
    println!("Trip:");
    if diff.trip_changes.is_empty() {
        println!("  （変更なし）");
    } else {
        for (field, old_value, new_value) in &diff.trip_changes {
            println!("- {field}: {old_value}");
            println!("+ {field}: {new_value}");
        }
    }

    println!();
    println!("Itinerary:");
    if diff.itinerary_added.is_empty()
        && diff.itinerary_removed.is_empty()
        && diff.itinerary_modified.is_empty()
    {
        println!("  （変更なし）");
        return;
    }

    for item in &diff.itinerary_removed {
        println!("- {}", format_itinerary_line(item));
    }
    for item in &diff.itinerary_added {
        println!("+ {}", format_itinerary_line(item));
    }

    let mut current_key: Option<(i64, Option<String>, String)> = None;
    for change in &diff.itinerary_modified {
        let key = (change.day, change.start_time.clone(), change.title.clone());
        if current_key.as_ref() != Some(&key) {
            let line_item = ItineraryItem {
                id: 0,
                trip_id: 0,
                day: change.day,
                title: change.title.clone(),
                note: None,
                start_time: change.start_time.clone(),
                sort_order: 0,
                duration_minutes: None,
                travel_minutes: None,
                location: None,
                category: None,
                created_at: String::new(),
                updated_at: String::new(),
            };
            println!("~ {}", format_itinerary_line(&line_item));
            current_key = Some(key);
        }
        println!(
            "  {}: {} -> {}",
            change.field, change.old_value, change.new_value
        );
    }
}

/// 2つの JSON ファイルを比較して差分を表示する
pub(crate) fn run_trip_diff(old_path: &str, new_path: &str) -> Result<()> {
    let old = crate::trip::load_trip_export_from_file(old_path)?;
    let new = crate::trip::load_trip_export_from_file(new_path)?;
    let diff = compute_trip_diff(&old, &new);
    print_trip_diff(&diff);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{ItineraryItem, Trip, TripExport};

    fn make_test_trip(name: &str) -> Trip {
        Trip {
            id: 1,
            name: name.to_string(),
            start_date: None,
            end_date: None,
            created_at: "2026-01-01 00:00:00".to_string(),
            updated_at: "2026-01-01 00:00:00".to_string(),
        }
    }

    fn make_test_item(day: i64, title: &str, start_time: Option<&str>) -> ItineraryItem {
        ItineraryItem {
            id: 1,
            trip_id: 1,
            day,
            title: title.to_string(),
            note: None,
            start_time: start_time.map(str::to_string),
            sort_order: 0,
            duration_minutes: None,
            travel_minutes: None,
            location: None,
            category: None,
            created_at: "2026-01-01 00:00:00".to_string(),
            updated_at: "2026-01-01 00:00:00".to_string(),
        }
    }

    #[test]
    fn test_diff_itinerary_added() {
        let old = TripExport {
            schema_version: None,
            generator: None,
            generator_version: None,
            exported_at: None,
            trip: make_test_trip("沖縄旅行"),
            itinerary_items: vec![],
            checklist_items: None,
            notes: None,
        };
        let new = TripExport {
            schema_version: None,
            generator: None,
            generator_version: None,
            exported_at: None,
            trip: make_test_trip("沖縄旅行"),
            itinerary_items: vec![make_test_item(1, "首里城", Some("09:00"))],
            checklist_items: None,
            notes: None,
        };

        let diff = compute_trip_diff(&old, &new);
        assert_eq!(diff.itinerary_added.len(), 1);
        assert_eq!(diff.itinerary_added[0].title, "首里城");
        assert!(diff.itinerary_removed.is_empty());
    }

    #[test]
    fn test_diff_itinerary_field_changes() {
        let mut old_item = make_test_item(1, "昼食", Some("12:30"));
        old_item.note = Some("沖縄そば".to_string());
        old_item.location = Some("那覇".to_string());
        old_item.duration_minutes = Some(60);
        old_item.travel_minutes = Some(15);

        let mut new_item = make_test_item(1, "昼食", Some("12:30"));
        new_item.note = Some("ステーキ".to_string());
        new_item.location = Some("恩納".to_string());
        new_item.duration_minutes = Some(90);
        new_item.travel_minutes = Some(20);

        let old = TripExport {
            schema_version: None,
            generator: None,
            generator_version: None,
            exported_at: None,
            trip: make_test_trip("沖縄旅行"),
            itinerary_items: vec![old_item],
            checklist_items: None,
            notes: None,
        };
        let new = TripExport {
            schema_version: None,
            generator: None,
            generator_version: None,
            exported_at: None,
            trip: make_test_trip("沖縄旅行"),
            itinerary_items: vec![new_item],
            checklist_items: None,
            notes: None,
        };

        let diff = compute_trip_diff(&old, &new);
        assert!(diff.itinerary_added.is_empty());
        assert!(diff.itinerary_removed.is_empty());
        assert_eq!(diff.itinerary_modified.len(), 4);

        let fields: Vec<&str> = diff
            .itinerary_modified
            .iter()
            .map(|c| c.field.as_str())
            .collect();
        assert!(fields.contains(&"note"));
        assert!(fields.contains(&"location"));
        assert!(fields.contains(&"duration_minutes"));
        assert!(fields.contains(&"travel_minutes"));

        let note = diff
            .itinerary_modified
            .iter()
            .find(|c| c.field == "note")
            .unwrap();
        assert_eq!(note.old_value, "沖縄そば");
        assert_eq!(note.new_value, "ステーキ");
    }

    #[test]
    fn test_diff_itinerary_removed() {
        let old = TripExport {
            schema_version: None,
            generator: None,
            generator_version: None,
            exported_at: None,
            trip: make_test_trip("沖縄旅行"),
            itinerary_items: vec![make_test_item(1, "万座毛", Some("10:00"))],
            checklist_items: None,
            notes: None,
        };
        let new = TripExport {
            schema_version: None,
            generator: None,
            generator_version: None,
            exported_at: None,
            trip: make_test_trip("沖縄旅行"),
            itinerary_items: vec![],
            checklist_items: None,
            notes: None,
        };

        let diff = compute_trip_diff(&old, &new);
        assert_eq!(diff.itinerary_removed.len(), 1);
        assert_eq!(diff.itinerary_removed[0].title, "万座毛");
        assert!(diff.itinerary_added.is_empty());
    }

    #[test]
    fn test_diff_trip_name_change() {
        let old = TripExport {
            schema_version: None,
            generator: None,
            generator_version: None,
            exported_at: None,
            trip: make_test_trip("沖縄旅行"),
            itinerary_items: vec![],
            checklist_items: None,
            notes: None,
        };
        let new = TripExport {
            schema_version: None,
            generator: None,
            generator_version: None,
            exported_at: None,
            trip: make_test_trip("沖縄・瀬底旅行"),
            itinerary_items: vec![],
            checklist_items: None,
            notes: None,
        };

        let diff = compute_trip_diff(&old, &new);
        assert_eq!(diff.trip_changes.len(), 1);
        assert_eq!(diff.trip_changes[0].0, "name");
        assert_eq!(diff.trip_changes[0].1, "沖縄旅行");
        assert_eq!(diff.trip_changes[0].2, "沖縄・瀬底旅行");
    }
}
