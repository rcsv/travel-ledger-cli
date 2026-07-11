use anyhow::{bail, Context, Result};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::day::{find_day_by_trip_and_day_number, validate_trip_date_range};
use crate::domain::models::{
    parse_itinerary_category, Estimate, ExportDayV3, ExportEstimateV3, ExportExpenseV3,
    ExportItineraryV3, ExportNote, ExportReservationV3, ItineraryCategory, ItineraryNoteKey,
    TripExportV3, TRIP_EXPORT_GENERATOR, TRIP_EXPORT_SCHEMA_VERSION,
};
use crate::estimate::{get_estimate, list_estimates_for_itinerary, normalize_optional_text};
use crate::itinerary::{parse_time_hhmm, SORT_ORDER_STEP};
use crate::money::{parse_amount_for_currency, validate_currency_code};
use crate::output::json::print_json;
use crate::reservation::{validate_provider_name, validate_reservation_type};
use crate::trip::{analyze_trip_export_json, build_trip_export_v3, get_trip};

use super::fragment::analyze_proposal_fragment_json;

pub const FRAGMENT_APPLY_REPORT_SCHEMA_VERSION: i32 = 2;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FragmentApplyResolvedTarget {
    pub target_type: String,
    pub trip_id: i64,
    pub trip_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub day_number: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub itinerary_sort_order: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub itinerary_title: Option<String>,
    pub resolution: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FragmentApplyItineraryFieldChange {
    pub field: String,
    pub before: String,
    pub after: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FragmentApplyEstimateFieldChange {
    pub field: String,
    pub before: String,
    pub after: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FragmentApplyEstimateUpdatePreview {
    pub target_itinerary_id: i64,
    pub target_itinerary_title: String,
    pub target_estimate_id: i64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FragmentApplyEstimateDeletePreview {
    pub target_itinerary_id: i64,
    pub target_itinerary_title: String,
    pub target_estimate_id: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    pub amount: i64,
    pub currency: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
    pub sort_order: i64,
    pub updated_at: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FragmentApplyPreviewSummary {
    pub intent: String,
    pub action: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub candidate_title: Option<String>,
    pub itineraries_before: usize,
    pub itineraries_after: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes_before: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes_after: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expenses_before: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expenses_after: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expense_preview: Option<FragmentApplyExpensePreview>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub estimates_before: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub estimates_after: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub estimate_preview: Option<FragmentApplyEstimatePreview>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub estimate_update_preview: Option<FragmentApplyEstimateUpdatePreview>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub estimate_field_changes: Option<Vec<FragmentApplyEstimateFieldChange>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub estimate_delete_preview: Option<FragmentApplyEstimateDeletePreview>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reservations_before: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reservations_after: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reservation_preview: Option<FragmentApplyReservationPreview>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub itinerary_field_changes: Option<Vec<FragmentApplyItineraryFieldChange>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reorder_preview: Option<FragmentApplyReorderPreview>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub move_preview: Option<FragmentApplyMovePreview>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delete_preview: Option<FragmentApplyDeletePreview>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FragmentApplyExpensePreview {
    pub amount: i64,
    pub currency: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FragmentApplyEstimatePreview {
    pub target_itinerary_id: i64,
    pub target_itinerary_title: String,
    pub amount: i64,
    pub currency: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
    pub sort_order: i64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FragmentApplyBlockingChildren {
    pub expenses: usize,
    pub estimates: usize,
    pub reservations: usize,
    pub notes: usize,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FragmentApplyDeletePreview {
    pub target_type: String,
    pub itinerary_id: i64,
    pub title: String,
    pub day_number: i64,
    pub sort_order: i64,
    pub blocking_children: FragmentApplyBlockingChildren,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub non_blocking_relations: Option<Value>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FragmentApplyReorderPreview {
    pub day_number: i64,
    pub itinerary_order_changes: Vec<FragmentApplyItineraryOrderChange>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FragmentApplyMovePreview {
    pub itinerary_id: i64,
    pub title: String,
    pub from_day_number: i64,
    pub to_day_number: i64,
    pub source_order_changes: Vec<FragmentApplyItineraryMoveOrderChange>,
    pub destination_order_changes: Vec<FragmentApplyItineraryMoveOrderChange>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FragmentApplyItineraryOrderChange {
    pub itinerary_id: i64,
    pub title: String,
    pub before_sort_order: i64,
    pub after_sort_order: i64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FragmentApplyItineraryMoveOrderChange {
    pub itinerary_id: i64,
    pub title: String,
    pub before_day_number: i64,
    pub after_day_number: i64,
    pub before_sort_order: i64,
    pub after_sort_order: i64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FragmentApplyReservationPreview {
    pub reservation_type: String,
    pub provider_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confirmation_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reservation_site_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remark: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_at: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FragmentApplyDryRunReport {
    pub schema_version: i32,
    pub file: String,
    pub dry_run: bool,
    pub confirm: bool,
    pub valid: bool,
    pub fragment_valid: bool,
    pub trip_export_valid: bool,
    pub trip_id: i64,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub required_decisions: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolved_target: Option<FragmentApplyResolvedTarget>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preview: Option<FragmentApplyPreviewSummary>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inserted_itinerary_id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inserted_note_id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inserted_expense_id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inserted_reservation_id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inserted_estimate_id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_estimate_id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_itinerary_id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deleted_itinerary_id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deleted_estimate_id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reordered_itineraries: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub moved_itinerary_id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub moved_itinerary_updated_rows: Option<usize>,
}

impl FragmentApplyDryRunReport {
    fn new(file: impl Into<String>, trip_id: i64, dry_run: bool, confirm: bool) -> Self {
        Self {
            schema_version: FRAGMENT_APPLY_REPORT_SCHEMA_VERSION,
            file: file.into(),
            dry_run,
            confirm,
            valid: false,
            fragment_valid: false,
            trip_export_valid: false,
            trip_id,
            errors: Vec::new(),
            warnings: Vec::new(),
            required_decisions: Vec::new(),
            resolved_target: None,
            preview: None,
            inserted_itinerary_id: None,
            inserted_note_id: None,
            inserted_expense_id: None,
            inserted_reservation_id: None,
            inserted_estimate_id: None,
            updated_estimate_id: None,
            updated_itinerary_id: None,
            deleted_itinerary_id: None,
            deleted_estimate_id: None,
            reordered_itineraries: None,
            moved_itinerary_id: None,
            moved_itinerary_updated_rows: None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FragmentApplyOptions {
    pub dry_run: bool,
    pub confirm: bool,
    pub trip_id: i64,
    pub output: Option<String>,
    pub json: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ResolvedApplyTarget {
    target_type: String,
    trip_id: i64,
    trip_name: String,
    day_number: Option<i64>,
    itinerary_sort_order: Option<i64>,
    itinerary_title: Option<String>,
    resolution: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ParsedAddItineraryFields {
    title: String,
    note: Option<String>,
    location: Option<String>,
    category: Option<ItineraryCategory>,
    start_time: Option<String>,
    duration_minutes: Option<i64>,
    travel_minutes: Option<i64>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ParsedAddNoteFields {
    title: Option<String>,
    body: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ParsedAddExpenseFields {
    title: Option<String>,
    amount: i64,
    currency: String,
    note: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ParsedAddEstimateFields {
    title: Option<String>,
    amount: i64,
    currency: String,
    note: Option<String>,
    sort_order: i64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ParsedUpdateEstimateFields {
    estimate_id: i64,
    has_amount: bool,
    amount_raw: Option<Value>,
    has_currency: bool,
    currency_text: Option<String>,
    title: Option<UpdateFieldPatch<String>>,
    note: Option<UpdateFieldPatch<String>>,
    sort_order: Option<i64>,
    clear_title: bool,
    clear_note: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ParsedAddReservationFields {
    reservation_type: String,
    provider_name: String,
    confirmation_code: Option<String>,
    reservation_site_url: Option<String>,
    remark: Option<String>,
    start_at: Option<String>,
    end_at: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ParsedUpdateItineraryFields {
    title: Option<String>,
    note: Option<UpdateFieldPatch<String>>,
    location: Option<UpdateFieldPatch<String>>,
    category: Option<UpdateFieldPatch<ItineraryCategory>>,
    start_time: Option<UpdateFieldPatch<String>>,
    duration_minutes: Option<i64>,
    travel_minutes: Option<i64>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct UpdateFieldPatch<T> {
    value: T,
}

enum ConfirmInsertResult {
    Itinerary(i64),
    Note(i64),
    Expense(i64),
    Reservation(i64),
    Estimate(i64),
    UpdatedItinerary(i64),
    UpdatedEstimate(i64),
    DeletedItinerary(i64),
    DeletedEstimate(i64),
    ReorderedItineraries(usize),
    MovedItinerary {
        itinerary_id: i64,
        updated_rows: usize,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum ItineraryRefKey {
    Number(i64),
    Title(String),
}

pub fn run_fragment_apply(
    path: &str,
    conn: &Connection,
    options: &FragmentApplyOptions,
) -> Result<()> {
    if !options.dry_run && !options.confirm {
        bail!("fragment apply には --dry-run または --confirm のいずれかが必要です");
    }
    if options.dry_run && options.confirm {
        bail!("--dry-run と --confirm は併用できません（dry-run means no Trip domain data side effects）");
    }

    let json = std::fs::read_to_string(path)
        .with_context(|| format!("ファイル '{path}' を読み込めませんでした"))?;
    let (mut report, preview_json) = fragment_apply_gate_json(
        conn,
        path,
        &json,
        options.trip_id,
        options.dry_run,
        options.confirm,
    );

    if options.confirm {
        if report.valid {
            let preview_action = report
                .preview
                .as_ref()
                .map(|preview| preview.action.as_str())
                .unwrap_or("");
            let confirm_result = match preview_action {
                "add_itinerary" => {
                    execute_confirm_add_itinerary(conn, options.trip_id, &json, &report)
                        .map(ConfirmInsertResult::Itinerary)
                }
                "add_note" => execute_confirm_add_note(conn, options.trip_id, &json, &report)
                    .map(ConfirmInsertResult::Note),
                "add_expense" => execute_confirm_add_expense(conn, options.trip_id, &json, &report)
                    .map(ConfirmInsertResult::Expense),
                "add_reservation" => {
                    execute_confirm_add_reservation(conn, options.trip_id, &json, &report)
                        .map(ConfirmInsertResult::Reservation)
                }
                "add_estimate" => {
                    execute_confirm_add_estimate(conn, options.trip_id, &json, &report)
                        .map(ConfirmInsertResult::Estimate)
                }
                "update_itinerary" => {
                    execute_confirm_update_itinerary(conn, options.trip_id, &json, &report)
                        .map(ConfirmInsertResult::UpdatedItinerary)
                }
                "update_estimate" => {
                    execute_confirm_update_estimate(conn, options.trip_id, &json, &report)
                        .map(ConfirmInsertResult::UpdatedEstimate)
                }
                "delete_estimate" => {
                    execute_confirm_delete_estimate(conn, options.trip_id, &json, &report)
                        .map(ConfirmInsertResult::DeletedEstimate)
                }
                "delete_itinerary" => {
                    execute_confirm_delete_itinerary(conn, options.trip_id, &report)
                        .map(ConfirmInsertResult::DeletedItinerary)
                }
                "reorder_itinerary" => {
                    execute_confirm_reorder_itinerary(conn, options.trip_id, &json, &report)
                        .map(ConfirmInsertResult::ReorderedItineraries)
                }
                "move_itinerary" => {
                    execute_confirm_move_itinerary(conn, options.trip_id, &json, &report).map(
                        |result| ConfirmInsertResult::MovedItinerary {
                            itinerary_id: result.itinerary_id,
                            updated_rows: result.updated_rows,
                        },
                    )
                }
                other => Err(anyhow::anyhow!("未対応の confirm action です: {other}")),
            };
            match confirm_result {
                Ok(ConfirmInsertResult::Itinerary(itinerary_id)) => {
                    report.inserted_itinerary_id = Some(itinerary_id);
                    let item = crate::itinerary::get_itinerary_item(conn, itinerary_id)?;
                    if !options.json {
                        println!();
                        println!("Itinerary を DB に追加しました（fragment apply --confirm）");
                        println!("  itinerary ID : {itinerary_id}");
                        println!("  旅行 ID      : {}", options.trip_id);
                        println!("  日目         : {}", item.day);
                        println!("  並び順       : {}", item.sort_order);
                        println!("  タイトル     : {}", item.title);
                    }
                }
                Ok(ConfirmInsertResult::Note(note_id)) => {
                    report.inserted_note_id = Some(note_id);
                    let note = crate::note::get_note(conn, note_id)?;
                    if !options.json {
                        println!();
                        println!("Note を DB に追加しました（fragment apply --confirm）");
                        println!("  note ID      : {note_id}");
                        println!("  旅行 ID      : {}", options.trip_id);
                        println!("  owner_type   : {}", note.owner_type.as_str());
                        if let Some(title) = &note.title {
                            println!("  タイトル     : {title}");
                        }
                        println!("  body         : {}", note.body);
                    }
                }
                Ok(ConfirmInsertResult::Expense(expense_id)) => {
                    report.inserted_expense_id = Some(expense_id);
                    let expense = crate::expense::get_expense(conn, expense_id)?;
                    if !options.json {
                        println!();
                        println!("Expense を DB に追加しました（fragment apply --confirm）");
                        println!("  expense ID   : {expense_id}");
                        println!("  旅行 ID      : {}", options.trip_id);
                        println!("  itinerary ID : {}", expense.itinerary_id);
                        println!("  amount       : {}", expense.amount);
                        println!("  currency     : {}", expense.currency);
                        if let Some(title) = &expense.title {
                            println!("  タイトル     : {title}");
                        }
                        if let Some(note) = &expense.note {
                            println!("  note         : {note}");
                        }
                    }
                }
                Ok(ConfirmInsertResult::Reservation(reservation_id)) => {
                    report.inserted_reservation_id = Some(reservation_id);
                    let reservation = crate::reservation::get_reservation(conn, reservation_id)?;
                    if !options.json {
                        println!();
                        println!("Reservation を DB に追加しました（fragment apply --confirm）");
                        println!("  reservation ID : {reservation_id}");
                        println!("  旅行 ID        : {}", options.trip_id);
                        println!("  itinerary ID   : {}", reservation.itinerary_id);
                        println!("  type           : {}", reservation.reservation_type);
                        println!("  provider       : {}", reservation.provider_name);
                        if let Some(code) = &reservation.confirmation_code {
                            println!("  confirmation   : {code}");
                        }
                        if let Some(remark) = &reservation.remark {
                            println!("  remark         : {remark}");
                        }
                    }
                }
                Ok(ConfirmInsertResult::Estimate(estimate_id)) => {
                    report.inserted_estimate_id = Some(estimate_id);
                    let estimate = crate::estimate::get_estimate(conn, estimate_id)?;
                    if !options.json {
                        println!();
                        println!("Estimate を DB に追加しました（fragment apply --confirm）");
                        println!("  estimate ID  : {estimate_id}");
                        println!("  旅行 ID      : {}", options.trip_id);
                        println!("  itinerary ID : {}", estimate.itinerary_id);
                        println!("  amount       : {}", estimate.amount);
                        println!("  currency     : {}", estimate.currency);
                        println!("  sort_order   : {}", estimate.sort_order);
                        if let Some(title) = &estimate.title {
                            println!("  タイトル     : {title}");
                        }
                        if let Some(note) = &estimate.note {
                            println!("  note         : {note}");
                        }
                    }
                }
                Ok(ConfirmInsertResult::UpdatedItinerary(itinerary_id)) => {
                    report.updated_itinerary_id = Some(itinerary_id);
                    let item = crate::itinerary::get_itinerary_item(conn, itinerary_id)?;
                    if !options.json {
                        println!();
                        println!("Itinerary を DB に更新しました（fragment apply --confirm）");
                        println!("  itinerary ID : {itinerary_id}");
                        println!("  旅行 ID      : {}", options.trip_id);
                        println!("  日目         : {}", item.day);
                        println!("  タイトル     : {}", item.title);
                        if let Some(category) = item.category {
                            println!("  category     : {}", category.as_str());
                        }
                        if let Some(note) = &item.note {
                            println!("  note         : {note}");
                        }
                    }
                }
                Ok(ConfirmInsertResult::UpdatedEstimate(estimate_id)) => {
                    report.updated_estimate_id = Some(estimate_id);
                    let estimate = crate::estimate::get_estimate(conn, estimate_id)?;
                    if !options.json {
                        println!();
                        println!("Estimate を DB に更新しました（fragment apply --confirm）");
                        println!("  estimate ID  : {estimate_id}");
                        println!("  旅行 ID      : {}", options.trip_id);
                        println!("  itinerary ID : {}", estimate.itinerary_id);
                        println!("  amount       : {}", estimate.amount);
                        println!("  currency     : {}", estimate.currency);
                        println!("  sort_order   : {}", estimate.sort_order);
                        if let Some(title) = &estimate.title {
                            println!("  タイトル     : {title}");
                        }
                        if let Some(note) = &estimate.note {
                            println!("  note         : {note}");
                        }
                    }
                }
                Ok(ConfirmInsertResult::DeletedEstimate(estimate_id)) => {
                    report.deleted_estimate_id = Some(estimate_id);
                    if !options.json {
                        println!();
                        println!("Estimate を DB から削除しました（fragment apply --confirm）");
                        println!("  deleted_estimate_id : {estimate_id}");
                        println!("  旅行 ID             : {}", options.trip_id);
                    }
                }
                Ok(ConfirmInsertResult::DeletedItinerary(itinerary_id)) => {
                    report.deleted_itinerary_id = Some(itinerary_id);
                    if !options.json {
                        println!();
                        println!("Itinerary を DB から削除しました（fragment apply --confirm）");
                        println!("  deleted_itinerary_id : {itinerary_id}");
                        println!("  旅行 ID              : {}", options.trip_id);
                    }
                }
                Ok(ConfirmInsertResult::ReorderedItineraries(updated)) => {
                    report.reordered_itineraries = Some(updated);
                    if !options.json {
                        println!();
                        println!(
                            "Itinerary の並び順を DB に反映しました（fragment apply --confirm）"
                        );
                        println!("  updated_rows : {updated}");
                        println!("  旅行 ID      : {}", options.trip_id);
                    }
                }
                Ok(ConfirmInsertResult::MovedItinerary {
                    itinerary_id,
                    updated_rows,
                }) => {
                    report.moved_itinerary_id = Some(itinerary_id);
                    report.moved_itinerary_updated_rows = Some(updated_rows);
                    if !options.json {
                        let item = crate::itinerary::get_itinerary_item(conn, itinerary_id)?;
                        println!();
                        println!("Itinerary を別 Day へ移動しました（fragment apply --confirm）");
                        println!("  moved_itinerary_id : {itinerary_id}");
                        println!("  updated_rows       : {updated_rows}");
                        println!("  旅行 ID            : {}", options.trip_id);
                        println!("  日目               : {}", item.day);
                        println!("  並び順             : {}", item.sort_order);
                        println!("  タイトル           : {}", item.title);
                    }
                }
                Err(error) => {
                    report.valid = false;
                    report.errors.push(error.to_string());
                }
            }
        }
    } else if report.valid {
        if let Some(output_path) = options.output.as_deref() {
            let preview_json = preview_json.expect("valid dry-run must produce preview JSON");
            std::fs::write(output_path, &preview_json)
                .with_context(|| format!("ファイル '{output_path}' への書き込みに失敗しました"))?;
            if !options.json {
                println!("apply preview（schema v8 Trip JSON）を書き込みました: {output_path}");
            }
        } else if let Some(preview_json) = preview_json {
            if !options.json {
                println!();
                println!("--- apply preview（schema v8 Trip JSON）---");
                println!("{preview_json}");
            }
        }
    }

    if options.json {
        print_json(&report)?;
    } else {
        print_fragment_apply_report(&report);
    }

    if !report.valid {
        let mode = if options.confirm {
            "--confirm"
        } else {
            "--dry-run"
        };
        anyhow::bail!("fragment apply {mode} に失敗しました");
    }

    Ok(())
}

#[allow(dead_code)]
pub fn fragment_apply_dry_run_json(
    conn: &Connection,
    path: &str,
    json: &str,
    trip_id: i64,
) -> (FragmentApplyDryRunReport, Option<String>) {
    fragment_apply_gate_json(conn, path, json, trip_id, true, false)
}

fn fragment_apply_gate_json(
    conn: &Connection,
    path: &str,
    json: &str,
    trip_id: i64,
    dry_run: bool,
    confirm: bool,
) -> (FragmentApplyDryRunReport, Option<String>) {
    let mut report = FragmentApplyDryRunReport::new(path, trip_id, dry_run, confirm);
    let validation = analyze_proposal_fragment_json(path, json);
    report.fragment_valid = validation.valid;
    report.warnings.extend(validation.warnings);

    if !validation.valid {
        report.errors.push(
            "Proposal Fragment の validation に失敗しました（fragment validate を先に通してください）"
                .to_string(),
        );
        report.errors.extend(validation.errors);
        return (report, None);
    }

    let root: Value = match serde_json::from_str(json) {
        Ok(value) => value,
        Err(error) => {
            report
                .errors
                .push(format!("JSON の parse に失敗しました: {error}"));
            return (report, None);
        }
    };

    let Some(root_obj) = root.as_object() else {
        report
            .errors
            .push("トップレベルが JSON object ではありません".to_string());
        return (report, None);
    };

    let target = match root_obj.get("target").and_then(Value::as_object) {
        Some(obj) => obj,
        None => {
            report.errors.push("target object が必要です".to_string());
            return (report, None);
        }
    };

    let fragment_body = match root_obj.get("fragment").and_then(Value::as_object) {
        Some(obj) => obj,
        None => {
            report.errors.push("fragment object が必要です".to_string());
            return (report, None);
        }
    };

    collect_required_decisions(
        root_obj.get("adoption_hints"),
        target,
        &mut report.required_decisions,
    );

    let trip = match get_trip(conn, trip_id) {
        Ok(trip) => trip,
        Err(error) => {
            report.errors.push(error.to_string());
            return (report, None);
        }
    };

    if let Some(trip_ref) = non_empty_string(target.get("trip_reference")) {
        if !trip.name.eq_ignore_ascii_case(trip_ref.trim()) {
            report.warnings.push(format!(
                "target.trip_reference ({trip_ref}) が --trip {trip_id} の名前 ({}) と一致しません — CLI の --trip を優先します",
                trip.name
            ));
        }
    }

    let resolved = match resolve_apply_target(conn, trip_id, &trip.name, target, &mut report) {
        Ok(Some(resolved)) => resolved,
        Ok(None) => return (report, None),
        Err(()) => return (report, None),
    };

    report.resolved_target = Some(FragmentApplyResolvedTarget {
        target_type: resolved.target_type.clone(),
        trip_id: resolved.trip_id,
        trip_name: resolved.trip_name.clone(),
        day_number: resolved.day_number,
        itinerary_sort_order: resolved.itinerary_sort_order,
        itinerary_title: resolved.itinerary_title.clone(),
        resolution: resolved.resolution.clone(),
    });

    let intent = non_empty_string(fragment_body.get("intent")).unwrap_or_default();
    let preview_export = match build_trip_export_v3(conn, trip_id) {
        Ok(export) => export,
        Err(error) => {
            report.errors.push(error.to_string());
            return (report, None);
        }
    };

    let itineraries_before = count_itineraries(&preview_export);
    let notes_before = count_notes(&preview_export);
    let expenses_before = count_expenses(&preview_export);
    let reservations_before = count_reservations(&preview_export);
    let estimates_before = count_estimates(&preview_export);
    let mut simulated = preview_export;
    let preview_summary = match simulate_apply_preview(
        conn,
        trip_id,
        &mut simulated,
        &resolved,
        fragment_body,
        &intent,
        itineraries_before,
        notes_before,
        expenses_before,
        reservations_before,
        estimates_before,
        &mut report,
    ) {
        Ok(summary) => summary,
        Err(message) => {
            report.errors.push(message);
            return (report, None);
        }
    };

    finalize_apply_preview_export(&mut simulated);

    let preview_json = match serde_json::to_string_pretty(&simulated) {
        Ok(json) => json,
        Err(error) => {
            report
                .errors
                .push(format!("apply preview JSON の生成に失敗しました: {error}"));
            return (report, None);
        }
    };

    let export_validation = analyze_trip_export_json(path, &preview_json);
    report.trip_export_valid = export_validation.valid;
    if !export_validation.valid {
        report
            .errors
            .push("apply preview が trip validate-export に合格しませんでした".to_string());
        report.errors.extend(export_validation.errors);
        return (report, None);
    }

    // v4.7.33: delete_itinerary の blocking child 等により errors が追加されうる。
    // preview.delete_preview を返したうえで valid=false とし、preview JSON は書き出さない。
    if !report.errors.is_empty() {
        report.preview = Some(preview_summary);
        return (report, None);
    }

    if confirm && !validate_confirm_scope(&resolved, &intent, &preview_summary, &mut report) {
        report.preview = Some(preview_summary);
        return (report, None);
    }

    if !report.required_decisions.is_empty() {
        report.errors.push(
            "required decisions が未解決です — apply preview は valid confirm candidate ではありません"
                .to_string(),
        );
        report.preview = Some(preview_summary);
        return (report, Some(preview_json));
    }

    report.preview = Some(preview_summary);
    report.valid = true;
    (report, Some(preview_json))
}

fn validate_confirm_scope(
    resolved: &ResolvedApplyTarget,
    intent: &str,
    preview: &FragmentApplyPreviewSummary,
    report: &mut FragmentApplyDryRunReport,
) -> bool {
    if resolved.resolution == "ambiguous" {
        report
            .errors
            .push("target が曖昧です — DB 更新しません".to_string());
        return false;
    }
    if !report.required_decisions.is_empty() {
        report
            .errors
            .push("required decisions が未解決です — DB 更新しません".to_string());
        return false;
    }

    match (intent, preview.action.as_str()) {
        ("add", "add_itinerary") => {
            if resolved.target_type != "day" {
                report.errors.push(
                    "v4.7.19 --confirm は Day target + add_itinerary のみサポートしています"
                        .to_string(),
                );
                return false;
            }
            true
        }
        ("add_note", "add_note") => {
            if !matches!(resolved.target_type.as_str(), "trip" | "day" | "itinerary") {
                report.errors.push(format!(
                    "v4.7.23 --confirm は trip / day / itinerary target + add_note のみサポートしています（現在: target_type={}）",
                    resolved.target_type
                ));
                return false;
            }
            true
        }
        ("add_expense", "add_expense") => {
            if resolved.target_type != "itinerary" {
                report.errors.push(format!(
                    "v4.7.25 --confirm は itinerary target + add_expense のみサポートしています（現在: target_type={}）",
                    resolved.target_type
                ));
                return false;
            }
            true
        }
        ("add_reservation", "add_reservation") => {
            if resolved.target_type != "itinerary" {
                report.errors.push(format!(
                    "v4.7.27 --confirm は itinerary target + add_reservation のみサポートしています（現在: target_type={}）",
                    resolved.target_type
                ));
                return false;
            }
            true
        }
        ("add_estimate", "add_estimate") => {
            if resolved.target_type != "itinerary" {
                report.errors.push(format!(
                    "v4.7.43 --confirm は itinerary target + add_estimate のみサポートしています（現在: target_type={}）",
                    resolved.target_type
                ));
                return false;
            }
            if preview.estimate_preview.is_none() {
                report
                    .errors
                    .push("add_estimate confirm には estimate_preview が必要です".to_string());
                return false;
            }
            true
        }
        ("update_itinerary", "update_itinerary") => {
            if resolved.target_type != "itinerary" {
                report.errors.push(format!(
                    "v4.7.29 --confirm は itinerary target + update_itinerary のみサポートしています（現在: target_type={}）",
                    resolved.target_type
                ));
                return false;
            }
            true
        }
        ("update_estimate", "update_estimate") => {
            if resolved.target_type != "itinerary" {
                report.errors.push(format!(
                    "v4.7.48 --confirm は itinerary target + update_estimate のみサポートしています（現在: target_type={}）",
                    resolved.target_type
                ));
                return false;
            }
            if preview.estimate_update_preview.is_none() {
                report.errors.push(
                    "update_estimate confirm には estimate_update_preview が必要です".to_string(),
                );
                return false;
            }
            if preview
                .estimate_field_changes
                .as_ref()
                .is_none_or(|changes| changes.is_empty())
            {
                report.errors.push(
                    "update_estimate confirm には estimate_field_changes が必要です".to_string(),
                );
                return false;
            }
            true
        }
        ("delete_estimate", "delete_estimate") => {
            if resolved.target_type != "itinerary" {
                report.errors.push(format!(
                    "v4.8.2 --confirm は itinerary target + delete_estimate のみサポートしています（現在: target_type={}）",
                    resolved.target_type
                ));
                return false;
            }
            if preview.estimate_delete_preview.is_none() {
                report.errors.push(
                    "delete_estimate confirm には estimate_delete_preview が必要です".to_string(),
                );
                return false;
            }
            true
        }
        ("delete_itinerary", "delete_itinerary") => {
            if resolved.target_type != "itinerary" {
                report.errors.push(format!(
                    "v4.7.32 --confirm は itinerary target + delete_itinerary のみサポートしています（現在: target_type={}）",
                    resolved.target_type
                ));
                return false;
            }
            if preview.delete_preview.is_none() {
                report
                    .errors
                    .push("delete_itinerary confirm には delete_preview が必要です".to_string());
                return false;
            }
            true
        }
        ("reorder_itinerary", "reorder_itinerary") => {
            if resolved.target_type != "day" {
                report.errors.push(format!(
                    "v4.7.36 --confirm は day target + reorder_itinerary のみサポートしています（現在: target_type={}）",
                    resolved.target_type
                ));
                return false;
            }
            if preview.reorder_preview.is_none() {
                report
                    .errors
                    .push("reorder_itinerary confirm には reorder_preview が必要です".to_string());
                return false;
            }
            true
        }
        ("move_itinerary", "move_itinerary") => {
            if resolved.target_type != "itinerary" {
                report.errors.push(format!(
                    "v4.7.39 --confirm は itinerary target + move_itinerary のみサポートしています（現在: target_type={}）",
                    resolved.target_type
                ));
                return false;
            }
            if preview.move_preview.is_none() {
                report
                    .errors
                    .push("move_itinerary confirm には move_preview が必要です".to_string());
                return false;
            }
            true
        }
        _ => {
            report.errors.push(format!(
                "v4.8.2 --confirm は intent add (add_itinerary)、add_note、add_expense、add_reservation、add_estimate、update_itinerary、update_estimate、delete_estimate、delete_itinerary、または将来の confirm 対象 intent のみサポートしています（現在: intent={intent}, action={}）",
                preview.action
            ));
            false
        }
    }
}

fn execute_confirm_add_itinerary(
    conn: &Connection,
    trip_id: i64,
    json: &str,
    report: &FragmentApplyDryRunReport,
) -> Result<i64> {
    let root: Value =
        serde_json::from_str(json).with_context(|| "Fragment JSON の parse に失敗しました")?;
    let root_obj = root
        .as_object()
        .context("トップレベルが JSON object ではありません")?;
    let fragment_body = root_obj
        .get("fragment")
        .and_then(Value::as_object)
        .context("fragment object が必要です")?;
    let day_number = report
        .resolved_target
        .as_ref()
        .and_then(|target| target.day_number)
        .context("target Day が解決されていません")?;
    let fields =
        parse_add_itinerary_fields(fragment_body, None).map_err(|error| anyhow::anyhow!(error))?;

    crate::itinerary::add_itinerary_item(
        conn,
        trip_id,
        day_number,
        &fields.title,
        fields.note.as_deref(),
        fields.start_time.as_deref(),
        None,
        fields.duration_minutes,
        fields.travel_minutes,
        fields.location.as_deref(),
        fields.category,
    )
}

fn execute_confirm_add_note(
    conn: &Connection,
    trip_id: i64,
    json: &str,
    report: &FragmentApplyDryRunReport,
) -> Result<i64> {
    let root: Value =
        serde_json::from_str(json).with_context(|| "Fragment JSON の parse に失敗しました")?;
    let root_obj = root
        .as_object()
        .context("トップレベルが JSON object ではありません")?;
    let fragment_body = root_obj
        .get("fragment")
        .and_then(Value::as_object)
        .context("fragment object が必要です")?;
    let target = report
        .resolved_target
        .as_ref()
        .context("target が解決されていません")?;
    let fields =
        parse_add_note_fields(fragment_body, None).map_err(|error| anyhow::anyhow!(error))?;
    let owner = resolve_note_owner_for_apply_target(conn, trip_id, target)?;
    crate::note::add_note(conn, owner, fields.title.as_deref(), &fields.body)
}

fn execute_confirm_add_expense(
    conn: &Connection,
    trip_id: i64,
    json: &str,
    report: &FragmentApplyDryRunReport,
) -> Result<i64> {
    let root: Value =
        serde_json::from_str(json).with_context(|| "Fragment JSON の parse に失敗しました")?;
    let root_obj = root
        .as_object()
        .context("トップレベルが JSON object ではありません")?;
    let fragment_body = root_obj
        .get("fragment")
        .and_then(Value::as_object)
        .context("fragment object が必要です")?;
    let target = report
        .resolved_target
        .as_ref()
        .context("target が解決されていません")?;
    let fields =
        parse_add_expense_fields(fragment_body, None).map_err(|error| anyhow::anyhow!(error))?;
    let itinerary_id = resolve_itinerary_id_for_apply_target(conn, trip_id, target)?;
    let result = crate::services::expense_add::add_expense(
        conn,
        crate::services::expense_add::ExpenseAddParams {
            itinerary: itinerary_id,
            amount: fields.amount.to_string(),
            currency: fields.currency,
            title: fields.title,
            note: fields.note,
            paid_by_name: None,
            paid_by_participant: None,
            beneficiary: vec![],
            shared_with: None,
            expense_date: None,
        },
    )?;
    Ok(result.id)
}

fn execute_confirm_add_reservation(
    conn: &Connection,
    trip_id: i64,
    json: &str,
    report: &FragmentApplyDryRunReport,
) -> Result<i64> {
    let root: Value =
        serde_json::from_str(json).with_context(|| "Fragment JSON の parse に失敗しました")?;
    let root_obj = root
        .as_object()
        .context("トップレベルが JSON object ではありません")?;
    let fragment_body = root_obj
        .get("fragment")
        .and_then(Value::as_object)
        .context("fragment object が必要です")?;
    let target = report
        .resolved_target
        .as_ref()
        .context("target が解決されていません")?;
    let fields = parse_add_reservation_fields(fragment_body, None)
        .map_err(|error| anyhow::anyhow!(error))?;
    let itinerary_id = resolve_itinerary_id_for_apply_target(conn, trip_id, target)?;
    let result = crate::services::reservation_add::add_reservation(
        conn,
        crate::services::reservation_add::ReservationAddParams {
            itinerary: itinerary_id,
            reservation_type: fields.reservation_type,
            provider: fields.provider_name,
            confirmation: fields.confirmation_code,
            site_url: fields.reservation_site_url,
            remark: fields.remark,
            start_at: fields.start_at,
            end_at: fields.end_at,
        },
    )?;
    Ok(result.id)
}

fn verify_estimate_preview_matches_fields(
    preview: &FragmentApplyEstimatePreview,
    fields: &ParsedAddEstimateFields,
    itinerary_id: i64,
) -> Result<(), String> {
    if itinerary_id != preview.target_itinerary_id {
        return Err(format!(
            "gate preview と target itinerary が一致しません（preview: {}, resolved: {itinerary_id}）— DB 更新しません",
            preview.target_itinerary_id
        ));
    }
    if fields.amount != preview.amount {
        return Err(format!(
            "gate preview と amount が一致しません（preview: {}, parsed: {}）— DB 更新しません",
            preview.amount, fields.amount
        ));
    }
    if fields.currency != preview.currency {
        return Err(format!(
            "gate preview と currency が一致しません（preview: {}, parsed: {}）— DB 更新しません",
            preview.currency, fields.currency
        ));
    }
    if fields.title != preview.title {
        return Err("gate preview と title が一致しません — DB 更新しません".to_string());
    }
    if fields.note != preview.note {
        return Err("gate preview と note が一致しません — DB 更新しません".to_string());
    }
    if fields.sort_order != preview.sort_order {
        return Err(format!(
            "gate preview と sort_order が一致しません（preview: {}, parsed: {}）— DB 更新しません",
            preview.sort_order, fields.sort_order
        ));
    }
    Ok(())
}

fn verify_inserted_estimate_matches_fields(
    estimate: &crate::domain::models::Estimate,
    itinerary_id: i64,
    fields: &ParsedAddEstimateFields,
) -> Result<(), String> {
    if estimate.id <= 0 {
        return Err("inserted estimate ID が不正です — DB 更新しません".to_string());
    }
    if estimate.itinerary_id != itinerary_id {
        return Err(format!(
            "inserted estimate の itinerary_id ({}) が期待値 ({itinerary_id}) と一致しません — DB 更新しません",
            estimate.itinerary_id
        ));
    }
    if estimate.amount != fields.amount {
        return Err(format!(
            "inserted estimate の amount ({}) が期待値 ({}) と一致しません — DB 更新しません",
            estimate.amount, fields.amount
        ));
    }
    if estimate.currency != fields.currency {
        return Err(format!(
            "inserted estimate の currency ({}) が期待値 ({}) と一致しません — DB 更新しません",
            estimate.currency, fields.currency
        ));
    }
    if estimate.title != fields.title {
        return Err(
            "inserted estimate の title が期待値と一致しません — DB 更新しません".to_string(),
        );
    }
    if estimate.note != fields.note {
        return Err(
            "inserted estimate の note が期待値と一致しません — DB 更新しません".to_string(),
        );
    }
    if estimate.sort_order != fields.sort_order {
        return Err(format!(
            "inserted estimate の sort_order ({}) が期待値 ({}) と一致しません — DB 更新しません",
            estimate.sort_order, fields.sort_order
        ));
    }
    Ok(())
}

fn execute_confirm_add_estimate(
    conn: &Connection,
    trip_id: i64,
    json: &str,
    report: &FragmentApplyDryRunReport,
) -> Result<i64> {
    let preview = report
        .preview
        .as_ref()
        .context("add_estimate confirm には preview が必要です")?;
    if preview.action != "add_estimate" {
        anyhow::bail!("gate preview の action が add_estimate ではありません — DB 更新しません");
    }
    let estimate_preview = preview
        .estimate_preview
        .as_ref()
        .context("add_estimate confirm には estimate_preview が必要です")?;

    let trip_name = report
        .resolved_target
        .as_ref()
        .map(|target| target.trip_name.clone())
        .or_else(|| {
            crate::trip::get_trip(conn, trip_id)
                .ok()
                .map(|trip| trip.name)
        })
        .context("Trip 名の解決に失敗しました")?;

    let mut estimate_id = 0i64;
    crate::storage::db::with_transaction(conn, "add_estimate confirm", |tx| {
        let root: Value =
            serde_json::from_str(json).with_context(|| "Fragment JSON の parse に失敗しました")?;
        let root_obj = root
            .as_object()
            .context("トップレベルが JSON object ではありません")?;
        let fragment_body = root_obj
            .get("fragment")
            .and_then(Value::as_object)
            .context("fragment object が必要です")?;
        let intent = non_empty_string(fragment_body.get("intent"))
            .ok_or_else(|| anyhow::anyhow!("fragment.intent が必要です"))?;
        if intent != "add_estimate" {
            anyhow::bail!("fragment.intent が add_estimate ではありません — DB 更新しません");
        }

        let fields = parse_add_estimate_fields(fragment_body, None)
            .map_err(|error| anyhow::anyhow!(error))?;

        let target_obj = root_obj
            .get("target")
            .and_then(Value::as_object)
            .context("target object が必要です")?;
        let mut resolve_report = FragmentApplyDryRunReport::new("", trip_id, false, true);
        let resolved =
            resolve_apply_target(tx, trip_id, &trip_name, target_obj, &mut resolve_report)
                .map_err(|_| {
                    anyhow::anyhow!(
                        "target の再解決に失敗しました: {}",
                        resolve_report.errors.join("; ")
                    )
                })?
                .context("target が解決されていません")?;
        if resolved.resolution == "ambiguous" {
            anyhow::bail!("target が曖昧です — DB 更新しません");
        }
        if resolved.target_type != "itinerary" {
            anyhow::bail!(
                "add_estimate confirm は itinerary target のみサポートしています（現在: {}）",
                resolved.target_type
            );
        }

        let itinerary_id = lookup_itinerary_db_id_from_resolved(tx, trip_id, &resolved)
            .map_err(|error| anyhow::anyhow!(error))?;

        verify_estimate_preview_matches_fields(estimate_preview, &fields, itinerary_id)
            .map_err(|error| anyhow::anyhow!(error))?;

        let id = crate::estimate::add_estimate_minor_units(
            tx,
            itinerary_id,
            fields.amount,
            &fields.currency,
            fields.title.as_deref(),
            fields.note.as_deref(),
            Some(fields.sort_order),
        )?;

        let stored = crate::estimate::get_estimate(tx, id)?;
        verify_inserted_estimate_matches_fields(&stored, itinerary_id, &fields)
            .map_err(|error| anyhow::anyhow!(error))?;

        estimate_id = id;
        Ok(())
    })?;

    Ok(estimate_id)
}

fn execute_confirm_update_itinerary(
    conn: &Connection,
    trip_id: i64,
    json: &str,
    report: &FragmentApplyDryRunReport,
) -> Result<i64> {
    let preview = report
        .preview
        .as_ref()
        .context("update_itinerary confirm には preview が必要です")?;
    let changes = preview
        .itinerary_field_changes
        .as_ref()
        .filter(|changes| !changes.is_empty())
        .context("update_itinerary confirm には itinerary_field_changes が必要です")?;

    let root: Value =
        serde_json::from_str(json).with_context(|| "Fragment JSON の parse に失敗しました")?;
    let root_obj = root
        .as_object()
        .context("トップレベルが JSON object ではありません")?;
    let fragment_body = root_obj
        .get("fragment")
        .and_then(Value::as_object)
        .context("fragment object が必要です")?;
    let candidate = fragment_body
        .get("candidate_content")
        .and_then(Value::as_object)
        .context("candidate_content object が必要です")?;
    let target = report
        .resolved_target
        .as_ref()
        .context("target が解決されていません")?;
    let fields = parse_update_itinerary_fields(fragment_body, None)
        .map_err(|error| anyhow::anyhow!(error))?;
    let itinerary_id = resolve_itinerary_id_for_apply_target(conn, trip_id, target)?;

    revalidate_update_itinerary_before_write(conn, itinerary_id, candidate, changes)
        .map_err(|error| anyhow::anyhow!(error))?;

    let note = fields.note.as_ref().map(|patch| {
        if patch.value.is_empty() {
            None
        } else {
            Some(patch.value.as_str())
        }
    });
    let location = fields.location.as_ref().map(|patch| {
        if patch.value.is_empty() {
            None
        } else {
            Some(patch.value.as_str())
        }
    });
    let start_time = fields.start_time.as_ref().map(|patch| {
        if patch.value.is_empty() {
            None
        } else {
            Some(patch.value.as_str())
        }
    });
    let category = fields.category.as_ref().map(|patch| Some(patch.value));

    crate::itinerary::update_itinerary_item(
        conn,
        itinerary_id,
        None,
        fields.title.as_deref(),
        note,
        start_time,
        None,
        fields.duration_minutes,
        fields.travel_minutes,
        location,
        category,
    )?;

    Ok(itinerary_id)
}

fn verify_update_estimate_gate_preview_matches(
    gate_preview: &FragmentApplyEstimateUpdatePreview,
    gate_changes: &[FragmentApplyEstimateFieldChange],
    itinerary_id: i64,
    fields: &ParsedUpdateEstimateFields,
    proposed: &Estimate,
    recomputed_changes: &[FragmentApplyEstimateFieldChange],
) -> Result<(), String> {
    if gate_preview.target_itinerary_id != itinerary_id {
        return Err(format!(
            "gate preview と target itinerary が一致しません（preview: {}, resolved: {itinerary_id}）— DB 更新しません",
            gate_preview.target_itinerary_id
        ));
    }
    if gate_preview.target_estimate_id != fields.estimate_id {
        return Err(format!(
            "gate preview と target estimate が一致しません（preview: {}, parsed: {}）— DB 更新しません",
            gate_preview.target_estimate_id, fields.estimate_id
        ));
    }
    if gate_changes != recomputed_changes {
        return Err(
            "gate preview と estimate_field_changes が一致しません — DB 更新しません".to_string(),
        );
    }
    for change in gate_changes {
        let expected_after = estimate_display_value_for_field(&change.field, proposed);
        if change.after != expected_after {
            return Err(format!(
                "gate preview の after ({}) と proposed {} ({expected_after}) が一致しません — DB 更新しません",
                change.after, change.field
            ));
        }
    }
    Ok(())
}

fn revalidate_update_estimate_before_write(
    conn: &Connection,
    estimate_id: i64,
    itinerary_id: i64,
    candidate: &Map<String, Value>,
    preview_changes: &[FragmentApplyEstimateFieldChange],
) -> Result<(), String> {
    let current = crate::estimate::get_estimate(conn, estimate_id)
        .map_err(|error| format!("Estimate not found: {error}"))?;
    if current.itinerary_id != itinerary_id {
        return Err(format!(
            "Estimate {estimate_id} は Itinerary {itinerary_id} 配下ではありません"
        ));
    }

    detect_update_estimate_baseline_conflicts(candidate, &current)?;

    for change in preview_changes {
        let actual_before = estimate_display_value_for_field(&change.field, &current);
        if actual_before != change.before {
            return Err(format!(
                "TOCTOU mismatch: estimate_field_changes.{} の before ({}) が現行 DB ({actual_before}) と一致しません — DB 更新しません",
                change.field, change.before
            ));
        }
    }
    Ok(())
}

fn verify_updated_estimate_matches_proposed(
    stored: &Estimate,
    itinerary_id: i64,
    proposed: &Estimate,
) -> Result<(), String> {
    if stored.id != proposed.id {
        return Err(format!(
            "updated estimate ID ({}) が期待値 ({}) と一致しません — DB 更新しません",
            stored.id, proposed.id
        ));
    }
    if stored.itinerary_id != itinerary_id {
        return Err(format!(
            "updated estimate の itinerary_id ({}) が期待値 ({itinerary_id}) と一致しません — DB 更新しません",
            stored.itinerary_id
        ));
    }
    if stored.amount != proposed.amount {
        return Err(format!(
            "updated estimate の amount ({}) が期待値 ({}) と一致しません — DB 更新しません",
            stored.amount, proposed.amount
        ));
    }
    if stored.currency != proposed.currency {
        return Err(format!(
            "updated estimate の currency ({}) が期待値 ({}) と一致しません — DB 更新しません",
            stored.currency, proposed.currency
        ));
    }
    if stored.title != proposed.title {
        return Err(
            "updated estimate の title が期待値と一致しません — DB 更新しません".to_string(),
        );
    }
    if stored.note != proposed.note {
        return Err(
            "updated estimate の note が期待値と一致しません — DB 更新しません".to_string(),
        );
    }
    if stored.sort_order != proposed.sort_order {
        return Err(format!(
            "updated estimate の sort_order ({}) が期待値 ({}) と一致しません — DB 更新しません",
            stored.sort_order, proposed.sort_order
        ));
    }
    Ok(())
}

fn execute_confirm_update_estimate(
    conn: &Connection,
    trip_id: i64,
    json: &str,
    report: &FragmentApplyDryRunReport,
) -> Result<i64> {
    let preview = report
        .preview
        .as_ref()
        .context("update_estimate confirm には preview が必要です")?;
    if preview.action != "update_estimate" {
        anyhow::bail!("gate preview の action が update_estimate ではありません — DB 更新しません");
    }
    let gate_preview = preview
        .estimate_update_preview
        .as_ref()
        .context("update_estimate confirm には estimate_update_preview が必要です")?;
    let gate_changes = preview
        .estimate_field_changes
        .as_ref()
        .filter(|changes| !changes.is_empty())
        .context("update_estimate confirm には estimate_field_changes が必要です")?;

    let trip_name = report
        .resolved_target
        .as_ref()
        .map(|target| target.trip_name.clone())
        .or_else(|| {
            crate::trip::get_trip(conn, trip_id)
                .ok()
                .map(|trip| trip.name)
        })
        .context("Trip 名の解決に失敗しました")?;

    let mut updated_estimate_id = 0i64;
    crate::storage::db::with_transaction(conn, "update_estimate confirm", |tx| {
        let root: Value =
            serde_json::from_str(json).with_context(|| "Fragment JSON の parse に失敗しました")?;
        let root_obj = root
            .as_object()
            .context("トップレベルが JSON object ではありません")?;
        let fragment_body = root_obj
            .get("fragment")
            .and_then(Value::as_object)
            .context("fragment object が必要です")?;
        let candidate = fragment_body
            .get("candidate_content")
            .and_then(Value::as_object)
            .context("candidate_content object が必要です")?;
        let intent = non_empty_string(fragment_body.get("intent"))
            .ok_or_else(|| anyhow::anyhow!("fragment.intent が必要です"))?;
        if intent != "update_estimate" {
            anyhow::bail!("fragment.intent が update_estimate ではありません — DB 更新しません");
        }

        let fields = parse_update_estimate_fields(fragment_body, None)
            .map_err(|error| anyhow::anyhow!(error))?;

        let target_obj = root_obj
            .get("target")
            .and_then(Value::as_object)
            .context("target object が必要です")?;
        let mut resolve_report = FragmentApplyDryRunReport::new("", trip_id, false, true);
        let resolved =
            resolve_apply_target(tx, trip_id, &trip_name, target_obj, &mut resolve_report)
                .map_err(|_| {
                    anyhow::anyhow!(
                        "target の再解決に失敗しました: {}",
                        resolve_report.errors.join("; ")
                    )
                })?
                .context("target が解決されていません")?;
        if resolved.resolution == "ambiguous" {
            anyhow::bail!("target が曖昧です — DB 更新しません");
        }
        if resolved.target_type != "itinerary" {
            anyhow::bail!(
                "update_estimate confirm は itinerary target のみサポートしています（現在: {}）",
                resolved.target_type
            );
        }

        let itinerary_id = lookup_itinerary_db_id_from_resolved(tx, trip_id, &resolved)
            .map_err(|error| anyhow::anyhow!(error))?;

        let current = crate::estimate::get_estimate(tx, fields.estimate_id)
            .map_err(|error| anyhow::anyhow!("Estimate not found: {error}"))?;
        if current.itinerary_id != itinerary_id {
            anyhow::bail!(
                "Estimate {} は Itinerary {} 配下ではありません",
                fields.estimate_id,
                itinerary_id
            );
        }

        let proposed = compute_update_estimate_proposed(&current, &fields)
            .map_err(|error| anyhow::anyhow!(error))?;
        let recomputed_changes = build_update_estimate_field_changes(&current, &proposed, &fields);

        verify_update_estimate_gate_preview_matches(
            gate_preview,
            gate_changes,
            itinerary_id,
            &fields,
            &proposed,
            &recomputed_changes,
        )
        .map_err(|error| anyhow::anyhow!(error))?;

        revalidate_update_estimate_before_write(
            tx,
            fields.estimate_id,
            itinerary_id,
            candidate,
            gate_changes,
        )
        .map_err(|error| anyhow::anyhow!(error))?;

        let now = crate::storage::db::now_string();
        crate::estimate::update_estimate_row_scoped(
            tx,
            fields.estimate_id,
            itinerary_id,
            proposed.title.as_deref(),
            proposed.amount,
            &proposed.currency,
            proposed.note.as_deref(),
            proposed.sort_order,
            &now,
        )?;

        let stored = crate::estimate::get_estimate(tx, fields.estimate_id)?;
        verify_updated_estimate_matches_proposed(&stored, itinerary_id, &proposed)
            .map_err(|error| anyhow::anyhow!(error))?;

        updated_estimate_id = fields.estimate_id;
        Ok(())
    })?;

    Ok(updated_estimate_id)
}

fn estimate_delete_snapshot_agreement_fields_match(
    gate: &FragmentApplyEstimateDeletePreview,
    other: &FragmentApplyEstimateDeletePreview,
) -> Result<(), String> {
    if gate.target_itinerary_id != other.target_itinerary_id {
        return Err(format!(
            "gate preview と recomputed target_itinerary_id が一致しません（preview: {}, recomputed: {}）— DB 更新しません",
            gate.target_itinerary_id, other.target_itinerary_id
        ));
    }
    if gate.target_estimate_id != other.target_estimate_id {
        return Err(format!(
            "gate preview と recomputed target_estimate_id が一致しません（preview: {}, recomputed: {}）— DB 更新しません",
            gate.target_estimate_id, other.target_estimate_id
        ));
    }
    if gate.title != other.title {
        return Err(
            "gate preview と recomputed title が一致しません — DB 更新しません".to_string(),
        );
    }
    if gate.amount != other.amount {
        return Err(format!(
            "gate preview と recomputed amount が一致しません（preview: {}, recomputed: {}）— DB 更新しません",
            gate.amount, other.amount
        ));
    }
    if gate.currency != other.currency {
        return Err(format!(
            "gate preview と recomputed currency が一致しません（preview: {}, recomputed: {}）— DB 更新しません",
            gate.currency, other.currency
        ));
    }
    if gate.note != other.note {
        return Err("gate preview と recomputed note が一致しません — DB 更新しません".to_string());
    }
    if gate.sort_order != other.sort_order {
        return Err(format!(
            "gate preview と recomputed sort_order が一致しません（preview: {}, recomputed: {}）— DB 更新しません",
            gate.sort_order, other.sort_order
        ));
    }
    if gate.updated_at != other.updated_at {
        return Err(format!(
            "gate preview と recomputed updated_at が一致しません（preview: {}, recomputed: {}）— DB 更新しません",
            gate.updated_at, other.updated_at
        ));
    }
    Ok(())
}

fn verify_delete_estimate_gate_preview_matches(
    gate: &FragmentApplyEstimateDeletePreview,
    itinerary_id: i64,
    estimate_id: i64,
    recomputed: &FragmentApplyEstimateDeletePreview,
) -> Result<(), String> {
    if gate.target_itinerary_id != itinerary_id {
        return Err(format!(
            "gate preview と target itinerary が一致しません（preview: {}, resolved: {itinerary_id}）— DB 更新しません",
            gate.target_itinerary_id
        ));
    }
    if gate.target_estimate_id != estimate_id {
        return Err(format!(
            "gate preview と target estimate が一致しません（preview: {}, parsed: {estimate_id}）— DB 更新しません",
            gate.target_estimate_id
        ));
    }
    estimate_delete_snapshot_agreement_fields_match(gate, recomputed)
}

fn revalidate_delete_estimate_before_write(
    conn: &Connection,
    estimate_id: i64,
    itinerary_id: i64,
    candidate: &Map<String, Value>,
    gate: &FragmentApplyEstimateDeletePreview,
) -> Result<(), String> {
    let current = crate::estimate::get_estimate(conn, estimate_id)
        .map_err(|error| format!("Estimate not found: {error}"))?;
    if current.itinerary_id != itinerary_id {
        return Err(format!(
            "Estimate {estimate_id} は Itinerary {itinerary_id} 配下ではありません"
        ));
    }

    detect_delete_estimate_baseline_conflicts(candidate, &current)?;

    if gate.target_estimate_id != current.id {
        return Err(format!(
            "TOCTOU mismatch: target_estimate_id の before ({}) が現行 DB ({}) と一致しません — DB 更新しません",
            gate.target_estimate_id, current.id
        ));
    }
    if gate.title != current.title {
        return Err(format!(
            "TOCTOU mismatch: title の before ({:?}) が現行 DB ({:?}) と一致しません — DB 更新しません",
            gate.title, current.title
        ));
    }
    if gate.amount != current.amount {
        return Err(format!(
            "TOCTOU mismatch: amount の before ({}) が現行 DB ({}) と一致しません — DB 更新しません",
            gate.amount, current.amount
        ));
    }
    if gate.currency != current.currency {
        return Err(format!(
            "TOCTOU mismatch: currency の before ({}) が現行 DB ({}) と一致しません — DB 更新しません",
            gate.currency, current.currency
        ));
    }
    if gate.note != current.note {
        return Err(format!(
            "TOCTOU mismatch: note の before ({:?}) が現行 DB ({:?}) と一致しません — DB 更新しません",
            gate.note, current.note
        ));
    }
    if gate.sort_order != current.sort_order {
        return Err(format!(
            "TOCTOU mismatch: sort_order の before ({}) が現行 DB ({}) と一致しません — DB 更新しません",
            gate.sort_order, current.sort_order
        ));
    }
    if gate.updated_at != current.updated_at {
        return Err(format!(
            "TOCTOU mismatch: updated_at の before ({}) が現行 DB ({}) と一致しません — DB 更新しません",
            gate.updated_at, current.updated_at
        ));
    }
    Ok(())
}

fn verify_estimate_deleted(conn: &Connection, estimate_id: i64) -> Result<(), String> {
    match crate::estimate::get_estimate(conn, estimate_id) {
        Err(_) => Ok(()),
        Ok(_) => Err(format!(
            "Estimate {estimate_id} が削除後も存在します — DB 更新しません"
        )),
    }
}

fn execute_confirm_delete_estimate(
    conn: &Connection,
    trip_id: i64,
    json: &str,
    report: &FragmentApplyDryRunReport,
) -> Result<i64> {
    let preview = report
        .preview
        .as_ref()
        .context("delete_estimate confirm には preview が必要です")?;
    if preview.action != "delete_estimate" {
        anyhow::bail!("gate preview の action が delete_estimate ではありません — DB 更新しません");
    }
    let gate_preview = preview
        .estimate_delete_preview
        .as_ref()
        .context("delete_estimate confirm には estimate_delete_preview が必要です")?;

    let trip_name = report
        .resolved_target
        .as_ref()
        .map(|target| target.trip_name.clone())
        .or_else(|| {
            crate::trip::get_trip(conn, trip_id)
                .ok()
                .map(|trip| trip.name)
        })
        .context("Trip 名の解決に失敗しました")?;

    let mut deleted_estimate_id = 0i64;
    crate::storage::db::with_transaction(conn, "delete_estimate confirm", |tx| {
        let root: Value =
            serde_json::from_str(json).with_context(|| "Fragment JSON の parse に失敗しました")?;
        let root_obj = root
            .as_object()
            .context("トップレベルが JSON object ではありません")?;
        let fragment_body = root_obj
            .get("fragment")
            .and_then(Value::as_object)
            .context("fragment object が必要です")?;
        let candidate = fragment_body
            .get("candidate_content")
            .and_then(Value::as_object)
            .context("candidate_content object が必要です")?;
        let intent = non_empty_string(fragment_body.get("intent"))
            .ok_or_else(|| anyhow::anyhow!("fragment.intent が必要です"))?;
        if intent != "delete_estimate" {
            anyhow::bail!("fragment.intent が delete_estimate ではありません — DB 更新しません");
        }

        if !report.required_decisions.is_empty() {
            anyhow::bail!("required decisions が未解決です — DB 更新しません");
        }

        let estimate_id = parse_delete_estimate_fields(fragment_body, None)
            .map_err(|error| anyhow::anyhow!(error))?;

        let target_obj = root_obj
            .get("target")
            .and_then(Value::as_object)
            .context("target object が必要です")?;
        let mut resolve_report = FragmentApplyDryRunReport::new("", trip_id, false, true);
        let resolved =
            resolve_apply_target(tx, trip_id, &trip_name, target_obj, &mut resolve_report)
                .map_err(|_| {
                    anyhow::anyhow!(
                        "target の再解決に失敗しました: {}",
                        resolve_report.errors.join("; ")
                    )
                })?
                .context("target が解決されていません")?;
        if resolved.resolution == "ambiguous" {
            anyhow::bail!("target が曖昧です — DB 更新しません");
        }
        if resolved.target_type != "itinerary" {
            anyhow::bail!(
                "delete_estimate confirm は itinerary target のみサポートしています（現在: {}）",
                resolved.target_type
            );
        }

        let itinerary_id = lookup_itinerary_db_id_from_resolved(tx, trip_id, &resolved)
            .map_err(|error| anyhow::anyhow!(error))?;

        let current = crate::estimate::get_estimate(tx, estimate_id)
            .map_err(|error| anyhow::anyhow!("Estimate not found: {error}"))?;
        if current.itinerary_id != itinerary_id {
            anyhow::bail!(
                "Estimate {} は Itinerary {} 配下ではありません",
                estimate_id,
                itinerary_id
            );
        }

        let itinerary_title = resolved.itinerary_title.clone().unwrap_or_default();
        let recomputed = build_estimate_delete_preview(itinerary_id, itinerary_title, &current);

        verify_delete_estimate_gate_preview_matches(
            gate_preview,
            itinerary_id,
            estimate_id,
            &recomputed,
        )
        .map_err(|error| anyhow::anyhow!(error))?;

        revalidate_delete_estimate_before_write(
            tx,
            estimate_id,
            itinerary_id,
            candidate,
            gate_preview,
        )
        .map_err(|error| anyhow::anyhow!(error))?;

        crate::estimate::delete_estimate_row_scoped(tx, estimate_id, itinerary_id)?;

        verify_estimate_deleted(tx, estimate_id).map_err(|error| anyhow::anyhow!(error))?;

        deleted_estimate_id = estimate_id;
        Ok(())
    })?;

    Ok(deleted_estimate_id)
}

fn count_blocking_children_for_itinerary_db(
    conn: &Connection,
    itinerary_id: i64,
) -> Result<FragmentApplyBlockingChildren> {
    use crate::domain::models::NoteOwnerType;

    Ok(FragmentApplyBlockingChildren {
        expenses: crate::expense::list_expenses_for_itinerary(conn, itinerary_id)?.len(),
        estimates: crate::estimate::list_estimates_for_itinerary(conn, itinerary_id)?.len(),
        reservations: crate::reservation::list_reservations_for_itinerary(conn, itinerary_id)?
            .len(),
        notes: crate::note::list_notes_for_owner(conn, NoteOwnerType::Itinerary, itinerary_id)?
            .len(),
    })
}

fn revalidate_delete_itinerary_before_write(
    conn: &Connection,
    trip_id: i64,
    expected: &FragmentApplyDeletePreview,
) -> Result<(), String> {
    let item = crate::itinerary::get_itinerary_item(conn, expected.itinerary_id)
        .map_err(|error| error.to_string())?;

    if item.trip_id != trip_id {
        return Err(
            "TOCTOU mismatch: itinerary の trip_id が gate と一致しません — DB 更新しません"
                .to_string(),
        );
    }
    if item.day != expected.day_number {
        return Err(format!(
            "TOCTOU mismatch: day_number の before ({}) が現行 DB ({}) と一致しません — DB 更新しません",
            expected.day_number, item.day
        ));
    }
    if item.sort_order != expected.sort_order {
        return Err(format!(
            "TOCTOU mismatch: sort_order の before ({}) が現行 DB ({}) と一致しません — DB 更新しません",
            expected.sort_order, item.sort_order
        ));
    }
    if item.title != expected.title {
        return Err(format!(
            "TOCTOU mismatch: title の before ({}) が現行 DB ({}) と一致しません — DB 更新しません",
            expected.title, item.title
        ));
    }

    let blocking_children = count_blocking_children_for_itinerary_db(conn, expected.itinerary_id)
        .map_err(|error| error.to_string())?;
    if blocking_children_total(&blocking_children) > 0 {
        return Err(format!(
            "TOCTOU mismatch: blocking child が存在します（expenses: {}, estimates: {}, reservations: {}, notes: {}）— DB 更新しません",
            blocking_children.expenses,
            blocking_children.estimates,
            blocking_children.reservations,
            blocking_children.notes,
        ));
    }

    if blocking_children != expected.blocking_children {
        return Err(
            "TOCTOU mismatch: blocking_children の件数が gate preview と一致しません — DB 更新しません"
                .to_string(),
        );
    }

    Ok(())
}

fn execute_confirm_delete_itinerary(
    conn: &Connection,
    trip_id: i64,
    report: &FragmentApplyDryRunReport,
) -> Result<i64> {
    let preview = report
        .preview
        .as_ref()
        .context("delete_itinerary confirm には preview が必要です")?;
    let delete_preview = preview
        .delete_preview
        .as_ref()
        .context("delete_itinerary confirm には delete_preview が必要です")?;
    let target = report
        .resolved_target
        .as_ref()
        .context("target が解決されていません")?;

    let itinerary_id = resolve_itinerary_id_for_apply_target(conn, trip_id, target)?;
    if itinerary_id != delete_preview.itinerary_id {
        anyhow::bail!(
            "TOCTOU mismatch: resolved itinerary_id ({itinerary_id}) が delete_preview.itinerary_id ({}) と一致しません — DB 更新しません",
            delete_preview.itinerary_id
        );
    }

    revalidate_delete_itinerary_before_write(conn, trip_id, delete_preview)
        .map_err(|error| anyhow::anyhow!(error))?;

    crate::itinerary::delete_itinerary_item_row_only(conn, itinerary_id)?;

    Ok(itinerary_id)
}

fn execute_confirm_reorder_itinerary(
    conn: &Connection,
    trip_id: i64,
    json: &str,
    report: &FragmentApplyDryRunReport,
) -> Result<usize> {
    let preview = report
        .preview
        .as_ref()
        .context("reorder_itinerary confirm には preview が必要です")?;
    let reorder_preview = preview
        .reorder_preview
        .as_ref()
        .context("reorder_itinerary confirm には reorder_preview が必要です")?;
    let target = report
        .resolved_target
        .as_ref()
        .context("target が解決されていません")?;
    if target.target_type != "day" {
        anyhow::bail!(
            "reorder_itinerary confirm は day target のみサポートしています（現在: {}）",
            target.target_type
        );
    }
    let day_number = target
        .day_number
        .context("reorder_itinerary の Day target が解決されていません")?;

    let root: Value =
        serde_json::from_str(json).with_context(|| "Fragment JSON の parse に失敗しました")?;
    let root_obj = root
        .as_object()
        .context("トップレベルが JSON object ではありません")?;
    let fragment_body = root_obj
        .get("fragment")
        .and_then(Value::as_object)
        .context("fragment object が必要です")?;
    let candidate = fragment_body
        .get("candidate_content")
        .and_then(Value::as_object)
        .context("candidate_content object が必要です")?;

    let expected = candidate
        .get("expected_order")
        .context("candidate_content.expected_order が必要です")?;
    let after = candidate
        .get("after_order")
        .context("candidate_content.after_order が必要です")?;
    let expected_refs =
        parse_reorder_order_refs(expected, "expected_order").map_err(|e| anyhow::anyhow!(e))?;
    let after_refs =
        parse_reorder_order_refs(after, "after_order").map_err(|e| anyhow::anyhow!(e))?;

    if expected_refs.is_empty() {
        anyhow::bail!("candidate_content.expected_order は空にできません");
    }
    if after_refs.is_empty() {
        anyhow::bail!("candidate_content.after_order は空にできません");
    }

    let mut updated_rows: usize = 0;
    crate::storage::db::with_transaction(conn, "reorder_itinerary confirm", |tx| {
        // TOCTOU: 現行 DB を読み直して再解決・再検証
        let day_items = crate::itinerary::list_itinerary_items_for_day(tx, trip_id, day_number)
            .context("Day itinerary の取得に失敗しました")?;
        if day_items.is_empty() {
            anyhow::bail!("reorder_itinerary: Day {day_number} に itinerary がありません");
        }

        let trip = crate::trip::get_trip(tx, trip_id)?;
        let start = trip
            .start_date
            .as_deref()
            .context("trip.start_date が必要です")?;
        let end = trip
            .end_date
            .as_deref()
            .context("trip.end_date が必要です")?;
        let day_count =
            validate_trip_date_range(start, end).context("trip date range の検証に失敗しました")?;

        let expected_resolved = resolve_reorder_order_in_day(
            tx,
            trip_id,
            day_number,
            day_count,
            &day_items,
            &expected_refs,
            "expected_order",
        )
        .map_err(|e| anyhow::anyhow!(e))?;
        let after_resolved = resolve_reorder_order_in_day(
            tx,
            trip_id,
            day_number,
            day_count,
            &day_items,
            &after_refs,
            "after_order",
        )
        .map_err(|e| anyhow::anyhow!(e))?;

        let current_ids: Vec<i64> = day_items.iter().map(|i| i.id).collect();
        if expected_resolved.iter().map(|r| r.id).collect::<Vec<_>>() != current_ids {
            anyhow::bail!(
                "TOCTOU mismatch: expected_order が現行 Day の順序と一致しません（baseline mismatch）— DB 更新しません"
            );
        }

        let expected_set = expected_resolved
            .iter()
            .map(|r| r.id)
            .collect::<std::collections::HashSet<_>>();
        let after_set = after_resolved
            .iter()
            .map(|r| r.id)
            .collect::<std::collections::HashSet<_>>();
        if expected_set != after_set {
            anyhow::bail!(
                "reorder_itinerary: after_order は expected_order と同じ itinerary 集合を含む必要があります"
            );
        }
        let expected_ids = expected_resolved.iter().map(|r| r.id).collect::<Vec<_>>();
        let after_ids = after_resolved.iter().map(|r| r.id).collect::<Vec<_>>();
        if expected_ids == after_ids {
            anyhow::bail!("reorder_itinerary: no-op reorder は許可されません");
        }

        // dry-run と同じ: sparse sort_order slot を after_order に割り当てる
        let mut slots: Vec<i64> = day_items.iter().map(|i| i.sort_order).collect();
        slots.sort();
        if slots.len() != after_ids.len() {
            anyhow::bail!("reorder_itinerary: internal mismatch（slot length）");
        }
        let mut id_to_after_sort: std::collections::HashMap<i64, i64> =
            std::collections::HashMap::new();
        for (idx, id) in after_ids.iter().enumerate() {
            id_to_after_sort.insert(*id, slots[idx]);
        }

        // preview と一致確認（TOCTOU guard）
        for change in &reorder_preview.itinerary_order_changes {
            let Some(expected_after) = id_to_after_sort.get(&change.itinerary_id) else {
                anyhow::bail!(
                    "TOCTOU mismatch: itinerary_id {} が after_order に存在しません — DB 更新しません",
                    change.itinerary_id
                );
            };
            if *expected_after != change.after_sort_order {
                anyhow::bail!(
                    "TOCTOU mismatch: after_sort_order が gate preview と一致しません（itinerary_id {}）— DB 更新しません",
                    change.itinerary_id
                );
            }
        }

        let now = crate::storage::db::now_string();
        let mut expected_updates: Vec<(i64, i64)> = Vec::new();
        for item in &day_items {
            let Some(new_sort) = id_to_after_sort.get(&item.id).copied() else {
                continue;
            };
            if new_sort != item.sort_order {
                expected_updates.push((item.id, new_sort));
            }
        }
        if expected_updates.is_empty() {
            anyhow::bail!("reorder_itinerary: no-op reorder は許可されません");
        }

        let mut actual_updated = 0usize;
        for (id, new_sort) in &expected_updates {
            let changed = tx
                .execute(
                    "UPDATE itinerary_items SET sort_order = ?1, updated_at = ?2 WHERE id = ?3 AND trip_id = ?4 AND day = ?5",
                    rusqlite::params![new_sort, now, id, trip_id, day_number],
                )
                .context("itinerary_items.sort_order の更新に失敗しました")?;
            if changed != 1 {
                anyhow::bail!(
                    "row count mismatch: itinerary_items UPDATE が 1 行ではありません（{changed}）— DB 更新しません"
                );
            }
            actual_updated += 1;
        }

        if actual_updated != expected_updates.len() {
            anyhow::bail!(
                "row count mismatch: updated rows ({actual_updated}) が想定 ({}) と一致しません — DB 更新しません",
                expected_updates.len()
            );
        }

        updated_rows = actual_updated;
        Ok(())
    })?;

    Ok(updated_rows)
}

fn itinerary_item_to_export_snapshot(
    item: &crate::domain::models::ItineraryItem,
) -> ExportItineraryV3 {
    ExportItineraryV3 {
        title: item.title.clone(),
        note: item.note.clone(),
        start_time: item.start_time.clone(),
        sort_order: item.sort_order,
        duration_minutes: item.duration_minutes,
        travel_minutes: item.travel_minutes,
        location: item.location.clone(),
        category: item.category,
        expenses: Vec::new(),
        estimates: Vec::new(),
        reservations: Vec::new(),
    }
}

fn revalidate_update_itinerary_before_write(
    conn: &Connection,
    itinerary_id: i64,
    candidate: &Map<String, Value>,
    preview_changes: &[FragmentApplyItineraryFieldChange],
) -> Result<(), String> {
    let item = crate::itinerary::get_itinerary_item(conn, itinerary_id)
        .map_err(|error| error.to_string())?;
    let current = itinerary_item_to_export_snapshot(&item);

    detect_update_itinerary_baseline_conflicts(candidate, &current)?;

    for change in preview_changes {
        let actual_before = export_itinerary_field_display(&current, &change.field);
        if actual_before != change.before {
            return Err(format!(
                "TOCTOU mismatch: itinerary_field_changes.{} の before ({}) が現行 DB ({actual_before}) と一致しません — DB 更新しません",
                change.field, change.before
            ));
        }
    }
    Ok(())
}

fn export_itinerary_field_display(current: &ExportItineraryV3, field: &str) -> String {
    match field {
        "title" => current.title.clone(),
        "note" => fmt_diff_option_str(&current.note),
        "location" => fmt_diff_option_str(&current.location),
        "category" => fmt_diff_option_category(current.category),
        "start_time" => fmt_diff_option_str(&current.start_time),
        "duration_minutes" => fmt_diff_option_i64(current.duration_minutes),
        "travel_minutes" => fmt_diff_option_i64(current.travel_minutes),
        other => other.to_string(),
    }
}

fn resolve_itinerary_id_for_apply_target(
    conn: &Connection,
    trip_id: i64,
    target: &FragmentApplyResolvedTarget,
) -> Result<i64> {
    if target.target_type != "itinerary" {
        anyhow::bail!(
            "itinerary target の apply のみサポートしています（現在: {}）",
            target.target_type
        );
    }
    let day_number = target
        .day_number
        .context("Itinerary target が解決されていません（day）")?;
    let sort_order = target
        .itinerary_sort_order
        .context("Itinerary target が解決されていません（sort_order）")?;
    let item = crate::itinerary::list_itinerary_items_for_day(conn, trip_id, day_number)?
        .into_iter()
        .find(|item| item.sort_order == sort_order)
        .with_context(|| {
            format!(
                "itinerary_reference (sort_order {sort_order}) が Day {day_number} に見つかりません"
            )
        })?;
    Ok(item.id)
}

fn resolve_note_owner_for_apply_target(
    conn: &Connection,
    trip_id: i64,
    target: &FragmentApplyResolvedTarget,
) -> Result<crate::note::ResolvedNoteOwner> {
    use crate::note::ResolvedNoteOwner;

    match target.target_type.as_str() {
        "trip" => Ok(ResolvedNoteOwner::Trip(trip_id)),
        "day" => {
            let day_number = target
                .day_number
                .context("add_note の Day target が解決されていません")?;
            let day_id = crate::day::find_day_id_by_trip_and_day_number(conn, trip_id, day_number)?;
            Ok(ResolvedNoteOwner::Day(day_id))
        }
        "itinerary" => {
            let day_number = target
                .day_number
                .context("add_note の Itinerary target が解決されていません（day）")?;
            let sort_order = target
                .itinerary_sort_order
                .context("add_note の Itinerary target が解決されていません（sort_order）")?;
            let item = crate::itinerary::list_itinerary_items_for_day(conn, trip_id, day_number)?
                .into_iter()
                .find(|item| item.sort_order == sort_order)
                .with_context(|| {
                    format!(
                        "itinerary_reference (sort_order {sort_order}) が Day {day_number} に見つかりません"
                    )
                })?;
            Ok(ResolvedNoteOwner::Itinerary(item.id))
        }
        other => anyhow::bail!("未対応の target_type です: {other}"),
    }
}

fn parse_add_itinerary_fields(
    fragment: &Map<String, Value>,
    report: Option<&mut FragmentApplyDryRunReport>,
) -> Result<ParsedAddItineraryFields, String> {
    let candidate = fragment
        .get("candidate_content")
        .and_then(Value::as_object)
        .ok_or_else(|| "candidate_content object が必要です".to_string())?;
    let title = non_empty_string(candidate.get("title"))
        .ok_or_else(|| "candidate_content.title が必要です".to_string())?;

    if let Some(report) = report {
        collect_add_ordering_hint_warnings(candidate, fragment, report);
        warn_unsupported_add_candidate_keys(candidate, report);
    }

    let category = match non_empty_string(candidate.get("category")) {
        None => None,
        Some(text) => Some(
            parse_itinerary_category(&text)
                .map_err(|error| format!("candidate_content.category が不正です: {error}"))?,
        ),
    };

    let start_time = match non_empty_string(candidate.get("start_time")) {
        None => None,
        Some(text) => {
            parse_time_hhmm(&text).map_err(|error| error.to_string())?;
            Some(text)
        }
    };

    let duration_minutes =
        parse_optional_non_negative_i64(candidate.get("duration_minutes"), "duration_minutes")?;
    let travel_minutes = parse_travel_minutes_field(candidate)?;

    Ok(ParsedAddItineraryFields {
        title,
        note: non_empty_string(fragment.get("notes")),
        location: non_empty_string(candidate.get("location")),
        category,
        start_time,
        duration_minutes,
        travel_minutes,
    })
}

fn parse_travel_minutes_field(candidate: &Map<String, Value>) -> Result<Option<i64>, String> {
    let travel_minutes =
        parse_optional_non_negative_i64(candidate.get("travel_minutes"), "travel_minutes")?;
    let travel_time_minutes = parse_optional_non_negative_i64(
        candidate.get("travel_time_minutes"),
        "travel_time_minutes",
    )?;
    match (travel_minutes, travel_time_minutes) {
        (Some(left), Some(right)) if left != right => {
            Err("travel_minutes と travel_time_minutes が矛盾しています".to_string())
        }
        (Some(value), _) | (_, Some(value)) => Ok(Some(value)),
        (None, None) => Ok(None),
    }
}

fn parse_optional_non_negative_i64(
    value: Option<&Value>,
    field: &str,
) -> Result<Option<i64>, String> {
    let Some(value) = value else {
        return Ok(None);
    };
    let number = value
        .as_i64()
        .ok_or_else(|| format!("{field} は整数である必要があります"))?;
    if number < 0 {
        return Err(format!("{field} は 0 以上である必要があります"));
    }
    Ok(Some(number))
}

fn collect_add_ordering_hint_warnings(
    candidate: &Map<String, Value>,
    fragment: &Map<String, Value>,
    report: &mut FragmentApplyDryRunReport,
) {
    if candidate.get("sort_order").is_some() {
        push_unique(
            &mut report.warnings,
            "ordering_hint: candidate_content.sort_order は v4.7.21 confirm では無視され、Day 末尾に append します".to_string(),
        );
    }
    if non_empty_string(candidate.get("placement_hint")).is_some() {
        push_unique(
            &mut report.warnings,
            "ordering_hint: placement_hint は preview のみ — confirm は Day 末尾 append（--after / reorder は未実装）".to_string(),
        );
    }
    if candidate.get("after").is_some() || candidate.get("before").is_some() {
        push_unique(
            &mut report.warnings,
            "ordering_hint: after / before は v4.7.21 confirm では未対応です — Day 末尾に append します".to_string(),
        );
    }
    if fragment
        .get("placement_hints")
        .is_some_and(|value| !value.is_null())
    {
        push_unique(
            &mut report.warnings,
            "ordering_hint: placement_hints は v4.7.21 confirm では未反映です — Day 末尾に append します".to_string(),
        );
    }
}

fn warn_unsupported_add_candidate_keys(
    candidate: &Map<String, Value>,
    report: &mut FragmentApplyDryRunReport,
) {
    const SUPPORTED: &[&str] = &[
        "title",
        "location",
        "category",
        "start_time",
        "duration_minutes",
        "travel_minutes",
        "travel_time_minutes",
        "sort_order",
        "placement_hint",
        "after",
        "before",
    ];
    for key in candidate.keys() {
        if SUPPORTED.contains(&key.as_str()) {
            continue;
        }
        push_unique(
            &mut report.warnings,
            format!("unsupported_field: candidate_content.{key} は add_itinerary では未反映です"),
        );
    }
}

fn append_sort_order_for_export_day(day: &ExportDayV3) -> i64 {
    day.itineraries
        .iter()
        .map(|item| item.sort_order)
        .max()
        .unwrap_or(0)
        + SORT_ORDER_STEP
}

fn build_export_itinerary_from_add_fields(
    fields: &ParsedAddItineraryFields,
    sort_order: i64,
) -> ExportItineraryV3 {
    ExportItineraryV3 {
        title: fields.title.clone(),
        note: fields.note.clone(),
        start_time: fields.start_time.clone(),
        sort_order,
        duration_minutes: fields.duration_minutes,
        travel_minutes: fields.travel_minutes,
        location: fields.location.clone(),
        category: fields.category,
        expenses: Vec::new(),
        estimates: Vec::new(),
        reservations: Vec::new(),
    }
}

fn non_empty_string(value: Option<&Value>) -> Option<String> {
    value
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
}

fn string_array(value: Option<&Value>) -> Vec<String> {
    value
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(|item| non_empty_string(Some(item)))
                .collect()
        })
        .unwrap_or_default()
}

fn collect_required_decisions(
    hints: Option<&Value>,
    target: &Map<String, Value>,
    required_decisions: &mut Vec<String>,
) {
    if let Some(items) = hints
        .and_then(Value::as_object)
        .and_then(|obj| obj.get("required_decisions"))
    {
        for item in string_array(Some(items)) {
            push_unique(required_decisions, item);
        }
    }

    if let Some(items) = hints
        .and_then(Value::as_object)
        .and_then(|obj| obj.get("conflicts"))
    {
        for item in string_array(Some(items)) {
            push_unique(required_decisions, item);
        }
    }

    if non_empty_string(target.get("target_type")).as_deref() == Some("unresolved") {
        push_unique(
            required_decisions,
            "target Day / Itinerary の確定".to_string(),
        );
        for hint in string_array(target.get("unresolved_target_hints")) {
            push_unique(required_decisions, hint);
        }
    }
}

fn push_unique(items: &mut Vec<String>, item: String) {
    if !items.iter().any(|existing| existing == &item) {
        items.push(item);
    }
}

fn resolve_apply_target(
    conn: &Connection,
    trip_id: i64,
    trip_name: &str,
    target: &Map<String, Value>,
    report: &mut FragmentApplyDryRunReport,
) -> std::result::Result<Option<ResolvedApplyTarget>, ()> {
    let target_type = match non_empty_string(target.get("target_type")) {
        Some(kind) => kind,
        None => {
            report
                .errors
                .push("target.target_type が必要です".to_string());
            return Err(());
        }
    };

    if target_type == "unresolved" {
        report.errors.push(
            "target が unresolved です — apply preview には Day / Itinerary の解決が必要です"
                .to_string(),
        );
        return Err(());
    }

    let mut resolved = ResolvedApplyTarget {
        target_type: target_type.clone(),
        trip_id,
        trip_name: trip_name.to_string(),
        day_number: None,
        itinerary_sort_order: None,
        itinerary_title: None,
        resolution: "resolved".to_string(),
    };

    match target_type.as_str() {
        "trip" => Ok(Some(resolved)),
        "day" => {
            let day_number = match day_reference_number(target, report) {
                Some(day) => day,
                None => return Err(()),
            };
            if find_day_by_trip_and_day_number(conn, trip_id, day_number).is_err() {
                report.errors.push(format!(
                    "target day_reference ({day_number}) が Trip {trip_id} に存在しません"
                ));
                return Err(());
            }
            resolved.day_number = Some(day_number);
            Ok(Some(resolved))
        }
        "itinerary" => {
            let day_number = match day_reference_number(target, report) {
                Some(day) => day,
                None => return Err(()),
            };
            if find_day_by_trip_and_day_number(conn, trip_id, day_number).is_err() {
                report.errors.push(format!(
                    "target day_reference ({day_number}) が Trip {trip_id} に存在しません"
                ));
                return Err(());
            }
            let items =
                match crate::itinerary::list_itinerary_items_for_day(conn, trip_id, day_number) {
                    Ok(items) => items,
                    Err(error) => {
                        report.errors.push(error.to_string());
                        return Err(());
                    }
                };
            let itinerary_ref = target.get("itinerary_reference");
            let matched = match itinerary_ref {
                Some(Value::Number(n)) if n.as_i64().is_some() => {
                    let v = n.as_i64().unwrap();
                    let id_matches: Vec<_> = items.iter().filter(|item| item.id == v).collect();
                    let sort_matches: Vec<_> =
                        items.iter().filter(|item| item.sort_order == v).collect();

                    if !id_matches.is_empty() && !sort_matches.is_empty() {
                        let id_item = id_matches[0];
                        let sort_item = sort_matches[0];
                        if id_item.id != sort_item.id {
                            report.errors.push(format!(
                                "itinerary_reference (数値 {v}) が itinerary_id と sort_order の両方に一致し曖昧です"
                            ));
                            resolved.resolution = "ambiguous".to_string();
                            return Err(());
                        }
                    }

                    let picked = if !id_matches.is_empty() {
                        if id_matches.len() > 1 {
                            report.errors.push(format!(
                                "itinerary_reference (itinerary_id {v}) が Day {day_number} で曖昧です"
                            ));
                            resolved.resolution = "ambiguous".to_string();
                            return Err(());
                        }
                        id_matches[0]
                    } else if !sort_matches.is_empty() {
                        if sort_matches.len() > 1 {
                            report.errors.push(format!(
                                "itinerary_reference (sort_order {v}) が Day {day_number} で曖昧です"
                            ));
                            resolved.resolution = "ambiguous".to_string();
                            return Err(());
                        }
                        sort_matches[0]
                    } else {
                        report.errors.push(format!(
                            "itinerary_reference (id/sort_order {v}) が Day {day_number} に見つかりません"
                        ));
                        return Err(());
                    };
                    (Some(picked.sort_order), Some(picked.title.clone()))
                }
                Some(Value::String(title)) => {
                    let needle = title.trim();
                    let matches: Vec<_> = items
                        .iter()
                        .filter(|item| item.title.trim() == needle)
                        .collect();
                    if matches.is_empty() {
                        report.errors.push(format!(
                            "itinerary_reference (title \"{needle}\") が Day {day_number} に見つかりません"
                        ));
                        return Err(());
                    }
                    if matches.len() > 1 {
                        report.errors.push(format!(
                            "itinerary_reference (title \"{needle}\") が Day {day_number} で曖昧です"
                        ));
                        resolved.resolution = "ambiguous".to_string();
                        return Err(());
                    }
                    (Some(matches[0].sort_order), Some(matches[0].title.clone()))
                }
                _ => {
                    report.errors.push(
                        "target_type が itinerary ですが itinerary_reference がありません"
                            .to_string(),
                    );
                    return Err(());
                }
            };
            resolved.day_number = Some(day_number);
            resolved.itinerary_sort_order = matched.0;
            resolved.itinerary_title = matched.1;
            Ok(Some(resolved))
        }
        other => {
            report
                .errors
                .push(format!("未対応の target_type です: {other}"));
            Err(())
        }
    }
}

fn day_reference_number(
    target: &Map<String, Value>,
    report: &mut FragmentApplyDryRunReport,
) -> Option<i64> {
    match target.get("day_reference").and_then(Value::as_i64) {
        Some(day) if day >= 1 => Some(day),
        Some(day) => {
            report.errors.push(format!(
                "target.day_reference ({day}) は 1 以上である必要があります"
            ));
            None
        }
        None => {
            report
                .errors
                .push("target.day_reference が必要です".to_string());
            None
        }
    }
}

fn finalize_apply_preview_export(export: &mut TripExportV3) {
    export.schema_version = Some(TRIP_EXPORT_SCHEMA_VERSION);
    if export.generator.is_none() {
        export.generator = Some(TRIP_EXPORT_GENERATOR.to_string());
    }
    if export.checklist_items.is_none() {
        export.checklist_items = Some(Vec::new());
    }
    if export.notes.is_none() {
        export.notes = Some(Vec::new());
    }
    if export.participants.is_none() {
        export.participants = Some(Vec::new());
    }
}

fn count_itineraries(export: &TripExportV3) -> usize {
    export.days.iter().map(|day| day.itineraries.len()).sum()
}

fn count_notes(export: &TripExportV3) -> usize {
    export.notes().len()
}

fn count_expenses(export: &TripExportV3) -> usize {
    export
        .days
        .iter()
        .flat_map(|day| day.itineraries.iter())
        .map(|item| item.expenses.len())
        .sum()
}

fn count_reservations(export: &TripExportV3) -> usize {
    export
        .days
        .iter()
        .flat_map(|day| day.itineraries.iter())
        .map(|item| item.reservations.len())
        .sum()
}

fn count_estimates(export: &TripExportV3) -> usize {
    export
        .days
        .iter()
        .flat_map(|day| day.itineraries.iter())
        .map(|item| item.estimates.len())
        .sum()
}

fn blocking_children_total(children: &FragmentApplyBlockingChildren) -> usize {
    children.expenses + children.estimates + children.reservations + children.notes
}

fn matches_itinerary_note(note: &ExportNote, day_number: i64, sort_order: i64) -> bool {
    matches!(
        note,
        ExportNote::Itinerary { itinerary_key, .. }
            if itinerary_key.day_number == day_number && itinerary_key.sort_order == sort_order
    )
}

fn count_itinerary_notes_in_export(
    export: &TripExportV3,
    day_number: i64,
    sort_order: i64,
) -> usize {
    export
        .notes()
        .iter()
        .filter(|note| matches_itinerary_note(note, day_number, sort_order))
        .count()
}

fn remove_itinerary_from_export(
    export: &mut TripExportV3,
    day_number: i64,
    sort_order: i64,
) -> Result<(), String> {
    let day = export
        .days
        .iter_mut()
        .find(|day| day.day_number == day_number)
        .ok_or_else(|| format!("preview 内に Day {day_number} が見つかりません"))?;
    let before = day.itineraries.len();
    day.itineraries.retain(|item| item.sort_order != sort_order);
    if day.itineraries.len() + 1 == before {
        Ok(())
    } else {
        Err(format!(
            "preview 内に itinerary (day {day_number}, sort_order {sort_order}) が見つかりません"
        ))
    }
}

fn remove_itinerary_notes_from_export(export: &mut TripExportV3, day_number: i64, sort_order: i64) {
    if let Some(notes) = export.notes.as_mut() {
        notes.retain(|note| !matches_itinerary_note(note, day_number, sort_order));
    }
}

fn resolved_apply_target_to_fragment_target(
    resolved: &ResolvedApplyTarget,
) -> FragmentApplyResolvedTarget {
    FragmentApplyResolvedTarget {
        target_type: resolved.target_type.clone(),
        trip_id: resolved.trip_id,
        trip_name: resolved.trip_name.clone(),
        day_number: resolved.day_number,
        itinerary_sort_order: resolved.itinerary_sort_order,
        itinerary_title: resolved.itinerary_title.clone(),
        resolution: resolved.resolution.clone(),
    }
}

fn apply_delete_itinerary_preview(
    conn: &Connection,
    trip_id: i64,
    export: &mut TripExportV3,
    resolved: &ResolvedApplyTarget,
    intent: &str,
    itineraries_before: usize,
    report: &mut FragmentApplyDryRunReport,
) -> Result<FragmentApplyPreviewSummary, String> {
    if resolved.target_type != "itinerary" {
        return Err(
            "delete_itinerary は itinerary target のみサポートしています（trip / day は未対応）"
                .to_string(),
        );
    }
    if resolved.resolution == "ambiguous" {
        return Err("target が曖昧です — apply preview を続行しません".to_string());
    }

    let day_number = resolved.day_number.ok_or_else(|| {
        "delete_itinerary の Itinerary target が解決されていません（day）".to_string()
    })?;
    let itinerary_sort_order = resolved.itinerary_sort_order.ok_or_else(|| {
        "delete_itinerary の Itinerary target が解決されていません（sort_order）".to_string()
    })?;
    ensure_day_in_range(export, day_number)?;

    let itinerary_id = resolve_itinerary_id_for_apply_target(
        conn,
        trip_id,
        &resolved_apply_target_to_fragment_target(resolved),
    )
    .map_err(|error| error.to_string())?;

    let current = lookup_itinerary_in_export(export, day_number, itinerary_sort_order)
        .ok_or_else(|| {
            format!(
                "preview 内に itinerary (day {day_number}, sort_order {itinerary_sort_order}) が見つかりません"
            )
        })?;

    let blocking_children = FragmentApplyBlockingChildren {
        expenses: current.expenses.len(),
        estimates: current.estimates.len(),
        reservations: current.reservations.len(),
        notes: count_itinerary_notes_in_export(export, day_number, itinerary_sort_order),
    };

    if blocking_children_total(&blocking_children) > 0 {
        report.errors.push(format!(
            "delete_itinerary は blocking child が存在します（expenses: {}, estimates: {}, reservations: {}, notes: {}）— DB 更新しません",
            blocking_children.expenses,
            blocking_children.estimates,
            blocking_children.reservations,
            blocking_children.notes,
        ));
    }

    let title = current.title.clone();
    let sort_order = current.sort_order;

    let itineraries_after = if blocking_children_total(&blocking_children) == 0 {
        remove_itinerary_from_export(export, day_number, itinerary_sort_order)?;
        remove_itinerary_notes_from_export(export, day_number, itinerary_sort_order);
        count_itineraries(export)
    } else {
        itineraries_before
    };

    Ok(FragmentApplyPreviewSummary {
        intent: intent.to_string(),
        action: "delete_itinerary".to_string(),
        candidate_title: resolved.itinerary_title.clone(),
        itineraries_before,
        itineraries_after,
        notes_before: None,
        notes_after: None,
        expenses_before: None,
        expenses_after: None,
        expense_preview: None,
        estimates_before: None,
        estimates_after: None,
        estimate_preview: None,
        estimate_update_preview: None,
        estimate_field_changes: None,
        estimate_delete_preview: None,
        reservations_before: None,
        reservations_after: None,
        reservation_preview: None,
        itinerary_field_changes: None,
        reorder_preview: None,
        move_preview: None,
        delete_preview: Some(FragmentApplyDeletePreview {
            target_type: "itinerary".to_string(),
            itinerary_id,
            title,
            day_number,
            sort_order,
            blocking_children,
            non_blocking_relations: None,
        }),
    })
}

#[allow(clippy::too_many_arguments)]
fn apply_reorder_itinerary_preview(
    conn: &Connection,
    trip_id: i64,
    export: &mut TripExportV3,
    resolved: &ResolvedApplyTarget,
    fragment: &Map<String, Value>,
    intent: &str,
    itineraries_before: usize,
    _report: &mut FragmentApplyDryRunReport,
) -> Result<FragmentApplyPreviewSummary, String> {
    if resolved.target_type != "day" {
        return Err(format!(
            "reorder_itinerary は day target のみサポートしています（現在: {}）",
            resolved.target_type
        ));
    }
    let day_number = resolved
        .day_number
        .ok_or_else(|| "reorder_itinerary の Day target が解決されていません".to_string())?;
    ensure_day_in_range(export, day_number)?;

    let candidate = fragment
        .get("candidate_content")
        .and_then(Value::as_object)
        .ok_or_else(|| "candidate_content object が必要です".to_string())?;

    let expected = candidate
        .get("expected_order")
        .ok_or_else(|| "candidate_content.expected_order が必要です".to_string())?;
    let after = candidate
        .get("after_order")
        .ok_or_else(|| "candidate_content.after_order が必要です".to_string())?;

    let expected_refs = parse_reorder_order_refs(expected, "expected_order")?;
    let after_refs = parse_reorder_order_refs(after, "after_order")?;

    if expected_refs.is_empty() {
        return Err("candidate_content.expected_order は空にできません".to_string());
    }
    if after_refs.is_empty() {
        return Err("candidate_content.after_order は空にできません".to_string());
    }

    let day_items = crate::itinerary::list_itinerary_items_for_day(conn, trip_id, day_number)
        .map_err(|e| e.to_string())?;
    if day_items.is_empty() {
        return Err(format!(
            "reorder_itinerary: Day {day_number} に itinerary がありません"
        ));
    }

    let day_count = {
        let start = export
            .trip
            .start_date
            .as_deref()
            .ok_or_else(|| "trip.start_date が必要です".to_string())?;
        let end = export
            .trip
            .end_date
            .as_deref()
            .ok_or_else(|| "trip.end_date が必要です".to_string())?;
        validate_trip_date_range(start, end).map_err(|e| e.to_string())?
    };

    let expected_resolved = resolve_reorder_order_in_day(
        conn,
        trip_id,
        day_number,
        day_count,
        &day_items,
        &expected_refs,
        "expected_order",
    )?;
    let after_resolved = resolve_reorder_order_in_day(
        conn,
        trip_id,
        day_number,
        day_count,
        &day_items,
        &after_refs,
        "after_order",
    )?;

    // full-day baseline: expected_order は「当該 Day の全 itinerary」を現在順で完全列挙する
    let mut current_ids: Vec<i64> = day_items.iter().map(|i| i.id).collect();
    current_ids.sort_by_key(|id| {
        day_items
            .iter()
            .find(|i| i.id == *id)
            .map(|i| i.sort_order)
            .unwrap_or(i64::MAX)
    });
    if expected_resolved.iter().map(|r| r.id).collect::<Vec<_>>() != current_ids {
        return Err("reorder_itinerary: expected_order が現行 Day の順序と一致しません（baseline mismatch）— DB 更新しません".to_string());
    }

    let expected_set = expected_resolved
        .iter()
        .map(|r| r.id)
        .collect::<std::collections::HashSet<_>>();
    let after_set = after_resolved
        .iter()
        .map(|r| r.id)
        .collect::<std::collections::HashSet<_>>();
    if expected_set != after_set {
        return Err(
            "reorder_itinerary: after_order は expected_order と同じ itinerary 集合を含む必要があります"
                .to_string(),
        );
    }
    let expected_ids = expected_resolved.iter().map(|r| r.id).collect::<Vec<_>>();
    let after_ids = after_resolved.iter().map(|r| r.id).collect::<Vec<_>>();
    if expected_ids == after_ids {
        return Err("reorder_itinerary: no-op reorder は許可されません".to_string());
    }

    // sparse sort_order を維持する: 現行 Day の sort_order スロットに after_order を割り当てる
    let mut slots: Vec<i64> = day_items.iter().map(|i| i.sort_order).collect();
    slots.sort();
    if slots.len() != after_ids.len() {
        return Err("reorder_itinerary: internal mismatch（slot length）".to_string());
    }

    let mut id_to_after_sort: std::collections::HashMap<i64, i64> =
        std::collections::HashMap::new();
    for (idx, id) in after_ids.iter().enumerate() {
        id_to_after_sort.insert(*id, slots[idx]);
    }

    let mut changes: Vec<FragmentApplyItineraryOrderChange> = Vec::new();
    for resolved_item in &after_resolved {
        let after_sort = *id_to_after_sort
            .get(&resolved_item.id)
            .expect("after sort assigned");
        changes.push(FragmentApplyItineraryOrderChange {
            itinerary_id: resolved_item.id,
            title: resolved_item.title.clone(),
            before_sort_order: resolved_item.sort_order,
            after_sort_order: after_sort,
        });
    }

    // preview export にも sort_order を反映（DB は更新しない）
    if let Some(day) = export.days.iter_mut().find(|d| d.day_number == day_number) {
        let mut sort_order_to_id: std::collections::HashMap<i64, i64> =
            std::collections::HashMap::new();
        for item in &day_items {
            sort_order_to_id.insert(item.sort_order, item.id);
        }
        for it in &mut day.itineraries {
            let Some(id) = sort_order_to_id.get(&it.sort_order).copied() else {
                continue;
            };
            if let Some(new_sort) = id_to_after_sort.get(&id).copied() {
                it.sort_order = new_sort;
            }
        }
        day.itineraries.sort_by_key(|i| i.sort_order);
    }

    Ok(FragmentApplyPreviewSummary {
        intent: intent.to_string(),
        action: "reorder_itinerary".to_string(),
        candidate_title: None,
        itineraries_before,
        itineraries_after: itineraries_before,
        notes_before: None,
        notes_after: None,
        expenses_before: None,
        expenses_after: None,
        expense_preview: None,
        estimates_before: None,
        estimates_after: None,
        estimate_preview: None,
        estimate_update_preview: None,
        estimate_field_changes: None,
        estimate_delete_preview: None,
        reservations_before: None,
        reservations_after: None,
        reservation_preview: None,
        itinerary_field_changes: None,
        reorder_preview: Some(FragmentApplyReorderPreview {
            day_number,
            itinerary_order_changes: changes,
        }),
        move_preview: None,
        delete_preview: None,
    })
}

#[allow(clippy::too_many_arguments)]
fn apply_move_itinerary_preview(
    conn: &Connection,
    trip_id: i64,
    export: &mut TripExportV3,
    resolved: &ResolvedApplyTarget,
    fragment: &Map<String, Value>,
    intent: &str,
    itineraries_before: usize,
    _report: &mut FragmentApplyDryRunReport,
) -> Result<FragmentApplyPreviewSummary, String> {
    if resolved.target_type != "itinerary" {
        return Err(format!(
            "move_itinerary は itinerary target のみサポートしています（現在: {}）",
            resolved.target_type
        ));
    }
    let target_day_number = resolved.day_number.ok_or_else(|| {
        "move_itinerary の Itinerary target が解決されていません（day）".to_string()
    })?;
    let target_itinerary_sort_order = resolved.itinerary_sort_order.ok_or_else(|| {
        "move_itinerary の Itinerary target が解決されていません（sort_order）".to_string()
    })?;

    ensure_day_in_range(export, target_day_number)?;

    let candidate = fragment
        .get("candidate_content")
        .and_then(Value::as_object)
        .ok_or_else(|| "candidate_content object が必要です".to_string())?;

    let from_day = candidate
        .get("from_day")
        .and_then(Value::as_i64)
        .ok_or_else(|| "candidate_content.from_day が必要です".to_string())?;
    let to_day = candidate
        .get("to_day")
        .and_then(Value::as_i64)
        .ok_or_else(|| "candidate_content.to_day が必要です".to_string())?;
    ensure_day_in_range(export, from_day)?;
    ensure_day_in_range(export, to_day)?;

    let plan = compute_move_itinerary_plan(
        conn,
        trip_id,
        target_day_number,
        target_itinerary_sort_order,
        candidate,
    )?;

    let mut source_sort_rewrites: std::collections::HashMap<i64, i64> =
        std::collections::HashMap::new();
    for item in &plan.after_source_resolved {
        let after_sort = *plan
            .source_after_sort
            .get(&item.id)
            .expect("source after sort");
        source_sort_rewrites.insert(item.sort_order, after_sort);
    }
    move_itinerary_in_export_preview(
        export,
        plan.from_day,
        plan.to_day,
        plan.moved.sort_order,
        plan.moved_after_sort,
        &source_sort_rewrites,
    )?;

    Ok(FragmentApplyPreviewSummary {
        intent: intent.to_string(),
        action: "move_itinerary".to_string(),
        candidate_title: resolved.itinerary_title.clone(),
        itineraries_before,
        itineraries_after: itineraries_before,
        notes_before: None,
        notes_after: None,
        expenses_before: None,
        expenses_after: None,
        expense_preview: None,
        estimates_before: None,
        estimates_after: None,
        estimate_preview: None,
        estimate_update_preview: None,
        estimate_field_changes: None,
        estimate_delete_preview: None,
        reservations_before: None,
        reservations_after: None,
        reservation_preview: None,
        itinerary_field_changes: None,
        reorder_preview: None,
        move_preview: Some(FragmentApplyMovePreview {
            itinerary_id: plan.moved.id,
            title: plan.moved.title.clone(),
            from_day_number: plan.from_day,
            to_day_number: plan.to_day,
            source_order_changes: plan.source_changes,
            destination_order_changes: plan.destination_changes,
        }),
        delete_preview: None,
    })
}

#[derive(Clone, Debug)]
struct ResolvedDayItinerary {
    id: i64,
    title: String,
    sort_order: i64,
}

fn resolve_move_order_in_day(
    conn: &Connection,
    trip_id: i64,
    day_number: i64,
    day_count: i64,
    day_items: &[crate::domain::models::ItineraryItem],
    refs: &[ItineraryRefKey],
    field: &str,
) -> Result<Vec<ResolvedDayItinerary>, String> {
    resolve_order_in_candidates(
        conn,
        trip_id,
        day_number,
        day_count,
        day_items,
        std::iter::empty(),
        refs,
        field,
        "move_itinerary",
    )
}

#[allow(clippy::too_many_arguments)]
fn resolve_move_after_destination_order(
    conn: &Connection,
    trip_id: i64,
    day_number: i64,
    day_count: i64,
    destination_items: &[crate::domain::models::ItineraryItem],
    moved_item: &crate::domain::models::ItineraryItem,
    refs: &[ItineraryRefKey],
    field: &str,
) -> Result<Vec<ResolvedDayItinerary>, String> {
    resolve_order_in_candidates(
        conn,
        trip_id,
        day_number,
        day_count,
        destination_items,
        std::iter::once(moved_item),
        refs,
        field,
        "move_itinerary",
    )
}

#[allow(clippy::too_many_arguments)]
fn resolve_order_in_candidates<'a, I>(
    conn: &Connection,
    trip_id: i64,
    day_number: i64,
    day_count: i64,
    day_items: &'a [crate::domain::models::ItineraryItem],
    extra_items: I,
    refs: &[ItineraryRefKey],
    field: &str,
    intent_name: &str,
) -> Result<Vec<ResolvedDayItinerary>, String>
where
    I: IntoIterator<Item = &'a crate::domain::models::ItineraryItem>,
{
    let mut candidates: Vec<&crate::domain::models::ItineraryItem> = day_items.iter().collect();
    candidates.extend(extra_items);

    let mut out: Vec<ResolvedDayItinerary> = Vec::new();
    for key in refs {
        match key {
            ItineraryRefKey::Number(v) => {
                let id_matches: Vec<_> = candidates.iter().filter(|item| item.id == *v).collect();
                let sort_matches: Vec<_> = candidates
                    .iter()
                    .filter(|item| item.sort_order == *v)
                    .collect();

                if !id_matches.is_empty() && !sort_matches.is_empty() {
                    return Err(format!(
                        "{intent_name}: {field} の数値 selector ({v}) が itinerary_id と sort_order の両方に一致し曖昧です — DB 更新しません"
                    ));
                }

                let picked = if !id_matches.is_empty() {
                    if id_matches.len() > 1 {
                        return Err(format!(
                            "{intent_name}: {field} の itinerary_id ({v}) が Day {day_number} で曖昧です"
                        ));
                    }
                    *id_matches[0]
                } else {
                    if sort_matches.is_empty() {
                        return Err(format!(
                            "{intent_name}: {field} の itinerary_reference (id/sort_order {v}) が Day {day_number} に見つかりません"
                        ));
                    }
                    if sort_matches.len() > 1 {
                        return Err(format!(
                            "{intent_name}: {field} の itinerary_reference (sort_order {v}) が Day {day_number} で曖昧です"
                        ));
                    }
                    *sort_matches[0]
                };

                out.push(ResolvedDayItinerary {
                    id: picked.id,
                    title: picked.title.clone(),
                    sort_order: picked.sort_order,
                });
            }
            ItineraryRefKey::Title(title) => {
                let matches: Vec<_> = candidates
                    .iter()
                    .filter(|item| item.title.trim() == title.trim())
                    .collect();
                if matches.is_empty() {
                    if itinerary_title_exists_in_other_day(
                        conn, trip_id, day_number, day_count, title,
                    ) {
                        return Err(format!(
                            "{intent_name}: {field} の itinerary_reference (title \"{title}\") は Day {day_number} ではなく別 Day に存在します — DB 更新しません"
                        ));
                    }
                    return Err(format!(
                        "{intent_name}: {field} の itinerary_reference (title \"{title}\") が Day {day_number} に見つかりません"
                    ));
                }
                if matches.len() > 1 {
                    return Err(format!(
                        "{intent_name}: {field} の itinerary_reference (title \"{title}\") が Day {day_number} で曖昧です"
                    ));
                }
                out.push(ResolvedDayItinerary {
                    id: matches[0].id,
                    title: matches[0].title.clone(),
                    sort_order: matches[0].sort_order,
                });
            }
        }
    }

    let mut seen_ids: std::collections::HashSet<i64> = std::collections::HashSet::new();
    for item in &out {
        if !seen_ids.insert(item.id) {
            return Err(format!(
                "{intent_name}: {field} に同一 itinerary の重複参照が含まれています"
            ));
        }
    }
    Ok(out)
}

#[derive(Clone, Debug)]
struct MoveItineraryComputedPlan {
    from_day: i64,
    to_day: i64,
    to_day_id: i64,
    moved: crate::domain::models::ItineraryItem,
    moved_after_sort: i64,
    after_source_resolved: Vec<ResolvedDayItinerary>,
    source_after_sort: std::collections::HashMap<i64, i64>,
    source_changes: Vec<FragmentApplyItineraryMoveOrderChange>,
    destination_changes: Vec<FragmentApplyItineraryMoveOrderChange>,
}

struct MoveItineraryConfirmOutcome {
    itinerary_id: i64,
    updated_rows: usize,
}

fn compute_move_itinerary_plan(
    conn: &Connection,
    trip_id: i64,
    target_day_number: i64,
    target_itinerary_sort_order: i64,
    candidate: &Map<String, Value>,
) -> Result<MoveItineraryComputedPlan, String> {
    let from_day = candidate
        .get("from_day")
        .and_then(Value::as_i64)
        .ok_or_else(|| "candidate_content.from_day が必要です".to_string())?;
    let to_day = candidate
        .get("to_day")
        .and_then(Value::as_i64)
        .ok_or_else(|| "candidate_content.to_day が必要です".to_string())?;
    if from_day == to_day {
        return Err(
            "move_itinerary: same-day move はサポートしていません — reorder_itinerary を使用してください"
                .to_string(),
        );
    }

    find_day_by_trip_and_day_number(conn, trip_id, from_day)
        .map_err(|_| format!("move_itinerary: source Day {from_day} が存在しません"))?;
    find_day_by_trip_and_day_number(conn, trip_id, to_day)
        .map_err(|_| format!("move_itinerary: destination Day {to_day} が存在しません"))?;
    let to_day_id = crate::day::find_day_id_by_trip_and_day_number(conn, trip_id, to_day)
        .map_err(|_| format!("move_itinerary: destination Day {to_day} が存在しません"))?;

    if target_day_number != from_day {
        return Err(format!(
            "move_itinerary: target itinerary は Day {target_day_number} にありますが、candidate_content.from_day ({from_day}) と一致しません — DB 更新しません"
        ));
    }

    let trip = crate::trip::get_trip(conn, trip_id).map_err(|e| e.to_string())?;
    let start = trip
        .start_date
        .as_deref()
        .ok_or_else(|| "trip.start_date が必要です".to_string())?;
    let end = trip
        .end_date
        .as_deref()
        .ok_or_else(|| "trip.end_date が必要です".to_string())?;
    let day_count = validate_trip_date_range(start, end).map_err(|e| e.to_string())?;
    for day in [from_day, to_day, target_day_number] {
        if day < 1 || day > day_count {
            return Err(format!(
                "day_number ({day}) が旅行期間 (1..={day_count}) の範囲外です"
            ));
        }
    }

    let expected_source = candidate
        .get("expected_source_order")
        .ok_or_else(|| "candidate_content.expected_source_order が必要です".to_string())?;
    let expected_destination = candidate
        .get("expected_destination_order")
        .ok_or_else(|| "candidate_content.expected_destination_order が必要です".to_string())?;
    let after_source = candidate
        .get("after_source_order")
        .ok_or_else(|| "candidate_content.after_source_order が必要です".to_string())?;
    let after_destination = candidate
        .get("after_destination_order")
        .ok_or_else(|| "candidate_content.after_destination_order が必要です".to_string())?;

    let expected_source_refs = parse_reorder_order_refs(expected_source, "expected_source_order")?;
    let expected_destination_refs =
        parse_reorder_order_refs(expected_destination, "expected_destination_order")?;
    let after_source_refs = parse_reorder_order_refs(after_source, "after_source_order")?;
    let after_destination_refs =
        parse_reorder_order_refs(after_destination, "after_destination_order")?;

    let source_items = crate::itinerary::list_itinerary_items_for_day(conn, trip_id, from_day)
        .map_err(|e| e.to_string())?;
    let destination_items = crate::itinerary::list_itinerary_items_for_day(conn, trip_id, to_day)
        .map_err(|e| e.to_string())?;

    let moved = source_items
        .iter()
        .find(|i| i.sort_order == target_itinerary_sort_order)
        .ok_or_else(|| {
            format!(
                "move_itinerary: target itinerary (day {from_day}, sort_order {target_itinerary_sort_order}) が見つかりません"
            )
        })?
        .clone();

    let expected_source_resolved = resolve_move_order_in_day(
        conn,
        trip_id,
        from_day,
        day_count,
        &source_items,
        &expected_source_refs,
        "expected_source_order",
    )?;
    let after_source_resolved = resolve_move_order_in_day(
        conn,
        trip_id,
        from_day,
        day_count,
        &source_items,
        &after_source_refs,
        "after_source_order",
    )?;
    let expected_destination_resolved = resolve_move_order_in_day(
        conn,
        trip_id,
        to_day,
        day_count,
        &destination_items,
        &expected_destination_refs,
        "expected_destination_order",
    )?;
    let after_destination_resolved = resolve_move_after_destination_order(
        conn,
        trip_id,
        to_day,
        day_count,
        &destination_items,
        &moved,
        &after_destination_refs,
        "after_destination_order",
    )?;

    let mut current_source_ids: Vec<i64> = source_items.iter().map(|i| i.id).collect();
    current_source_ids.sort_by_key(|id| {
        source_items
            .iter()
            .find(|i| i.id == *id)
            .map(|i| i.sort_order)
            .unwrap_or(i64::MAX)
    });
    if expected_source_resolved
        .iter()
        .map(|r| r.id)
        .collect::<Vec<_>>()
        != current_source_ids
    {
        return Err("move_itinerary: expected_source_order が現行 source Day の順序と一致しません（baseline mismatch）— DB 更新しません".to_string());
    }

    let mut current_destination_ids: Vec<i64> = destination_items.iter().map(|i| i.id).collect();
    current_destination_ids.sort_by_key(|id| {
        destination_items
            .iter()
            .find(|i| i.id == *id)
            .map(|i| i.sort_order)
            .unwrap_or(i64::MAX)
    });
    if expected_destination_resolved
        .iter()
        .map(|r| r.id)
        .collect::<Vec<_>>()
        != current_destination_ids
    {
        return Err("move_itinerary: expected_destination_order が現行 destination Day の順序と一致しません（baseline mismatch）— DB 更新しません".to_string());
    }

    let moved_id = moved.id;

    if expected_source_resolved
        .iter()
        .filter(|r| r.id == moved_id)
        .count()
        != 1
    {
        return Err("move_itinerary: moved itinerary は expected_source_order に 1 回だけ含まれる必要があります — DB 更新しません".to_string());
    }
    if expected_destination_resolved
        .iter()
        .any(|r| r.id == moved_id)
    {
        return Err("move_itinerary: moved itinerary は expected_destination_order に含められません — DB 更新しません".to_string());
    }
    if after_source_resolved.iter().any(|r| r.id == moved_id) {
        return Err("move_itinerary: moved itinerary は after_source_order に含められません — DB 更新しません".to_string());
    }
    if after_destination_resolved
        .iter()
        .filter(|r| r.id == moved_id)
        .count()
        != 1
    {
        return Err("move_itinerary: moved itinerary は after_destination_order に 1 回だけ含まれる必要があります — DB 更新しません".to_string());
    }

    let expected_source_ids = expected_source_resolved
        .iter()
        .map(|r| r.id)
        .collect::<Vec<_>>();
    let after_source_ids = after_source_resolved
        .iter()
        .map(|r| r.id)
        .collect::<Vec<_>>();
    let expected_source_minus_moved: Vec<i64> = expected_source_ids
        .iter()
        .copied()
        .filter(|id| *id != moved_id)
        .collect();
    if after_source_ids != expected_source_minus_moved {
        return Err("move_itinerary: after_source_order は expected_source_order から moved itinerary を 1 つ除いた結果と一致する必要があります — DB 更新しません".to_string());
    }

    let expected_destination_ids = expected_destination_resolved
        .iter()
        .map(|r| r.id)
        .collect::<Vec<_>>();
    let after_destination_ids = after_destination_resolved
        .iter()
        .map(|r| r.id)
        .collect::<Vec<_>>();
    let after_destination_minus_moved: Vec<i64> = after_destination_ids
        .iter()
        .copied()
        .filter(|id| *id != moved_id)
        .collect();
    if after_destination_minus_moved != expected_destination_ids {
        return Err("move_itinerary: after_destination_order は expected_destination_order に moved itinerary を 1 回だけ挿入した結果と一致する必要があります — DB 更新しません".to_string());
    }

    let mut source_slots: Vec<i64> = source_items.iter().map(|i| i.sort_order).collect();
    source_slots.sort();
    if source_slots.len() != expected_source_ids.len() {
        return Err("move_itinerary: internal mismatch（source slot length）".to_string());
    }
    let remaining_slots: Vec<i64> = source_slots
        .into_iter()
        .take(after_source_ids.len())
        .collect();
    let mut source_after_sort: std::collections::HashMap<i64, i64> =
        std::collections::HashMap::new();
    for (idx, id) in after_source_ids.iter().enumerate() {
        source_after_sort.insert(*id, remaining_slots[idx]);
    }

    let moved_insert_index = after_destination_ids
        .iter()
        .position(|id| *id == moved_id)
        .ok_or_else(|| "move_itinerary: internal mismatch（moved insert index）".to_string())?;
    let destination_sort_by_id: std::collections::HashMap<i64, i64> = destination_items
        .iter()
        .map(|i| (i.id, i.sort_order))
        .collect();
    let prev_dest_sort = if moved_insert_index == 0 {
        None
    } else {
        let prev_id = after_destination_ids[moved_insert_index - 1];
        destination_sort_by_id.get(&prev_id).copied()
    };
    let next_dest_sort = if moved_insert_index + 1 >= after_destination_ids.len() {
        None
    } else {
        let next_id = after_destination_ids[moved_insert_index + 1];
        destination_sort_by_id.get(&next_id).copied()
    };

    let dest_existing_sorts: std::collections::HashSet<i64> =
        destination_items.iter().map(|i| i.sort_order).collect();

    let moved_after_sort = if let (Some(prev), Some(next)) = (prev_dest_sort, next_dest_sort) {
        if prev + 1 < next {
            let midpoint = (prev + next) / 2;
            if midpoint > prev && midpoint < next && !dest_existing_sorts.contains(&midpoint) {
                midpoint
            } else {
                return Err("move_itinerary: destination に安全な sort_order slot を生成できません（midpoint collision）— DB 更新しません".to_string());
            }
        } else {
            return Err("move_itinerary: destination に安全な sort_order slot を生成できません（no gap）— DB 更新しません".to_string());
        }
    } else if prev_dest_sort.is_none() && next_dest_sort.is_some() {
        let Some(first) = next_dest_sort else {
            return Err("move_itinerary: internal mismatch（next dest sort）".to_string());
        };
        let head = first - SORT_ORDER_STEP;
        if head <= 0 {
            return Err("move_itinerary: destination 先頭への挿入に必要な正の sort_order slot を生成できません — DB 更新しません".to_string());
        }
        if dest_existing_sorts.contains(&head) {
            return Err("move_itinerary: destination に安全な sort_order slot を生成できません（head collision）— DB 更新しません".to_string());
        }
        head
    } else if prev_dest_sort.is_some() && next_dest_sort.is_none() {
        let last = prev_dest_sort.expect("prev");
        let tail = last + SORT_ORDER_STEP;
        if dest_existing_sorts.contains(&tail) {
            return Err("move_itinerary: destination に安全な sort_order slot を生成できません（tail collision）— DB 更新しません".to_string());
        }
        tail
    } else {
        SORT_ORDER_STEP
    };

    let mut source_changes: Vec<FragmentApplyItineraryMoveOrderChange> = Vec::new();
    for item in &after_source_resolved {
        let after_sort = *source_after_sort.get(&item.id).expect("source after sort");
        source_changes.push(FragmentApplyItineraryMoveOrderChange {
            itinerary_id: item.id,
            title: item.title.clone(),
            before_day_number: from_day,
            after_day_number: from_day,
            before_sort_order: item.sort_order,
            after_sort_order: after_sort,
        });
    }
    source_changes.push(FragmentApplyItineraryMoveOrderChange {
        itinerary_id: moved.id,
        title: moved.title.clone(),
        before_day_number: from_day,
        after_day_number: to_day,
        before_sort_order: moved.sort_order,
        after_sort_order: moved_after_sort,
    });

    let mut destination_changes: Vec<FragmentApplyItineraryMoveOrderChange> = Vec::new();
    for item in &expected_destination_resolved {
        destination_changes.push(FragmentApplyItineraryMoveOrderChange {
            itinerary_id: item.id,
            title: item.title.clone(),
            before_day_number: to_day,
            after_day_number: to_day,
            before_sort_order: item.sort_order,
            after_sort_order: item.sort_order,
        });
    }
    destination_changes.insert(
        moved_insert_index,
        FragmentApplyItineraryMoveOrderChange {
            itinerary_id: moved.id,
            title: moved.title.clone(),
            before_day_number: from_day,
            after_day_number: to_day,
            before_sort_order: moved.sort_order,
            after_sort_order: moved_after_sort,
        },
    );

    Ok(MoveItineraryComputedPlan {
        from_day,
        to_day,
        to_day_id,
        moved,
        moved_after_sort,
        after_source_resolved,
        source_after_sort,
        source_changes,
        destination_changes,
    })
}

fn verify_move_plan_matches_preview(
    plan: &MoveItineraryComputedPlan,
    preview: &FragmentApplyMovePreview,
) -> Result<(), String> {
    if plan.moved.id != preview.itinerary_id {
        return Err(format!(
            "TOCTOU mismatch: moved itinerary_id が gate preview と一致しません（{} vs {}）— DB 更新しません",
            plan.moved.id, preview.itinerary_id
        ));
    }
    if plan.from_day != preview.from_day_number {
        return Err(
            "TOCTOU mismatch: from_day_number が gate preview と一致しません — DB 更新しません"
                .to_string(),
        );
    }
    if plan.to_day != preview.to_day_number {
        return Err(
            "TOCTOU mismatch: to_day_number が gate preview と一致しません — DB 更新しません"
                .to_string(),
        );
    }
    if plan.source_changes.len() != preview.source_order_changes.len() {
        return Err(
            "TOCTOU mismatch: source_order_changes の件数が gate preview と一致しません — DB 更新しません"
                .to_string(),
        );
    }
    for (planned, previewed) in plan
        .source_changes
        .iter()
        .zip(preview.source_order_changes.iter())
    {
        if planned.itinerary_id != previewed.itinerary_id
            || planned.before_day_number != previewed.before_day_number
            || planned.after_day_number != previewed.after_day_number
            || planned.before_sort_order != previewed.before_sort_order
            || planned.after_sort_order != previewed.after_sort_order
        {
            return Err(format!(
                "TOCTOU mismatch: source_order_changes が gate preview と一致しません（itinerary_id {}）— DB 更新しません",
                planned.itinerary_id
            ));
        }
    }
    if plan.destination_changes.len() != preview.destination_order_changes.len() {
        return Err(
            "TOCTOU mismatch: destination_order_changes の件数が gate preview と一致しません — DB 更新しません"
                .to_string(),
        );
    }
    for (planned, previewed) in plan
        .destination_changes
        .iter()
        .zip(preview.destination_order_changes.iter())
    {
        if planned.itinerary_id != previewed.itinerary_id
            || planned.before_day_number != previewed.before_day_number
            || planned.after_day_number != previewed.after_day_number
            || planned.before_sort_order != previewed.before_sort_order
            || planned.after_sort_order != previewed.after_sort_order
        {
            return Err(format!(
                "TOCTOU mismatch: destination_order_changes が gate preview と一致しません（itinerary_id {}）— DB 更新しません",
                planned.itinerary_id
            ));
        }
    }
    Ok(())
}

fn execute_confirm_move_itinerary(
    conn: &Connection,
    trip_id: i64,
    json: &str,
    report: &FragmentApplyDryRunReport,
) -> Result<MoveItineraryConfirmOutcome> {
    let preview = report
        .preview
        .as_ref()
        .context("move_itinerary confirm には preview が必要です")?;
    let move_preview = preview
        .move_preview
        .as_ref()
        .context("move_itinerary confirm には move_preview が必要です")?;
    let target = report
        .resolved_target
        .as_ref()
        .context("target が解決されていません")?;
    if target.target_type != "itinerary" {
        anyhow::bail!(
            "move_itinerary confirm は itinerary target のみサポートしています（現在: {}）",
            target.target_type
        );
    }
    let target_day_number = target
        .day_number
        .context("move_itinerary の Itinerary target が解決されていません（day）")?;
    let target_itinerary_sort_order = target
        .itinerary_sort_order
        .context("move_itinerary の Itinerary target が解決されていません（sort_order）")?;

    let root: Value =
        serde_json::from_str(json).with_context(|| "Fragment JSON の parse に失敗しました")?;
    let root_obj = root
        .as_object()
        .context("トップレベルが JSON object ではありません")?;
    let fragment_body = root_obj
        .get("fragment")
        .and_then(Value::as_object)
        .context("fragment object が必要です")?;
    let candidate = fragment_body
        .get("candidate_content")
        .and_then(Value::as_object)
        .context("candidate_content object が必要です")?;

    let mut outcome = MoveItineraryConfirmOutcome {
        itinerary_id: 0,
        updated_rows: 0,
    };
    crate::storage::db::with_transaction(conn, "move_itinerary confirm", |tx| {
        let plan = compute_move_itinerary_plan(
            tx,
            trip_id,
            target_day_number,
            target_itinerary_sort_order,
            candidate,
        )
        .map_err(|error| anyhow::anyhow!(error))?;
        verify_move_plan_matches_preview(&plan, move_preview)
            .map_err(|error| anyhow::anyhow!(error))?;

        let now = crate::storage::db::now_string();
        let mut expected_updates: Vec<(MoveItineraryUpdateKind, i64, i64, i64)> = Vec::new();

        for change in &plan.source_changes {
            if change.itinerary_id == plan.moved.id {
                continue;
            }
            if change.before_sort_order != change.after_sort_order {
                expected_updates.push((
                    MoveItineraryUpdateKind::SourceSort,
                    change.itinerary_id,
                    change.after_sort_order,
                    plan.from_day,
                ));
            }
        }

        expected_updates.push((
            MoveItineraryUpdateKind::Moved,
            plan.moved.id,
            plan.moved_after_sort,
            plan.from_day,
        ));

        for change in &plan.destination_changes {
            if change.itinerary_id == plan.moved.id {
                continue;
            }
            if change.before_sort_order != change.after_sort_order {
                expected_updates.push((
                    MoveItineraryUpdateKind::DestinationSort,
                    change.itinerary_id,
                    change.after_sort_order,
                    plan.to_day,
                ));
            }
        }

        let mut actual_updated = 0usize;
        for (kind, id, new_sort, day_number) in expected_updates {
            let changed = match kind {
                MoveItineraryUpdateKind::SourceSort | MoveItineraryUpdateKind::DestinationSort => {
                    tx.execute(
                        "UPDATE itinerary_items SET sort_order = ?1, updated_at = ?2 WHERE id = ?3 AND trip_id = ?4 AND day = ?5",
                        rusqlite::params![new_sort, now, id, trip_id, day_number],
                    )
                    .context("itinerary_items.sort_order の更新に失敗しました")?
                }
                MoveItineraryUpdateKind::Moved => tx
                    .execute(
                        "UPDATE itinerary_items SET day_id = ?1, day = ?2, sort_order = ?3, updated_at = ?4 WHERE id = ?5 AND trip_id = ?6 AND day = ?7",
                        rusqlite::params![
                            plan.to_day_id,
                            plan.to_day,
                            new_sort,
                            now,
                            id,
                            trip_id,
                            day_number
                        ],
                    )
                    .context("itinerary_items の cross-day 更新に失敗しました")?,
            };
            if changed != 1 {
                anyhow::bail!(
                    "row count mismatch: itinerary_items UPDATE が 1 行ではありません（{changed}）— DB 更新しません"
                );
            }
            actual_updated += 1;
        }

        outcome.itinerary_id = plan.moved.id;
        outcome.updated_rows = actual_updated;
        Ok(())
    })?;

    Ok(outcome)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum MoveItineraryUpdateKind {
    SourceSort,
    DestinationSort,
    Moved,
}

fn move_itinerary_in_export_preview(
    export: &mut TripExportV3,
    from_day: i64,
    to_day: i64,
    from_sort_order: i64,
    to_sort_order: i64,
    source_sort_rewrites: &std::collections::HashMap<i64, i64>,
) -> Result<(), String> {
    let source_day = export
        .days
        .iter_mut()
        .find(|d| d.day_number == from_day)
        .ok_or_else(|| format!("preview export に source Day {from_day} がありません"))?;
    let moved_idx = source_day
        .itineraries
        .iter()
        .position(|it| it.sort_order == from_sort_order)
        .ok_or_else(|| {
            format!(
                "preview export 内に itinerary (day {from_day}, sort_order {from_sort_order}) が見つかりません"
            )
        })?;
    let mut moved_it = source_day.itineraries.remove(moved_idx);

    for it in &mut source_day.itineraries {
        if let Some(new_sort) = source_sort_rewrites.get(&it.sort_order).copied() {
            it.sort_order = new_sort;
        }
    }
    source_day.itineraries.sort_by_key(|i| i.sort_order);

    // destination day
    let dest_day = find_or_create_day(export, to_day);
    moved_it.sort_order = to_sort_order;
    dest_day.itineraries.push(moved_it);
    dest_day.itineraries.sort_by_key(|i| i.sort_order);

    Ok(())
}

fn parse_reorder_order_refs(value: &Value, field: &str) -> Result<Vec<ItineraryRefKey>, String> {
    let arr = value
        .as_array()
        .ok_or_else(|| format!("candidate_content.{field} は配列である必要があります"))?;
    let mut out: Vec<ItineraryRefKey> = Vec::new();
    for item in arr {
        match item {
            Value::Number(n) => {
                let Some(v) = n.as_i64() else {
                    return Err(format!(
                        "candidate_content.{field} の要素は整数（sort_order）または文字列（title）である必要があります"
                    ));
                };
                out.push(ItineraryRefKey::Number(v));
            }
            Value::String(s) => {
                let needle = s.trim();
                if needle.is_empty() {
                    return Err(format!(
                        "candidate_content.{field} に空文字が含まれています"
                    ));
                }
                out.push(ItineraryRefKey::Title(needle.to_string()));
            }
            _ => {
                return Err(format!(
                    "candidate_content.{field} の要素は整数（sort_order）または文字列（title）である必要があります"
                ));
            }
        }
    }
    // duplicate check（入力自体の重複）
    let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
    for key in &out {
        let k = match key {
            ItineraryRefKey::Number(v) => format!("number:{v}"),
            ItineraryRefKey::Title(t) => format!("title:{t}"),
        };
        if !seen.insert(k) {
            return Err(format!("candidate_content.{field} に重複が含まれています"));
        }
    }
    Ok(out)
}

fn resolve_reorder_order_in_day(
    conn: &Connection,
    trip_id: i64,
    day_number: i64,
    day_count: i64,
    day_items: &[crate::domain::models::ItineraryItem],
    refs: &[ItineraryRefKey],
    field: &str,
) -> Result<Vec<ResolvedDayItinerary>, String> {
    let mut out: Vec<ResolvedDayItinerary> = Vec::new();
    for key in refs {
        match key {
            ItineraryRefKey::Number(v) => {
                // GUI 想定: itinerary_id の配列（推奨）
                // CLI 想定: sort_order selector（数値）
                // v4.7.35 では number を id 優先で解釈し、id 不一致なら sort_order として解釈する。
                let id_matches: Vec<_> = day_items.iter().filter(|item| item.id == *v).collect();
                let sort_matches: Vec<_> = day_items
                    .iter()
                    .filter(|item| item.sort_order == *v)
                    .collect();

                if !id_matches.is_empty() && !sort_matches.is_empty() {
                    // 極端にまれだが、同一数値が id と sort_order の双方に一致する場合は曖昧として reject
                    return Err(format!(
                        "reorder_itinerary: {field} の数値 selector ({v}) が itinerary_id と sort_order の両方に一致し曖昧です — DB 更新しません"
                    ));
                }

                let picked = if !id_matches.is_empty() {
                    if id_matches.len() > 1 {
                        return Err(format!(
                            "reorder_itinerary: {field} の itinerary_id ({v}) が Day {day_number} で曖昧です"
                        ));
                    }
                    id_matches[0]
                } else {
                    if sort_matches.is_empty() {
                        return Err(format!(
                            "reorder_itinerary: {field} の itinerary_reference (id/sort_order {v}) が Day {day_number} に見つかりません"
                        ));
                    }
                    if sort_matches.len() > 1 {
                        return Err(format!(
                            "reorder_itinerary: {field} の itinerary_reference (sort_order {v}) が Day {day_number} で曖昧です"
                        ));
                    }
                    sort_matches[0]
                };

                out.push(ResolvedDayItinerary {
                    id: picked.id,
                    title: picked.title.clone(),
                    sort_order: picked.sort_order,
                });
            }
            ItineraryRefKey::Title(title) => {
                let matches: Vec<_> = day_items
                    .iter()
                    .filter(|item| item.title.trim() == title.trim())
                    .collect();
                if matches.is_empty() {
                    // cross-day move attempt hint: same title exists in another Day
                    if itinerary_title_exists_in_other_day(
                        conn, trip_id, day_number, day_count, title,
                    ) {
                        return Err(format!(
                            "reorder_itinerary: cross-day move は未対応です（別 intent として defer）。{field} の itinerary_reference (title \"{title}\") は Day {day_number} ではなく別 Day に存在します — DB 更新しません"
                        ));
                    }
                    return Err(format!(
                        "reorder_itinerary: {field} の itinerary_reference (title \"{title}\") が Day {day_number} に見つかりません"
                    ));
                }
                if matches.len() > 1 {
                    return Err(format!(
                        "reorder_itinerary: {field} の itinerary_reference (title \"{title}\") が Day {day_number} で曖昧です"
                    ));
                }
                out.push(ResolvedDayItinerary {
                    id: matches[0].id,
                    title: matches[0].title.clone(),
                    sort_order: matches[0].sort_order,
                });
            }
        }
    }
    // resolved duplicate（同一 itinerary を異なる selector で二重参照した場合）
    let mut seen_ids: std::collections::HashSet<i64> = std::collections::HashSet::new();
    for item in &out {
        if !seen_ids.insert(item.id) {
            return Err(format!(
                "reorder_itinerary: {field} に同一 itinerary の重複参照が含まれています"
            ));
        }
    }
    Ok(out)
}

fn itinerary_title_exists_in_other_day(
    conn: &Connection,
    trip_id: i64,
    day_number: i64,
    day_count: i64,
    title: &str,
) -> bool {
    // 低コストな cross-day 検出（title 一致のみ）。ID ベース authoring 前提の過渡期向け。
    for other_day in 1..=day_count {
        if other_day == day_number {
            continue;
        }
        let Ok(items) = crate::itinerary::list_itinerary_items_for_day(conn, trip_id, other_day)
        else {
            continue;
        };
        if items.iter().any(|i| i.title.trim() == title.trim()) {
            return true;
        }
    }
    false
}

fn parse_add_expense_fields(
    fragment: &Map<String, Value>,
    report: Option<&mut FragmentApplyDryRunReport>,
) -> Result<ParsedAddExpenseFields, String> {
    let candidate = fragment
        .get("candidate_content")
        .and_then(Value::as_object)
        .ok_or_else(|| "candidate_content object が必要です".to_string())?;

    if let Some(report) = report {
        warn_unsupported_add_expense_candidate_keys(candidate, report);
    }

    let currency_text = non_empty_string(candidate.get("currency"))
        .ok_or_else(|| "candidate_content.currency が必要です".to_string())?;
    let currency = validate_currency_code(&currency_text)
        .map_err(|error| format!("candidate_content.currency が不正です: {error}"))?;

    let amount = parse_expense_amount_field(candidate.get("amount"), &currency)?;

    let title = non_empty_string(candidate.get("title"))
        .or_else(|| non_empty_string(candidate.get("description")))
        .or_else(|| non_empty_string(candidate.get("label")));

    let note = non_empty_string(candidate.get("note"))
        .or_else(|| non_empty_string(candidate.get("memo")))
        .or_else(|| non_empty_string(fragment.get("notes")));

    Ok(ParsedAddExpenseFields {
        title,
        amount,
        currency,
        note,
    })
}

fn parse_expense_amount_field(value: Option<&Value>, currency: &str) -> Result<i64, String> {
    let Some(value) = value else {
        return Err("candidate_content.amount が必要です".to_string());
    };
    match value {
        Value::Number(number) => {
            let amount = number
                .as_i64()
                .ok_or_else(|| "candidate_content.amount は整数である必要があります".to_string())?;
            if amount < 0 {
                return Err("candidate_content.amount は 0 以上である必要があります".to_string());
            }
            Ok(amount)
        }
        Value::String(text) => parse_amount_for_currency(text, currency)
            .map_err(|error| format!("candidate_content.amount が不正です: {error}")),
        _ => Err("candidate_content.amount は数値または文字列である必要があります".to_string()),
    }
}

fn warn_unsupported_add_expense_candidate_keys(
    candidate: &Map<String, Value>,
    report: &mut FragmentApplyDryRunReport,
) {
    const SUPPORTED: &[&str] = &[
        "title",
        "description",
        "label",
        "amount",
        "currency",
        "note",
        "memo",
    ];
    for key in candidate.keys() {
        if SUPPORTED.contains(&key.as_str()) {
            continue;
        }
        push_unique(
            &mut report.warnings,
            format!("unsupported_field: candidate_content.{key} は add_expense では未反映です"),
        );
    }
}

fn parse_add_estimate_fields(
    fragment: &Map<String, Value>,
    report: Option<&mut FragmentApplyDryRunReport>,
) -> Result<ParsedAddEstimateFields, String> {
    let candidate = fragment
        .get("candidate_content")
        .and_then(Value::as_object)
        .ok_or_else(|| "candidate_content object が必要です".to_string())?;

    if let Some(report) = report {
        warn_unsupported_add_estimate_candidate_keys(candidate, report);
    }

    validate_optional_string_candidate_field(candidate, "title")?;
    validate_optional_string_candidate_field(candidate, "description")?;
    validate_optional_string_candidate_field(candidate, "label")?;
    validate_optional_string_candidate_field(candidate, "note")?;
    validate_optional_string_candidate_field(candidate, "memo")?;

    let currency_text = non_empty_string(candidate.get("currency"))
        .ok_or_else(|| "candidate_content.currency が必要です".to_string())?;
    let currency = validate_currency_code(&currency_text)
        .map_err(|error| format!("candidate_content.currency が不正です: {error}"))?;

    let amount = parse_expense_amount_field(candidate.get("amount"), &currency)?;

    let title = non_empty_string(candidate.get("title"))
        .or_else(|| non_empty_string(candidate.get("description")))
        .or_else(|| non_empty_string(candidate.get("label")));

    let note = non_empty_string(candidate.get("note"))
        .or_else(|| non_empty_string(candidate.get("memo")))
        .or_else(|| non_empty_string(fragment.get("notes")));

    let sort_order = parse_estimate_sort_order_field(candidate.get("sort_order"))?;

    Ok(ParsedAddEstimateFields {
        title,
        amount,
        currency,
        note,
        sort_order,
    })
}

fn validate_optional_string_candidate_field(
    candidate: &Map<String, Value>,
    key: &str,
) -> Result<(), String> {
    match candidate.get(key) {
        None | Some(Value::Null) => Ok(()),
        Some(Value::String(_)) => Ok(()),
        _ => Err(format!(
            "candidate_content.{key} は文字列である必要があります"
        )),
    }
}

fn parse_estimate_sort_order_field(value: Option<&Value>) -> Result<i64, String> {
    match value {
        None => Ok(0),
        Some(Value::Number(number)) => number
            .as_i64()
            .ok_or_else(|| "candidate_content.sort_order は整数である必要があります".to_string()),
        _ => Err("candidate_content.sort_order は整数である必要があります".to_string()),
    }
}

fn warn_unsupported_add_estimate_candidate_keys(
    candidate: &Map<String, Value>,
    report: &mut FragmentApplyDryRunReport,
) {
    const SUPPORTED: &[&str] = &[
        "title",
        "description",
        "label",
        "amount",
        "currency",
        "note",
        "memo",
        "sort_order",
    ];
    for key in candidate.keys() {
        if SUPPORTED.contains(&key.as_str()) {
            continue;
        }
        push_unique(
            &mut report.warnings,
            format!("unsupported_field: candidate_content.{key} は add_estimate では未反映です"),
        );
    }
}

fn parse_add_reservation_fields(
    fragment: &Map<String, Value>,
    report: Option<&mut FragmentApplyDryRunReport>,
) -> Result<ParsedAddReservationFields, String> {
    let candidate = fragment
        .get("candidate_content")
        .and_then(Value::as_object)
        .ok_or_else(|| "candidate_content object が必要です".to_string())?;

    if let Some(report) = report {
        warn_unsupported_add_reservation_candidate_keys(candidate, report);
    }

    let reservation_type_text = non_empty_string(candidate.get("reservation_type"))
        .or_else(|| non_empty_string(candidate.get("type")))
        .ok_or_else(|| "candidate_content.reservation_type が必要です".to_string())?;
    let reservation_type = validate_reservation_type(&reservation_type_text)
        .map_err(|error| format!("candidate_content.reservation_type が不正です: {error}"))?;

    let provider_text = non_empty_string(candidate.get("provider"))
        .or_else(|| non_empty_string(candidate.get("provider_name")))
        .ok_or_else(|| "candidate_content.provider が必要です".to_string())?;
    let provider_name = validate_provider_name(&provider_text)
        .map_err(|error| format!("candidate_content.provider が不正です: {error}"))?;

    let confirmation_code = non_empty_string(candidate.get("confirmation"))
        .or_else(|| non_empty_string(candidate.get("confirmation_code")));
    let reservation_site_url = non_empty_string(candidate.get("site_url"))
        .or_else(|| non_empty_string(candidate.get("reservation_site_url")));
    let remark = non_empty_string(candidate.get("remark"))
        .or_else(|| non_empty_string(candidate.get("note")))
        .or_else(|| non_empty_string(candidate.get("memo")));
    let start_at = non_empty_string(candidate.get("start_at"));
    let end_at = non_empty_string(candidate.get("end_at"));

    Ok(ParsedAddReservationFields {
        reservation_type,
        provider_name,
        confirmation_code,
        reservation_site_url,
        remark,
        start_at,
        end_at,
    })
}

fn warn_unsupported_add_reservation_candidate_keys(
    candidate: &Map<String, Value>,
    report: &mut FragmentApplyDryRunReport,
) {
    const SUPPORTED: &[&str] = &[
        "reservation_type",
        "type",
        "provider",
        "provider_name",
        "confirmation",
        "confirmation_code",
        "site_url",
        "reservation_site_url",
        "remark",
        "note",
        "memo",
        "start_at",
        "end_at",
    ];
    for key in candidate.keys() {
        if SUPPORTED.contains(&key.as_str()) {
            continue;
        }
        push_unique(
            &mut report.warnings,
            format!("unsupported_field: candidate_content.{key} は add_reservation では未反映です"),
        );
    }
}

fn build_export_reservation_from_add_fields(
    fields: &ParsedAddReservationFields,
) -> ExportReservationV3 {
    ExportReservationV3 {
        reservation_type: fields.reservation_type.clone(),
        provider_name: fields.provider_name.clone(),
        confirmation_code: fields.confirmation_code.clone(),
        reservation_site_url: fields.reservation_site_url.clone(),
        remark: fields.remark.clone(),
        start_at: fields.start_at.clone(),
        end_at: fields.end_at.clone(),
    }
}

fn parse_update_itinerary_fields(
    fragment: &Map<String, Value>,
    report: Option<&mut FragmentApplyDryRunReport>,
) -> Result<ParsedUpdateItineraryFields, String> {
    let candidate = fragment
        .get("candidate_content")
        .and_then(Value::as_object)
        .ok_or_else(|| "candidate_content object が必要です".to_string())?;

    if let Some(report) = report {
        warn_unsupported_update_itinerary_candidate_keys(candidate, report);
    }

    let title = if candidate.contains_key("title") {
        Some(
            non_empty_string(candidate.get("title"))
                .ok_or_else(|| "candidate_content.title が必要です".to_string())?,
        )
    } else {
        None
    };

    let note = patch_optional_string_field(candidate, "note")?;
    let location = patch_optional_string_field(candidate, "location")?;
    let category = patch_optional_category_field(candidate)?;
    let start_time = parse_update_start_time_field(candidate)?;
    let duration_minutes = parse_update_duration_minutes_field(candidate)?;
    let travel_minutes = parse_update_travel_minutes_field(candidate)?;

    let has_update_field = title.is_some()
        || note.is_some()
        || location.is_some()
        || category.is_some()
        || start_time.is_some()
        || duration_minutes.is_some()
        || travel_minutes.is_some();
    if !has_update_field {
        return Err("update_itinerary には少なくとも 1 つの更新フィールドが必要です".to_string());
    }

    Ok(ParsedUpdateItineraryFields {
        title,
        note,
        location,
        category,
        start_time,
        duration_minutes,
        travel_minutes,
    })
}

fn patch_optional_string_field(
    candidate: &Map<String, Value>,
    key: &str,
) -> Result<Option<UpdateFieldPatch<String>>, String> {
    if !candidate.contains_key(key) {
        return Ok(None);
    }
    let value = match candidate.get(key) {
        None | Some(Value::Null) => None,
        Some(Value::String(text)) => {
            if text.is_empty() {
                None
            } else {
                Some(text.clone())
            }
        }
        _ => {
            return Err(format!(
                "candidate_content.{key} は文字列である必要があります"
            ))
        }
    };
    Ok(Some(UpdateFieldPatch {
        value: value.unwrap_or_default(),
    }))
}

fn patch_optional_category_field(
    candidate: &Map<String, Value>,
) -> Result<Option<UpdateFieldPatch<ItineraryCategory>>, String> {
    if !candidate.contains_key("category") {
        return Ok(None);
    }
    let text = non_empty_string(candidate.get("category"))
        .ok_or_else(|| "candidate_content.category が必要です".to_string())?;
    let category = parse_itinerary_category(&text)
        .map_err(|error| format!("candidate_content.category が不正です: {error}"))?;
    Ok(Some(UpdateFieldPatch { value: category }))
}

fn parse_update_start_time_field(
    candidate: &Map<String, Value>,
) -> Result<Option<UpdateFieldPatch<String>>, String> {
    let has_start_time = candidate.contains_key("start_time");
    let has_time = candidate.contains_key("time");
    if !has_start_time && !has_time {
        return Ok(None);
    }

    let start_time = read_optional_time_value(candidate.get("start_time"), "start_time")?;
    let time = read_optional_time_value(candidate.get("time"), "time")?;
    match (start_time, time) {
        (Some(left), Some(right)) if left != right => {
            Err("start_time と time が矛盾しています".to_string())
        }
        (Some(value), _) | (_, Some(value)) => Ok(Some(UpdateFieldPatch { value })),
        (None, None) => Ok(Some(UpdateFieldPatch {
            value: String::new(),
        })),
    }
}

fn read_optional_time_value(value: Option<&Value>, field: &str) -> Result<Option<String>, String> {
    let Some(value) = value else {
        return Ok(None);
    };
    if value.is_null() {
        return Ok(None);
    }
    let text = non_empty_string(Some(value))
        .ok_or_else(|| format!("candidate_content.{field} が必要です"))?;
    parse_time_hhmm(&text).map_err(|error| error.to_string())?;
    Ok(Some(text))
}

fn parse_update_duration_minutes_field(
    candidate: &Map<String, Value>,
) -> Result<Option<i64>, String> {
    let duration_minutes =
        parse_optional_non_negative_i64(candidate.get("duration_minutes"), "duration_minutes")?;
    let duration = parse_optional_non_negative_i64(candidate.get("duration"), "duration")?;
    match (duration_minutes, duration) {
        (Some(left), Some(right)) if left != right => {
            Err("duration_minutes と duration が矛盾しています".to_string())
        }
        (Some(value), _) | (_, Some(value)) => Ok(Some(value)),
        (None, None) => Ok(None),
    }
}

fn parse_update_travel_minutes_field(
    candidate: &Map<String, Value>,
) -> Result<Option<i64>, String> {
    let travel_minutes =
        parse_optional_non_negative_i64(candidate.get("travel_minutes"), "travel_minutes")?;
    let travel_time_minutes = parse_optional_non_negative_i64(
        candidate.get("travel_time_minutes"),
        "travel_time_minutes",
    )?;
    let travel = parse_optional_non_negative_i64(candidate.get("travel"), "travel")?;
    let values = [travel_minutes, travel_time_minutes, travel]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();
    if values.is_empty() {
        return Ok(None);
    }
    let first = values[0];
    if values.iter().any(|value| *value != first) {
        return Err("travel_minutes / travel_time_minutes / travel が矛盾しています".to_string());
    }
    Ok(Some(first))
}

fn warn_unsupported_update_itinerary_candidate_keys(
    candidate: &Map<String, Value>,
    report: &mut FragmentApplyDryRunReport,
) {
    const SUPPORTED: &[&str] = &[
        "title",
        "note",
        "location",
        "category",
        "start_time",
        "time",
        "duration_minutes",
        "duration",
        "travel_minutes",
        "travel_time_minutes",
        "travel",
        "expected_title",
        "expected_note",
        "expected_location",
        "expected_category",
        "expected_start_time",
        "expected_duration_minutes",
        "expected_travel_minutes",
    ];
    for key in candidate.keys() {
        if SUPPORTED.contains(&key.as_str()) {
            continue;
        }
        push_unique(
            &mut report.warnings,
            format!(
                "unsupported_field: candidate_content.{key} は update_itinerary では未反映です"
            ),
        );
    }
}

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
        .map(|category| category.as_str().to_string())
        .unwrap_or_else(|| "-".to_string())
}

fn detect_update_itinerary_baseline_conflicts(
    candidate: &Map<String, Value>,
    current: &ExportItineraryV3,
) -> Result<(), String> {
    let checks = [
        ("expected_title", current.title.clone()),
        ("expected_note", fmt_diff_option_str(&current.note)),
        ("expected_location", fmt_diff_option_str(&current.location)),
        (
            "expected_category",
            fmt_diff_option_category(current.category),
        ),
        (
            "expected_start_time",
            fmt_diff_option_str(&current.start_time),
        ),
        (
            "expected_duration_minutes",
            fmt_diff_option_i64(current.duration_minutes),
        ),
        (
            "expected_travel_minutes",
            fmt_diff_option_i64(current.travel_minutes),
        ),
    ];

    for (expected_key, current_value) in checks {
        let Some(expected_text) = non_empty_string(candidate.get(expected_key)) else {
            continue;
        };
        if expected_text != current_value {
            return Err(format!(
                "baseline mismatch: {expected_key} ({expected_text}) が現行値 ({current_value}) と一致しません"
            ));
        }
    }
    Ok(())
}

fn build_update_itinerary_field_changes(
    current: &ExportItineraryV3,
    fields: &ParsedUpdateItineraryFields,
) -> Result<Vec<FragmentApplyItineraryFieldChange>, String> {
    let mut changes = Vec::new();

    if let Some(title) = &fields.title {
        push_itinerary_field_change(&mut changes, "title", &current.title, title);
    }
    if let Some(note) = &fields.note {
        let after = if note.value.is_empty() {
            "-".to_string()
        } else {
            note.value.clone()
        };
        push_itinerary_field_change(
            &mut changes,
            "note",
            &fmt_diff_option_str(&current.note),
            &after,
        );
    }
    if let Some(location) = &fields.location {
        let after = if location.value.is_empty() {
            "-".to_string()
        } else {
            location.value.clone()
        };
        push_itinerary_field_change(
            &mut changes,
            "location",
            &fmt_diff_option_str(&current.location),
            &after,
        );
    }
    if let Some(category) = &fields.category {
        push_itinerary_field_change(
            &mut changes,
            "category",
            &fmt_diff_option_category(current.category),
            &fmt_diff_option_category(Some(category.value)),
        );
    }
    if let Some(start_time) = &fields.start_time {
        let after = if start_time.value.is_empty() {
            "-".to_string()
        } else {
            start_time.value.clone()
        };
        push_itinerary_field_change(
            &mut changes,
            "start_time",
            &fmt_diff_option_str(&current.start_time),
            &after,
        );
    }
    if let Some(duration_minutes) = fields.duration_minutes {
        push_itinerary_field_change(
            &mut changes,
            "duration_minutes",
            &fmt_diff_option_i64(current.duration_minutes),
            &duration_minutes.to_string(),
        );
    }
    if let Some(travel_minutes) = fields.travel_minutes {
        push_itinerary_field_change(
            &mut changes,
            "travel_minutes",
            &fmt_diff_option_i64(current.travel_minutes),
            &travel_minutes.to_string(),
        );
    }

    if changes.is_empty() {
        return Err(
            "有効な itinerary 更新がありません — 現行値と同一のため no-op です".to_string(),
        );
    }
    Ok(changes)
}

fn push_itinerary_field_change(
    changes: &mut Vec<FragmentApplyItineraryFieldChange>,
    field: &str,
    before: &str,
    after: &str,
) {
    if before == after {
        return;
    }
    changes.push(FragmentApplyItineraryFieldChange {
        field: field.to_string(),
        before: before.to_string(),
        after: after.to_string(),
    });
}

fn apply_update_itinerary_patch(
    itinerary: &mut ExportItineraryV3,
    fields: &ParsedUpdateItineraryFields,
) {
    if let Some(title) = &fields.title {
        itinerary.title = title.clone();
    }
    if let Some(note) = &fields.note {
        itinerary.note = if note.value.is_empty() {
            None
        } else {
            Some(note.value.clone())
        };
    }
    if let Some(location) = &fields.location {
        itinerary.location = if location.value.is_empty() {
            None
        } else {
            Some(location.value.clone())
        };
    }
    if let Some(category) = &fields.category {
        itinerary.category = Some(category.value);
    }
    if let Some(start_time) = &fields.start_time {
        itinerary.start_time = if start_time.value.is_empty() {
            None
        } else {
            Some(start_time.value.clone())
        };
    }
    if let Some(duration_minutes) = fields.duration_minutes {
        itinerary.duration_minutes = Some(duration_minutes);
    }
    if let Some(travel_minutes) = fields.travel_minutes {
        itinerary.travel_minutes = Some(travel_minutes);
    }
}

fn reject_explicit_null_candidate_field(
    candidate: &Map<String, Value>,
    key: &str,
) -> Result<(), String> {
    if candidate.get(key) == Some(&Value::Null) {
        return Err(format!("candidate_content.{key} に null は指定できません"));
    }
    Ok(())
}

fn reject_explicit_null_update_estimate_keys(candidate: &Map<String, Value>) -> Result<(), String> {
    for key in [
        "estimate_id",
        "amount",
        "currency",
        "title",
        "note",
        "sort_order",
        "expected_amount",
        "expected_currency",
        "expected_title",
        "expected_note",
        "expected_sort_order",
        "clear_title",
        "clear_note",
    ] {
        reject_explicit_null_candidate_field(candidate, key)?;
    }
    Ok(())
}

fn parse_estimate_id_field(candidate: &Map<String, Value>) -> Result<i64, String> {
    reject_explicit_null_candidate_field(candidate, "estimate_id")?;
    match candidate.get("estimate_id") {
        None => Err("candidate_content.estimate_id が必要です".to_string()),
        Some(Value::Number(number)) => number
            .as_i64()
            .ok_or_else(|| "candidate_content.estimate_id は整数である必要があります".to_string()),
        _ => Err("candidate_content.estimate_id は整数である必要があります".to_string()),
    }
}

fn parse_clear_flag_field(candidate: &Map<String, Value>, key: &str) -> Result<bool, String> {
    reject_explicit_null_candidate_field(candidate, key)?;
    match candidate.get(key) {
        None => Ok(false),
        Some(Value::Bool(value)) => Ok(*value),
        _ => Err(format!(
            "candidate_content.{key} は真偽値である必要があります"
        )),
    }
}

fn patch_optional_estimate_text_field(
    candidate: &Map<String, Value>,
    key: &str,
) -> Result<Option<UpdateFieldPatch<String>>, String> {
    if !candidate.contains_key(key) {
        return Ok(None);
    }
    reject_explicit_null_candidate_field(candidate, key)?;
    let Value::String(text) = candidate.get(key).expect("contains key") else {
        return Err(format!(
            "candidate_content.{key} は文字列である必要があります"
        ));
    };
    Ok(Some(UpdateFieldPatch {
        value: text.clone(),
    }))
}

fn parse_update_estimate_sort_order_field(value: Option<&Value>) -> Result<Option<i64>, String> {
    match value {
        None => Ok(None),
        Some(Value::Null) => {
            Err("candidate_content.sort_order に null は指定できません".to_string())
        }
        Some(Value::Number(number)) => Ok(Some(number.as_i64().ok_or_else(|| {
            "candidate_content.sort_order は整数である必要があります".to_string()
        })?)),
        _ => Err("candidate_content.sort_order は整数である必要があります".to_string()),
    }
}

fn warn_unsupported_update_estimate_candidate_keys(
    candidate: &Map<String, Value>,
    report: &mut FragmentApplyDryRunReport,
) {
    const SUPPORTED: &[&str] = &[
        "estimate_id",
        "amount",
        "currency",
        "title",
        "note",
        "sort_order",
        "clear_title",
        "clear_note",
        "expected_amount",
        "expected_currency",
        "expected_title",
        "expected_note",
        "expected_sort_order",
    ];
    for key in candidate.keys() {
        if SUPPORTED.contains(&key.as_str()) {
            continue;
        }
        push_unique(
            &mut report.warnings,
            format!("unsupported_field: candidate_content.{key} は update_estimate では未反映です"),
        );
    }
}

fn parse_update_estimate_fields(
    fragment: &Map<String, Value>,
    report: Option<&mut FragmentApplyDryRunReport>,
) -> Result<ParsedUpdateEstimateFields, String> {
    let candidate = fragment
        .get("candidate_content")
        .and_then(Value::as_object)
        .ok_or_else(|| "candidate_content object が必要です".to_string())?;

    if let Some(report) = report {
        warn_unsupported_update_estimate_candidate_keys(candidate, report);
    }

    reject_explicit_null_update_estimate_keys(candidate)?;

    for unsupported_clear in ["clear_amount", "clear_currency", "clear_sort_order"] {
        if candidate.contains_key(unsupported_clear) {
            return Err(format!(
                "candidate_content.{unsupported_clear} は update_estimate では未対応です"
            ));
        }
    }

    let estimate_id = parse_estimate_id_field(candidate)?;
    let has_amount = candidate.contains_key("amount");
    let has_currency = candidate.contains_key("currency");
    let amount_raw = if has_amount {
        Some(candidate.get("amount").cloned().expect("amount key"))
    } else {
        None
    };
    let currency_text = if has_currency {
        Some(
            non_empty_string(candidate.get("currency"))
                .ok_or_else(|| "candidate_content.currency が必要です".to_string())?,
        )
    } else {
        None
    };

    if has_currency && !has_amount {
        return Err("currency を変更する場合は amount も指定してください".to_string());
    }

    let title = patch_optional_estimate_text_field(candidate, "title")?;
    let note = patch_optional_estimate_text_field(candidate, "note")?;
    let sort_order = parse_update_estimate_sort_order_field(candidate.get("sort_order"))?;
    let clear_title = parse_clear_flag_field(candidate, "clear_title")?;
    let clear_note = parse_clear_flag_field(candidate, "clear_note")?;

    if title.is_some() && clear_title {
        return Err("title と clear_title は同時に指定できません".to_string());
    }
    if note.is_some() && clear_note {
        return Err("note と clear_note は同時に指定できません".to_string());
    }

    let has_update_field = has_amount
        || has_currency
        || title.is_some()
        || note.is_some()
        || sort_order.is_some()
        || clear_title
        || clear_note;
    if !has_update_field {
        return Err("update_estimate には少なくとも 1 つの更新フィールドが必要です".to_string());
    }

    Ok(ParsedUpdateEstimateFields {
        estimate_id,
        has_amount,
        amount_raw,
        has_currency,
        currency_text,
        title,
        note,
        sort_order,
        clear_title,
        clear_note,
    })
}

fn parse_update_estimate_amount_value(value: &Value, currency: &str) -> Result<i64, String> {
    match value {
        Value::Number(number) => {
            let amount = number
                .as_i64()
                .ok_or_else(|| "candidate_content.amount は整数である必要があります".to_string())?;
            if amount < 0 {
                return Err("candidate_content.amount は 0 以上である必要があります".to_string());
            }
            Ok(amount)
        }
        Value::String(text) => parse_amount_for_currency(text, currency)
            .map_err(|error| format!("candidate_content.amount が不正です: {error}")),
        _ => Err("candidate_content.amount は数値または文字列である必要があります".to_string()),
    }
}

fn compute_update_estimate_proposed(
    current: &Estimate,
    fields: &ParsedUpdateEstimateFields,
) -> Result<Estimate, String> {
    let mut proposed = current.clone();

    if fields.clear_title {
        proposed.title = None;
    } else if let Some(title) = &fields.title {
        proposed.title = normalize_optional_text(Some(&title.value));
    }

    if fields.clear_note {
        proposed.note = None;
    } else if let Some(note) = &fields.note {
        proposed.note = normalize_optional_text(Some(&note.value));
    }

    if let Some(sort_order) = fields.sort_order {
        proposed.sort_order = sort_order;
    }

    let currency_for_amount = if fields.has_currency {
        fields
            .currency_text
            .as_deref()
            .ok_or_else(|| "candidate_content.currency が必要です".to_string())?
    } else {
        current.currency.as_str()
    };

    if fields.has_currency {
        proposed.currency = validate_currency_code(currency_for_amount)
            .map_err(|error| format!("candidate_content.currency が不正です: {error}"))?;
    }

    if fields.has_amount {
        let amount_raw = fields
            .amount_raw
            .as_ref()
            .ok_or_else(|| "candidate_content.amount が必要です".to_string())?;
        proposed.amount = parse_update_estimate_amount_value(amount_raw, currency_for_amount)?;
    }

    Ok(proposed)
}

fn detect_update_estimate_baseline_conflicts(
    candidate: &Map<String, Value>,
    current: &Estimate,
) -> Result<(), String> {
    let checks = [
        ("expected_amount", current.amount.to_string()),
        ("expected_currency", current.currency.clone()),
        ("expected_title", fmt_diff_option_str(&current.title)),
        ("expected_note", fmt_diff_option_str(&current.note)),
        (
            "expected_sort_order",
            fmt_diff_option_i64(Some(current.sort_order)),
        ),
    ];

    for (expected_key, current_value) in checks {
        let Some(expected_text) = non_empty_string(candidate.get(expected_key)) else {
            continue;
        };
        if expected_text != current_value {
            return Err(format!(
                "baseline mismatch: {expected_key} ({expected_text}) が現行値 ({current_value}) と一致しません"
            ));
        }
    }
    Ok(())
}

fn reject_explicit_null_delete_estimate_keys(candidate: &Map<String, Value>) -> Result<(), String> {
    for key in [
        "estimate_id",
        "expected_amount",
        "expected_currency",
        "expected_title",
        "expected_note",
        "expected_sort_order",
        "expected_updated_at",
    ] {
        reject_explicit_null_candidate_field(candidate, key)?;
    }
    Ok(())
}

fn reject_delete_estimate_mutation_fields(candidate: &Map<String, Value>) -> Result<(), String> {
    for key in [
        "amount",
        "currency",
        "title",
        "note",
        "sort_order",
        "clear_title",
        "clear_note",
        "clear_amount",
        "clear_currency",
        "clear_sort_order",
    ] {
        if candidate.contains_key(key) {
            return Err(format!(
                "candidate_content.{key} は delete_estimate では指定できません"
            ));
        }
    }
    Ok(())
}

fn warn_unsupported_delete_estimate_candidate_keys(
    candidate: &Map<String, Value>,
    report: &mut FragmentApplyDryRunReport,
) {
    const SUPPORTED: &[&str] = &[
        "estimate_id",
        "expected_amount",
        "expected_currency",
        "expected_title",
        "expected_note",
        "expected_sort_order",
        "expected_updated_at",
    ];
    for key in candidate.keys() {
        if SUPPORTED.contains(&key.as_str()) {
            continue;
        }
        push_unique(
            &mut report.warnings,
            format!("unsupported_field: candidate_content.{key} は delete_estimate では未反映です"),
        );
    }
}

fn parse_delete_estimate_fields(
    fragment: &Map<String, Value>,
    report: Option<&mut FragmentApplyDryRunReport>,
) -> Result<i64, String> {
    let candidate = fragment
        .get("candidate_content")
        .and_then(Value::as_object)
        .ok_or_else(|| "candidate_content object が必要です".to_string())?;

    if let Some(report) = report {
        warn_unsupported_delete_estimate_candidate_keys(candidate, report);
    }

    reject_explicit_null_delete_estimate_keys(candidate)?;
    reject_delete_estimate_mutation_fields(candidate)?;

    let estimate_id = parse_estimate_id_field(candidate)?;
    if estimate_id <= 0 {
        return Err("candidate_content.estimate_id は正の整数である必要があります".to_string());
    }
    Ok(estimate_id)
}

fn detect_delete_estimate_baseline_conflicts(
    candidate: &Map<String, Value>,
    current: &Estimate,
) -> Result<(), String> {
    let checks = [
        ("expected_amount", current.amount.to_string()),
        ("expected_currency", current.currency.clone()),
        ("expected_title", fmt_diff_option_str(&current.title)),
        ("expected_note", fmt_diff_option_str(&current.note)),
        (
            "expected_sort_order",
            fmt_diff_option_i64(Some(current.sort_order)),
        ),
        ("expected_updated_at", current.updated_at.clone()),
    ];

    for (expected_key, current_value) in checks {
        let Some(expected_text) = non_empty_string(candidate.get(expected_key)) else {
            continue;
        };
        if expected_text != current_value {
            return Err(format!(
                "baseline mismatch: {expected_key} ({expected_text}) が現行値 ({current_value}) と一致しません"
            ));
        }
    }
    Ok(())
}

fn build_estimate_delete_preview(
    target_itinerary_id: i64,
    target_itinerary_title: String,
    current: &Estimate,
) -> FragmentApplyEstimateDeletePreview {
    FragmentApplyEstimateDeletePreview {
        target_itinerary_id,
        target_itinerary_title,
        target_estimate_id: current.id,
        title: current.title.clone(),
        amount: current.amount,
        currency: current.currency.clone(),
        note: current.note.clone(),
        sort_order: current.sort_order,
        updated_at: current.updated_at.clone(),
    }
}

fn estimate_display_value_for_field(field: &str, estimate: &Estimate) -> String {
    match field {
        "amount" => estimate.amount.to_string(),
        "currency" => estimate.currency.clone(),
        "title" => fmt_diff_option_str(&estimate.title),
        "note" => fmt_diff_option_str(&estimate.note),
        "sort_order" => estimate.sort_order.to_string(),
        _ => "-".to_string(),
    }
}

fn push_estimate_field_change(
    changes: &mut Vec<FragmentApplyEstimateFieldChange>,
    field: &str,
    before: &Estimate,
    after: &Estimate,
) {
    changes.push(FragmentApplyEstimateFieldChange {
        field: field.to_string(),
        before: estimate_display_value_for_field(field, before),
        after: estimate_display_value_for_field(field, after),
    });
}

fn build_update_estimate_field_changes(
    current: &Estimate,
    proposed: &Estimate,
    fields: &ParsedUpdateEstimateFields,
) -> Vec<FragmentApplyEstimateFieldChange> {
    let mut changes = Vec::new();
    if fields.has_amount {
        push_estimate_field_change(&mut changes, "amount", current, proposed);
    }
    if fields.has_currency {
        push_estimate_field_change(&mut changes, "currency", current, proposed);
    }
    if fields.title.is_some() || fields.clear_title {
        push_estimate_field_change(&mut changes, "title", current, proposed);
    }
    if fields.note.is_some() || fields.clear_note {
        push_estimate_field_change(&mut changes, "note", current, proposed);
    }
    if fields.sort_order.is_some() {
        push_estimate_field_change(&mut changes, "sort_order", current, proposed);
    }
    changes
}

fn find_estimate_index_in_itinerary(
    conn: &Connection,
    itinerary_id: i64,
    estimate_id: i64,
) -> Result<usize, String> {
    let estimates = list_estimates_for_itinerary(conn, itinerary_id)
        .map_err(|error| format!("Estimate 一覧の取得に失敗しました: {error}"))?;
    estimates
        .iter()
        .position(|estimate| estimate.id == estimate_id)
        .ok_or_else(|| format!("Estimate not found: {estimate_id}"))
}

fn apply_update_estimate_patch(export_estimate: &mut ExportEstimateV3, proposed: &Estimate) {
    export_estimate.title = proposed.title.clone();
    export_estimate.amount = proposed.amount;
    export_estimate.currency = proposed.currency.clone();
    export_estimate.note = proposed.note.clone();
    export_estimate.sort_order = proposed.sort_order;
}

#[allow(clippy::too_many_arguments)]
fn apply_update_estimate_preview(
    conn: &Connection,
    export: &mut TripExportV3,
    resolved: &ResolvedApplyTarget,
    fragment: &Map<String, Value>,
    intent: &str,
    itineraries_before: usize,
    estimates_before: usize,
    report: &mut FragmentApplyDryRunReport,
) -> Result<FragmentApplyPreviewSummary, String> {
    if resolved.target_type != "itinerary" {
        return Err(
            "update_estimate は itinerary target のみサポートしています（trip / day は未対応）"
                .to_string(),
        );
    }
    if resolved.resolution == "ambiguous" {
        return Err("target が曖昧です — apply preview を続行しません".to_string());
    }

    let day_number = resolved.day_number.ok_or_else(|| {
        "update_estimate の Itinerary target が解決されていません（day）".to_string()
    })?;
    let itinerary_sort_order = resolved.itinerary_sort_order.ok_or_else(|| {
        "update_estimate の Itinerary target が解決されていません（sort_order）".to_string()
    })?;
    let target_itinerary_id =
        lookup_itinerary_db_id_from_resolved(conn, resolved.trip_id, resolved)?;
    let target_itinerary_title = resolved.itinerary_title.clone().ok_or_else(|| {
        "update_estimate の Itinerary target が解決されていません（title）".to_string()
    })?;

    let fields = parse_update_estimate_fields(fragment, Some(report))?;
    let candidate = fragment
        .get("candidate_content")
        .and_then(Value::as_object)
        .ok_or_else(|| "candidate_content object が必要です".to_string())?;

    let current = get_estimate(conn, fields.estimate_id)
        .map_err(|error| format!("Estimate not found: {error}"))?;
    if current.itinerary_id != target_itinerary_id {
        return Err(format!(
            "Estimate {id} は Itinerary {target_itinerary_id} 配下ではありません",
            id = fields.estimate_id
        ));
    }

    detect_update_estimate_baseline_conflicts(candidate, &current)?;
    let proposed = compute_update_estimate_proposed(&current, &fields)?;
    let changes = build_update_estimate_field_changes(&current, &proposed, &fields);

    ensure_day_in_range(export, day_number)?;
    let estimate_index =
        find_estimate_index_in_itinerary(conn, target_itinerary_id, fields.estimate_id)?;
    let day = find_or_create_day(export, day_number);
    let itinerary = find_itinerary_mut_in_export_day(day, itinerary_sort_order).ok_or_else(|| {
        format!(
            "preview 内に itinerary (day {day_number}, sort_order {itinerary_sort_order}) が見つかりません"
        )
    })?;
    let export_estimate = itinerary
        .estimates
        .get_mut(estimate_index)
        .ok_or_else(|| format!("preview 内に estimate index {estimate_index} が見つかりません"))?;
    apply_update_estimate_patch(export_estimate, &proposed);

    Ok(FragmentApplyPreviewSummary {
        intent: intent.to_string(),
        action: "update_estimate".to_string(),
        candidate_title: proposed.title.clone(),
        itineraries_before,
        itineraries_after: itineraries_before,
        notes_before: None,
        notes_after: None,
        expenses_before: None,
        expenses_after: None,
        expense_preview: None,
        estimates_before: Some(estimates_before),
        estimates_after: Some(estimates_before),
        estimate_preview: None,
        estimate_update_preview: Some(FragmentApplyEstimateUpdatePreview {
            target_itinerary_id,
            target_itinerary_title,
            target_estimate_id: fields.estimate_id,
        }),
        estimate_field_changes: Some(changes),
        reservations_before: None,
        reservations_after: None,
        reservation_preview: None,
        itinerary_field_changes: None,
        reorder_preview: None,
        move_preview: None,
        delete_preview: None,
        estimate_delete_preview: None,
    })
}

#[allow(clippy::too_many_arguments)]
fn apply_delete_estimate_preview(
    conn: &Connection,
    export: &mut TripExportV3,
    resolved: &ResolvedApplyTarget,
    fragment: &Map<String, Value>,
    intent: &str,
    itineraries_before: usize,
    estimates_before: usize,
    report: &mut FragmentApplyDryRunReport,
) -> Result<FragmentApplyPreviewSummary, String> {
    if resolved.target_type != "itinerary" {
        return Err(
            "delete_estimate は itinerary target のみサポートしています（trip / day は未対応）"
                .to_string(),
        );
    }
    if resolved.resolution == "ambiguous" {
        return Err("target が曖昧です — apply preview を続行しません".to_string());
    }

    let day_number = resolved.day_number.ok_or_else(|| {
        "delete_estimate の Itinerary target が解決されていません（day）".to_string()
    })?;
    let itinerary_sort_order = resolved.itinerary_sort_order.ok_or_else(|| {
        "delete_estimate の Itinerary target が解決されていません（sort_order）".to_string()
    })?;
    let target_itinerary_id =
        lookup_itinerary_db_id_from_resolved(conn, resolved.trip_id, resolved)?;
    let target_itinerary_title = resolved.itinerary_title.clone().ok_or_else(|| {
        "delete_estimate の Itinerary target が解決されていません（title）".to_string()
    })?;

    let estimate_id = parse_delete_estimate_fields(fragment, Some(report))?;
    let candidate = fragment
        .get("candidate_content")
        .and_then(Value::as_object)
        .ok_or_else(|| "candidate_content object が必要です".to_string())?;

    let current =
        get_estimate(conn, estimate_id).map_err(|error| format!("Estimate not found: {error}"))?;
    if current.itinerary_id != target_itinerary_id {
        return Err(format!(
            "Estimate {id} は Itinerary {target_itinerary_id} 配下ではありません",
            id = estimate_id
        ));
    }

    detect_delete_estimate_baseline_conflicts(candidate, &current)?;
    let delete_preview =
        build_estimate_delete_preview(target_itinerary_id, target_itinerary_title, &current);

    push_unique(
        &mut report.warnings,
        "delete_estimate は hard delete です — confirm 成功後の undo は保証されません".to_string(),
    );

    ensure_day_in_range(export, day_number)?;
    let estimate_index = find_estimate_index_in_itinerary(conn, target_itinerary_id, estimate_id)?;
    let day = find_or_create_day(export, day_number);
    let itinerary = find_itinerary_mut_in_export_day(day, itinerary_sort_order).ok_or_else(|| {
        format!(
            "preview 内に itinerary (day {day_number}, sort_order {itinerary_sort_order}) が見つかりません"
        )
    })?;
    itinerary.estimates.remove(estimate_index);
    let estimates_after = count_estimates(export);

    Ok(FragmentApplyPreviewSummary {
        intent: intent.to_string(),
        action: "delete_estimate".to_string(),
        candidate_title: current.title.clone(),
        itineraries_before,
        itineraries_after: itineraries_before,
        notes_before: None,
        notes_after: None,
        expenses_before: None,
        expenses_after: None,
        expense_preview: None,
        estimates_before: Some(estimates_before),
        estimates_after: Some(estimates_after),
        estimate_preview: None,
        estimate_update_preview: None,
        estimate_field_changes: None,
        estimate_delete_preview: Some(delete_preview),
        reservations_before: None,
        reservations_after: None,
        reservation_preview: None,
        itinerary_field_changes: None,
        reorder_preview: None,
        move_preview: None,
        delete_preview: None,
    })
}

fn apply_update_itinerary_preview(
    export: &mut TripExportV3,
    resolved: &ResolvedApplyTarget,
    fragment: &Map<String, Value>,
    intent: &str,
    itineraries_before: usize,
    report: &mut FragmentApplyDryRunReport,
) -> Result<FragmentApplyPreviewSummary, String> {
    if resolved.target_type != "itinerary" {
        return Err(
            "update_itinerary は itinerary target のみサポートしています（trip / day は未対応）"
                .to_string(),
        );
    }
    if resolved.resolution == "ambiguous" {
        return Err("target が曖昧です — apply preview を続行しません".to_string());
    }

    let day_number = resolved.day_number.ok_or_else(|| {
        "update_itinerary の Itinerary target が解決されていません（day）".to_string()
    })?;
    let itinerary_sort_order = resolved.itinerary_sort_order.ok_or_else(|| {
        "update_itinerary の Itinerary target が解決されていません（sort_order）".to_string()
    })?;
    ensure_day_in_range(export, day_number)?;

    let fields = parse_update_itinerary_fields(fragment, Some(report))?;
    let candidate = fragment
        .get("candidate_content")
        .and_then(Value::as_object)
        .ok_or_else(|| "candidate_content object が必要です".to_string())?;

    let current = lookup_itinerary_in_export(export, day_number, itinerary_sort_order)
        .ok_or_else(|| {
            format!(
                "preview 内に itinerary (day {day_number}, sort_order {itinerary_sort_order}) が見つかりません"
            )
        })?
        .clone();

    detect_update_itinerary_baseline_conflicts(candidate, &current)?;
    let changes = build_update_itinerary_field_changes(&current, &fields)?;

    let day = find_or_create_day(export, day_number);
    let itinerary = find_itinerary_mut_in_export_day(day, itinerary_sort_order).ok_or_else(|| {
        format!(
            "preview 内に itinerary (day {day_number}, sort_order {itinerary_sort_order}) が見つかりません"
        )
    })?;
    apply_update_itinerary_patch(itinerary, &fields);

    Ok(FragmentApplyPreviewSummary {
        intent: intent.to_string(),
        action: "update_itinerary".to_string(),
        candidate_title: fields
            .title
            .clone()
            .or_else(|| resolved.itinerary_title.clone()),
        itineraries_before,
        itineraries_after: itineraries_before,
        notes_before: None,
        notes_after: None,
        expenses_before: None,
        expenses_after: None,
        expense_preview: None,
        estimates_before: None,
        estimates_after: None,
        estimate_preview: None,
        estimate_update_preview: None,
        estimate_field_changes: None,
        estimate_delete_preview: None,
        reservations_before: None,
        reservations_after: None,
        reservation_preview: None,
        itinerary_field_changes: Some(changes),
        reorder_preview: None,
        move_preview: None,
        delete_preview: None,
    })
}

fn append_sort_order_for_export_itinerary(itinerary: &ExportItineraryV3) -> i64 {
    itinerary
        .expenses
        .iter()
        .map(|expense| expense.sort_order)
        .max()
        .unwrap_or(0)
        + 1
}

fn build_export_expense_from_add_fields(
    fields: &ParsedAddExpenseFields,
    sort_order: i64,
) -> ExportExpenseV3 {
    ExportExpenseV3 {
        title: fields.title.clone(),
        amount: fields.amount,
        currency: fields.currency.clone(),
        paid_by_name: None,
        paid_by_participant_ref: None,
        beneficiaries: Vec::new(),
        expense_date: None,
        note: fields.note.clone(),
        sort_order,
    }
}

fn build_export_estimate_from_add_fields(fields: &ParsedAddEstimateFields) -> ExportEstimateV3 {
    ExportEstimateV3 {
        title: fields.title.clone(),
        amount: fields.amount,
        currency: fields.currency.clone(),
        note: fields.note.clone(),
        sort_order: fields.sort_order,
    }
}

fn lookup_itinerary_db_id_from_resolved(
    conn: &Connection,
    trip_id: i64,
    resolved: &ResolvedApplyTarget,
) -> Result<i64, String> {
    let day_number = resolved.day_number.ok_or_else(|| {
        "add_estimate の Itinerary target が解決されていません（day）".to_string()
    })?;
    let itinerary_sort_order = resolved.itinerary_sort_order.ok_or_else(|| {
        "add_estimate の Itinerary target が解決されていません（sort_order）".to_string()
    })?;
    let items = crate::itinerary::list_itinerary_items_for_day(conn, trip_id, day_number)
        .map_err(|error| format!("Itinerary target の解決に失敗しました: {error}"))?;
    items
        .iter()
        .find(|item| item.sort_order == itinerary_sort_order)
        .map(|item| item.id)
        .ok_or_else(|| {
            format!(
                "target itinerary (day {day_number}, sort_order {itinerary_sort_order}) の DB ID を解決できません"
            )
        })
}

fn find_itinerary_mut_in_export_day(
    day: &mut ExportDayV3,
    sort_order: i64,
) -> Option<&mut ExportItineraryV3> {
    day.itineraries
        .iter_mut()
        .find(|item| item.sort_order == sort_order)
}

fn parse_add_note_fields(
    fragment: &Map<String, Value>,
    report: Option<&mut FragmentApplyDryRunReport>,
) -> Result<ParsedAddNoteFields, String> {
    if let Some(report) = report {
        if let Some(candidate) = fragment.get("candidate_content").and_then(Value::as_object) {
            warn_unsupported_add_note_candidate_keys(candidate, report);
        }
    }

    let body = non_empty_string(
        fragment
            .get("candidate_content")
            .and_then(Value::as_object)
            .and_then(|obj| obj.get("body")),
    )
    .or_else(|| {
        fragment
            .get("candidate_content")
            .and_then(|value| non_empty_string(Some(value)))
    })
    .or_else(|| non_empty_string(fragment.get("notes")))
    .ok_or_else(|| {
        "add_note には candidate_content.body または fragment.notes が必要です".to_string()
    })?;

    let title = fragment
        .get("candidate_content")
        .and_then(Value::as_object)
        .and_then(|obj| non_empty_string(obj.get("title")));

    Ok(ParsedAddNoteFields { title, body })
}

fn warn_unsupported_add_note_candidate_keys(
    candidate: &Map<String, Value>,
    report: &mut FragmentApplyDryRunReport,
) {
    const SUPPORTED: &[&str] = &["title", "body"];
    for key in candidate.keys() {
        if SUPPORTED.contains(&key.as_str()) {
            continue;
        }
        push_unique(
            &mut report.warnings,
            format!("unsupported_field: candidate_content.{key} は add_note では未反映です"),
        );
    }
}

fn build_export_note_from_add_fields(
    fields: &ParsedAddNoteFields,
    resolved: &ResolvedApplyTarget,
    export: &TripExportV3,
) -> Result<ExportNote, String> {
    match resolved.target_type.as_str() {
        "trip" => Ok(ExportNote::Trip {
            title: fields.title.clone(),
            body: fields.body.clone(),
        }),
        "day" => {
            let day_number = resolved
                .day_number
                .ok_or_else(|| "add_note の Day target が解決されていません".to_string())?;
            Ok(ExportNote::Day {
                day_number,
                title: fields.title.clone(),
                body: fields.body.clone(),
            })
        }
        "itinerary" => {
            let day_number = resolved.day_number.ok_or_else(|| {
                "add_note の Itinerary target が解決されていません（day）".to_string()
            })?;
            let sort_order = resolved.itinerary_sort_order.ok_or_else(|| {
                "add_note の Itinerary target が解決されていません（sort_order）".to_string()
            })?;
            let itinerary_title = resolved.itinerary_title.clone().ok_or_else(|| {
                "add_note の Itinerary target が解決されていません（title）".to_string()
            })?;
            let start_time = lookup_itinerary_in_export(export, day_number, sort_order)
                .and_then(|item| item.start_time.clone());
            Ok(ExportNote::Itinerary {
                itinerary_key: ItineraryNoteKey {
                    day_number,
                    sort_order,
                    start_time,
                    title: itinerary_title,
                },
                title: fields.title.clone(),
                body: fields.body.clone(),
            })
        }
        other => Err(format!(
            "add_note は trip / day / itinerary target のみサポートしています（現在: {other}）"
        )),
    }
}

fn lookup_itinerary_in_export(
    export: &TripExportV3,
    day_number: i64,
    sort_order: i64,
) -> Option<&ExportItineraryV3> {
    export
        .days
        .iter()
        .find(|day| day.day_number == day_number)
        .and_then(|day| {
            day.itineraries
                .iter()
                .find(|item| item.sort_order == sort_order)
        })
}

#[allow(clippy::too_many_arguments)]
fn simulate_apply_preview(
    conn: &Connection,
    trip_id: i64,
    export: &mut TripExportV3,
    resolved: &ResolvedApplyTarget,
    fragment: &Map<String, Value>,
    intent: &str,
    itineraries_before: usize,
    notes_before: usize,
    expenses_before: usize,
    reservations_before: usize,
    estimates_before: usize,
    report: &mut FragmentApplyDryRunReport,
) -> Result<FragmentApplyPreviewSummary, String> {
    let candidate = fragment.get("candidate_content");
    let candidate_title = candidate
        .and_then(|value| {
            value
                .as_object()
                .and_then(|obj| non_empty_string(obj.get("title")))
        })
        .or_else(|| non_empty_string(candidate));

    match intent {
        "add" => {
            let day_number = resolved.day_number.ok_or_else(|| {
                "intent が add ですが target Day が解決されていません".to_string()
            })?;
            ensure_day_in_range(export, day_number)?;
            let fields = parse_add_itinerary_fields(fragment, Some(report))?;
            let day = find_or_create_day(export, day_number);
            let sort_order = append_sort_order_for_export_day(day);
            day.itineraries
                .push(build_export_itinerary_from_add_fields(&fields, sort_order));
            let itineraries_after = count_itineraries(export);
            Ok(FragmentApplyPreviewSummary {
                intent: intent.to_string(),
                action: "add_itinerary".to_string(),
                candidate_title: Some(fields.title),
                itineraries_before,
                itineraries_after,
                notes_before: None,
                notes_after: None,
                expenses_before: None,
                expenses_after: None,
                expense_preview: None,
                estimates_before: None,
                estimates_after: None,
                estimate_preview: None,
                estimate_update_preview: None,
                estimate_field_changes: None,
                estimate_delete_preview: None,
                reservations_before: None,
                reservations_after: None,
                reservation_preview: None,
                itinerary_field_changes: None,
                reorder_preview: None,
                move_preview: None,
                delete_preview: None,
            })
        }
        "add_note" => {
            let fields = parse_add_note_fields(fragment, Some(report))?;
            let export_note = build_export_note_from_add_fields(&fields, resolved, export)?;
            let notes = export.notes.get_or_insert_with(Vec::new);
            notes.push(export_note);
            Ok(FragmentApplyPreviewSummary {
                intent: intent.to_string(),
                action: "add_note".to_string(),
                candidate_title: fields.title.clone(),
                itineraries_before,
                itineraries_after: itineraries_before,
                notes_before: Some(notes_before),
                notes_after: Some(notes_before + 1),
                expenses_before: None,
                expenses_after: None,
                expense_preview: None,
                estimates_before: None,
                estimates_after: None,
                estimate_preview: None,
                estimate_update_preview: None,
                estimate_field_changes: None,
                estimate_delete_preview: None,
                reservations_before: None,
                reservations_after: None,
                reservation_preview: None,
                itinerary_field_changes: None,
                reorder_preview: None,
                move_preview: None,
                delete_preview: None,
            })
        }
        "add_expense" => {
            if resolved.target_type != "itinerary" {
                return Err(
                    "add_expense は itinerary target のみサポートしています（trip / day は未対応）"
                        .to_string(),
                );
            }
            let day_number = resolved.day_number.ok_or_else(|| {
                "add_expense の Itinerary target が解決されていません（day）".to_string()
            })?;
            let itinerary_sort_order = resolved.itinerary_sort_order.ok_or_else(|| {
                "add_expense の Itinerary target が解決されていません（sort_order）".to_string()
            })?;
            ensure_day_in_range(export, day_number)?;
            let fields = parse_add_expense_fields(fragment, Some(report))?;
            let day = find_or_create_day(export, day_number);
            let itinerary = find_itinerary_mut_in_export_day(day, itinerary_sort_order)
                .ok_or_else(|| {
                    format!(
                        "preview 内に itinerary (day {day_number}, sort_order {itinerary_sort_order}) が見つかりません"
                    )
                })?;
            let expense_sort_order = append_sort_order_for_export_itinerary(itinerary);
            let export_expense = build_export_expense_from_add_fields(&fields, expense_sort_order);
            itinerary.expenses.push(export_expense);
            let expenses_after = count_expenses(export);
            Ok(FragmentApplyPreviewSummary {
                intent: intent.to_string(),
                action: "add_expense".to_string(),
                candidate_title: fields.title.clone(),
                itineraries_before,
                itineraries_after: itineraries_before,
                notes_before: None,
                notes_after: None,
                expenses_before: Some(expenses_before),
                expenses_after: Some(expenses_after),
                expense_preview: Some(FragmentApplyExpensePreview {
                    amount: fields.amount,
                    currency: fields.currency.clone(),
                    title: fields.title.clone(),
                    note: fields.note.clone(),
                }),
                estimates_before: None,
                estimates_after: None,
                estimate_preview: None,
                estimate_update_preview: None,
                estimate_field_changes: None,
                estimate_delete_preview: None,
                reservations_before: None,
                reservations_after: None,
                reservation_preview: None,
                itinerary_field_changes: None,
                reorder_preview: None,
                move_preview: None,
                delete_preview: None,
            })
        }
        "add_estimate" => {
            if resolved.target_type != "itinerary" {
                return Err(
                    "add_estimate は itinerary target のみサポートしています（trip / day は未対応）"
                        .to_string(),
                );
            }
            let day_number = resolved.day_number.ok_or_else(|| {
                "add_estimate の Itinerary target が解決されていません（day）".to_string()
            })?;
            let itinerary_sort_order = resolved.itinerary_sort_order.ok_or_else(|| {
                "add_estimate の Itinerary target が解決されていません（sort_order）".to_string()
            })?;
            let target_itinerary_id =
                lookup_itinerary_db_id_from_resolved(conn, trip_id, resolved)?;
            let target_itinerary_title = resolved.itinerary_title.clone().ok_or_else(|| {
                "add_estimate の Itinerary target が解決されていません（title）".to_string()
            })?;
            ensure_day_in_range(export, day_number)?;
            let fields = parse_add_estimate_fields(fragment, Some(report))?;
            let day = find_or_create_day(export, day_number);
            let itinerary = find_itinerary_mut_in_export_day(day, itinerary_sort_order)
                .ok_or_else(|| {
                    format!(
                        "preview 内に itinerary (day {day_number}, sort_order {itinerary_sort_order}) が見つかりません"
                    )
                })?;
            let export_estimate = build_export_estimate_from_add_fields(&fields);
            itinerary.estimates.push(export_estimate);
            let estimates_after = count_estimates(export);
            Ok(FragmentApplyPreviewSummary {
                intent: intent.to_string(),
                action: "add_estimate".to_string(),
                candidate_title: fields.title.clone(),
                itineraries_before,
                itineraries_after: itineraries_before,
                notes_before: None,
                notes_after: None,
                expenses_before: None,
                expenses_after: None,
                expense_preview: None,
                estimates_before: Some(estimates_before),
                estimates_after: Some(estimates_after),
                estimate_preview: Some(FragmentApplyEstimatePreview {
                    target_itinerary_id,
                    target_itinerary_title,
                    amount: fields.amount,
                    currency: fields.currency.clone(),
                    title: fields.title.clone(),
                    note: fields.note.clone(),
                    sort_order: fields.sort_order,
                }),
                estimate_update_preview: None,
                estimate_field_changes: None,
                estimate_delete_preview: None,
                reservations_before: None,
                reservations_after: None,
                reservation_preview: None,
                itinerary_field_changes: None,
                reorder_preview: None,
                move_preview: None,
                delete_preview: None,
            })
        }
        "add_reservation" => {
            if resolved.target_type != "itinerary" {
                return Err(
                    "add_reservation は itinerary target のみサポートしています（trip / day は未対応）"
                        .to_string(),
                );
            }
            let day_number = resolved.day_number.ok_or_else(|| {
                "add_reservation の Itinerary target が解決されていません（day）".to_string()
            })?;
            let itinerary_sort_order = resolved.itinerary_sort_order.ok_or_else(|| {
                "add_reservation の Itinerary target が解決されていません（sort_order）".to_string()
            })?;
            ensure_day_in_range(export, day_number)?;
            let fields = parse_add_reservation_fields(fragment, Some(report))?;
            let day = find_or_create_day(export, day_number);
            let itinerary = find_itinerary_mut_in_export_day(day, itinerary_sort_order)
                .ok_or_else(|| {
                    format!(
                        "preview 内に itinerary (day {day_number}, sort_order {itinerary_sort_order}) が見つかりません"
                    )
                })?;
            itinerary
                .reservations
                .push(build_export_reservation_from_add_fields(&fields));
            let reservations_after = count_reservations(export);
            Ok(FragmentApplyPreviewSummary {
                intent: intent.to_string(),
                action: "add_reservation".to_string(),
                candidate_title: Some(fields.provider_name.clone()),
                itineraries_before,
                itineraries_after: itineraries_before,
                notes_before: None,
                notes_after: None,
                expenses_before: None,
                expenses_after: None,
                expense_preview: None,
                estimates_before: None,
                estimates_after: None,
                estimate_preview: None,
                estimate_update_preview: None,
                estimate_field_changes: None,
                estimate_delete_preview: None,
                reservations_before: Some(reservations_before),
                reservations_after: Some(reservations_after),
                reservation_preview: Some(FragmentApplyReservationPreview {
                    reservation_type: fields.reservation_type,
                    provider_name: fields.provider_name,
                    confirmation_code: fields.confirmation_code,
                    reservation_site_url: fields.reservation_site_url,
                    remark: fields.remark,
                    start_at: fields.start_at,
                    end_at: fields.end_at,
                }),
                itinerary_field_changes: None,
                reorder_preview: None,
                move_preview: None,
                delete_preview: None,
            })
        }
        "update_itinerary" => apply_update_itinerary_preview(
            export,
            resolved,
            fragment,
            intent,
            itineraries_before,
            report,
        ),
        "update_estimate" => apply_update_estimate_preview(
            conn,
            export,
            resolved,
            fragment,
            intent,
            itineraries_before,
            estimates_before,
            report,
        ),
        "delete_estimate" => apply_delete_estimate_preview(
            conn,
            export,
            resolved,
            fragment,
            intent,
            itineraries_before,
            estimates_before,
            report,
        ),
        "delete_itinerary" => apply_delete_itinerary_preview(
            conn,
            trip_id,
            export,
            resolved,
            intent,
            itineraries_before,
            report,
        ),
        "enrich" => {
            if let Some(day_number) = resolved.day_number {
                ensure_day_in_range(export, day_number)?;
                if let Some(summary) = candidate
                    .and_then(|v| v.as_object())
                    .and_then(|obj| non_empty_string(obj.get("summary")))
                {
                    let day = find_or_create_day(export, day_number);
                    day.summary = Some(summary);
                } else {
                    report.warnings.push(
                        "intent が enrich ですが candidate_content.summary がありません — preview は変更なし"
                            .to_string(),
                    );
                }
            } else {
                report.warnings.push(
                    "intent が enrich ですが Day target が解決されていません — preview は変更なし"
                        .to_string(),
                );
            }
            Ok(FragmentApplyPreviewSummary {
                intent: intent.to_string(),
                action: "enrich_preview".to_string(),
                candidate_title,
                itineraries_before,
                itineraries_after: itineraries_before,
                notes_before: None,
                notes_after: None,
                expenses_before: None,
                expenses_after: None,
                expense_preview: None,
                estimates_before: None,
                estimates_after: None,
                estimate_preview: None,
                estimate_update_preview: None,
                estimate_field_changes: None,
                estimate_delete_preview: None,
                reservations_before: None,
                reservations_after: None,
                reservation_preview: None,
                itinerary_field_changes: None,
                reorder_preview: None,
                move_preview: None,
                delete_preview: None,
            })
        }
        "reorder_itinerary" => apply_reorder_itinerary_preview(
            conn,
            trip_id,
            export,
            resolved,
            fragment,
            intent,
            itineraries_before,
            report,
        ),
        "move_itinerary" => apply_move_itinerary_preview(
            conn,
            trip_id,
            export,
            resolved,
            fragment,
            intent,
            itineraries_before,
            report,
        ),
        "warning" => Ok(FragmentApplyPreviewSummary {
            intent: intent.to_string(),
            action: "none".to_string(),
            candidate_title,
            itineraries_before,
            itineraries_after: itineraries_before,
            notes_before: None,
            notes_after: None,
            expenses_before: None,
            expenses_after: None,
            expense_preview: None,
            estimates_before: None,
            estimates_after: None,
            estimate_preview: None,
            estimate_update_preview: None,
            estimate_field_changes: None,
            estimate_delete_preview: None,
            reservations_before: None,
            reservations_after: None,
            reservation_preview: None,
            itinerary_field_changes: None,
            reorder_preview: None,
            move_preview: None,
            delete_preview: None,
        }),
        "replace_candidate" | "reorder_hint" => {
            report.warnings.push(format!(
                "intent が {intent} です — v4.7.18 preview は構造変更をシミュレートしません"
            ));
            Ok(FragmentApplyPreviewSummary {
                intent: intent.to_string(),
                action: "preview_only".to_string(),
                candidate_title,
                itineraries_before,
                itineraries_after: itineraries_before,
                notes_before: None,
                notes_after: None,
                expenses_before: None,
                expenses_after: None,
                expense_preview: None,
                estimates_before: None,
                estimates_after: None,
                estimate_preview: None,
                estimate_update_preview: None,
                estimate_field_changes: None,
                estimate_delete_preview: None,
                reservations_before: None,
                reservations_after: None,
                reservation_preview: None,
                itinerary_field_changes: None,
                reorder_preview: None,
                move_preview: None,
                delete_preview: None,
            })
        }
        other => Err(format!("未対応の intent です: {other}")),
    }
}

fn ensure_day_in_range(export: &TripExportV3, day_number: i64) -> Result<(), String> {
    let start = export
        .trip
        .start_date
        .as_deref()
        .ok_or_else(|| "trip.start_date が必要です".to_string())?;
    let end = export
        .trip
        .end_date
        .as_deref()
        .ok_or_else(|| "trip.end_date が必要です".to_string())?;
    let day_count = validate_trip_date_range(start, end).map_err(|error| error.to_string())?;
    if day_number < 1 || day_number > day_count {
        return Err(format!(
            "day_number ({day_number}) が旅行期間 (1..={day_count}) の範囲外です"
        ));
    }
    Ok(())
}

fn find_or_create_day(export: &mut TripExportV3, day_number: i64) -> &mut ExportDayV3 {
    if let Some(index) = export
        .days
        .iter()
        .position(|day| day.day_number == day_number)
    {
        return &mut export.days[index];
    }
    export.days.push(ExportDayV3 {
        day_number,
        summary: None,
        itineraries: Vec::new(),
    });
    export
        .days
        .iter_mut()
        .find(|day| day.day_number == day_number)
        .expect("day just inserted")
}

fn print_fragment_apply_report(report: &FragmentApplyDryRunReport) {
    if report.confirm {
        println!("Fragment apply confirm result:");
    } else {
        println!("Fragment apply dry-run result (apply preview / simulation):");
    }
    println!("  file: {}", report.file);
    println!("  dry_run: {}", report.dry_run);
    println!("  confirm: {}", report.confirm);
    println!("  trip_id: {}", report.trip_id);
    println!("  valid: {}", report.valid);
    println!("  fragment_valid: {}", report.fragment_valid);
    println!("  trip_export_valid: {}", report.trip_export_valid);

    if let Some(target) = &report.resolved_target {
        println!("  target_type: {}", target.target_type);
        println!("  trip_name: {}", target.trip_name);
        println!("  resolution: {}", target.resolution);
        if let Some(day) = target.day_number {
            println!("  day_number: {day}");
        }
        if let Some(sort_order) = target.itinerary_sort_order {
            println!("  itinerary_sort_order: {sort_order}");
        }
        if let Some(title) = &target.itinerary_title {
            println!("  itinerary_title: {title}");
        }
    }

    if let Some(preview) = &report.preview {
        println!("  intent: {}", preview.intent);
        println!("  action: {}", preview.action);
        if let Some(title) = &preview.candidate_title {
            println!("  candidate_title: {title}");
        }
        println!("  itineraries_before: {}", preview.itineraries_before);
        println!("  itineraries_after: {}", preview.itineraries_after);
        if let Some(notes_before) = preview.notes_before {
            println!("  notes_before: {notes_before}");
        }
        if let Some(notes_after) = preview.notes_after {
            println!("  notes_after: {notes_after}");
        }
        if let Some(expenses_before) = preview.expenses_before {
            println!("  expenses_before: {expenses_before}");
        }
        if let Some(expenses_after) = preview.expenses_after {
            println!("  expenses_after: {expenses_after}");
        }
        if let Some(expense_preview) = &preview.expense_preview {
            println!("  expense_preview.amount: {}", expense_preview.amount);
            println!("  expense_preview.currency: {}", expense_preview.currency);
            if let Some(title) = &expense_preview.title {
                println!("  expense_preview.title: {title}");
            }
            if let Some(note) = &expense_preview.note {
                println!("  expense_preview.note: {note}");
            }
        }
        if let Some(estimates_before) = preview.estimates_before {
            println!("  estimates_before: {estimates_before}");
        }
        if let Some(estimates_after) = preview.estimates_after {
            println!("  estimates_after: {estimates_after}");
        }
        if let Some(estimate_preview) = &preview.estimate_preview {
            println!(
                "  estimate_preview.target_itinerary_id: {}",
                estimate_preview.target_itinerary_id
            );
            println!(
                "  estimate_preview.target_itinerary_title: {}",
                estimate_preview.target_itinerary_title
            );
            println!("  estimate_preview.amount: {}", estimate_preview.amount);
            println!("  estimate_preview.currency: {}", estimate_preview.currency);
            if let Some(title) = &estimate_preview.title {
                println!("  estimate_preview.title: {title}");
            }
            if let Some(note) = &estimate_preview.note {
                println!("  estimate_preview.note: {note}");
            }
            println!(
                "  estimate_preview.sort_order: {}",
                estimate_preview.sort_order
            );
        }
        if let Some(reservations_before) = preview.reservations_before {
            println!("  reservations_before: {reservations_before}");
        }
        if let Some(reservations_after) = preview.reservations_after {
            println!("  reservations_after: {reservations_after}");
        }
        if let Some(reservation_preview) = &preview.reservation_preview {
            println!(
                "  reservation_preview.reservation_type: {}",
                reservation_preview.reservation_type
            );
            println!(
                "  reservation_preview.provider_name: {}",
                reservation_preview.provider_name
            );
            if let Some(code) = &reservation_preview.confirmation_code {
                println!("  reservation_preview.confirmation_code: {code}");
            }
            if let Some(url) = &reservation_preview.reservation_site_url {
                println!("  reservation_preview.reservation_site_url: {url}");
            }
            if let Some(remark) = &reservation_preview.remark {
                println!("  reservation_preview.remark: {remark}");
            }
            if let Some(start_at) = &reservation_preview.start_at {
                println!("  reservation_preview.start_at: {start_at}");
            }
            if let Some(end_at) = &reservation_preview.end_at {
                println!("  reservation_preview.end_at: {end_at}");
            }
        }
        if let Some(changes) = &preview.itinerary_field_changes {
            for change in changes {
                println!(
                    "  itinerary_field_change.{}: {} -> {}",
                    change.field, change.before, change.after
                );
            }
        }
        if let Some(reorder_preview) = &preview.reorder_preview {
            println!(
                "  reorder_preview.day_number: {}",
                reorder_preview.day_number
            );
            for change in &reorder_preview.itinerary_order_changes {
                println!(
                    "  reorder.itinerary_id {}: {} ({} -> {})",
                    change.itinerary_id,
                    change.title,
                    change.before_sort_order,
                    change.after_sort_order
                );
            }
        }
        if let Some(move_preview) = &preview.move_preview {
            println!("  move_preview.itinerary_id: {}", move_preview.itinerary_id);
            println!("  move_preview.title: {}", move_preview.title);
            println!(
                "  move_preview.from_day_number: {}",
                move_preview.from_day_number
            );
            println!(
                "  move_preview.to_day_number: {}",
                move_preview.to_day_number
            );
            for change in &move_preview.source_order_changes {
                println!(
                    "  move.source.itinerary_id {}: {} (day {} -> {}, {} -> {})",
                    change.itinerary_id,
                    change.title,
                    change.before_day_number,
                    change.after_day_number,
                    change.before_sort_order,
                    change.after_sort_order
                );
            }
            for change in &move_preview.destination_order_changes {
                println!(
                    "  move.destination.itinerary_id {}: {} (day {} -> {}, {} -> {})",
                    change.itinerary_id,
                    change.title,
                    change.before_day_number,
                    change.after_day_number,
                    change.before_sort_order,
                    change.after_sort_order
                );
            }
        }
        if let Some(delete_preview) = &preview.delete_preview {
            println!(
                "  delete_preview.itinerary_id: {}",
                delete_preview.itinerary_id
            );
            println!("  delete_preview.title: {}", delete_preview.title);
            println!("  delete_preview.day_number: {}", delete_preview.day_number);
            println!("  delete_preview.sort_order: {}", delete_preview.sort_order);
            println!(
                "  delete_preview.blocking_children.expenses: {}",
                delete_preview.blocking_children.expenses
            );
            println!(
                "  delete_preview.blocking_children.estimates: {}",
                delete_preview.blocking_children.estimates
            );
            println!(
                "  delete_preview.blocking_children.reservations: {}",
                delete_preview.blocking_children.reservations
            );
            println!(
                "  delete_preview.blocking_children.notes: {}",
                delete_preview.blocking_children.notes
            );
        }
        if let Some(estimate_delete_preview) = &preview.estimate_delete_preview {
            println!(
                "  estimate_delete_preview.target_itinerary_id: {}",
                estimate_delete_preview.target_itinerary_id
            );
            println!(
                "  estimate_delete_preview.target_estimate_id: {}",
                estimate_delete_preview.target_estimate_id
            );
            if let Some(title) = &estimate_delete_preview.title {
                println!("  estimate_delete_preview.title: {title}");
            }
            println!(
                "  estimate_delete_preview.amount: {}",
                estimate_delete_preview.amount
            );
            println!(
                "  estimate_delete_preview.currency: {}",
                estimate_delete_preview.currency
            );
            println!(
                "  estimate_delete_preview.sort_order: {}",
                estimate_delete_preview.sort_order
            );
            println!(
                "  estimate_delete_preview.updated_at: {}",
                estimate_delete_preview.updated_at
            );
        }
    }

    if let Some(itinerary_id) = report.inserted_itinerary_id {
        println!("  inserted_itinerary_id: {itinerary_id}");
    }

    if let Some(note_id) = report.inserted_note_id {
        println!("  inserted_note_id: {note_id}");
    }

    if let Some(expense_id) = report.inserted_expense_id {
        println!("  inserted_expense_id: {expense_id}");
    }

    if let Some(reservation_id) = report.inserted_reservation_id {
        println!("  inserted_reservation_id: {reservation_id}");
    }

    if let Some(estimate_id) = report.inserted_estimate_id {
        println!("  inserted_estimate_id: {estimate_id}");
    }

    if let Some(itinerary_id) = report.updated_itinerary_id {
        println!("  updated_itinerary_id: {itinerary_id}");
    }

    if let Some(itinerary_id) = report.deleted_itinerary_id {
        println!("  deleted_itinerary_id: {itinerary_id}");
    }
    if let Some(count) = report.reordered_itineraries {
        println!("  reordered_itineraries: {count}");
    }

    if let Some(itinerary_id) = report.moved_itinerary_id {
        println!("  moved_itinerary_id: {itinerary_id}");
    }

    if let Some(count) = report.moved_itinerary_updated_rows {
        println!("  moved_itinerary_updated_rows: {count}");
    }

    if !report.required_decisions.is_empty() {
        println!("Required decisions:");
        for item in &report.required_decisions {
            println!("  - {item}");
        }
    }

    if !report.errors.is_empty() {
        println!("Blocking errors:");
        for error in &report.errors {
            println!("  - {error}");
        }
    }

    if !report.warnings.is_empty() {
        println!("Warnings:");
        for warning in &report.warnings {
            println!("  - {warning}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::db::open_db_at;

    const APPLY_READY_FRAGMENT: &str = r#"{
      "metadata": {
        "fragment_id": "frag-apply-ready",
        "created_at": "2026-03-15T14:00:00Z",
        "source": "manual",
        "provider": "fixture"
      },
      "target": {
        "target_type": "day",
        "day_reference": 1
      },
      "fragment": {
        "intent": "add",
        "candidate_content": { "title": "Lunch candidate" },
        "notes": "Preview only."
      },
      "adoption_hints": { "required_decisions": [] }
    }"#;

    const APPLY_EXPANDED_FRAGMENT: &str = r#"{
      "metadata": {
        "fragment_id": "frag-apply-expanded",
        "created_at": "2026-03-15T14:00:00Z",
        "source": "manual",
        "provider": "fixture"
      },
      "target": {
        "target_type": "day",
        "day_reference": 1
      },
      "fragment": {
        "intent": "add",
        "candidate_content": {
          "title": "Afternoon temple visit",
          "location": "Kiyomizu area",
          "category": "activity",
          "start_time": "14:30",
          "duration_minutes": 90,
          "travel_time_minutes": 20
        },
        "notes": "Ticket not purchased yet."
      },
      "adoption_hints": { "required_decisions": [] }
    }"#;

    #[test]
    fn expanded_add_fields_match_between_preview_and_confirm() {
        let conn = open_db_at(":memory:").unwrap();
        let trip_id =
            crate::trip::add_trip(&conn, "Trip", "2026-05-01", "2026-05-01", None).unwrap();
        crate::itinerary::add_itinerary_item(
            &conn,
            trip_id,
            1,
            "Morning temple",
            None,
            None,
            Some(1000),
            None,
            None,
            None,
            None,
        )
        .unwrap();

        let (report, preview_json) =
            fragment_apply_dry_run_json(&conn, "test.json", APPLY_EXPANDED_FRAGMENT, trip_id);
        assert!(report.valid, "errors: {:?}", report.errors);
        let preview_json = preview_json.expect("preview json");
        let preview: TripExportV3 = serde_json::from_str(&preview_json).unwrap();
        let preview_day = preview
            .days
            .iter()
            .find(|day| day.day_number == 1)
            .expect("day 1");
        let preview_item = preview_day
            .itineraries
            .iter()
            .find(|item| item.title == "Afternoon temple visit")
            .expect("preview item");
        assert_eq!(preview_item.category, Some(ItineraryCategory::Activity));
        assert_eq!(preview_item.start_time.as_deref(), Some("14:30"));
        assert_eq!(preview_item.duration_minutes, Some(90));
        assert_eq!(preview_item.travel_minutes, Some(20));
        assert_eq!(preview_item.sort_order, 2000);

        let options = FragmentApplyOptions {
            dry_run: false,
            confirm: true,
            trip_id,
            output: None,
            json: false,
        };
        let path = std::env::temp_dir().join(format!(
            "caglla-fragment-expanded-confirm-{}",
            std::process::id()
        ));
        std::fs::write(&path, APPLY_EXPANDED_FRAGMENT).unwrap();
        run_fragment_apply(path.to_str().unwrap(), &conn, &options).expect("confirm apply");

        let item = crate::itinerary::list_itinerary_items(&conn, trip_id)
            .unwrap()
            .into_iter()
            .find(|item| item.title == "Afternoon temple visit")
            .expect("db item");
        assert_eq!(item.category, Some(ItineraryCategory::Activity));
        assert_eq!(item.start_time.as_deref(), Some("14:30"));
        assert_eq!(item.duration_minutes, Some(90));
        assert_eq!(item.travel_minutes, Some(20));
        assert_eq!(item.sort_order, preview_item.sort_order);
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn confirm_add_note_writes_db() {
        let conn = open_db_at(":memory:").unwrap();
        let trip_id =
            crate::trip::add_trip(&conn, "Trip", "2026-05-01", "2026-05-01", None).unwrap();
        let json = r#"{
          "metadata": { "created_at": "2026-03-15T14:00:00Z", "source": "ai" },
          "target": { "target_type": "trip" },
          "fragment": {
            "intent": "add_note",
            "candidate_content": {
              "title": "Memo",
              "body": "Pack rain jacket."
            }
          }
        }"#;
        let before = crate::note::list_all_notes_for_trip(&conn, trip_id)
            .unwrap()
            .len();
        let options = FragmentApplyOptions {
            dry_run: false,
            confirm: true,
            trip_id,
            output: None,
            json: false,
        };
        let path = std::env::temp_dir().join(format!(
            "caglla-fragment-add-note-confirm-{}",
            std::process::id()
        ));
        std::fs::write(&path, json).unwrap();
        run_fragment_apply(path.to_str().unwrap(), &conn, &options).expect("confirm apply");
        let after = crate::note::list_all_notes_for_trip(&conn, trip_id)
            .unwrap()
            .len();
        assert_eq!(after, before + 1);
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn confirm_add_expense_writes_db() {
        let conn = open_db_at(":memory:").unwrap();
        let trip_id =
            crate::trip::add_trip(&conn, "Trip", "2026-05-01", "2026-05-01", None).unwrap();
        let itinerary_id = crate::itinerary::add_itinerary_item(
            &conn,
            trip_id,
            1,
            "Morning temple",
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .unwrap();
        let json = r#"{
          "metadata": { "created_at": "2026-03-15T14:00:00Z", "source": "ai" },
          "target": {
            "target_type": "itinerary",
            "day_reference": 1,
            "itinerary_reference": "Morning temple"
          },
          "fragment": {
            "intent": "add_expense",
            "candidate_content": {
              "title": "Temple admission",
              "amount": 500,
              "currency": "JPY",
              "note": "Cash only."
            }
          }
        }"#;
        let before = crate::expense::list_expenses_for_itinerary(&conn, itinerary_id)
            .unwrap()
            .len();
        let options = FragmentApplyOptions {
            dry_run: false,
            confirm: true,
            trip_id,
            output: None,
            json: true,
        };
        let path = std::env::temp_dir().join(format!(
            "caglla-fragment-add-expense-confirm-{}",
            std::process::id()
        ));
        std::fs::write(&path, json).unwrap();
        run_fragment_apply(path.to_str().unwrap(), &conn, &options).expect("confirm apply");
        let after = crate::expense::list_expenses_for_itinerary(&conn, itinerary_id)
            .unwrap()
            .len();
        assert_eq!(after, before + 1);
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn add_note_dry_run_appends_trip_note_to_preview() {
        let conn = open_db_at(":memory:").unwrap();
        let trip_id =
            crate::trip::add_trip(&conn, "Trip", "2026-05-01", "2026-05-01", None).unwrap();
        let json = r#"{
          "metadata": { "created_at": "2026-03-15T14:00:00Z", "source": "ai" },
          "target": { "target_type": "trip" },
          "fragment": {
            "intent": "add_note",
            "candidate_content": {
              "title": "Memo",
              "body": "Pack rain jacket."
            }
          }
        }"#;
        let before = crate::note::list_all_notes_for_trip(&conn, trip_id)
            .unwrap()
            .len();
        let (report, preview_json) = fragment_apply_dry_run_json(&conn, "test.json", json, trip_id);
        assert!(report.valid, "errors: {:?}", report.errors);
        let preview = preview_json.expect("preview json");
        let export: TripExportV3 = serde_json::from_str(&preview).unwrap();
        assert_eq!(export.notes().len(), 1);
        assert_eq!(report.preview.unwrap().action, "add_note");
        let after = crate::note::list_all_notes_for_trip(&conn, trip_id)
            .unwrap()
            .len();
        assert_eq!(before, after);
    }

    #[test]
    fn invalid_category_blocks_apply_without_db_write() {
        let conn = open_db_at(":memory:").unwrap();
        let trip_id =
            crate::trip::add_trip(&conn, "Trip", "2026-05-01", "2026-05-01", None).unwrap();
        let json = r#"{
          "metadata": { "created_at": "2026-03-15T14:00:00Z", "source": "ai" },
          "target": { "target_type": "day", "day_reference": 1 },
          "fragment": {
            "intent": "add",
            "candidate_content": { "title": "Bad", "category": "meal" },
            "notes": "n"
          }
        }"#;
        let before = crate::itinerary::list_itinerary_items(&conn, trip_id)
            .unwrap()
            .len();
        let (report, preview) = fragment_apply_dry_run_json(&conn, "test.json", json, trip_id);
        assert!(!report.valid);
        assert!(preview.is_none());
        assert!(report.errors.iter().any(|e| e.contains("category")));
        let after = crate::itinerary::list_itinerary_items(&conn, trip_id)
            .unwrap()
            .len();
        assert_eq!(before, after);
    }

    #[test]
    fn dry_run_simulates_add_without_db_write() {
        let conn = open_db_at(":memory:").unwrap();
        let trip_id =
            crate::trip::add_trip(&conn, "Apply Test Trip", "2026-05-01", "2026-05-02", None)
                .unwrap();
        crate::itinerary::add_itinerary_item(
            &conn,
            trip_id,
            1,
            "Morning walk",
            None,
            None,
            Some(1),
            None,
            None,
            None,
            None,
        )
        .unwrap();

        let before = crate::itinerary::list_itinerary_items(&conn, trip_id)
            .unwrap()
            .len();
        let (report, preview_json) =
            fragment_apply_dry_run_json(&conn, "test.json", APPLY_READY_FRAGMENT, trip_id);
        assert!(report.valid, "errors: {:?}", report.errors);
        assert!(report.trip_export_valid);
        let preview_json = preview_json.expect("preview json");
        let export_report = analyze_trip_export_json("preview.json", &preview_json);
        assert!(export_report.valid, "errors: {:?}", export_report.errors);
        let after = crate::itinerary::list_itinerary_items(&conn, trip_id)
            .unwrap()
            .len();
        assert_eq!(before, after, "DB must not change on dry-run");
        assert_eq!(report.preview.unwrap().itineraries_after, before + 1);
    }

    #[test]
    fn unresolved_target_blocks_apply() {
        let conn = open_db_at(":memory:").unwrap();
        let trip_id =
            crate::trip::add_trip(&conn, "Trip", "2026-05-01", "2026-05-01", None).unwrap();
        let json = r#"{
          "metadata": { "created_at": "2026-03-15T14:00:00Z", "source": "ai" },
          "target": { "target_type": "unresolved" },
          "fragment": {
            "intent": "add",
            "candidate_content": { "title": "TBD" },
            "notes": "n"
          }
        }"#;
        let (report, preview) = fragment_apply_dry_run_json(&conn, "test.json", json, trip_id);
        assert!(!report.valid);
        assert!(preview.is_none());
        assert!(report.errors.iter().any(|e| e.contains("unresolved")));
    }

    #[test]
    fn confirm_add_itinerary_writes_db() {
        let conn = open_db_at(":memory:").unwrap();
        let trip_id =
            crate::trip::add_trip(&conn, "Apply Test Trip", "2026-05-01", "2026-05-02", None)
                .unwrap();
        crate::itinerary::add_itinerary_item(
            &conn,
            trip_id,
            1,
            "Morning walk",
            None,
            None,
            Some(1),
            None,
            None,
            None,
            None,
        )
        .unwrap();

        let path = std::env::temp_dir().join(format!(
            "caglla-fragment-confirm-unit-{}",
            std::process::id()
        ));
        std::fs::write(&path, APPLY_READY_FRAGMENT).unwrap();

        let before = crate::itinerary::list_itinerary_items(&conn, trip_id)
            .unwrap()
            .len();
        let options = FragmentApplyOptions {
            dry_run: false,
            confirm: true,
            trip_id,
            output: None,
            json: false,
        };
        run_fragment_apply(path.to_str().unwrap(), &conn, &options).expect("confirm apply");
        let after = crate::itinerary::list_itinerary_items(&conn, trip_id)
            .unwrap()
            .len();
        assert_eq!(after, before + 1);
        assert!(crate::itinerary::list_itinerary_items(&conn, trip_id)
            .unwrap()
            .iter()
            .any(|item| item.title == "Lunch candidate"));
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn confirm_rejects_unsupported_intent_without_db_write() {
        let conn = open_db_at(":memory:").unwrap();
        let trip_id =
            crate::trip::add_trip(&conn, "Trip", "2026-05-01", "2026-05-01", None).unwrap();
        let json = r#"{
          "metadata": { "created_at": "2026-03-15T14:00:00Z", "source": "ai" },
          "target": { "target_type": "day", "day_reference": 1 },
          "fragment": {
            "intent": "enrich",
            "candidate_content": { "summary": "Updated" },
            "notes": "n"
          }
        }"#;
        let before = crate::itinerary::list_itinerary_items(&conn, trip_id)
            .unwrap()
            .len();
        let (report, _) = fragment_apply_gate_json(&conn, "test.json", json, trip_id, false, true);
        assert!(!report.valid);
        assert!(report.errors.iter().any(|e| e.contains("add_itinerary")));
        let after = crate::itinerary::list_itinerary_items(&conn, trip_id)
            .unwrap()
            .len();
        assert_eq!(before, after);
    }

    #[test]
    fn update_itinerary_parse_requires_update_field() {
        let fragment: Map<String, Value> = serde_json::from_str(
            r#"{
          "intent": "update_itinerary",
          "candidate_content": { "expected_title": "Morning temple" }
        }"#,
        )
        .unwrap();
        let error = parse_update_itinerary_fields(&fragment, None).unwrap_err();
        assert!(error.contains("更新フィールド"));
    }

    #[test]
    fn update_itinerary_baseline_conflict_detected() {
        let current = ExportItineraryV3 {
            title: "Morning temple".to_string(),
            note: None,
            start_time: None,
            sort_order: 1000,
            duration_minutes: None,
            travel_minutes: None,
            location: None,
            category: None,
            expenses: Vec::new(),
            estimates: Vec::new(),
            reservations: Vec::new(),
        };
        let candidate: Map<String, Value> =
            serde_json::from_str(r#"{"expected_title": "Wrong title"}"#).unwrap();
        let error = detect_update_itinerary_baseline_conflicts(&candidate, &current).unwrap_err();
        assert!(error.contains("baseline mismatch"));
    }

    #[test]
    fn update_itinerary_dry_run_preview_patches_export_without_db_write() {
        let conn = open_db_at(":memory:").unwrap();
        let trip_id =
            crate::trip::add_trip(&conn, "Trip", "2026-05-01", "2026-05-01", None).unwrap();
        crate::itinerary::add_itinerary_item(
            &conn,
            trip_id,
            1,
            "Morning temple",
            None,
            None,
            Some(1000),
            None,
            None,
            None,
            None,
        )
        .unwrap();

        let json = r#"{
          "metadata": { "created_at": "2026-03-15T14:00:00Z", "source": "manual" },
          "target": {
            "target_type": "itinerary",
            "day_reference": 1,
            "itinerary_reference": "Morning temple"
          },
          "fragment": {
            "intent": "update_itinerary",
            "candidate_content": {
              "title": "Morning temple visit",
              "category": "museum"
            }
          },
          "adoption_hints": { "required_decisions": [] }
        }"#;

        let item_before = crate::itinerary::list_itinerary_items(&conn, trip_id)
            .unwrap()
            .into_iter()
            .find(|item| item.title == "Morning temple")
            .expect("seed itinerary");

        let (report, preview_json) = fragment_apply_dry_run_json(&conn, "test.json", json, trip_id);
        assert!(report.valid, "errors: {:?}", report.errors);
        let preview = report.preview.expect("preview summary");
        assert_eq!(preview.action, "update_itinerary");
        let changes = preview.itinerary_field_changes.expect("field changes");
        assert_eq!(changes.len(), 2);
        assert_eq!(changes[0].field, "title");
        assert_eq!(changes[1].field, "category");

        let preview_json = preview_json.expect("preview json");
        let export: TripExportV3 = serde_json::from_str(&preview_json).unwrap();
        let item = export
            .days
            .iter()
            .flat_map(|day| day.itineraries.iter())
            .find(|item| item.title == "Morning temple visit")
            .expect("patched itinerary");
        assert_eq!(item.category, Some(ItineraryCategory::Museum));

        let item_after = crate::itinerary::list_itinerary_items(&conn, trip_id)
            .unwrap()
            .into_iter()
            .find(|item| item.id == item_before.id)
            .expect("db itinerary");
        assert_eq!(item_after.title, "Morning temple");
        assert_eq!(item_after.category, None);
    }

    const UPDATE_ITINERARY_FRAGMENT: &str = r#"{
          "metadata": { "created_at": "2026-03-15T14:00:00Z", "source": "manual" },
          "target": {
            "target_type": "itinerary",
            "day_reference": 1,
            "itinerary_reference": "Morning temple"
          },
          "fragment": {
            "intent": "update_itinerary",
            "candidate_content": {
              "title": "Morning temple visit",
              "category": "museum",
              "note": "Arrive 15 minutes early."
            }
          },
          "adoption_hints": { "required_decisions": [] }
        }"#;

    #[test]
    fn confirm_update_itinerary_writes_db() {
        let conn = open_db_at(":memory:").unwrap();
        let trip_id =
            crate::trip::add_trip(&conn, "Trip", "2026-05-01", "2026-05-01", None).unwrap();
        let item_id = crate::itinerary::add_itinerary_item(
            &conn,
            trip_id,
            1,
            "Morning temple",
            None,
            None,
            Some(1000),
            None,
            None,
            None,
            None,
        )
        .unwrap();

        let path = std::env::temp_dir().join(format!(
            "caglla-fragment-update-confirm-{}",
            std::process::id()
        ));
        std::fs::write(&path, UPDATE_ITINERARY_FRAGMENT).unwrap();
        let options = FragmentApplyOptions {
            dry_run: false,
            confirm: true,
            trip_id,
            output: None,
            json: false,
        };
        run_fragment_apply(path.to_str().unwrap(), &conn, &options).expect("confirm apply");

        let item = crate::itinerary::get_itinerary_item(&conn, item_id).unwrap();
        assert_eq!(item.title, "Morning temple visit");
        assert_eq!(item.category, Some(ItineraryCategory::Museum));
        assert_eq!(item.note.as_deref(), Some("Arrive 15 minutes early."));
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn confirm_update_itinerary_toctou_blocks_db_write() {
        let conn = open_db_at(":memory:").unwrap();
        let trip_id =
            crate::trip::add_trip(&conn, "Trip", "2026-05-01", "2026-05-01", None).unwrap();
        let item_id = crate::itinerary::add_itinerary_item(
            &conn,
            trip_id,
            1,
            "Morning temple",
            None,
            None,
            Some(1000),
            None,
            None,
            None,
            None,
        )
        .unwrap();

        let (report, _) = fragment_apply_gate_json(
            &conn,
            "test.json",
            UPDATE_ITINERARY_FRAGMENT,
            trip_id,
            false,
            true,
        );
        assert!(report.valid, "errors: {:?}", report.errors);

        crate::itinerary::update_itinerary_item(
            &conn,
            item_id,
            None,
            Some("Changed by another writer"),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .unwrap();

        let error =
            execute_confirm_update_itinerary(&conn, trip_id, UPDATE_ITINERARY_FRAGMENT, &report)
                .unwrap_err();
        assert!(error.to_string().contains("TOCTOU"));

        let item = crate::itinerary::get_itinerary_item(&conn, item_id).unwrap();
        assert_eq!(item.title, "Changed by another writer");
        assert_eq!(item.category, None);
    }

    const DELETE_ITINERARY_FRAGMENT: &str = r#"{
          "metadata": { "created_at": "2026-03-15T14:00:00Z", "source": "manual" },
          "target": {
            "target_type": "itinerary",
            "day_reference": 1,
            "itinerary_reference": "Morning temple"
          },
          "fragment": {
            "intent": "delete_itinerary"
          },
          "adoption_hints": { "required_decisions": [] }
        }"#;

    #[test]
    fn delete_itinerary_dry_run_preview_removes_itinerary_without_db_write() {
        let conn = open_db_at(":memory:").unwrap();
        let trip_id =
            crate::trip::add_trip(&conn, "Trip", "2026-05-01", "2026-05-01", None).unwrap();
        let item_id = crate::itinerary::add_itinerary_item(
            &conn,
            trip_id,
            1,
            "Morning temple",
            None,
            None,
            Some(1000),
            None,
            None,
            None,
            None,
        )
        .unwrap();

        let (report, preview_json) =
            fragment_apply_dry_run_json(&conn, "test.json", DELETE_ITINERARY_FRAGMENT, trip_id);
        assert!(report.valid, "errors: {:?}", report.errors);
        let preview = report.preview.expect("preview summary");
        assert_eq!(preview.action, "delete_itinerary");
        assert_eq!(preview.itineraries_before, 1);
        assert_eq!(preview.itineraries_after, 0);
        let delete_preview = preview.delete_preview.expect("delete_preview");
        assert_eq!(delete_preview.itinerary_id, item_id);
        assert_eq!(delete_preview.title, "Morning temple");
        assert_eq!(delete_preview.blocking_children.expenses, 0);
        assert_eq!(delete_preview.blocking_children.estimates, 0);
        assert_eq!(delete_preview.blocking_children.reservations, 0);
        assert_eq!(delete_preview.blocking_children.notes, 0);
        assert!(delete_preview.non_blocking_relations.is_none());

        let preview_json = preview_json.expect("preview json");
        let export: TripExportV3 = serde_json::from_str(&preview_json).unwrap();
        assert_eq!(count_itineraries(&export), 0);

        let items = crate::itinerary::list_itinerary_items(&conn, trip_id).unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].id, item_id);
        assert_eq!(items[0].title, "Morning temple");
    }

    #[test]
    fn delete_itinerary_blocking_children_block_preview() {
        use crate::expense::ExpenseSharedOptions;

        let conn = open_db_at(":memory:").unwrap();
        let trip_id =
            crate::trip::add_trip(&conn, "Trip", "2026-05-01", "2026-05-01", None).unwrap();
        let item_id = crate::itinerary::add_itinerary_item(
            &conn,
            trip_id,
            1,
            "Morning temple",
            None,
            None,
            Some(1000),
            None,
            None,
            None,
            None,
        )
        .unwrap();
        crate::expense::add_expense(
            &conn,
            item_id,
            "500",
            "JPY",
            Some("Admission"),
            None,
            None,
            None,
            &ExpenseSharedOptions::default(),
        )
        .unwrap();

        let (report, preview_json) =
            fragment_apply_dry_run_json(&conn, "test.json", DELETE_ITINERARY_FRAGMENT, trip_id);
        assert!(!report.valid);
        assert!(preview_json.is_none());
        assert!(report
            .errors
            .iter()
            .any(|error| error.contains("blocking child") && error.contains("expenses: 1")));
    }

    #[test]
    fn delete_itinerary_trip_target_blocks_preview() {
        let conn = open_db_at(":memory:").unwrap();
        let trip_id =
            crate::trip::add_trip(&conn, "Trip", "2026-05-01", "2026-05-01", None).unwrap();
        crate::itinerary::add_itinerary_item(
            &conn,
            trip_id,
            1,
            "Morning temple",
            None,
            None,
            Some(1000),
            None,
            None,
            None,
            None,
        )
        .unwrap();

        let json = r#"{
          "metadata": { "created_at": "2026-03-15T14:00:00Z", "source": "manual" },
          "target": { "target_type": "trip" },
          "fragment": { "intent": "delete_itinerary" },
          "adoption_hints": { "required_decisions": [] }
        }"#;

        let (report, preview_json) = fragment_apply_dry_run_json(&conn, "test.json", json, trip_id);
        assert!(!report.valid);
        assert!(preview_json.is_none());
        assert!(report
            .errors
            .iter()
            .any(|error| error.contains("itinerary target")));
    }

    #[test]
    fn confirm_delete_itinerary_writes_db() {
        let conn = open_db_at(":memory:").unwrap();
        let trip_id =
            crate::trip::add_trip(&conn, "Trip", "2026-05-01", "2026-05-01", None).unwrap();
        let item_id = crate::itinerary::add_itinerary_item(
            &conn,
            trip_id,
            1,
            "Morning temple",
            None,
            None,
            Some(1000),
            None,
            None,
            None,
            None,
        )
        .unwrap();

        let path = std::env::temp_dir().join(format!(
            "caglla-fragment-delete-confirm-{}",
            std::process::id()
        ));
        std::fs::write(&path, DELETE_ITINERARY_FRAGMENT).unwrap();
        let options = FragmentApplyOptions {
            dry_run: false,
            confirm: true,
            trip_id,
            output: None,
            json: false,
        };
        run_fragment_apply(path.to_str().unwrap(), &conn, &options).expect("confirm apply");

        assert!(crate::itinerary::get_itinerary_item(&conn, item_id).is_err());
        let items = crate::itinerary::list_itinerary_items(&conn, trip_id).unwrap();
        assert!(items.is_empty());
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn confirm_delete_itinerary_toctou_blocks_db_write() {
        let conn = open_db_at(":memory:").unwrap();
        let trip_id =
            crate::trip::add_trip(&conn, "Trip", "2026-05-01", "2026-05-01", None).unwrap();
        let item_id = crate::itinerary::add_itinerary_item(
            &conn,
            trip_id,
            1,
            "Morning temple",
            None,
            None,
            Some(1000),
            None,
            None,
            None,
            None,
        )
        .unwrap();

        let (report, _) = fragment_apply_gate_json(
            &conn,
            "test.json",
            DELETE_ITINERARY_FRAGMENT,
            trip_id,
            false,
            true,
        );
        assert!(report.valid, "errors: {:?}", report.errors);

        crate::itinerary::update_itinerary_item(
            &conn,
            item_id,
            None,
            Some("Changed by another writer"),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .unwrap();

        let error = execute_confirm_delete_itinerary(&conn, trip_id, &report).unwrap_err();
        assert!(error.to_string().contains("TOCTOU"));

        let item = crate::itinerary::get_itinerary_item(&conn, item_id).unwrap();
        assert_eq!(item.title, "Changed by another writer");
    }

    #[test]
    fn confirm_delete_itinerary_inline_note_does_not_block() {
        let conn = open_db_at(":memory:").unwrap();
        let trip_id =
            crate::trip::add_trip(&conn, "Trip", "2026-05-01", "2026-05-01", None).unwrap();
        let item_id = crate::itinerary::add_itinerary_item(
            &conn,
            trip_id,
            1,
            "Morning temple",
            Some("Inline memo only"),
            None,
            Some(1000),
            None,
            None,
            None,
            None,
        )
        .unwrap();

        let path = std::env::temp_dir().join(format!(
            "caglla-fragment-delete-inline-note-{}",
            std::process::id()
        ));
        std::fs::write(&path, DELETE_ITINERARY_FRAGMENT).unwrap();
        let options = FragmentApplyOptions {
            dry_run: false,
            confirm: true,
            trip_id,
            output: None,
            json: false,
        };
        run_fragment_apply(path.to_str().unwrap(), &conn, &options).expect("confirm apply");
        assert!(crate::itinerary::get_itinerary_item(&conn, item_id).is_err());
        let _ = std::fs::remove_file(path);
    }

    fn seed_update_estimate_confirm_fixture(conn: &Connection) -> (i64, i64, i64) {
        let trip_id =
            crate::trip::add_trip(&conn, "Trip", "2026-05-01", "2026-05-01", None).unwrap();
        let itinerary_id = crate::itinerary::add_itinerary_item(
            &conn,
            trip_id,
            1,
            "Morning temple",
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .unwrap();
        let estimate_id = crate::estimate::add_estimate(
            &conn,
            itinerary_id,
            "10000",
            "JPY",
            Some("Lunch estimate"),
            Some("Original note"),
            None,
        )
        .unwrap();
        (trip_id, itinerary_id, estimate_id)
    }

    const UPDATE_ESTIMATE_GATE_AMOUNT_1250_JSON: &str = r#"{
      "metadata": { "created_at": "2026-03-15T14:00:00Z", "source": "manual" },
      "target": {
        "target_type": "itinerary",
        "day_reference": 1,
        "itinerary_reference": "Morning temple"
      },
      "fragment": {
        "intent": "update_estimate",
        "candidate_content": {
          "estimate_id": 1,
          "amount": 1250,
          "currency": "USD"
        }
      },
      "adoption_hints": { "required_decisions": [] }
    }"#;

    const UPDATE_ESTIMATE_CONFIRM_AMOUNT_1300_JSON: &str = r#"{
      "metadata": { "created_at": "2026-03-15T14:00:00Z", "source": "manual" },
      "target": {
        "target_type": "itinerary",
        "day_reference": 1,
        "itinerary_reference": "Morning temple"
      },
      "fragment": {
        "intent": "update_estimate",
        "candidate_content": {
          "estimate_id": 1,
          "amount": 1300,
          "currency": "USD"
        }
      },
      "adoption_hints": { "required_decisions": [] }
    }"#;

    #[test]
    fn confirm_update_estimate_candidate_preview_mismatch_blocks_db_write() {
        let conn = open_db_at(":memory:").unwrap();
        let (trip_id, itinerary_id, estimate_id) = seed_update_estimate_confirm_fixture(&conn);
        let before = crate::estimate::get_estimate(&conn, estimate_id).unwrap();
        let before_expenses = crate::expense::list_expenses_for_itinerary(&conn, itinerary_id)
            .unwrap()
            .len();

        let (gate_report, _) = fragment_apply_gate_json(
            &conn,
            "test.json",
            UPDATE_ESTIMATE_GATE_AMOUNT_1250_JSON,
            trip_id,
            false,
            true,
        );
        assert!(gate_report.valid, "errors: {:?}", gate_report.errors);

        let error = execute_confirm_update_estimate(
            &conn,
            trip_id,
            UPDATE_ESTIMATE_CONFIRM_AMOUNT_1300_JSON,
            &gate_report,
        )
        .unwrap_err();
        let message = format!("{error:#}");
        assert!(
            message.contains("estimate_field_changes") || message.contains("gate preview"),
            "got: {message}"
        );
        assert!(gate_report.updated_estimate_id.is_none());

        let after = crate::estimate::get_estimate(&conn, estimate_id).unwrap();
        assert_eq!(after.amount, before.amount);
        assert_eq!(after.currency, before.currency);
        assert_eq!(after.title, before.title);
        assert_eq!(after.note, before.note);
        assert_eq!(after.sort_order, before.sort_order);
        assert_eq!(after.updated_at, before.updated_at);
        assert_eq!(
            crate::expense::list_expenses_for_itinerary(&conn, itinerary_id)
                .unwrap()
                .len(),
            before_expenses
        );
    }

    #[test]
    fn verify_updated_estimate_matches_proposed_rejects_amount_mismatch() {
        let stored = crate::domain::models::Estimate {
            id: 1,
            itinerary_id: 10,
            title: Some("Title".to_string()),
            amount: 1300,
            currency: "USD".to_string(),
            note: Some("Note".to_string()),
            sort_order: 1,
            created_at: "2026-01-01T00:00:00Z".to_string(),
            updated_at: "2026-01-01T00:00:00Z".to_string(),
        };
        let proposed = crate::domain::models::Estimate {
            amount: 1250,
            ..stored.clone()
        };

        let error = verify_updated_estimate_matches_proposed(&stored, 10, &proposed).unwrap_err();
        assert!(error.contains("amount"));
        assert!(error.contains("DB 更新しません"));
    }

    const DELETE_ESTIMATE_FRAGMENT: &str = r#"{
      "metadata": { "created_at": "2026-03-15T14:00:00Z", "source": "manual" },
      "target": {
        "target_type": "itinerary",
        "day_reference": 1,
        "itinerary_reference": "Morning temple"
      },
      "fragment": {
        "intent": "delete_estimate",
        "candidate_content": {
          "estimate_id": 1
        }
      },
      "adoption_hints": { "required_decisions": [] }
    }"#;

    #[test]
    fn delete_estimate_dry_run_preview_removes_estimate_without_db_write() {
        let conn = open_db_at(":memory:").unwrap();
        let (trip_id, itinerary_id, estimate_id) = seed_update_estimate_confirm_fixture(&conn);
        let before = crate::estimate::get_estimate(&conn, estimate_id).unwrap();

        let (report, preview_json) =
            fragment_apply_dry_run_json(&conn, "test.json", DELETE_ESTIMATE_FRAGMENT, trip_id);
        assert!(report.valid, "errors: {:?}", report.errors);
        assert!(report.deleted_estimate_id.is_none());
        let preview = report.preview.expect("preview summary");
        assert_eq!(preview.action, "delete_estimate");
        assert_eq!(preview.estimates_before, Some(1));
        assert_eq!(preview.estimates_after, Some(0));
        let delete_preview = preview
            .estimate_delete_preview
            .expect("estimate_delete_preview");
        assert_eq!(delete_preview.target_itinerary_id, itinerary_id);
        assert_eq!(delete_preview.target_estimate_id, estimate_id);
        assert_eq!(delete_preview.title, before.title);
        assert_eq!(delete_preview.amount, before.amount);
        assert_eq!(delete_preview.currency, before.currency);
        assert_eq!(delete_preview.note, before.note);
        assert_eq!(delete_preview.sort_order, before.sort_order);
        assert_eq!(delete_preview.updated_at, before.updated_at);
        assert!(report
            .warnings
            .iter()
            .any(|warning| warning.contains("hard delete")));

        let preview_json = preview_json.expect("preview json");
        let export: TripExportV3 = serde_json::from_str(&preview_json).unwrap();
        assert_eq!(count_estimates(&export), 0);

        let after = crate::estimate::get_estimate(&conn, estimate_id).unwrap();
        assert_eq!(after, before);
    }

    #[test]
    fn delete_estimate_not_found_blocks_preview() {
        let conn = open_db_at(":memory:").unwrap();
        let trip_id =
            crate::trip::add_trip(&conn, "Trip", "2026-05-01", "2026-05-01", None).unwrap();
        crate::itinerary::add_itinerary_item(
            &conn,
            trip_id,
            1,
            "Morning temple",
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .unwrap();

        let (report, preview_json) =
            fragment_apply_dry_run_json(&conn, "test.json", DELETE_ESTIMATE_FRAGMENT, trip_id);
        assert!(!report.valid);
        assert!(preview_json.is_none());
        assert!(report
            .errors
            .iter()
            .any(|error| error.contains("Estimate not found")));
    }

    #[test]
    fn delete_estimate_cross_itinerary_blocks_preview() {
        let conn = open_db_at(":memory:").unwrap();
        let trip_id =
            crate::trip::add_trip(&conn, "Trip", "2026-05-01", "2026-05-01", None).unwrap();
        crate::itinerary::add_itinerary_item(
            &conn,
            trip_id,
            1,
            "Morning temple",
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .unwrap();
        let second_itinerary_id = crate::itinerary::add_itinerary_item(
            &conn,
            trip_id,
            1,
            "Second stop",
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .unwrap();
        let foreign_estimate_id = crate::estimate::add_estimate(
            &conn,
            second_itinerary_id,
            "5000",
            "JPY",
            None,
            None,
            None,
        )
        .unwrap();
        let before = crate::estimate::get_estimate(&conn, foreign_estimate_id).unwrap();

        let fragment = format!(
            r#"{{
      "metadata": {{ "created_at": "2026-03-15T14:00:00Z", "source": "manual" }},
      "target": {{
        "target_type": "itinerary",
        "day_reference": 1,
        "itinerary_reference": "Morning temple"
      }},
      "fragment": {{
        "intent": "delete_estimate",
        "candidate_content": {{
          "estimate_id": {foreign_estimate_id}
        }}
      }},
      "adoption_hints": {{ "required_decisions": [] }}
    }}"#
        );

        let (report, preview_json) =
            fragment_apply_dry_run_json(&conn, "test.json", &fragment, trip_id);
        assert!(!report.valid);
        assert!(preview_json.is_none());
        assert!(report
            .errors
            .iter()
            .any(|error| error.contains("配下ではありません")));
        assert_eq!(
            crate::estimate::get_estimate(&conn, foreign_estimate_id).unwrap(),
            before
        );
    }

    #[test]
    fn delete_estimate_mutation_field_blocks_preview() {
        let conn = open_db_at(":memory:").unwrap();
        let (trip_id, _itinerary_id, estimate_id) = seed_update_estimate_confirm_fixture(&conn);
        let before = crate::estimate::get_estimate(&conn, estimate_id).unwrap();
        let fragment = r#"{
      "metadata": { "created_at": "2026-03-15T14:00:00Z", "source": "manual" },
      "target": {
        "target_type": "itinerary",
        "day_reference": 1,
        "itinerary_reference": "Morning temple"
      },
      "fragment": {
        "intent": "delete_estimate",
        "candidate_content": {
          "estimate_id": 1,
          "amount": 15000
        }
      },
      "adoption_hints": { "required_decisions": [] }
    }"#;

        let (report, preview_json) =
            fragment_apply_dry_run_json(&conn, "test.json", fragment, trip_id);
        assert!(!report.valid);
        assert!(preview_json.is_none());
        assert!(report
            .errors
            .iter()
            .any(|error| error.contains("指定できません")));
        assert_eq!(
            crate::estimate::get_estimate(&conn, estimate_id).unwrap(),
            before
        );
    }

    const DELETE_ESTIMATE_CONFIRM_FRAGMENT: &str = r#"{
      "metadata": { "created_at": "2026-03-15T14:00:00Z", "source": "manual" },
      "target": {
        "target_type": "itinerary",
        "day_reference": 1,
        "itinerary_reference": "Morning temple"
      },
      "fragment": {
        "intent": "delete_estimate",
        "candidate_content": {
          "estimate_id": 1
        }
      },
      "adoption_hints": { "required_decisions": [] }
    }"#;

    #[test]
    fn confirm_delete_estimate_writes_db() {
        let conn = open_db_at(":memory:").unwrap();
        let (trip_id, itinerary_id, estimate_id) = seed_update_estimate_confirm_fixture(&conn);
        let before_expenses = crate::expense::list_expenses_for_itinerary(&conn, itinerary_id)
            .unwrap()
            .len();

        let (gate_report, _) = fragment_apply_gate_json(
            &conn,
            "test.json",
            DELETE_ESTIMATE_FRAGMENT,
            trip_id,
            false,
            true,
        );
        assert!(gate_report.valid, "errors: {:?}", gate_report.errors);

        let deleted_id = execute_confirm_delete_estimate(
            &conn,
            trip_id,
            DELETE_ESTIMATE_CONFIRM_FRAGMENT,
            &gate_report,
        )
        .unwrap();
        assert_eq!(deleted_id, estimate_id);
        assert!(crate::estimate::get_estimate(&conn, estimate_id).is_err());
        assert_eq!(
            crate::estimate::list_estimates_for_itinerary(&conn, itinerary_id)
                .unwrap()
                .len(),
            0
        );
        assert_eq!(
            crate::expense::list_expenses_for_itinerary(&conn, itinerary_id)
                .unwrap()
                .len(),
            before_expenses
        );
    }

    #[test]
    fn confirm_delete_estimate_candidate_preview_mismatch_blocks_db_write() {
        let conn = open_db_at(":memory:").unwrap();
        let (trip_id, itinerary_id, estimate_id) = seed_update_estimate_confirm_fixture(&conn);
        let before = crate::estimate::get_estimate(&conn, estimate_id).unwrap();

        let (gate_report, _) = fragment_apply_gate_json(
            &conn,
            "test.json",
            DELETE_ESTIMATE_FRAGMENT,
            trip_id,
            false,
            true,
        );
        assert!(gate_report.valid, "errors: {:?}", gate_report.errors);

        let mismatch_fragment = r#"{
      "metadata": { "created_at": "2026-03-15T14:00:00Z", "source": "manual" },
      "target": {
        "target_type": "itinerary",
        "day_reference": 1,
        "itinerary_reference": "Morning temple"
      },
      "fragment": {
        "intent": "delete_estimate",
        "candidate_content": {
          "estimate_id": 1,
          "expected_amount": "99999"
        }
      },
      "adoption_hints": { "required_decisions": [] }
    }"#;

        let error =
            execute_confirm_delete_estimate(&conn, trip_id, mismatch_fragment, &gate_report)
                .unwrap_err();
        let message = format!("{error:#}");
        assert!(
            message.contains("baseline mismatch") || message.contains("gate preview"),
            "got: {message}"
        );

        assert_eq!(
            crate::estimate::get_estimate(&conn, estimate_id).unwrap(),
            before
        );
        assert_eq!(
            crate::estimate::list_estimates_for_itinerary(&conn, itinerary_id)
                .unwrap()
                .len(),
            1
        );
    }

    #[test]
    fn confirm_delete_estimate_toctou_blocks_db_write() {
        let conn = open_db_at(":memory:").unwrap();
        let (trip_id, _itinerary_id, estimate_id) = seed_update_estimate_confirm_fixture(&conn);

        let (gate_report, _) = fragment_apply_gate_json(
            &conn,
            "test.json",
            DELETE_ESTIMATE_FRAGMENT,
            trip_id,
            false,
            true,
        );
        assert!(gate_report.valid, "errors: {:?}", gate_report.errors);

        crate::estimate::update_estimate(
            &conn,
            estimate_id,
            &crate::estimate::UpdateEstimateParams {
                amount_input: Some("20000"),
                ..Default::default()
            },
        )
        .unwrap();
        let after_update = crate::estimate::get_estimate(&conn, estimate_id).unwrap();

        let error = execute_confirm_delete_estimate(
            &conn,
            trip_id,
            DELETE_ESTIMATE_CONFIRM_FRAGMENT,
            &gate_report,
        )
        .unwrap_err();
        let message = format!("{error:#}");
        assert!(
            message.contains("TOCTOU") || message.contains("gate preview"),
            "got: {message}"
        );

        assert_eq!(
            crate::estimate::get_estimate(&conn, estimate_id).unwrap(),
            after_update
        );
    }
}
