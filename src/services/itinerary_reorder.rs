//! Same-Day one-step Itinerary reorder use case for Desktop Activity controls.

use std::collections::{HashMap, HashSet};
use std::fmt;

use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

use super::read_errors::{classify_read_error, ReadServiceErrorCode};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ItineraryReorderDirection {
    Up,
    Down,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReorderItineraryParams {
    pub trip_id: i64,
    pub day_number: i64,
    pub itinerary_id: i64,
    pub direction: ItineraryReorderDirection,
    pub expected_order: Vec<i64>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReorderItineraryResult {
    pub itinerary_id: i64,
    pub day_number: i64,
    pub moved: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ItineraryReorderErrorCode {
    TargetNotFound,
    PlacementInvalid,
    PlacementConflict,
    StorageFailure,
}

impl ItineraryReorderErrorCode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::TargetNotFound => "ITINERARY_TARGET_NOT_FOUND",
            Self::PlacementInvalid => "ITINERARY_PLACEMENT_INVALID",
            Self::PlacementConflict => "ITINERARY_PLACEMENT_CONFLICT",
            Self::StorageFailure => "STORAGE_FAILURE",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ItineraryReorderError {
    pub code: ItineraryReorderErrorCode,
    pub message: String,
}

impl ItineraryReorderError {
    fn new(code: ItineraryReorderErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }

    fn target_not_found(message: impl Into<String>) -> Self {
        Self::new(ItineraryReorderErrorCode::TargetNotFound, message)
    }

    fn placement_invalid(message: impl Into<String>) -> Self {
        Self::new(ItineraryReorderErrorCode::PlacementInvalid, message)
    }

    fn placement_conflict(message: impl Into<String>) -> Self {
        Self::new(ItineraryReorderErrorCode::PlacementConflict, message)
    }

    fn storage(message: impl Into<String>) -> Self {
        Self::new(ItineraryReorderErrorCode::StorageFailure, message)
    }
}

impl fmt::Display for ItineraryReorderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for ItineraryReorderError {}

pub fn reorder_itinerary(
    conn: &Connection,
    params: ReorderItineraryParams,
) -> Result<ReorderItineraryResult, ItineraryReorderError> {
    validate_expected_order(&params)?;

    let tx = conn
        .unchecked_transaction()
        .map_err(|err| ItineraryReorderError::storage(err.to_string()))?;
    let items =
        crate::itinerary::list_itinerary_items_for_day(&tx, params.trip_id, params.day_number)
            .map_err(classify_target_or_storage)?;

    let current_order = items.iter().map(|item| item.id).collect::<Vec<_>>();
    let Some(target_index) = current_order
        .iter()
        .position(|id| *id == params.itinerary_id)
    else {
        return Err(ItineraryReorderError::target_not_found(format!(
            "Itinerary target not found in trip {} Day {}: {}",
            params.trip_id, params.day_number, params.itinerary_id
        )));
    };

    if params.expected_order != current_order {
        return Err(ItineraryReorderError::placement_conflict(format!(
            "Itinerary order changed for trip {} Day {}; refresh and try again",
            params.trip_id, params.day_number
        )));
    }

    let adjacent_index = match params.direction {
        ItineraryReorderDirection::Up => target_index.checked_sub(1),
        ItineraryReorderDirection::Down => {
            (target_index + 1 < current_order.len()).then_some(target_index + 1)
        }
    };
    let Some(adjacent_index) = adjacent_index else {
        tx.commit()
            .map_err(|err| ItineraryReorderError::storage(err.to_string()))?;
        return Ok(ReorderItineraryResult {
            itinerary_id: params.itinerary_id,
            day_number: params.day_number,
            moved: false,
        });
    };

    let mut desired_order = current_order;
    desired_order.swap(target_index, adjacent_index);

    let mut slots = items.iter().map(|item| item.sort_order).collect::<Vec<_>>();
    slots.sort_unstable();
    let mut visible_order = desired_order
        .iter()
        .copied()
        .zip(slots.iter().copied())
        .collect::<Vec<_>>();
    visible_order.sort_unstable_by_key(|(itinerary_id, sort_order)| (*sort_order, *itinerary_id));
    if visible_order
        .iter()
        .map(|(itinerary_id, _)| itinerary_id)
        .ne(desired_order.iter())
    {
        slots = (1..=items.len())
            .map(|position| {
                i64::try_from(position)
                    .ok()
                    .and_then(|value| value.checked_mul(crate::itinerary::SORT_ORDER_STEP))
                    .ok_or_else(|| ItineraryReorderError::storage("Itinerary sort order overflow"))
            })
            .collect::<Result<Vec<_>, _>>()?;
    }

    let old_orders = items
        .iter()
        .map(|item| (item.id, item.sort_order))
        .collect::<HashMap<_, _>>();
    let now = crate::storage::db::now_string();
    for (itinerary_id, new_order) in desired_order.iter().zip(slots.iter()) {
        if old_orders.get(itinerary_id) == Some(new_order) {
            continue;
        }
        let changed = tx
            .execute(
                "UPDATE itinerary_items
                 SET sort_order = ?1, updated_at = ?2
                 WHERE id = ?3 AND trip_id = ?4 AND day = ?5",
                params![
                    new_order,
                    &now,
                    itinerary_id,
                    params.trip_id,
                    params.day_number
                ],
            )
            .map_err(|err| ItineraryReorderError::storage(err.to_string()))?;
        if changed != 1 {
            return Err(ItineraryReorderError::storage(format!(
                "Itinerary reorder expected one updated row, got {changed}"
            )));
        }
    }

    tx.commit()
        .map_err(|err| ItineraryReorderError::storage(err.to_string()))?;
    Ok(ReorderItineraryResult {
        itinerary_id: params.itinerary_id,
        day_number: params.day_number,
        moved: true,
    })
}

fn validate_expected_order(params: &ReorderItineraryParams) -> Result<(), ItineraryReorderError> {
    if params.expected_order.is_empty() {
        return Err(ItineraryReorderError::placement_invalid(
            "Expected itinerary order must not be empty",
        ));
    }
    let unique = params
        .expected_order
        .iter()
        .copied()
        .collect::<HashSet<_>>();
    if unique.len() != params.expected_order.len() {
        return Err(ItineraryReorderError::placement_invalid(
            "Expected itinerary order must not contain duplicate IDs",
        ));
    }
    if !unique.contains(&params.itinerary_id) {
        return Err(ItineraryReorderError::placement_invalid(format!(
            "Expected itinerary order does not contain target {}",
            params.itinerary_id
        )));
    }
    Ok(())
}

fn classify_target_or_storage(err: anyhow::Error) -> ItineraryReorderError {
    let classified = classify_read_error(err);
    match classified.code {
        ReadServiceErrorCode::TripNotFound | ReadServiceErrorCode::DayNotFound => {
            ItineraryReorderError::target_not_found(classified.message)
        }
        ReadServiceErrorCode::StorageFailure | ReadServiceErrorCode::DataMappingFailure => {
            ItineraryReorderError::storage(classified.message)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::models::ItineraryCategory;

    fn connection() -> Connection {
        crate::storage::db::open_db_at(":memory:").unwrap()
    }

    fn trip(conn: &Connection, name: &str) -> i64 {
        crate::trip::add_test_trip(conn, name).unwrap()
    }

    fn add(conn: &Connection, trip_id: i64, title: &str, order: i64) -> i64 {
        crate::itinerary::add_itinerary_item(
            conn,
            trip_id,
            1,
            title,
            None,
            None,
            Some(order),
            None,
            None,
            None,
            None,
        )
        .unwrap()
    }

    fn setup_three(conn: &Connection, orders: [i64; 3]) -> (i64, [i64; 3]) {
        let trip_id = trip(conn, "Reorder Trip");
        let ids = [
            add(conn, trip_id, "First", orders[0]),
            add(conn, trip_id, "Second", orders[1]),
            add(conn, trip_id, "Third", orders[2]),
        ];
        (trip_id, ids)
    }

    fn params(
        trip_id: i64,
        itinerary_id: i64,
        direction: ItineraryReorderDirection,
        expected_order: Vec<i64>,
    ) -> ReorderItineraryParams {
        ReorderItineraryParams {
            trip_id,
            day_number: 1,
            itinerary_id,
            direction,
            expected_order,
        }
    }

    fn timeline_ids(conn: &Connection, trip_id: i64) -> Vec<i64> {
        crate::services::get_day_timeline(conn, trip_id, 1)
            .unwrap()
            .itineraries
            .into_iter()
            .map(|item| item.id)
            .collect()
    }

    #[test]
    fn moves_middle_up_and_down_using_existing_slots() {
        let conn = connection();
        let (trip_id, ids) = setup_three(&conn, [1000, 2000, 3000]);

        let up = reorder_itinerary(
            &conn,
            params(trip_id, ids[1], ItineraryReorderDirection::Up, ids.to_vec()),
        )
        .unwrap();
        assert!(up.moved);
        assert_eq!(timeline_ids(&conn, trip_id), vec![ids[1], ids[0], ids[2]]);
        assert_eq!(
            crate::itinerary::get_itinerary_item(&conn, ids[1])
                .unwrap()
                .sort_order,
            1000
        );
        assert_eq!(
            crate::itinerary::get_itinerary_item(&conn, ids[0])
                .unwrap()
                .sort_order,
            2000
        );

        let current = vec![ids[1], ids[0], ids[2]];
        let down = reorder_itinerary(
            &conn,
            params(trip_id, ids[0], ItineraryReorderDirection::Down, current),
        )
        .unwrap();
        assert!(down.moved);
        assert_eq!(timeline_ids(&conn, trip_id), vec![ids[1], ids[2], ids[0]]);
    }

    #[test]
    fn returns_noop_at_boundaries_and_for_one_item() {
        let conn = connection();
        let (trip_id, ids) = setup_three(&conn, [1000, 2000, 3000]);
        let first = reorder_itinerary(
            &conn,
            params(trip_id, ids[0], ItineraryReorderDirection::Up, ids.to_vec()),
        )
        .unwrap();
        assert!(!first.moved);
        let last = reorder_itinerary(
            &conn,
            params(
                trip_id,
                ids[2],
                ItineraryReorderDirection::Down,
                ids.to_vec(),
            ),
        )
        .unwrap();
        assert!(!last.moved);

        let other_trip = trip(&conn, "One Item");
        let only = add(&conn, other_trip, "Only", 0);
        let result = reorder_itinerary(
            &conn,
            params(
                other_trip,
                only,
                ItineraryReorderDirection::Down,
                vec![only],
            ),
        )
        .unwrap();
        assert!(!result.moved);
    }

    #[test]
    fn rejects_invalid_and_stale_expected_orders() {
        let conn = connection();
        let (trip_id, ids) = setup_three(&conn, [1000, 2000, 3000]);
        for expected in [vec![], vec![ids[0], ids[1], ids[1]], vec![ids[0], ids[2]]] {
            let error = reorder_itinerary(
                &conn,
                params(trip_id, ids[1], ItineraryReorderDirection::Up, expected),
            )
            .unwrap_err();
            assert_eq!(error.code, ItineraryReorderErrorCode::PlacementInvalid);
        }

        for expected in [
            vec![ids[1], ids[0], ids[2]],
            vec![ids[0], ids[1]],
            vec![ids[0], ids[1], ids[2], 999],
        ] {
            let error = reorder_itinerary(
                &conn,
                params(trip_id, ids[1], ItineraryReorderDirection::Up, expected),
            )
            .unwrap_err();
            assert_eq!(error.code, ItineraryReorderErrorCode::PlacementConflict);
        }
        assert_eq!(timeline_ids(&conn, trip_id), ids);
    }

    #[test]
    fn rejects_missing_and_mismatched_targets() {
        let conn = connection();
        let (trip_id, ids) = setup_three(&conn, [1000, 2000, 3000]);
        let missing_trip = reorder_itinerary(
            &conn,
            params(999, ids[1], ItineraryReorderDirection::Up, ids.to_vec()),
        )
        .unwrap_err();
        assert_eq!(missing_trip.code, ItineraryReorderErrorCode::TargetNotFound);

        let missing_day = reorder_itinerary(
            &conn,
            ReorderItineraryParams {
                day_number: 99,
                ..params(trip_id, ids[1], ItineraryReorderDirection::Up, ids.to_vec())
            },
        )
        .unwrap_err();
        assert_eq!(missing_day.code, ItineraryReorderErrorCode::TargetNotFound);

        let other_trip = trip(&conn, "Other Trip");
        let other_id = add(&conn, other_trip, "Other", 1000);
        let wrong_day = reorder_itinerary(
            &conn,
            params(
                trip_id,
                other_id,
                ItineraryReorderDirection::Up,
                vec![other_id],
            ),
        )
        .unwrap_err();
        assert_eq!(wrong_day.code, ItineraryReorderErrorCode::TargetNotFound);
    }

    #[test]
    fn conditionally_normalizes_duplicate_slots() {
        let conn = connection();
        let (trip_id, ids) = setup_three(&conn, [0, 0, 1]);
        reorder_itinerary(
            &conn,
            params(trip_id, ids[1], ItineraryReorderDirection::Up, ids.to_vec()),
        )
        .unwrap();
        let timeline = crate::services::get_day_timeline(&conn, trip_id, 1).unwrap();
        assert_eq!(
            timeline
                .itineraries
                .iter()
                .map(|item| item.id)
                .collect::<Vec<_>>(),
            vec![ids[1], ids[0], ids[2]]
        );
        assert_eq!(
            timeline
                .itineraries
                .iter()
                .map(|item| item.sort_order)
                .collect::<Vec<_>>(),
            vec![1000, 2000, 3000]
        );
    }

    #[test]
    fn reuses_duplicate_slots_when_they_still_express_the_desired_order() {
        let conn = connection();
        let (trip_id, ids) = setup_three(&conn, [0, 1, 1]);
        reorder_itinerary(
            &conn,
            params(
                trip_id,
                ids[0],
                ItineraryReorderDirection::Down,
                ids.to_vec(),
            ),
        )
        .unwrap();

        let timeline = crate::services::get_day_timeline(&conn, trip_id, 1).unwrap();
        assert_eq!(
            timeline
                .itineraries
                .iter()
                .map(|item| item.id)
                .collect::<Vec<_>>(),
            vec![ids[1], ids[0], ids[2]]
        );
        assert_eq!(
            timeline
                .itineraries
                .iter()
                .map(|item| item.sort_order)
                .collect::<Vec<_>>(),
            vec![0, 1, 1]
        );
    }

    #[test]
    fn reuses_distinct_negative_dense_sparse_and_near_limit_slots() {
        for orders in [
            [i64::MIN + 1, 0, 1],
            [-10, 10, 50_000],
            [i64::MAX - 2, i64::MAX - 1, i64::MAX],
        ] {
            let conn = connection();
            let (trip_id, ids) = setup_three(&conn, orders);
            reorder_itinerary(
                &conn,
                params(
                    trip_id,
                    ids[1],
                    ItineraryReorderDirection::Down,
                    ids.to_vec(),
                ),
            )
            .unwrap();
            assert_eq!(timeline_ids(&conn, trip_id), vec![ids[0], ids[2], ids[1]]);
            let mut resulting_slots = crate::services::get_day_timeline(&conn, trip_id, 1)
                .unwrap()
                .itineraries
                .into_iter()
                .map(|item| item.sort_order)
                .collect::<Vec<_>>();
            resulting_slots.sort_unstable();
            let mut expected_slots = orders.to_vec();
            expected_slots.sort_unstable();
            assert_eq!(resulting_slots, expected_slots);
        }
    }

    #[test]
    fn rolls_back_all_updates_on_storage_failure() {
        let conn = connection();
        let (trip_id, ids) = setup_three(&conn, [1000, 2000, 3000]);
        conn.execute_batch(&format!(
            "CREATE TRIGGER fail_second_reorder
             BEFORE UPDATE OF sort_order ON itinerary_items
             WHEN OLD.id = {}
             BEGIN
                 SELECT RAISE(ABORT, 'forced reorder failure');
             END;",
            ids[1]
        ))
        .unwrap();

        let error = reorder_itinerary(
            &conn,
            params(
                trip_id,
                ids[0],
                ItineraryReorderDirection::Down,
                ids.to_vec(),
            ),
        )
        .unwrap_err();
        assert_eq!(error.code, ItineraryReorderErrorCode::StorageFailure);
        assert_eq!(timeline_ids(&conn, trip_id), ids);
        assert_eq!(
            crate::itinerary::get_itinerary_item(&conn, ids[0])
                .unwrap()
                .sort_order,
            1000
        );
    }

    #[test]
    fn preserves_non_placement_fields_and_child_association() {
        let conn = connection();
        let trip_id = trip(&conn, "Preserve Trip");
        let first = crate::itinerary::add_itinerary_item(
            &conn,
            trip_id,
            1,
            "Museum",
            Some("Note"),
            Some("09:30"),
            Some(1000),
            Some(90),
            Some(15),
            Some("Naha"),
            Some(ItineraryCategory::Museum),
        )
        .unwrap();
        let second = add(&conn, trip_id, "Lunch", 2000);
        let before = crate::itinerary::get_itinerary_item(&conn, first).unwrap();
        let note_id = crate::note::add_note(
            &conn,
            crate::note::ResolvedNoteOwner::Itinerary(first),
            Some("Child"),
            "Keep me",
        )
        .unwrap();

        reorder_itinerary(
            &conn,
            params(
                trip_id,
                first,
                ItineraryReorderDirection::Down,
                vec![first, second],
            ),
        )
        .unwrap();
        let after = crate::itinerary::get_itinerary_item(&conn, first).unwrap();
        assert_eq!(after.id, before.id);
        assert_eq!(after.trip_id, before.trip_id);
        assert_eq!(after.day, before.day);
        assert_eq!(after.title, before.title);
        assert_eq!(after.note, before.note);
        assert_eq!(after.start_time, before.start_time);
        assert_eq!(after.duration_minutes, before.duration_minutes);
        assert_eq!(after.travel_minutes, before.travel_minutes);
        assert_eq!(after.location, before.location);
        assert_eq!(after.category, before.category);
        assert_eq!(after.created_at, before.created_at);
        let notes = crate::note::list_notes_for_owner(
            &conn,
            crate::domain::models::NoteOwnerType::Itinerary,
            first,
        )
        .unwrap();
        assert_eq!(notes.len(), 1);
        assert_eq!(notes[0].id, note_id);
    }

    #[test]
    fn read_facade_json_export_and_markdown_follow_reordered_sequence() {
        let conn = connection();
        let (trip_id, ids) = setup_three(&conn, [1000, 2000, 3000]);
        reorder_itinerary(
            &conn,
            params(trip_id, ids[1], ItineraryReorderDirection::Up, ids.to_vec()),
        )
        .unwrap();

        let read_titles = crate::services::get_day_timeline(&conn, trip_id, 1)
            .unwrap()
            .itineraries
            .into_iter()
            .map(|item| item.title)
            .collect::<Vec<_>>();
        assert_eq!(read_titles, ["Second", "First", "Third"]);

        let json = crate::trip::export_trip_to_json(&conn, trip_id).unwrap();
        let exported: serde_json::Value = serde_json::from_str(&json).unwrap();
        let export_titles = exported["days"][0]["itineraries"]
            .as_array()
            .unwrap()
            .iter()
            .map(|item| item["title"].as_str().unwrap())
            .collect::<Vec<_>>();
        assert_eq!(export_titles, ["Second", "First", "Third"]);

        let markdown = crate::io::markdown::generate_trip_markdown(&conn, trip_id).unwrap();
        let second = markdown.find("#### Second").unwrap();
        let first = markdown.find("#### First").unwrap();
        let third = markdown.find("#### Third").unwrap();
        assert!(second < first && first < third);
    }
}
