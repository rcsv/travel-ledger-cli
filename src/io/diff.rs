use std::cmp::Ordering;
use std::collections::HashMap;

use anyhow::Result;

use crate::domain::models::{
    effective_export_schema_version, ExportEstimate, ExportExpense, ExportNote,
    ExportParticipantV4, ExportReceiptV7, ExportReservation, ItineraryCategory, ItineraryItem,
    TripExport, TRIP_EXPORT_SCHEMA_VERSION, TRIP_EXPORT_SCHEMA_VERSION_V5,
    TRIP_EXPORT_SCHEMA_VERSION_V6,
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

/// Expense の比較キー（Itinerary コンテキスト + expense 識別）
#[derive(Clone, Eq, PartialEq, Hash, PartialOrd, Ord)]
struct ExpenseKey {
    day_number: i64,
    sort_order: i64,
    start_time: Option<String>,
    itinerary_title: String,
    expense_sort_order: i64,
    expense_title: Option<String>,
    amount: i64,
    currency: String,
}

/// 1件の Expense におけるフィールド変更
struct ExpenseFieldChange {
    line: String,
    field: String,
    old_value: String,
    new_value: String,
}

/// Estimate の比較キー（Itinerary コンテキスト + estimate 識別）
#[derive(Clone, Eq, PartialEq, Hash, PartialOrd, Ord)]
struct EstimateKey {
    day_number: i64,
    sort_order: i64,
    start_time: Option<String>,
    itinerary_title: String,
    estimate_sort_order: i64,
    estimate_title: Option<String>,
}

/// 1件の Estimate におけるフィールド変更
struct EstimateFieldChange {
    line: String,
    field: String,
    old_value: String,
    new_value: String,
}

/// Receipt の比較キー（export 安定参照 — DB id は使わない）
#[derive(Clone, Eq, PartialEq, Hash, PartialOrd, Ord)]
struct ReceiptKey {
    day_number: Option<i64>,
    itinerary_day: Option<i64>,
    itinerary_sort: Option<i64>,
    itinerary_start_time: Option<String>,
    itinerary_title: Option<String>,
    amount: Option<i64>,
    currency: Option<String>,
    occurred_date: Option<String>,
    memo: Option<String>,
    status: String,
}

/// 1件の Receipt におけるフィールド変更
struct ReceiptFieldChange {
    line: String,
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
    expense_added: Vec<ExportExpense>,
    expense_removed: Vec<ExportExpense>,
    expense_modified: Vec<ExpenseFieldChange>,
    estimate_added: Vec<ExportEstimate>,
    estimate_removed: Vec<ExportEstimate>,
    estimate_modified: Vec<EstimateFieldChange>,
    receipt_added: Vec<ExportReceiptV7>,
    receipt_removed: Vec<ExportReceiptV7>,
    receipt_modified: Vec<ReceiptFieldChange>,
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

fn expense_key(expense: &ExportExpense) -> ExpenseKey {
    ExpenseKey {
        day_number: expense.itinerary_key.day_number,
        sort_order: expense.itinerary_key.sort_order,
        start_time: expense.itinerary_key.start_time.clone(),
        itinerary_title: expense.itinerary_key.title.clone(),
        expense_sort_order: expense.expense.sort_order,
        expense_title: expense.expense.title.clone(),
        amount: expense.expense.amount,
        currency: expense.expense.currency.clone(),
    }
}

fn compare_export_expenses(a: &ExportExpense, b: &ExportExpense) -> Ordering {
    expense_key(a).cmp(&expense_key(b))
}

fn format_expense_line(expense: &ExportExpense) -> String {
    let key = &expense.itinerary_key;
    let time = key.start_time.as_deref().unwrap_or("-");
    let title = expense.expense.title.as_deref().unwrap_or("-");
    format!(
        "Day{} {time} {} / {} / {} {}",
        key.day_number, key.title, title, expense.expense.amount, expense.expense.currency
    )
}

fn beneficiary_refs(expense: &ExportExpense) -> Vec<String> {
    let mut refs: Vec<String> = expense
        .expense
        .beneficiaries
        .iter()
        .map(|b| b.participant_ref.clone())
        .collect();
    refs.sort_unstable();
    refs
}

fn fmt_payer_ref(expense: &ExportExpense) -> String {
    expense
        .expense
        .paid_by_participant_ref
        .clone()
        .or(expense.expense.paid_by_name.clone())
        .unwrap_or_else(|| "-".to_string())
}

fn expense_shared_fields_changed(
    old: &ExportExpense,
    new: &ExportExpense,
    compare_shared_fields: bool,
) -> Vec<(String, String, String)> {
    if !compare_shared_fields {
        return Vec::new();
    }
    let mut changes = Vec::new();
    let old_payer = fmt_payer_ref(old);
    let new_payer = fmt_payer_ref(new);
    if old_payer != new_payer {
        changes.push(("payer".to_string(), old_payer, new_payer));
    }
    let old_beneficiaries = beneficiary_refs(old).join(", ");
    let new_beneficiaries = beneficiary_refs(new).join(", ");
    let old_display = if old_beneficiaries.is_empty() {
        "-".to_string()
    } else {
        old_beneficiaries
    };
    let new_display = if new_beneficiaries.is_empty() {
        "-".to_string()
    } else {
        new_beneficiaries
    };
    if old_display != new_display {
        changes.push(("beneficiaries".to_string(), old_display, new_display));
    }
    changes
}

fn schema_supports_shared_expense_diff(schema_version: Option<i32>) -> bool {
    effective_export_schema_version(schema_version) >= TRIP_EXPORT_SCHEMA_VERSION_V5
}

fn schema_supports_estimate_diff(schema_version: Option<i32>) -> bool {
    effective_export_schema_version(schema_version) >= TRIP_EXPORT_SCHEMA_VERSION_V6
}

fn schema_supports_receipt_diff(schema_version: Option<i32>) -> bool {
    effective_export_schema_version(schema_version) >= TRIP_EXPORT_SCHEMA_VERSION
}

fn estimate_key(estimate: &ExportEstimate) -> EstimateKey {
    EstimateKey {
        day_number: estimate.itinerary_key.day_number,
        sort_order: estimate.itinerary_key.sort_order,
        start_time: estimate.itinerary_key.start_time.clone(),
        itinerary_title: estimate.itinerary_key.title.clone(),
        estimate_sort_order: estimate.estimate.sort_order,
        estimate_title: estimate.estimate.title.clone(),
    }
}

fn compare_export_estimates(a: &ExportEstimate, b: &ExportEstimate) -> Ordering {
    estimate_key(a).cmp(&estimate_key(b))
}

fn format_estimate_line(estimate: &ExportEstimate) -> String {
    let key = &estimate.itinerary_key;
    let time = key.start_time.as_deref().unwrap_or("-");
    let title = estimate.estimate.title.as_deref().unwrap_or("-");
    format!(
        "Day{} {time} {} / {} / {} {}",
        key.day_number, key.title, title, estimate.estimate.amount, estimate.estimate.currency
    )
}

fn estimate_content_changed(
    old: &ExportEstimate,
    new: &ExportEstimate,
) -> Vec<(String, String, String)> {
    let fields = [
        (
            "title",
            fmt_diff_option_str(&old.estimate.title),
            fmt_diff_option_str(&new.estimate.title),
        ),
        (
            "amount",
            old.estimate.amount.to_string(),
            new.estimate.amount.to_string(),
        ),
        (
            "currency",
            old.estimate.currency.clone(),
            new.estimate.currency.clone(),
        ),
        (
            "note",
            fmt_diff_option_str(&old.estimate.note),
            fmt_diff_option_str(&new.estimate.note),
        ),
        (
            "sort_order",
            old.estimate.sort_order.to_string(),
            new.estimate.sort_order.to_string(),
        ),
    ];
    fields
        .into_iter()
        .filter(|(_, old_value, new_value)| old_value != new_value)
        .map(|(field, old_value, new_value)| (field.to_string(), old_value, new_value))
        .collect()
}

fn compute_estimates_diff(
    old_estimates: &[ExportEstimate],
    new_estimates: &[ExportEstimate],
    compare_estimates: bool,
) -> (
    Vec<ExportEstimate>,
    Vec<ExportEstimate>,
    Vec<EstimateFieldChange>,
) {
    if !compare_estimates {
        return (Vec::new(), Vec::new(), Vec::new());
    }

    let old_map: HashMap<EstimateKey, &ExportEstimate> = old_estimates
        .iter()
        .map(|estimate| (estimate_key(estimate), estimate))
        .collect();
    let new_map: HashMap<EstimateKey, &ExportEstimate> = new_estimates
        .iter()
        .map(|estimate| (estimate_key(estimate), estimate))
        .collect();

    let mut estimate_removed: Vec<ExportEstimate> = old_estimates
        .iter()
        .filter(|estimate| !new_map.contains_key(&estimate_key(estimate)))
        .cloned()
        .collect();
    let mut estimate_added: Vec<ExportEstimate> = new_estimates
        .iter()
        .filter(|estimate| !old_map.contains_key(&estimate_key(estimate)))
        .cloned()
        .collect();

    let mut estimate_modified = Vec::new();
    for old_estimate in old_map.values() {
        let Some(new_estimate) = new_map.get(&estimate_key(old_estimate)) else {
            continue;
        };
        let line = format_estimate_line(new_estimate);
        for (field, old_value, new_value) in estimate_content_changed(old_estimate, new_estimate) {
            estimate_modified.push(EstimateFieldChange {
                line: line.clone(),
                field,
                old_value,
                new_value,
            });
        }
    }

    estimate_removed.sort_by(compare_export_estimates);
    estimate_added.sort_by(compare_export_estimates);
    estimate_modified.sort_by(|a, b| a.line.cmp(&b.line).then_with(|| a.field.cmp(&b.field)));

    (estimate_added, estimate_removed, estimate_modified)
}

fn receipt_key(receipt: &ExportReceiptV7) -> ReceiptKey {
    ReceiptKey {
        day_number: receipt.day_ref.as_ref().map(|d| d.day_number),
        itinerary_day: receipt.itinerary_ref.as_ref().map(|it| it.day_number),
        itinerary_sort: receipt.itinerary_ref.as_ref().map(|it| it.sort_order),
        itinerary_start_time: receipt
            .itinerary_ref
            .as_ref()
            .and_then(|it| it.start_time.clone()),
        itinerary_title: receipt.itinerary_ref.as_ref().map(|it| it.title.clone()),
        amount: receipt.amount,
        currency: receipt.currency.clone(),
        occurred_date: receipt.occurred_date.clone(),
        memo: receipt.memo.clone(),
        status: receipt.status.clone(),
    }
}

fn compare_export_receipts(a: &ExportReceiptV7, b: &ExportReceiptV7) -> Ordering {
    receipt_key(a).cmp(&receipt_key(b))
}

fn format_receipt_line(receipt: &ExportReceiptV7) -> String {
    let day = receipt
        .day_ref
        .as_ref()
        .map(|d| d.day_number.to_string())
        .unwrap_or_else(|| "-".to_string());
    let amount = receipt
        .amount
        .map(|v| v.to_string())
        .unwrap_or_else(|| "-".to_string());
    let currency = receipt.currency.as_deref().unwrap_or("-");
    let memo = receipt.memo.as_deref().unwrap_or("-");
    format!(
        "Day {day} / {amount} {currency} / {memo} / {}",
        receipt.status
    )
}

fn receipt_content_changed(
    old: &ExportReceiptV7,
    new: &ExportReceiptV7,
) -> Vec<(String, String, String)> {
    let old_day = old
        .day_ref
        .as_ref()
        .map(|d| d.day_number.to_string())
        .unwrap_or_else(|| "-".to_string());
    let new_day = new
        .day_ref
        .as_ref()
        .map(|d| d.day_number.to_string())
        .unwrap_or_else(|| "-".to_string());
    let fields = [
        ("day_ref", old_day, new_day),
        (
            "amount",
            fmt_diff_option_i64(old.amount),
            fmt_diff_option_i64(new.amount),
        ),
        (
            "currency",
            fmt_diff_option_str(&old.currency),
            fmt_diff_option_str(&new.currency),
        ),
        (
            "occurred_date",
            fmt_diff_option_str(&old.occurred_date),
            fmt_diff_option_str(&new.occurred_date),
        ),
        (
            "memo",
            fmt_diff_option_str(&old.memo),
            fmt_diff_option_str(&new.memo),
        ),
        ("status", old.status.clone(), new.status.clone()),
    ];
    fields
        .into_iter()
        .filter(|(_, old_value, new_value)| old_value != new_value)
        .map(|(field, old_value, new_value)| (field.to_string(), old_value, new_value))
        .collect()
}

fn compute_receipts_diff(
    old_receipts: &[ExportReceiptV7],
    new_receipts: &[ExportReceiptV7],
    compare_receipts: bool,
) -> (
    Vec<ExportReceiptV7>,
    Vec<ExportReceiptV7>,
    Vec<ReceiptFieldChange>,
) {
    if !compare_receipts {
        return (Vec::new(), Vec::new(), Vec::new());
    }

    let old_map: HashMap<ReceiptKey, &ExportReceiptV7> = old_receipts
        .iter()
        .map(|receipt| (receipt_key(receipt), receipt))
        .collect();
    let new_map: HashMap<ReceiptKey, &ExportReceiptV7> = new_receipts
        .iter()
        .map(|receipt| (receipt_key(receipt), receipt))
        .collect();

    let mut receipt_removed: Vec<ExportReceiptV7> = old_receipts
        .iter()
        .filter(|receipt| !new_map.contains_key(&receipt_key(receipt)))
        .cloned()
        .collect();
    let mut receipt_added: Vec<ExportReceiptV7> = new_receipts
        .iter()
        .filter(|receipt| !old_map.contains_key(&receipt_key(receipt)))
        .cloned()
        .collect();

    let mut receipt_modified = Vec::new();
    for old_receipt in old_map.values() {
        let Some(new_receipt) = new_map.get(&receipt_key(old_receipt)) else {
            continue;
        };
        let line = format_receipt_line(new_receipt);
        for (field, old_value, new_value) in receipt_content_changed(old_receipt, new_receipt) {
            receipt_modified.push(ReceiptFieldChange {
                line: line.clone(),
                field,
                old_value,
                new_value,
            });
        }
    }

    receipt_removed.sort_by(compare_export_receipts);
    receipt_added.sort_by(compare_export_receipts);
    receipt_modified.sort_by(|a, b| a.line.cmp(&b.line).then_with(|| a.field.cmp(&b.field)));

    (receipt_added, receipt_removed, receipt_modified)
}

fn compute_expenses_diff(
    old_expenses: &[ExportExpense],
    new_expenses: &[ExportExpense],
    compare_shared_fields: bool,
) -> (
    Vec<ExportExpense>,
    Vec<ExportExpense>,
    Vec<ExpenseFieldChange>,
) {
    let old_map: HashMap<ExpenseKey, &ExportExpense> = old_expenses
        .iter()
        .map(|expense| (expense_key(expense), expense))
        .collect();
    let new_map: HashMap<ExpenseKey, &ExportExpense> = new_expenses
        .iter()
        .map(|expense| (expense_key(expense), expense))
        .collect();

    let mut expense_removed: Vec<ExportExpense> = old_expenses
        .iter()
        .filter(|expense| !new_map.contains_key(&expense_key(expense)))
        .cloned()
        .collect();
    let mut expense_added: Vec<ExportExpense> = new_expenses
        .iter()
        .filter(|expense| !old_map.contains_key(&expense_key(expense)))
        .cloned()
        .collect();

    let mut expense_modified = Vec::new();
    for (key, old_expense) in &old_map {
        let Some(new_expense) = new_map.get(key) else {
            continue;
        };
        let line = format_expense_line(new_expense);
        for (field, old_value, new_value) in
            expense_shared_fields_changed(old_expense, new_expense, compare_shared_fields)
        {
            expense_modified.push(ExpenseFieldChange {
                line: line.clone(),
                field,
                old_value,
                new_value,
            });
        }
    }

    expense_removed.sort_by(compare_export_expenses);
    expense_added.sort_by(compare_export_expenses);
    expense_modified.sort_by(|a, b| a.line.cmp(&b.line).then_with(|| a.field.cmp(&b.field)));

    (expense_added, expense_removed, expense_modified)
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
    let compare_shared_fields = schema_supports_shared_expense_diff(old.schema_version)
        && schema_supports_shared_expense_diff(new.schema_version);
    let (expense_added, expense_removed, expense_modified) =
        compute_expenses_diff(old.expenses(), new.expenses(), compare_shared_fields);
    let compare_estimates = schema_supports_estimate_diff(old.schema_version)
        && schema_supports_estimate_diff(new.schema_version);
    let (estimate_added, estimate_removed, estimate_modified) =
        compute_estimates_diff(old.estimates(), new.estimates(), compare_estimates);
    let compare_receipts = schema_supports_receipt_diff(old.schema_version)
        && schema_supports_receipt_diff(new.schema_version);
    let (receipt_added, receipt_removed, receipt_modified) =
        compute_receipts_diff(old.receipts(), new.receipts(), compare_receipts);

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
        expense_added,
        expense_removed,
        expense_modified,
        estimate_added,
        estimate_removed,
        estimate_modified,
        receipt_added,
        receipt_removed,
        receipt_modified,
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

    println!();
    println!("Expenses:");
    if diff.expense_added.is_empty()
        && diff.expense_removed.is_empty()
        && diff.expense_modified.is_empty()
    {
        println!("  （変更なし）");
    } else {
        for expense in &diff.expense_removed {
            println!("- Expense removed: {}", format_expense_line(expense));
        }
        for expense in &diff.expense_added {
            println!("+ Expense added: {}", format_expense_line(expense));
        }
        let mut current_line: Option<String> = None;
        for change in &diff.expense_modified {
            if current_line.as_deref() != Some(change.line.as_str()) {
                println!("~ Expense changed: {}", change.line);
                current_line = Some(change.line.clone());
            }
            println!(
                "  {}: {} -> {}",
                change.field, change.old_value, change.new_value
            );
        }
    }

    println!();
    println!("Estimates:");
    if diff.estimate_added.is_empty()
        && diff.estimate_removed.is_empty()
        && diff.estimate_modified.is_empty()
    {
        println!("  （変更なし）");
    } else {
        for estimate in &diff.estimate_removed {
            println!("- Estimate removed: {}", format_estimate_line(estimate));
        }
        for estimate in &diff.estimate_added {
            println!("+ Estimate added: {}", format_estimate_line(estimate));
        }
        let mut current_line: Option<String> = None;
        for change in &diff.estimate_modified {
            if current_line.as_deref() != Some(change.line.as_str()) {
                println!("~ Estimate modified: {}", change.line);
                current_line = Some(change.line.clone());
            }
            println!(
                "  {}: {} -> {}",
                change.field, change.old_value, change.new_value
            );
        }
    }

    println!();
    println!("Receipts:");
    if diff.receipt_added.is_empty()
        && diff.receipt_removed.is_empty()
        && diff.receipt_modified.is_empty()
    {
        println!("  （変更なし）");
    } else {
        for receipt in &diff.receipt_removed {
            println!("- Receipt removed: {}", format_receipt_line(receipt));
        }
        for receipt in &diff.receipt_added {
            println!("+ Receipt added: {}", format_receipt_line(receipt));
        }
        let mut current_line: Option<String> = None;
        for change in &diff.receipt_modified {
            if current_line.as_deref() != Some(change.line.as_str()) {
                println!("~ Receipt modified: {}", change.line);
                current_line = Some(change.line.clone());
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
    use crate::domain::models::{
        ExportNote, ExportReservation, ExportReservationV3, ItineraryItem, ItineraryNoteKey, Trip,
        TripExport, TRIP_EXPORT_SCHEMA_VERSION_V5,
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
            expenses: vec![],
            estimates: vec![],
            receipts: vec![],
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
            expenses: vec![],
            estimates: vec![],
            receipts: vec![],
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
            expenses: vec![],
            estimates: vec![],
            receipts: vec![],
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
            expenses: vec![],
            estimates: vec![],
            receipts: vec![],
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
            expenses: vec![],
            estimates: vec![],
            receipts: vec![],
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
            expenses: vec![],
            estimates: vec![],
            receipts: vec![],
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
            expenses: vec![],
            estimates: vec![],
            receipts: vec![],
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
            expenses: vec![],
            estimates: vec![],
            receipts: vec![],
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
            expenses: vec![],
            estimates: vec![],
            receipts: vec![],
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
        use crate::domain::models::ExportParticipantV4;

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

    #[test]
    fn test_diff_expense_payer_and_beneficiaries_changed() {
        use crate::domain::models::{
            ExportExpense, ExportExpenseBeneficiaryV5, ExportExpenseV3, TRIP_EXPORT_SCHEMA_VERSION,
            TRIP_EXPORT_SCHEMA_VERSION_V4,
        };

        let itinerary_key = ItineraryNoteKey {
            day_number: 1,
            sort_order: 0,
            start_time: Some("12:00".to_string()),
            title: "Lunch".to_string(),
        };
        let base_expense = ExportExpense {
            itinerary_key: itinerary_key.clone(),
            expense: ExportExpenseV3 {
                title: Some("Meal".to_string()),
                amount: 4000,
                currency: "JPY".to_string(),
                paid_by_name: None,
                paid_by_participant_ref: None,
                beneficiaries: vec![],
                expense_date: None,
                note: None,
                sort_order: 0,
            },
        };

        let mut old = make_base_export(make_test_trip("Trip"));
        old.schema_version = Some(TRIP_EXPORT_SCHEMA_VERSION);
        old.expenses = vec![base_expense.clone()];

        let mut new = make_base_export(make_test_trip("Trip"));
        new.schema_version = Some(TRIP_EXPORT_SCHEMA_VERSION);
        new.expenses = vec![ExportExpense {
            expense: ExportExpenseV3 {
                paid_by_participant_ref: Some("Alice".to_string()),
                beneficiaries: vec![
                    ExportExpenseBeneficiaryV5 {
                        participant_ref: "Alice".to_string(),
                        sort_order: Some(0),
                    },
                    ExportExpenseBeneficiaryV5 {
                        participant_ref: "Bob".to_string(),
                        sort_order: Some(1),
                    },
                ],
                ..base_expense.expense.clone()
            },
            ..base_expense.clone()
        }];

        let diff = compute_trip_diff(&old, &new);
        assert!(diff.expense_added.is_empty());
        assert!(diff.expense_removed.is_empty());
        let fields: Vec<&str> = diff
            .expense_modified
            .iter()
            .map(|c| c.field.as_str())
            .collect();
        assert!(fields.contains(&"payer"));
        assert!(fields.contains(&"beneficiaries"));

        let mut old_v4 = make_base_export(make_test_trip("Trip"));
        old_v4.schema_version = Some(TRIP_EXPORT_SCHEMA_VERSION_V4);
        old_v4.expenses = vec![base_expense.clone()];
        let mut new_v4 = make_base_export(make_test_trip("Trip"));
        new_v4.schema_version = Some(TRIP_EXPORT_SCHEMA_VERSION_V4);
        new_v4.expenses = vec![ExportExpense {
            expense: ExportExpenseV3 {
                paid_by_participant_ref: Some("Alice".to_string()),
                ..base_expense.expense
            },
            itinerary_key,
        }];
        let diff_v4 = compute_trip_diff(&old_v4, &new_v4);
        assert!(diff_v4.expense_modified.is_empty());
    }

    #[test]
    fn test_diff_estimate_added_and_amount_changed() {
        use crate::domain::models::{ExportEstimate, ExportEstimateV3, TRIP_EXPORT_SCHEMA_VERSION};

        let itinerary_key = ItineraryNoteKey {
            day_number: 1,
            sort_order: 0,
            start_time: Some("08:00".to_string()),
            title: "Hotel".to_string(),
        };
        let base_estimate = ExportEstimate {
            itinerary_key: itinerary_key.clone(),
            estimate: ExportEstimateV3 {
                title: Some("Breakfast".to_string()),
                amount: 14000,
                currency: "JPY".to_string(),
                note: Some("5 people".to_string()),
                sort_order: 0,
            },
        };

        let mut old = make_base_export(make_test_trip("Trip"));
        old.schema_version = Some(TRIP_EXPORT_SCHEMA_VERSION);
        old.estimates = vec![base_estimate.clone()];

        let mut new = make_base_export(make_test_trip("Trip"));
        new.schema_version = Some(TRIP_EXPORT_SCHEMA_VERSION);
        new.estimates = vec![
            ExportEstimate {
                estimate: ExportEstimateV3 {
                    amount: 15000,
                    ..base_estimate.estimate.clone()
                },
                ..base_estimate.clone()
            },
            ExportEstimate {
                itinerary_key,
                estimate: ExportEstimateV3 {
                    title: Some("Parking".to_string()),
                    amount: 5000,
                    currency: "JPY".to_string(),
                    note: None,
                    sort_order: 1,
                },
            },
        ];

        let diff = compute_trip_diff(&old, &new);
        assert_eq!(diff.estimate_added.len(), 1);
        assert_eq!(
            diff.estimate_added[0].estimate.title.as_deref(),
            Some("Parking")
        );
        assert_eq!(diff.estimate_modified.len(), 1);
        assert_eq!(diff.estimate_modified[0].field, "amount");
        assert_eq!(diff.estimate_modified[0].old_value, "14000");
        assert_eq!(diff.estimate_modified[0].new_value, "15000");

        let mut old_v5 = make_base_export(make_test_trip("Trip"));
        old_v5.schema_version = Some(TRIP_EXPORT_SCHEMA_VERSION_V5);
        old_v5.estimates = vec![base_estimate];
        let diff_v5 = compute_trip_diff(&old_v5, &new);
        assert!(diff_v5.estimate_added.is_empty());
        assert!(diff_v5.estimate_modified.is_empty());
    }
}
