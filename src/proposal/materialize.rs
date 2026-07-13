use anyhow::{bail, Context, Result};
use chrono::Utc;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::day::validate_trip_date_range;
use crate::domain::models::{
    ExportDayV3, ExportItineraryV3, Trip, TripExportV3, TRIP_EXPORT_GENERATOR,
    TRIP_EXPORT_SCHEMA_VERSION,
};
use crate::output::json::print_json;
use crate::storage::db;
use crate::trip::{
    analyze_trip_export_json, import_trip_from_json_with_summary, print_trip_import_summary,
};

use super::envelope::analyze_proposal_envelope_json;

pub const PROPOSAL_MATERIALIZE_REPORT_SCHEMA_VERSION: i32 = 2;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProposalMaterializeOutputSummary {
    pub trip_name: String,
    pub start_date: String,
    pub end_date: String,
    pub day_count: i64,
    pub itinerary_count: usize,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProposalMaterializeDryRunReport {
    pub schema_version: i32,
    pub file: String,
    pub dry_run: bool,
    pub confirm: bool,
    pub valid: bool,
    pub envelope_valid: bool,
    pub trip_export_valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub required_decisions: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output: Option<ProposalMaterializeOutputSummary>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trip_id: Option<i64>,
}

impl ProposalMaterializeDryRunReport {
    fn new(file: impl Into<String>, dry_run: bool, confirm: bool) -> Self {
        Self {
            schema_version: PROPOSAL_MATERIALIZE_REPORT_SCHEMA_VERSION,
            file: file.into(),
            dry_run,
            confirm,
            valid: false,
            envelope_valid: false,
            trip_export_valid: false,
            errors: Vec::new(),
            warnings: Vec::new(),
            required_decisions: Vec::new(),
            output: None,
            trip_id: None,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ProposalMaterializeParams {
    pub start_date: Option<String>,
    pub end_date: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProposalMaterializeOptions {
    pub dry_run: bool,
    pub confirm: bool,
    pub output: Option<String>,
    pub params: ProposalMaterializeParams,
    pub json: bool,
}

pub fn run_proposal_materialize(
    path: &str,
    conn: Option<&Connection>,
    options: &ProposalMaterializeOptions,
) -> Result<()> {
    if !options.dry_run && !options.confirm {
        bail!("proposal materialize には --dry-run または --confirm のいずれかが必要です");
    }
    if options.dry_run && options.confirm {
        bail!("--dry-run と --confirm は併用できません（dry-run means no side effects）");
    }
    if options.confirm && conn.is_none() {
        bail!("internal error: --confirm requires database connection");
    }

    let (mut report, trip_json) =
        materialize_proposal_envelope(path, &options.params, options.dry_run, options.confirm)?;

    if !report.valid {
        if options.json {
            print_json(&report)?;
        } else {
            print_proposal_materialize_report(&report);
        }
        anyhow::bail!("proposal materialize に失敗しました");
    }

    let trip_json = trip_json.expect("valid materialize must produce trip JSON");

    let import_summary = if options.confirm {
        Some(import_trip_from_json_with_summary(
            conn.expect("--confirm DB"),
            &trip_json,
        )?)
    } else {
        None
    };

    if let Some(summary) = &import_summary {
        report.trip_id = Some(summary.trip_id);
    }

    if options.dry_run {
        match options.output.as_deref() {
            Some(path) => {
                std::fs::write(path, &trip_json)
                    .with_context(|| format!("ファイル '{path}' への書き込みに失敗しました"))?;
            }
            None if !options.json && !options.confirm => println!("{trip_json}"),
            _ => {}
        }
    }

    if options.json {
        print_json(&report)?;
    } else {
        print_proposal_materialize_report(&report);
        if let Some(path) = options.output.as_deref() {
            if options.dry_run {
                println!("schema v8 Trip JSON 候補を書き込みました: {path}");
            }
        }
        if let Some(summary) = import_summary {
            println!();
            println!("Trip を DB に保存しました（proposal materialize --confirm）");
            print_trip_import_summary(&summary);
        }
    }

    Ok(())
}

pub fn materialize_proposal_envelope(
    path: &str,
    params: &ProposalMaterializeParams,
    dry_run: bool,
    confirm: bool,
) -> Result<(ProposalMaterializeDryRunReport, Option<String>)> {
    let json = std::fs::read_to_string(path)
        .with_context(|| format!("ファイル '{path}' を読み込めませんでした"))?;
    Ok(materialize_proposal_envelope_json(
        path, &json, params, dry_run, confirm,
    ))
}

pub fn materialize_proposal_envelope_json(
    path: &str,
    json: &str,
    params: &ProposalMaterializeParams,
    dry_run: bool,
    confirm: bool,
) -> (ProposalMaterializeDryRunReport, Option<String>) {
    let mut report = ProposalMaterializeDryRunReport::new(path, dry_run, confirm);
    let validation = analyze_proposal_envelope_json(path, json);
    report.envelope_valid = validation.valid;
    report.warnings.extend(validation.warnings.clone());

    if !validation.valid {
        report
            .errors
            .push("Trip Proposal Envelope の validation に失敗しました（proposal validate を先に通してください）".to_string());
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

    let proposal = match root.get("proposal").and_then(Value::as_object) {
        Some(obj) => obj,
        None => {
            report.errors.push("proposal object が必要です".to_string());
            return (report, None);
        }
    };

    collect_required_decisions(
        root.get("materialize_hints"),
        &mut report.required_decisions,
    );
    if report.required_decisions.is_empty() && needs_date_decision(proposal) {
        report
            .required_decisions
            .push("start_date / end_date の確定".to_string());
    }

    let (start_date, end_date) = match resolve_materialize_dates(proposal, params) {
        Ok(dates) => dates,
        Err(error) => {
            report.errors.push(error);
            return (report, None);
        }
    };

    let day_count = match validate_trip_date_range(&start_date, &end_date) {
        Ok(count) => count,
        Err(error) => {
            report.errors.push(error.to_string());
            return (report, None);
        }
    };

    let days = build_export_days(proposal, day_count, &mut report.warnings);
    if !has_adoptable_content(&days) {
        report
            .errors
            .push("採用可能な Day / Itinerary がありません（candidate_days / candidate_itineraries が空です）".to_string());
        return (report, None);
    }

    for day in &days {
        if day.day_number < 1 || day.day_number > day_count {
            report.errors.push(format!(
                "days[].day_number ({}) が旅行期間 (1..={day_count}) の範囲外です",
                day.day_number
            ));
        }
    }
    if !report.errors.is_empty() {
        return (report, None);
    }

    let trip_name =
        non_empty_string(proposal.get("title")).unwrap_or_else(|| "Untitled trip".to_string());
    let now = db::now_string();
    let export = TripExportV3 {
        schema_version: Some(TRIP_EXPORT_SCHEMA_VERSION),
        generator: Some(TRIP_EXPORT_GENERATOR.to_string()),
        generator_version: Some(env!("CARGO_PKG_VERSION").to_string()),
        exported_at: Some(Utc::now().to_rfc3339()),
        trip: Trip {
            id: 0,
            name: trip_name.clone(),
            start_date: Some(start_date.clone()),
            end_date: Some(end_date.clone()),
            summary: non_empty_string(proposal.get("notes")),
            main_destination: None,
            main_destination_country_code: None,
            default_currency: None,
            created_at: now.clone(),
            updated_at: now,
        },
        days,
        checklist_items: Some(Vec::new()),
        notes: Some(Vec::new()),
        participants: Some(Vec::new()),
        receipts: Vec::new(),
    };

    let itinerary_count = export.days.iter().map(|d| d.itineraries.len()).sum();
    report.output = Some(ProposalMaterializeOutputSummary {
        trip_name,
        start_date: start_date.clone(),
        end_date: end_date.clone(),
        day_count,
        itinerary_count,
    });

    let trip_json = match serde_json::to_string_pretty(&export) {
        Ok(json) => json,
        Err(error) => {
            report
                .errors
                .push(format!("schema v8 Trip JSON の生成に失敗しました: {error}"));
            return (report, None);
        }
    };

    let export_validation = analyze_trip_export_json(path, &trip_json);
    report.trip_export_valid = export_validation.valid;
    if !export_validation.valid {
        report.errors.push(
            "生成した schema v8 Trip JSON が trip validate-export に合格しませんでした".to_string(),
        );
        report.errors.extend(export_validation.errors);
        return (report, None);
    }

    report.valid = true;
    (report, Some(trip_json))
}

fn non_empty_string(value: Option<&Value>) -> Option<String> {
    value
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
}

fn needs_date_decision(proposal: &Map<String, Value>) -> bool {
    resolve_materialize_dates(proposal, &ProposalMaterializeParams::default()).is_err()
}

fn collect_required_decisions(hints: Option<&Value>, required_decisions: &mut Vec<String>) {
    let Some(items) = hints
        .and_then(Value::as_object)
        .and_then(|obj| obj.get("required_decisions"))
        .and_then(Value::as_array)
    else {
        return;
    };

    for item in items {
        if let Some(text) = non_empty_string(Some(item)) {
            if !required_decisions.iter().any(|existing| existing == &text) {
                required_decisions.push(text);
            }
        }
    }
}

fn resolve_materialize_dates(
    proposal: &Map<String, Value>,
    params: &ProposalMaterializeParams,
) -> Result<(String, String), String> {
    if params.start_date.is_some() ^ params.end_date.is_some() {
        return Err("--start と --end は両方指定してください".to_string());
    }

    if let (Some(start), Some(end)) = (&params.start_date, &params.end_date) {
        return Ok((start.clone(), end.clone()));
    }

    let start = non_empty_string(proposal.get("start_date"));
    let end = non_empty_string(proposal.get("end_date"));
    if let (Some(start), Some(end)) = (start, end) {
        return Ok((start, end));
    }

    if let Some(ranges) = proposal
        .get("candidate_date_ranges")
        .and_then(Value::as_array)
    {
        for range in ranges {
            let Some(obj) = range.as_object() else {
                continue;
            };
            let confirmed = obj
                .get("confirmed")
                .and_then(Value::as_bool)
                .unwrap_or(false);
            if !confirmed {
                continue;
            }
            let start = non_empty_string(obj.get("start"));
            let end = non_empty_string(obj.get("end"));
            if let (Some(start), Some(end)) = (start, end) {
                return Ok((start, end));
            }
        }
    }

    Err("start_date / end_date が未確定です（proposal 内の確定日付、confirmed な candidate_date_ranges、または --start / --end を指定してください）".to_string())
}

fn build_export_days(
    proposal: &Map<String, Value>,
    day_count: i64,
    warnings: &mut Vec<String>,
) -> Vec<ExportDayV3> {
    let mut days_by_number: std::collections::BTreeMap<i64, ExportDayV3> =
        std::collections::BTreeMap::new();

    if let Some(candidate_days) = proposal.get("candidate_days").and_then(Value::as_array) {
        for (index, day) in candidate_days.iter().enumerate() {
            let Some(obj) = day.as_object() else {
                warnings.push(format!(
                    "candidate_days[{index}] は object である必要があります — スキップしました"
                ));
                continue;
            };
            let day_number = obj
                .get("day_number")
                .and_then(Value::as_i64)
                .filter(|n| *n >= 1)
                .unwrap_or((index as i64) + 1);
            if day_number > day_count {
                warnings.push(format!(
                    "candidate_days[{index}].day_number ({day_number}) が旅行期間外です — スキップしました"
                ));
                continue;
            }
            let summary =
                non_empty_string(obj.get("summary")).or_else(|| non_empty_string(obj.get("label")));
            days_by_number
                .entry(day_number)
                .or_insert_with(|| ExportDayV3 {
                    day_number,
                    summary: summary.clone(),
                    itineraries: Vec::new(),
                });
            if let Some(entry) = days_by_number.get_mut(&day_number) {
                if entry.summary.is_none() {
                    entry.summary = summary;
                }
            }
        }
    }

    if let Some(candidate_itineraries) = proposal
        .get("candidate_itineraries")
        .and_then(Value::as_array)
    {
        for (index, itinerary) in candidate_itineraries.iter().enumerate() {
            let Some(obj) = itinerary.as_object() else {
                warnings.push(format!(
                    "candidate_itineraries[{index}] は object である必要があります — スキップしました"
                ));
                continue;
            };
            let title = match non_empty_string(obj.get("title")) {
                Some(title) => title,
                None => {
                    warnings.push(format!(
                        "candidate_itineraries[{index}].title が空です — スキップしました"
                    ));
                    continue;
                }
            };
            let day_number = obj
                .get("day_number")
                .and_then(Value::as_i64)
                .filter(|n| *n >= 1)
                .unwrap_or(1);
            if day_number > day_count {
                warnings.push(format!(
                    "candidate_itineraries[{index}].day_number ({day_number}) が旅行期間外です — スキップしました"
                ));
                continue;
            }
            let sort_order = obj
                .get("sort_order")
                .and_then(Value::as_i64)
                .unwrap_or_else(|| {
                    days_by_number
                        .get(&day_number)
                        .map(|day| day.itineraries.len() as i64 + 1)
                        .unwrap_or(1)
                });
            let entry = days_by_number
                .entry(day_number)
                .or_insert_with(|| ExportDayV3 {
                    day_number,
                    summary: None,
                    itineraries: Vec::new(),
                });
            entry.itineraries.push(ExportItineraryV3 {
                title,
                note: non_empty_string(obj.get("note")),
                start_time: non_empty_string(obj.get("start_time")),
                sort_order,
                duration_minutes: obj.get("duration_minutes").and_then(Value::as_i64),
                travel_minutes: obj.get("travel_minutes").and_then(Value::as_i64),
                location: non_empty_string(obj.get("location")),
                category: parse_itinerary_category(obj.get("category")),
                expenses: Vec::new(),
                estimates: Vec::new(),
                reservations: Vec::new(),
            });
        }
    }

    days_by_number.into_values().collect()
}

fn parse_itinerary_category(
    value: Option<&Value>,
) -> Option<crate::domain::models::ItineraryCategory> {
    let text = non_empty_string(value)?;
    serde_json::from_value(Value::String(text)).ok()
}

fn has_adoptable_content(days: &[ExportDayV3]) -> bool {
    days.iter().any(|day| {
        day.summary
            .as_deref()
            .is_some_and(|summary| !summary.trim().is_empty())
            || day
                .itineraries
                .iter()
                .any(|item| !item.title.trim().is_empty())
    })
}

fn print_proposal_materialize_report(report: &ProposalMaterializeDryRunReport) {
    let title = if report.confirm {
        "Materialize confirm result:"
    } else {
        "Materialize dry-run result:"
    };
    println!("{title}");
    println!("  file: {}", report.file);
    println!("  dry_run: {}", report.dry_run);
    println!("  confirm: {}", report.confirm);
    println!("  valid: {}", report.valid);
    println!("  envelope_valid: {}", report.envelope_valid);
    println!("  trip_export_valid: {}", report.trip_export_valid);

    if let Some(output) = &report.output {
        println!("  trip_name: {}", output.trip_name);
        println!("  start_date: {}", output.start_date);
        println!("  end_date: {}", output.end_date);
        println!("  day_count: {}", output.day_count);
        println!("  itinerary_count: {}", output.itinerary_count);
    }

    if let Some(trip_id) = report.trip_id {
        println!("  trip_id: {trip_id}");
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

    const MATERIALIZE_READY: &str = r#"{
      "metadata": {
        "proposal_id": "prop-materialize-ready",
        "created_at": "2026-03-01T09:00:00Z",
        "source": "manual",
        "provider": "fixture"
      },
      "proposal": {
        "title": "Okinawa weekend draft",
        "destination": "Okinawa, Japan",
        "date_policy": "fixed_dates",
        "start_date": "2026-04-26",
        "end_date": "2026-04-27",
        "candidate_days": [
          { "day_number": 1, "summary": "Arrival" },
          { "day_number": 2, "summary": "Sightseeing" }
        ],
        "candidate_itineraries": [
          {
            "day_number": 1,
            "title": "Flight to Naha",
            "sort_order": 1,
            "category": "flight"
          },
          {
            "day_number": 2,
            "title": "Churaumi Aquarium",
            "sort_order": 1,
            "category": "activity"
          }
        ]
      },
      "materialize_hints": {
        "required_decisions": [],
        "warnings": []
      }
    }"#;

    #[test]
    fn dry_run_generates_valid_trip_export() {
        let (report, json) = materialize_proposal_envelope_json(
            "test.json",
            MATERIALIZE_READY,
            &ProposalMaterializeParams::default(),
            true,
            false,
        );
        assert!(report.valid, "errors: {:?}", report.errors);
        assert!(report.envelope_valid);
        assert!(report.trip_export_valid);
        let json = json.expect("trip json");
        let export_report = analyze_trip_export_json("candidate.json", &json);
        assert!(export_report.valid, "errors: {:?}", export_report.errors);
        assert_eq!(
            export_report.export_schema_version,
            Some(TRIP_EXPORT_SCHEMA_VERSION)
        );
    }

    #[test]
    fn unresolved_dates_block_materialize() {
        let json = r#"{
          "metadata": { "created_at": "2026-03-01T09:00:00Z", "source": "ai" },
          "proposal": {
            "title": "Undated trip",
            "destination": "Kyoto",
            "date_policy": "undated",
            "candidate_days": [{ "label": "Day 1", "summary": "Walk" }],
            "notes": "Dates TBD"
          }
        }"#;
        let (report, trip_json) = materialize_proposal_envelope_json(
            "test.json",
            json,
            &ProposalMaterializeParams::default(),
            true,
            false,
        );
        assert!(!report.valid);
        assert!(trip_json.is_none());
        assert!(report
            .errors
            .iter()
            .any(|e| e.contains("start_date") || e.contains("end_date")));
    }

    #[test]
    fn cli_date_flags_resolve_flexible_envelope() {
        let json = r#"{
          "metadata": { "created_at": "2026-03-01T09:00:00Z", "source": "ai" },
          "proposal": {
            "title": "Flexible trip",
            "destination": "Kyoto",
            "date_policy": "flexible_dates",
            "candidate_days": [{ "summary": "Temple visit" }],
            "notes": "Hotel pending"
          }
        }"#;
        let params = ProposalMaterializeParams {
            start_date: Some("2026-05-01".to_string()),
            end_date: Some("2026-05-01".to_string()),
        };
        let (report, json) =
            materialize_proposal_envelope_json("test.json", json, &params, true, false);
        assert!(report.valid, "errors: {:?}", report.errors);
        let json = json.expect("trip json");
        let export_report = analyze_trip_export_json("candidate.json", &json);
        assert!(export_report.valid);
    }
}
