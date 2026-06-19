use std::cmp::Ordering;
use std::collections::HashMap;

use anyhow::Result;

use crate::models::{
    ExportNote, ExportParticipantV4, ExportReservation, ItineraryCategory, ItineraryItem,
    TripExport,
};

/// export Note の比較キー
#[derive(Clone, Eq, PartialEq, Hash, PartialOrd, Ord)]
enum NoteKey {
    Trip {
        title: Option<String>,
    },
    Day {
        day_number: i64,
        title: Option<String>,
    },
    Itinerary {
        day_number: i64,
        sort_order: i64,
        title: String,
    },
}

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

/// Reservation の比較キー（Itinerary コンテキスト + 予約識別）
#[derive(Clone, Eq, PartialEq, Hash, PartialOrd, Ord)]
struct ReservationKey {
    day_number: i64,
    sort_order: i64,
    start_time: Option<String>,
    itinerary_title: String,
    reservation_type: String,
    provider_name: String,
    confirmation_code: Option<String>,
}

/// 1件の Reservation におけるフィールド変更
struct ReservationFieldChange {
    line: String,
    field: String,
    old_value: String,
    new_value: String,
}

/// Participant の比較キー（export 上の identity）
#[derive(Clone, Eq, PartialEq, Hash, PartialOrd, Ord)]
struct ParticipantKey {
    sort_order: i64,
    name: String,
}

/// 1件の Participant におけるフィールド変更
struct ParticipantFieldChange {
    sort_order: i64,
    name: String,
    field: String,
    old_value: String,
    new_value: String,
}

