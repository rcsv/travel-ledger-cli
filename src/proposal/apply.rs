use anyhow::{bail, Context, Result};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::day::{find_day_by_trip_and_day_number, validate_trip_date_range};
use crate::domain::models::{
    ExportDayV3, ExportItineraryV3, TripExportV3, TRIP_EXPORT_GENERATOR, TRIP_EXPORT_SCHEMA_VERSION,
};
use crate::output::json::print_json;
use crate::trip::{analyze_trip_export_json, build_trip_export_v3, get_trip};

use super::fragment::analyze_proposal_fragment_json;

pub const FRAGMENT_APPLY_REPORT_SCHEMA_VERSION: i32 = 1;

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
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FragmentApplyDryRunReport {
    pub schema_version: i32,
    pub file: String,
    pub dry_run: bool,
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
}

impl FragmentApplyDryRunReport {
    fn new(file: impl Into<String>, trip_id: i64) -> Self {
        Self {
            schema_version: FRAGMENT_APPLY_REPORT_SCHEMA_VERSION,
            file: file.into(),
            dry_run: true,
            valid: false,
            fragment_valid: false,
            trip_export_valid: false,
            trip_id,
            errors: Vec::new(),
            warnings: Vec::new(),
            required_decisions: Vec::new(),
            resolved_target: None,
            preview: None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FragmentApplyOptions {
    pub dry_run: bool,
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

pub fn run_fragment_apply(
    path: &str,
    conn: &Connection,
    options: &FragmentApplyOptions,
) -> Result<()> {
    if !options.dry_run {
        bail!("v4.7.18 では fragment apply は --dry-run のみサポートしています（DB 更新は未実装）");
    }

    let json = std::fs::read_to_string(path)
        .with_context(|| format!("ファイル '{path}' を読み込めませんでした"))?;
    let (report, preview_json) = fragment_apply_dry_run_json(conn, path, &json, options.trip_id);

    if options.json {
        print_json(&report)?;
    } else {
        print_fragment_apply_report(&report);
    }

    if report.valid {
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
    } else {
        anyhow::bail!("fragment apply --dry-run に失敗しました");
    }

    Ok(())
}

pub fn fragment_apply_dry_run_json(
    conn: &Connection,
    path: &str,
    json: &str,
    trip_id: i64,
) -> (FragmentApplyDryRunReport, Option<String>) {
    let mut report = FragmentApplyDryRunReport::new(path, trip_id);
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
    let mut simulated = preview_export;
    let preview_summary = match simulate_apply_preview(
        &mut simulated,
        &resolved,
        fragment_body,
        &intent,
        itineraries_before,
        &mut report,
    ) {
        Ok(summary) => summary,
        Err(message) => {
            report.errors.push(message);
            return (report, None);
        }
    };
    report.preview = Some(preview_summary);

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

    report.valid = true;
    (report, Some(preview_json))
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

fn simulate_apply_preview(
    export: &mut TripExportV3,
    resolved: &ResolvedApplyTarget,
    fragment: &Map<String, Value>,
    intent: &str,
    itineraries_before: usize,
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
            let title = candidate_title.ok_or_else(|| {
                "intent が add ですが candidate_content.title がありません".to_string()
            })?;
            let day_number = resolved.day_number.ok_or_else(|| {
                "intent が add ですが target Day が解決されていません".to_string()
            })?;
            ensure_day_in_range(export, day_number)?;
            let day = find_or_create_day(export, day_number);
            let sort_order = day
                .itineraries
                .iter()
                .map(|item| item.sort_order)
                .max()
                .unwrap_or(0)
                + 1;
            day.itineraries.push(ExportItineraryV3 {
                title: title.clone(),
                note: non_empty_string(fragment.get("notes")),
                start_time: None,
                sort_order,
                duration_minutes: None,
                travel_minutes: None,
                location: candidate
                    .and_then(|v| v.as_object())
                    .and_then(|obj| non_empty_string(obj.get("location"))),
                category: None,
                expenses: Vec::new(),
                estimates: Vec::new(),
                reservations: Vec::new(),
            });
            let itineraries_after = count_itineraries(export);
            Ok(FragmentApplyPreviewSummary {
                intent: intent.to_string(),
                action: "add_itinerary".to_string(),
                candidate_title: Some(title),
                itineraries_before,
                itineraries_after,
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
            })
        }
        "warning" => Ok(FragmentApplyPreviewSummary {
            intent: intent.to_string(),
            action: "none".to_string(),
            candidate_title,
            itineraries_before,
            itineraries_after: itineraries_before,
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
    println!("Fragment apply dry-run result (apply preview / simulation):");
    println!("  file: {}", report.file);
    println!("  dry_run: {}", report.dry_run);
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
}
