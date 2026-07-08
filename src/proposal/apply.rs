use anyhow::{bail, Context, Result};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::day::{find_day_by_trip_and_day_number, validate_trip_date_range};
use crate::domain::models::{
    parse_itinerary_category, ExportDayV3, ExportExpenseV3, ExportItineraryV3, ExportNote,
    ExportReservationV3, ItineraryCategory, ItineraryNoteKey, TripExportV3, TRIP_EXPORT_GENERATOR,
    TRIP_EXPORT_SCHEMA_VERSION,
};
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
pub struct FragmentApplyItineraryOrderChange {
    pub itinerary_id: i64,
    pub title: String,
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
    pub updated_itinerary_id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deleted_itinerary_id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reordered_itineraries: Option<usize>,
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
            updated_itinerary_id: None,
            deleted_itinerary_id: None,
            reordered_itineraries: None,
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
    UpdatedItinerary(i64),
    DeletedItinerary(i64),
    ReorderedItineraries(usize),
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
                "update_itinerary" => {
                    execute_confirm_update_itinerary(conn, options.trip_id, &json, &report)
                        .map(ConfirmInsertResult::UpdatedItinerary)
                }
                "delete_itinerary" => {
                    execute_confirm_delete_itinerary(conn, options.trip_id, &report)
                        .map(ConfirmInsertResult::DeletedItinerary)
                }
                "reorder_itinerary" => {
                    execute_confirm_reorder_itinerary(conn, options.trip_id, &json, &report)
                        .map(ConfirmInsertResult::ReorderedItineraries)
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
        _ => {
            report.errors.push(format!(
                "v4.7.32 --confirm は intent add (add_itinerary)、add_note、add_expense、add_reservation、update_itinerary、delete_itinerary、または将来の confirm 対象 intent のみサポートしています（現在: intent={intent}, action={}）",
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
                    let sort_order = n.as_i64().unwrap();
                    let matches: Vec<_> = items
                        .iter()
                        .filter(|item| item.sort_order == sort_order)
                        .collect();
                    if matches.is_empty() {
                        report.errors.push(format!(
                            "itinerary_reference (sort_order {sort_order}) が Day {day_number} に見つかりません"
                        ));
                        return Err(());
                    }
                    if matches.len() > 1 {
                        report.errors.push(format!(
                            "itinerary_reference (sort_order {sort_order}) が Day {day_number} で曖昧です"
                        ));
                        resolved.resolution = "ambiguous".to_string();
                        return Err(());
                    }
                    (Some(sort_order), Some(matches[0].title.clone()))
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
        reservations_before: None,
        reservations_after: None,
        reservation_preview: None,
        itinerary_field_changes: None,
        reorder_preview: None,
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
        reservations_before: None,
        reservations_after: None,
        reservation_preview: None,
        itinerary_field_changes: None,
        reorder_preview: Some(FragmentApplyReorderPreview {
            day_number,
            itinerary_order_changes: changes,
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
        reservations_before: None,
        reservations_after: None,
        reservation_preview: None,
        itinerary_field_changes: Some(changes),
        reorder_preview: None,
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
                reservations_before: None,
                reservations_after: None,
                reservation_preview: None,
                itinerary_field_changes: None,
                reorder_preview: None,
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
                reservations_before: None,
                reservations_after: None,
                reservation_preview: None,
                itinerary_field_changes: None,
                reorder_preview: None,
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
                reservations_before: None,
                reservations_after: None,
                reservation_preview: None,
                itinerary_field_changes: None,
                reorder_preview: None,
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
                reservations_before: None,
                reservations_after: None,
                reservation_preview: None,
                itinerary_field_changes: None,
                reorder_preview: None,
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
            reservations_before: None,
            reservations_after: None,
            reservation_preview: None,
            itinerary_field_changes: None,
            reorder_preview: None,
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
                reservations_before: None,
                reservations_after: None,
                reservation_preview: None,
                itinerary_field_changes: None,
                reorder_preview: None,
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

    if let Some(itinerary_id) = report.updated_itinerary_id {
        println!("  updated_itinerary_id: {itinerary_id}");
    }

    if let Some(itinerary_id) = report.deleted_itinerary_id {
        println!("  deleted_itinerary_id: {itinerary_id}");
    }
    if let Some(count) = report.reordered_itineraries {
        println!("  reordered_itineraries: {count}");
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
}