/// trip diff の結果
pub(crate) struct TripDiff {
    trip_changes: Vec<(String, String, String)>,
    day_summary_changes: Vec<(i64, String, String)>,
    itinerary_added: Vec<ItineraryItem>,
    itinerary_removed: Vec<ItineraryItem>,
    itinerary_modified: Vec<ItineraryFieldChange>,
    note_added: Vec<ExportNote>,
    note_removed: Vec<ExportNote>,
    note_changed: Vec<ExportNote>,
    reservation_added: Vec<ExportReservation>,
    reservation_removed: Vec<ExportReservation>,
    reservation_modified: Vec<ReservationFieldChange>,
    participant_added: Vec<ExportParticipantV4>,
    participant_removed: Vec<ExportParticipantV4>,
    participant_changed: Vec<ParticipantFieldChange>,
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

fn note_key(note: &ExportNote) -> NoteKey {
    match note {
        ExportNote::Trip { title, .. } => NoteKey::Trip {
            title: title.clone(),
        },
        ExportNote::Day {
            day_number, title, ..
        } => NoteKey::Day {
            day_number: *day_number,
            title: title.clone(),
        },
        ExportNote::Itinerary { itinerary_key, .. } => NoteKey::Itinerary {
            day_number: itinerary_key.day_number,
            sort_order: itinerary_key.sort_order,
            title: itinerary_key.title.clone(),
        },
    }
}

fn compare_export_notes(a: &ExportNote, b: &ExportNote) -> Ordering {
    note_key(a).cmp(&note_key(b))
}

fn format_note_line(note: &ExportNote) -> String {
    match note {
        ExportNote::Trip { title, .. } => {
            format!("Trip / {}", title.as_deref().unwrap_or("-"))
        }
        ExportNote::Day {
            day_number, title, ..
        } => format!("Day {day_number} / {}", title.as_deref().unwrap_or("-")),
        ExportNote::Itinerary { itinerary_key, .. } => format!(
            "Itinerary / Day {} / {}",
            itinerary_key.day_number, itinerary_key.title
        ),
    }
}

fn note_body(note: &ExportNote) -> &str {
    match note {
        ExportNote::Trip { body, .. }
        | ExportNote::Day { body, .. }
        | ExportNote::Itinerary { body, .. } => body,
    }
}

fn note_content_changed(old: &ExportNote, new: &ExportNote) -> bool {
    if note_body(old) != note_body(new) {
        return true;
    }
    matches!(
        (old, new),
        (
            ExportNote::Itinerary { title: old_title, .. },
            ExportNote::Itinerary { title: new_title, .. }
        ) if old_title != new_title
    )
}

fn reservation_key(reservation: &ExportReservation) -> ReservationKey {
    ReservationKey {
        day_number: reservation.itinerary_key.day_number,
        sort_order: reservation.itinerary_key.sort_order,
        start_time: reservation.itinerary_key.start_time.clone(),
        itinerary_title: reservation.itinerary_key.title.clone(),
        reservation_type: reservation.reservation.reservation_type.clone(),
        provider_name: reservation.reservation.provider_name.clone(),
        confirmation_code: reservation.reservation.confirmation_code.clone(),
    }
}

fn compare_export_reservations(a: &ExportReservation, b: &ExportReservation) -> Ordering {
    reservation_key(a).cmp(&reservation_key(b))
}

fn format_reservation_line(reservation: &ExportReservation) -> String {
    let key = &reservation.itinerary_key;
    let time = key.start_time.as_deref().unwrap_or("-");
    let confirmation = reservation
        .reservation
        .confirmation_code
        .as_deref()
        .unwrap_or("-");
    format!(
        "Day{} {time} {} / {} / {} / {}",
        key.day_number,
        key.title,
        reservation.reservation.reservation_type,
        reservation.reservation.provider_name,
        confirmation
    )
}

fn reservation_content_changed(
    old: &ExportReservation,
    new: &ExportReservation,
) -> Vec<(String, String, String)> {
    let fields = [
        (
            "reservation_site_url",
            fmt_diff_option_str(&old.reservation.reservation_site_url),
            fmt_diff_option_str(&new.reservation.reservation_site_url),
        ),
        (
            "remark",
            fmt_diff_option_str(&old.reservation.remark),
            fmt_diff_option_str(&new.reservation.remark),
        ),
        (
            "start_at",
            fmt_diff_option_str(&old.reservation.start_at),
            fmt_diff_option_str(&new.reservation.start_at),
        ),
        (
            "end_at",
            fmt_diff_option_str(&old.reservation.end_at),
            fmt_diff_option_str(&new.reservation.end_at),
        ),
    ];
    fields
        .into_iter()
        .filter(|(_, old_value, new_value)| old_value != new_value)
        .map(|(field, old_value, new_value)| (field.to_string(), old_value, new_value))
        .collect()
}

fn compute_reservations_diff(
    old_reservations: &[ExportReservation],
    new_reservations: &[ExportReservation],
) -> (
    Vec<ExportReservation>,
    Vec<ExportReservation>,
    Vec<ReservationFieldChange>,
) {
    let old_map: HashMap<ReservationKey, &ExportReservation> = old_reservations
        .iter()
        .map(|reservation| (reservation_key(reservation), reservation))
        .collect();
    let new_map: HashMap<ReservationKey, &ExportReservation> = new_reservations
        .iter()
        .map(|reservation| (reservation_key(reservation), reservation))
        .collect();

    let mut reservation_removed: Vec<ExportReservation> = old_reservations
        .iter()
        .filter(|reservation| !new_map.contains_key(&reservation_key(reservation)))
        .cloned()
        .collect();
    let mut reservation_added: Vec<ExportReservation> = new_reservations
        .iter()
        .filter(|reservation| !old_map.contains_key(&reservation_key(reservation)))
        .cloned()
        .collect();

    let mut reservation_modified = Vec::new();
    for (key, old_reservation) in &old_map {
        let Some(new_reservation) = new_map.get(key) else {
            continue;
        };
        let line = format_reservation_line(new_reservation);
        for (field, old_value, new_value) in
            reservation_content_changed(old_reservation, new_reservation)
        {
            reservation_modified.push(ReservationFieldChange {
                line: line.clone(),
                field,
                old_value,
                new_value,
            });
        }
    }

    reservation_removed.sort_by(compare_export_reservations);
    reservation_added.sort_by(compare_export_reservations);
    reservation_modified.sort_by(|a, b| a.line.cmp(&b.line).then_with(|| a.field.cmp(&b.field)));

    (reservation_added, reservation_removed, reservation_modified)
}

fn compute_notes_diff(
    old_notes: &[ExportNote],
    new_notes: &[ExportNote],
) -> (Vec<ExportNote>, Vec<ExportNote>, Vec<ExportNote>) {
    let old_map: HashMap<NoteKey, &ExportNote> = old_notes
        .iter()
        .map(|note| (note_key(note), note))
        .collect();
    let new_map: HashMap<NoteKey, &ExportNote> = new_notes
        .iter()
        .map(|note| (note_key(note), note))
        .collect();

    let mut note_removed: Vec<ExportNote> = old_notes
        .iter()
        .filter(|note| !new_map.contains_key(&note_key(note)))
        .cloned()
        .collect();
    let mut note_added: Vec<ExportNote> = new_notes
        .iter()
        .filter(|note| !old_map.contains_key(&note_key(note)))
        .cloned()
        .collect();

    let mut note_changed = Vec::new();
    for (key, old_note) in &old_map {
        let Some(new_note) = new_map.get(key) else {
            continue;
        };
        if note_content_changed(old_note, new_note) {
            note_changed.push((*new_note).clone());
        }
    }

    note_removed.sort_by(compare_export_notes);
    note_added.sort_by(compare_export_notes);
    note_changed.sort_by(compare_export_notes);

    (note_added, note_removed, note_changed)
}

fn participant_key(participant: &ExportParticipantV4) -> ParticipantKey {
    ParticipantKey {
        sort_order: participant.sort_order,
        name: participant.name.clone(),
    }
}

fn compare_export_participants(a: &ExportParticipantV4, b: &ExportParticipantV4) -> Ordering {
    participant_key(a)
        .cmp(&participant_key(b))
        .then_with(|| a.is_self.cmp(&b.is_self))
}

fn format_participant_line(participant: &ExportParticipantV4) -> String {
    let self_mark = if participant.is_self { "yes" } else { "no" };
    format!(
        "#{} {} (self: {self_mark})",
        participant.sort_order, participant.name
    )
}

fn group_participants_by_key<'a>(
    participants: &'a [ExportParticipantV4],
) -> HashMap<ParticipantKey, Vec<&'a ExportParticipantV4>> {
    let mut grouped: HashMap<ParticipantKey, Vec<&'a ExportParticipantV4>> = HashMap::new();
    for participant in participants {
        grouped
            .entry(participant_key(participant))
            .or_default()
            .push(participant);
    }
    grouped
}

