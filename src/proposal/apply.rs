use anyhow::{bail, Context, Result};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::day::{find_day_by_trip_and_day_number, validate_trip_date_range};
use crate::domain::models::{
    parse_itinerary_category, ExportDayV3, ExportItineraryV3, ExportNote, ItineraryCategory,
    ItineraryNoteKey, TripExportV3, TRIP_EXPORT_GENERATOR, TRIP_EXPORT_SCHEMA_VERSION,
};
use crate::itinerary::{parse_time_hhmm, SORT_ORDER_STEP};
use crate::output::json::print_json;
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

enum ConfirmInsertResult {
    Itinerary(i64),
    Note(i64),
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
    let mut simulated = preview_export;
    let preview_summary = match simulate_apply_preview(
        &mut simulated,
        &resolved,
        fragment_body,
        &intent,
        itineraries_before,
        notes_before,
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
        _ => {
            report.errors.push(format!(
                "v4.7.23 --confirm は intent add (add_itinerary) または add_note のみサポートしています（現在: intent={intent}, action={}）",
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

fn simulate_apply_preview(
    export: &mut TripExportV3,
    resolved: &ResolvedApplyTarget,
    fragment: &Map<String, Value>,
    intent: &str,
    itineraries_before: usize,
    notes_before: usize,
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
            })
        }
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
            })
        }
        "warning" => Ok(FragmentApplyPreviewSummary {
            intent: intent.to_string(),
            action: "none".to_string(),
            candidate_title,
            itineraries_before,
            itineraries_after: itineraries_before,
            notes_before: None,
            notes_after: None,
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
    }

    if let Some(itinerary_id) = report.inserted_itinerary_id {
        println!("  inserted_itinerary_id: {itinerary_id}");
    }

    if let Some(note_id) = report.inserted_note_id {
        println!("  inserted_note_id: {note_id}");
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
}