fn compute_participants_diff(
    old_participants: &[ExportParticipantV4],
    new_participants: &[ExportParticipantV4],
) -> (
    Vec<ExportParticipantV4>,
    Vec<ExportParticipantV4>,
    Vec<ParticipantFieldChange>,
) {
    let old_by_key = group_participants_by_key(old_participants);
    let new_by_key = group_participants_by_key(new_participants);

    let mut keys: Vec<ParticipantKey> = old_by_key
        .keys()
        .chain(new_by_key.keys())
        .cloned()
        .collect();
    keys.sort_unstable();
    keys.dedup();

    let mut participant_added = Vec::new();
    let mut participant_removed = Vec::new();
    let mut participant_changed = Vec::new();

    for key in keys {
        let old_list = old_by_key.get(&key).map(Vec::as_slice).unwrap_or(&[]);
        let new_list = new_by_key.get(&key).map(Vec::as_slice).unwrap_or(&[]);
        let paired = old_list.len().min(new_list.len());

        for index in 0..paired {
            let old_participant = old_list[index];
            let new_participant = new_list[index];
            if old_participant.is_self != new_participant.is_self {
                participant_changed.push(ParticipantFieldChange {
                    sort_order: key.sort_order,
                    name: key.name.clone(),
                    field: "is_self".to_string(),
                    old_value: old_participant.is_self.to_string(),
                    new_value: new_participant.is_self.to_string(),
                });
            }
        }

        for participant in &old_list[paired..] {
            participant_removed.push((*participant).clone());
        }
        for participant in &new_list[paired..] {
            participant_added.push((*participant).clone());
        }
    }

    participant_removed.sort_by(compare_export_participants);
    participant_added.sort_by(compare_export_participants);
    participant_changed.sort_by(|a, b| {
        participant_key(&ExportParticipantV4 {
            name: a.name.clone(),
            sort_order: a.sort_order,
            is_self: false,
        })
        .cmp(&participant_key(&ExportParticipantV4 {
            name: b.name.clone(),
            sort_order: b.sort_order,
            is_self: false,
        }))
        .then_with(|| a.field.cmp(&b.field))
    });

    (participant_added, participant_removed, participant_changed)
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
    if old.trip.summary != new.trip.summary {
        trip_changes.push((
            "summary".to_string(),
            fmt_diff_option_str(&old.trip.summary),
            fmt_diff_option_str(&new.trip.summary),
        ));
    }

    let mut day_summary_changes = Vec::new();
    let old_days: HashMap<i64, Option<String>> = old
        .day_summaries
        .iter()
        .map(|d| (d.day_number, d.summary.clone()))
        .collect();
    let new_days: HashMap<i64, Option<String>> = new
        .day_summaries
        .iter()
        .map(|d| (d.day_number, d.summary.clone()))
        .collect();
    let mut day_numbers: Vec<i64> = old_days.keys().chain(new_days.keys()).copied().collect();
    day_numbers.sort_unstable();
    day_numbers.dedup();
    for day_number in day_numbers {
        let old_summary = old_days.get(&day_number).cloned().unwrap_or(None);
        let new_summary = new_days.get(&day_number).cloned().unwrap_or(None);
        if old_summary != new_summary {
            day_summary_changes.push((
                day_number,
                fmt_diff_option_str(&old_summary),
                fmt_diff_option_str(&new_summary),
            ));
        }
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

    let (note_added, note_removed, note_changed) = compute_notes_diff(old.notes(), new.notes());
    let (reservation_added, reservation_removed, reservation_modified) =
        compute_reservations_diff(&old.reservations, &new.reservations);
    let (participant_added, participant_removed, participant_changed) =
        compute_participants_diff(old.participants(), new.participants());

    TripDiff {
        trip_changes,
        day_summary_changes,
        itinerary_added,
        itinerary_removed,
        itinerary_modified,
        note_added,
        note_removed,
        note_changed,
        reservation_added,
        reservation_removed,
        reservation_modified,
        participant_added,
        participant_removed,
        participant_changed,
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
    println!("Day summary:");
    if diff.day_summary_changes.is_empty() {
        println!("  （変更なし）");
    } else {
        for (day_number, old_value, new_value) in &diff.day_summary_changes {
            println!("~ Day {day_number} summary: {old_value} -> {new_value}");
        }
    }

    println!();
    println!("Itinerary:");
    if diff.itinerary_added.is_empty()
        && diff.itinerary_removed.is_empty()
        && diff.itinerary_modified.is_empty()
    {
        println!("  （変更なし）");
    } else {
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

    println!();
    println!("Notes:");
    if diff.note_added.is_empty() && diff.note_removed.is_empty() && diff.note_changed.is_empty() {
        println!("  （変更なし）");
    } else {
        for note in &diff.note_removed {
            println!("- Note removed: {}", format_note_line(note));
        }
        for note in &diff.note_added {
            println!("+ Note added: {}", format_note_line(note));
        }
        for note in &diff.note_changed {
            println!("~ Note changed: {}", format_note_line(note));
        }
    }

    println!();
    println!("Reservations:");
    if diff.reservation_added.is_empty()
        && diff.reservation_removed.is_empty()
        && diff.reservation_modified.is_empty()
    {
        println!("  （変更なし）");
    } else {
        for reservation in &diff.reservation_removed {
            println!(
                "- Reservation removed: {}",
                format_reservation_line(reservation)
            );
        }
        for reservation in &diff.reservation_added {
            println!(
                "+ Reservation added: {}",
                format_reservation_line(reservation)
            );
        }
        let mut current_line: Option<String> = None;
        for change in &diff.reservation_modified {
            if current_line.as_deref() != Some(change.line.as_str()) {
                println!("~ Reservation modified: {}", change.line);
                current_line = Some(change.line.clone());
            }
            println!(
                "  {}: {} -> {}",
                change.field, change.old_value, change.new_value
            );
        }
    }

    println!();
    println!("Participants:");
    if diff.participant_added.is_empty()
        && diff.participant_removed.is_empty()
        && diff.participant_changed.is_empty()
    {
        println!("  （変更なし）");
    } else {
        for participant in &diff.participant_removed {
            println!(
                "- Participant removed: {}",
                format_participant_line(participant)
            );
        }
        for participant in &diff.participant_added {
            println!(
                "+ Participant added: {}",
                format_participant_line(participant)
            );
        }
        let mut current_line: Option<String> = None;
        for change in &diff.participant_changed {
            let line = format!("#{} {}", change.sort_order, change.name);
            if current_line.as_deref() != Some(line.as_str()) {
                println!("~ Participant changed: {line}");
                current_line = Some(line);
            }
            println!(
                "  {}: {} -> {}",
                change.field, change.old_value, change.new_value
            );
        }
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
    use crate::models::{
        ExportNote, ExportReservation, ExportReservationV3, ItineraryItem, ItineraryNoteKey, Trip,
        TripExport,
    };

    fn make_test_trip(name: &str) -> Trip {
        Trip {
            id: 1,
            name: name.to_string(),
            start_date: None,
            end_date: None,
            summary: None,
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
            day_summaries: vec![],
            reservations: vec![],
            participants: vec![],
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
            day_summaries: vec![],
            reservations: vec![],
            participants: vec![],
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
            day_summaries: vec![],
            reservations: vec![],
            participants: vec![],
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
            day_summaries: vec![],
            reservations: vec![],
            participants: vec![],
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
            day_summaries: vec![],
            reservations: vec![],
            participants: vec![],
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
            day_summaries: vec![],
            reservations: vec![],
            participants: vec![],
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
            day_summaries: vec![],
            reservations: vec![],
            participants: vec![],
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
            day_summaries: vec![],
            reservations: vec![],
            participants: vec![],
        };

        let diff = compute_trip_diff(&old, &new);
        assert_eq!(diff.trip_changes.len(), 1);
        assert_eq!(diff.trip_changes[0].0, "name");
        assert_eq!(diff.trip_changes[0].1, "沖縄旅行");
        assert_eq!(diff.trip_changes[0].2, "沖縄・瀬底旅行");
    }

    #[test]
    fn test_diff_trip_summary_change() {
        let mut old_trip = make_test_trip("Trip");
        old_trip.summary = Some("old overview".to_string());
        let mut new_trip = make_test_trip("Trip");
        new_trip.summary = Some("new overview".to_string());

        let old = make_base_export(old_trip);
        let new = make_base_export(new_trip);

        let diff = compute_trip_diff(&old, &new);
        assert_eq!(diff.trip_changes.len(), 1);
        assert_eq!(diff.trip_changes[0].0, "summary");
        assert_eq!(diff.trip_changes[0].1, "old overview");
        assert_eq!(diff.trip_changes[0].2, "new overview");
    }

    fn make_base_export(trip: Trip) -> TripExport {
        TripExport {
            schema_version: None,
            generator: None,
            generator_version: None,
            exported_at: None,
            trip,
            itinerary_items: vec![],
            checklist_items: None,
            notes: None,
            day_summaries: vec![],
            reservations: vec![],
            participants: vec![],
        }
    }

    #[test]
    fn test_diff_notes_v1_vs_v2_empty_does_not_panic() {
        let old = make_base_export(make_test_trip("Trip"));
        let mut new = make_base_export(make_test_trip("Trip"));
        new.schema_version = Some(2);
        new.notes = Some(vec![]);

        let diff = compute_trip_diff(&old, &new);
        assert!(diff.note_added.is_empty());
        assert!(diff.note_removed.is_empty());
        assert!(diff.note_changed.is_empty());
    }

    #[test]
    fn test_diff_trip_note_added_removed_body_changed() {
        let old = make_base_export(make_test_trip("Trip"));
        let mut new = make_base_export(make_test_trip("Trip"));

        let added = ExportNote::Trip {
            title: Some("持ち物メモ".to_string()),
            body: "passport".to_string(),
        };
        new.notes = Some(vec![added.clone()]);

        let diff = compute_trip_diff(&old, &new);
        assert_eq!(diff.note_added.len(), 1);
        assert!(diff.note_removed.is_empty());
        assert!(diff.note_changed.is_empty());

        let diff = compute_trip_diff(&new, &old);
        assert_eq!(diff.note_removed.len(), 1);
        assert_eq!(diff.note_removed[0], added);

        let mut changed_old = make_base_export(make_test_trip("Trip"));
        changed_old.notes = Some(vec![ExportNote::Trip {
            title: Some("持ち物メモ".to_string()),
            body: "before".to_string(),
        }]);
        let mut changed_new = make_base_export(make_test_trip("Trip"));
        changed_new.notes = Some(vec![ExportNote::Trip {
            title: Some("持ち物メモ".to_string()),
            body: "after".to_string(),
        }]);

        let diff = compute_trip_diff(&changed_old, &changed_new);
        assert!(diff.note_added.is_empty());
        assert!(diff.note_removed.is_empty());
        assert_eq!(diff.note_changed.len(), 1);
    }

    #[test]
    fn test_diff_day_note_added_removed_body_changed() {
        let old = make_base_export(make_test_trip("Trip"));
        let mut new = make_base_export(make_test_trip("Trip"));

        let added = ExportNote::Day {
            day_number: 2,
            title: Some("夕食候補".to_string()),
            body: "steak".to_string(),
        };
        new.notes = Some(vec![added.clone()]);

        let diff = compute_trip_diff(&old, &new);
        assert_eq!(diff.note_added.len(), 1);

        let diff = compute_trip_diff(&new, &old);
        assert_eq!(diff.note_removed.len(), 1);

        let mut changed_old = make_base_export(make_test_trip("Trip"));
        changed_old.notes = Some(vec![ExportNote::Day {
            day_number: 2,
            title: Some("夕食候補".to_string()),
            body: "before".to_string(),
        }]);
        let mut changed_new = make_base_export(make_test_trip("Trip"));
        changed_new.notes = Some(vec![ExportNote::Day {
            day_number: 2,
            title: Some("夕食候補".to_string()),
            body: "after".to_string(),
        }]);

        let diff = compute_trip_diff(&changed_old, &changed_new);
        assert_eq!(diff.note_changed.len(), 1);
    }

    #[test]
    fn test_diff_itinerary_note_added_removed_body_changed() {
        let old = make_base_export(make_test_trip("Trip"));
        let mut new = make_base_export(make_test_trip("Trip"));

        let added = ExportNote::Itinerary {
            itinerary_key: ItineraryNoteKey {
                day_number: 2,
                sort_order: 3,
                start_time: Some("09:00".to_string()),
                title: "美ら海水族館".to_string(),
            },
            title: Some("水族館メモ".to_string()),
            body: "ticket info".to_string(),
        };
        new.notes = Some(vec![added.clone()]);

        let diff = compute_trip_diff(&old, &new);
        assert_eq!(diff.note_added.len(), 1);

        let diff = compute_trip_diff(&new, &old);
        assert_eq!(diff.note_removed.len(), 1);

        let mut changed_old = make_base_export(make_test_trip("Trip"));
        changed_old.notes = Some(vec![ExportNote::Itinerary {
            itinerary_key: ItineraryNoteKey {
                day_number: 2,
                sort_order: 3,
                start_time: Some("09:00".to_string()),
                title: "美ら海水族館".to_string(),
            },
            title: Some("水族館メモ".to_string()),
            body: "before".to_string(),
        }]);
        let mut changed_new = make_base_export(make_test_trip("Trip"));
        changed_new.notes = Some(vec![ExportNote::Itinerary {
            itinerary_key: ItineraryNoteKey {
                day_number: 2,
                sort_order: 3,
                start_time: Some("09:00".to_string()),
                title: "美ら海水族館".to_string(),
            },
            title: Some("水族館メモ".to_string()),
            body: "after".to_string(),
        }]);

        let diff = compute_trip_diff(&changed_old, &changed_new);
        assert_eq!(diff.note_changed.len(), 1);
    }

    #[test]
    fn test_diff_reservation_added_removed_modified() {
        let reservation = ExportReservation {
            itinerary_key: ItineraryNoteKey {
                day_number: 1,
                sort_order: 0,
                start_time: Some("16:40".to_string()),
                title: "Check-in".to_string(),
            },
            reservation: ExportReservationV3 {
                reservation_type: "hotel".to_string(),
                provider_name: "Hilton Sesoko Resort".to_string(),
                confirmation_code: Some("ABC123".to_string()),
                reservation_site_url: None,
                remark: None,
                start_at: None,
                end_at: None,
            },
        };

        let mut old = make_base_export(make_test_trip("Trip"));
        let new = make_base_export(make_test_trip("Trip"));
        old.reservations = vec![reservation.clone()];

        let diff = compute_trip_diff(&old, &new);
        assert_eq!(diff.reservation_removed.len(), 1);
        assert!(diff.reservation_added.is_empty());

        let diff = compute_trip_diff(&new, &old);
        assert_eq!(diff.reservation_added.len(), 1);
        assert!(diff.reservation_removed.is_empty());

        let mut modified_old = make_base_export(make_test_trip("Trip"));
        let mut modified_new = make_base_export(make_test_trip("Trip"));
        modified_old.reservations = vec![reservation];
        modified_new.reservations = vec![ExportReservation {
            itinerary_key: modified_old.reservations[0].itinerary_key.clone(),
            reservation: ExportReservationV3 {
                remark: Some("Twin room".to_string()),
                ..modified_old.reservations[0].reservation.clone()
            },
        }];
        let diff = compute_trip_diff(&modified_old, &modified_new);
        assert!(diff.reservation_added.is_empty());
        assert!(diff.reservation_removed.is_empty());
        assert_eq!(diff.reservation_modified.len(), 1);
        assert_eq!(diff.reservation_modified[0].field, "remark");
    }

    #[test]
    fn test_diff_itinerary_note_title_field_change() {
        let mut old = make_base_export(make_test_trip("Trip"));
        old.notes = Some(vec![ExportNote::Itinerary {
            itinerary_key: ItineraryNoteKey {
                day_number: 2,
                sort_order: 3,
                start_time: None,
                title: "美ら海水族館".to_string(),
            },
            title: Some("旧タイトル".to_string()),
            body: "same body".to_string(),
        }]);
        let mut new = make_base_export(make_test_trip("Trip"));
        new.notes = Some(vec![ExportNote::Itinerary {
            itinerary_key: ItineraryNoteKey {
                day_number: 2,
                sort_order: 3,
                start_time: None,
                title: "美ら海水族館".to_string(),
            },
            title: Some("新タイトル".to_string()),
            body: "same body".to_string(),
        }]);

        let diff = compute_trip_diff(&old, &new);
        assert_eq!(diff.note_changed.len(), 1);
        assert!(diff.note_added.is_empty());
        assert!(diff.note_removed.is_empty());
    }

    #[test]
    fn test_diff_participants_added_removed_and_is_self_changed() {
        use crate::models::ExportParticipantV4;

        let old = make_base_export(make_test_trip("Trip"));
        let mut new = make_base_export(make_test_trip("Trip"));
        new.participants = vec![
            ExportParticipantV4 {
                name: "ともさん".to_string(),
                sort_order: 0,
                is_self: true,
            },
            ExportParticipantV4 {
                name: "妻".to_string(),
                sort_order: 1,
                is_self: false,
            },
        ];

        let diff = compute_trip_diff(&old, &new);
        assert_eq!(diff.participant_added.len(), 2);
        assert!(diff.participant_removed.is_empty());
        assert!(diff.participant_changed.is_empty());

        let diff = compute_trip_diff(&new, &old);
        assert_eq!(diff.participant_removed.len(), 2);
        assert!(diff.participant_added.is_empty());

        let mut changed = make_base_export(make_test_trip("Trip"));
        changed.participants = vec![
            ExportParticipantV4 {
                name: "ともさん".to_string(),
                sort_order: 0,
                is_self: false,
            },
            ExportParticipantV4 {
                name: "妻".to_string(),
                sort_order: 1,
                is_self: true,
            },
        ];
        let diff = compute_trip_diff(&new, &changed);
        assert_eq!(diff.participant_changed.len(), 2);
        assert!(diff
            .participant_changed
            .iter()
            .all(|c| c.field == "is_self"));
    }
}
