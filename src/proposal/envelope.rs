use anyhow::{Context, Result};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::output::json::print_json;

pub const PROPOSAL_VALIDATION_REPORT_SCHEMA_VERSION: i32 = 1;

const VALID_DATE_POLICIES: &[&str] = &["fixed_dates", "flexible_dates", "undated"];

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProposalEnvelopeSummary {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub destination: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date_policy: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub valid_until: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProposalEnvelopeValidationReport {
    pub schema_version: i32,
    pub file: String,
    pub valid: bool,
    pub document_kind: String,
    pub summary: ProposalEnvelopeSummary,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

impl ProposalEnvelopeValidationReport {
    fn new(file: impl Into<String>) -> Self {
        Self {
            schema_version: PROPOSAL_VALIDATION_REPORT_SCHEMA_VERSION,
            file: file.into(),
            valid: false,
            document_kind: "trip_proposal_envelope".to_string(),
            summary: ProposalEnvelopeSummary {
                title: None,
                destination: None,
                date_policy: None,
                created_at: None,
                valid_until: None,
                source: None,
                provider: None,
            },
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }
}

/// Proposal Envelope JSON ファイルを検証する（DB 不要）
pub fn analyze_proposal_envelope(path: &str) -> Result<ProposalEnvelopeValidationReport> {
    let json = std::fs::read_to_string(path)
        .with_context(|| format!("ファイル '{path}' を読み込めませんでした"))?;
    Ok(analyze_proposal_envelope_json(path, &json))
}

pub fn analyze_proposal_envelope_json(path: &str, json: &str) -> ProposalEnvelopeValidationReport {
    let mut report = ProposalEnvelopeValidationReport::new(path);

    let root: Value = match serde_json::from_str(json) {
        Ok(value) => value,
        Err(error) => {
            report
                .errors
                .push(format!("JSON の parse に失敗しました: {error}"));
            return report;
        }
    };

    let Some(root_obj) = root.as_object() else {
        report
            .errors
            .push("トップレベルが JSON object ではありません".to_string());
        return report;
    };

    collect_trip_export_like_errors(root_obj, &mut report.errors);

    let proposal = match root_obj.get("proposal") {
        Some(Value::Object(obj)) => obj,
        Some(_) => {
            report
                .errors
                .push("proposal は object である必要があります".to_string());
            report.valid = report.errors.is_empty();
            return report;
        }
        None => {
            report
                .errors
                .push("Trip Proposal Envelope には proposal object が必要です".to_string());
            report.valid = report.errors.is_empty();
            return report;
        }
    };

    let metadata = root_obj
        .get("metadata")
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default();

    let materialize_hints = root_obj.get("materialize_hints").and_then(Value::as_object);

    validate_proposal_fields(proposal, &mut report);
    fill_summary(proposal, &metadata, &mut report.summary);
    collect_metadata_warnings(&metadata, &mut report.warnings);
    collect_materialize_hint_warnings(materialize_hints, &mut report.warnings);
    collect_body_warnings(proposal, &mut report.warnings);

    report.valid = report.errors.is_empty();
    report
}

fn collect_trip_export_like_errors(
    root: &serde_json::Map<String, Value>,
    errors: &mut Vec<String>,
) {
    if root.contains_key("schema_version") {
        errors.push(
            "schema_version が含まれています — これは schema v8 Trip export の可能性が高く、Trip Proposal Envelope ではありません（trip validate-export を使用してください）".to_string(),
        );
    }

    if let Some(trip) = root.get("trip") {
        if trip.is_object() {
            errors.push(
                "トップレベル trip object が含まれています — schema v8 Trip export の可能性が高く、Trip Proposal Envelope ではありません".to_string(),
            );
        }
    }

    if root.contains_key("itinerary_items") {
        errors.push(
            "itinerary_items が含まれています — schema v8 Trip export（legacy flat）の可能性が高く、Trip Proposal Envelope ではありません".to_string(),
        );
    }

    if root.contains_key("days")
        && root.get("days").and_then(Value::as_array).is_some()
        && !root.contains_key("proposal")
    {
        errors.push(
            "days 配列のみの export 形式です — Trip Proposal Envelope ではありません".to_string(),
        );
    }
}

fn non_empty_string(value: Option<&Value>) -> Option<String> {
    value
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
}

fn validate_proposal_fields(
    proposal: &serde_json::Map<String, Value>,
    report: &mut ProposalEnvelopeValidationReport,
) {
    let title = non_empty_string(proposal.get("title"));
    if title.is_none() {
        report.errors.push("proposal.title が必要です".to_string());
    }

    let destination = non_empty_string(proposal.get("destination"));
    if destination.is_none() {
        report
            .errors
            .push("proposal.destination が必要です".to_string());
    }

    let date_policy = non_empty_string(proposal.get("date_policy"));
    match &date_policy {
        None => report
            .errors
            .push("proposal.date_policy が必要です".to_string()),
        Some(policy) if !VALID_DATE_POLICIES.contains(&policy.as_str()) => {
            report.errors.push(format!(
                "proposal.date_policy が不明です: {policy}（fixed_dates / flexible_dates / undated のいずれか）"
            ));
        }
        Some(policy) if policy == "fixed_dates" && !fixed_dates_requirements_met(proposal) => {
            report.errors.push(
                "date_policy が fixed_dates ですが、確定日付（proposal.start_date / proposal.end_date、または confirmed な candidate_date_ranges）が不足しています".to_string(),
            );
        }
        _ => {}
    }

    if proposal_body_nearly_empty(proposal) {
        report.errors.push(
            "proposal body が空に近いです（candidate_days / candidate_itineraries / candidate_date_ranges / notes のいずれかが必要）".to_string(),
        );
    }
}

fn fixed_dates_requirements_met(proposal: &serde_json::Map<String, Value>) -> bool {
    let start = non_empty_string(proposal.get("start_date"));
    let end = non_empty_string(proposal.get("end_date"));
    if start.is_some() && end.is_some() {
        return true;
    }

    proposal
        .get("candidate_date_ranges")
        .and_then(Value::as_array)
        .is_some_and(|ranges| {
            ranges.iter().any(|range| {
                let Some(obj) = range.as_object() else {
                    return false;
                };
                let confirmed = obj
                    .get("confirmed")
                    .and_then(Value::as_bool)
                    .unwrap_or(false);
                confirmed
                    && non_empty_string(obj.get("start")).is_some()
                    && non_empty_string(obj.get("end")).is_some()
            })
        })
}

fn proposal_body_nearly_empty(proposal: &serde_json::Map<String, Value>) -> bool {
    let has_days = proposal
        .get("candidate_days")
        .and_then(Value::as_array)
        .is_some_and(|a| !a.is_empty());
    let has_itineraries = proposal
        .get("candidate_itineraries")
        .and_then(Value::as_array)
        .is_some_and(|a| !a.is_empty());
    let has_ranges = proposal
        .get("candidate_date_ranges")
        .and_then(Value::as_array)
        .is_some_and(|a| !a.is_empty());
    let has_notes = non_empty_string(proposal.get("notes")).is_some();

    !has_days && !has_itineraries && !has_ranges && !has_notes
}

fn fill_summary(
    proposal: &serde_json::Map<String, Value>,
    metadata: &serde_json::Map<String, Value>,
    summary: &mut ProposalEnvelopeSummary,
) {
    summary.title = non_empty_string(proposal.get("title"));
    summary.destination = non_empty_string(proposal.get("destination"));
    summary.date_policy = non_empty_string(proposal.get("date_policy"));
    summary.created_at = non_empty_string(metadata.get("created_at"));
    summary.valid_until = non_empty_string(metadata.get("valid_until"));
    summary.source = non_empty_string(metadata.get("source"));
    summary.provider = non_empty_string(metadata.get("provider"));
}

fn collect_metadata_warnings(
    metadata: &serde_json::Map<String, Value>,
    warnings: &mut Vec<String>,
) {
    let created_at = non_empty_string(metadata.get("created_at"));
    let valid_until = non_empty_string(metadata.get("valid_until"));

    if created_at.is_none() {
        warnings.push("metadata.created_at がありません".to_string());
    }

    if non_empty_string(metadata.get("source")).is_none() {
        warnings.push("metadata.source がありません".to_string());
    }

    if non_empty_string(metadata.get("provider")).is_none() {
        warnings.push("metadata.provider がありません".to_string());
    }

    let now = Utc::now();

    if let Some(until_str) = &valid_until {
        if let Some(until) = parse_datetime(until_str) {
            if until < now {
                warnings.push(format!("metadata.valid_until を過ぎています: {until_str}"));
            }
        } else {
            warnings.push(format!(
                "metadata.valid_until の形式を解釈できません: {until_str}"
            ));
        }
    } else if let Some(created_str) = &created_at {
        if let Some(created) = parse_datetime(created_str) {
            if now > created + Duration::days(365) {
                warnings.push(
                    "metadata.valid_until がなく、created_at から 1 年を超えています（提案が古い可能性）"
                        .to_string(),
                );
            }
        }
    }
}

fn collect_materialize_hint_warnings(
    hints: Option<&serde_json::Map<String, Value>>,
    warnings: &mut Vec<String>,
) {
    let Some(hints) = hints else {
        return;
    };

    if hints
        .get("missing_fields")
        .and_then(Value::as_array)
        .is_some_and(|a| !a.is_empty())
    {
        warnings.push("materialize_hints.missing_fields があります".to_string());
    }

    if hints
        .get("assumptions")
        .and_then(Value::as_array)
        .is_some_and(|a| !a.is_empty())
    {
        warnings.push("materialize_hints.assumptions があります".to_string());
    }

    if hints
        .get("warnings")
        .and_then(Value::as_array)
        .is_some_and(|a| !a.is_empty())
    {
        warnings.push("materialize_hints.warnings があります".to_string());
    }
}

fn collect_body_warnings(proposal: &serde_json::Map<String, Value>, warnings: &mut Vec<String>) {
    if proposal
        .get("candidate_days")
        .and_then(Value::as_array)
        .is_none_or(|a| a.is_empty())
    {
        warnings.push("proposal.candidate_days が空です".to_string());
    }

    if proposal
        .get("candidate_itineraries")
        .and_then(Value::as_array)
        .is_none_or(|a| a.is_empty())
    {
        warnings.push("proposal.candidate_itineraries が空です".to_string());
    }
}

fn parse_datetime(text: &str) -> Option<DateTime<Utc>> {
    if let Ok(dt) = DateTime::parse_from_rfc3339(text) {
        return Some(dt.with_timezone(&Utc));
    }
    if let Ok(naive) = chrono::NaiveDateTime::parse_from_str(text, "%Y-%m-%d %H:%M:%S") {
        return Some(naive.and_utc());
    }
    if let Ok(date) = chrono::NaiveDate::parse_from_str(text, "%Y-%m-%d") {
        return Some(date.and_hms_opt(0, 0, 0)?.and_utc());
    }
    None
}

pub fn print_proposal_envelope_validation_report(report: &ProposalEnvelopeValidationReport) {
    println!("Proposal file: {}", report.file);
    println!();
    println!(
        "Validation result:\n  {}",
        if report.valid { "valid" } else { "invalid" }
    );

    println!();
    println!("Blocking errors:");
    if report.errors.is_empty() {
        println!("  なし");
    } else {
        for error in &report.errors {
            println!("  - {error}");
        }
    }

    println!();
    println!("Warnings:");
    if report.warnings.is_empty() {
        println!("  なし");
    } else {
        for warning in &report.warnings {
            println!("  - {warning}");
        }
    }

    println!();
    println!("Summary:");
    println!(
        "  Title       : {}",
        report.summary.title.as_deref().unwrap_or("-")
    );
    println!(
        "  Destination : {}",
        report.summary.destination.as_deref().unwrap_or("-")
    );
    println!(
        "  Date policy : {}",
        report.summary.date_policy.as_deref().unwrap_or("-")
    );
    println!(
        "  Created at  : {}",
        report.summary.created_at.as_deref().unwrap_or("-")
    );
    println!(
        "  Valid until : {}",
        report.summary.valid_until.as_deref().unwrap_or("-")
    );
    println!(
        "  Source      : {}",
        report.summary.source.as_deref().unwrap_or("-")
    );
    println!(
        "  Provider    : {}",
        report.summary.provider.as_deref().unwrap_or("-")
    );

    println!();
    println!("Result:");
    if report.valid {
        println!("  有効な Trip Proposal Envelope ファイル");
    } else {
        println!("  無効な Trip Proposal Envelope ファイル");
    }
}

pub fn run_proposal_validate(path: &str, json: bool) -> Result<()> {
    let report = analyze_proposal_envelope(path)?;
    if json {
        print_json(&report)?;
    } else {
        print_proposal_envelope_validation_report(&report);
    }
    if !report.valid {
        anyhow::bail!("無効な Trip Proposal Envelope ファイルです");
    }
    Ok(())
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ProposalEnvelopeDetails {
    pub proposal_id: Option<String>,
    pub notes: Option<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub candidate_days_count: usize,
    pub candidate_itineraries_count: usize,
    pub candidate_date_ranges_count: usize,
    pub missing_fields_count: usize,
    pub assumptions_count: usize,
    pub hint_warnings_count: usize,
    pub required_decisions_count: usize,
    pub missing_fields: Vec<String>,
    pub assumptions: Vec<String>,
    pub hint_warnings: Vec<String>,
    pub required_decisions: Vec<String>,
    pub candidate_days_preview: Vec<String>,
    pub candidate_itineraries_preview: Vec<String>,
    pub candidate_date_ranges_preview: Vec<String>,
    pub expired: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProposalEnvelopeView {
    pub report: ProposalEnvelopeValidationReport,
    pub details: ProposalEnvelopeDetails,
}

pub fn load_proposal_envelope_view(path: &str) -> Result<ProposalEnvelopeView> {
    let json = std::fs::read_to_string(path)
        .with_context(|| format!("ファイル '{path}' を読み込めませんでした"))?;
    let report = analyze_proposal_envelope_json(path, &json);
    let details = extract_proposal_envelope_details(&json, &report);
    Ok(ProposalEnvelopeView { report, details })
}

fn extract_proposal_envelope_details(
    json: &str,
    report: &ProposalEnvelopeValidationReport,
) -> ProposalEnvelopeDetails {
    let mut details = ProposalEnvelopeDetails::default();

    let Ok(root) = serde_json::from_str::<Value>(json) else {
        return details;
    };
    let Some(root_obj) = root.as_object() else {
        return details;
    };

    if let Some(metadata) = root_obj.get("metadata").and_then(Value::as_object) {
        details.proposal_id = non_empty_string(metadata.get("proposal_id"));
    }

    if let Some(proposal) = root_obj.get("proposal").and_then(Value::as_object) {
        details.notes = non_empty_string(proposal.get("notes"));
        details.start_date = non_empty_string(proposal.get("start_date"));
        details.end_date = non_empty_string(proposal.get("end_date"));

        if let Some(days) = proposal.get("candidate_days").and_then(Value::as_array) {
            details.candidate_days_count = days.len();
            details.candidate_days_preview =
                days.iter().filter_map(preview_candidate_day).collect();
        }

        if let Some(items) = proposal
            .get("candidate_itineraries")
            .and_then(Value::as_array)
        {
            details.candidate_itineraries_count = items.len();
            details.candidate_itineraries_preview = items
                .iter()
                .filter_map(preview_candidate_itinerary)
                .collect();
        }

        if let Some(ranges) = proposal
            .get("candidate_date_ranges")
            .and_then(Value::as_array)
        {
            details.candidate_date_ranges_count = ranges.len();
            details.candidate_date_ranges_preview =
                ranges.iter().filter_map(preview_date_range).collect();
        }
    }

    if let Some(hints) = root_obj.get("materialize_hints").and_then(Value::as_object) {
        details.missing_fields = string_array(hints.get("missing_fields"));
        details.assumptions = string_array(hints.get("assumptions"));
        details.hint_warnings = string_array(hints.get("warnings"));
        details.required_decisions = string_array(hints.get("required_decisions"));
        details.missing_fields_count = details.missing_fields.len();
        details.assumptions_count = details.assumptions.len();
        details.hint_warnings_count = details.hint_warnings.len();
        details.required_decisions_count = details.required_decisions.len();
    }

    details.expired = report
        .warnings
        .iter()
        .any(|w| w.contains("valid_until を過ぎ") || w.contains("1 年を超え"));

    details
}

fn string_array(value: Option<&Value>) -> Vec<String> {
    value
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(|item| item.as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default()
}

fn preview_candidate_day(value: &Value) -> Option<String> {
    let obj = value.as_object()?;
    let label = non_empty_string(obj.get("label")).unwrap_or_else(|| "(no label)".to_string());
    let summary = non_empty_string(obj.get("summary"));
    let date = non_empty_string(obj.get("date")).unwrap_or_else(|| "date TBD".to_string());
    Some(match summary {
        Some(summary) => format!("{label}: {summary} [{date}]"),
        None => format!("{label} [{date}]"),
    })
}

fn preview_candidate_itinerary(value: &Value) -> Option<String> {
    let obj = value.as_object()?;
    let title = non_empty_string(obj.get("title")).unwrap_or_else(|| "(no title)".to_string());
    Some(title)
}

fn preview_date_range(value: &Value) -> Option<String> {
    let obj = value.as_object()?;
    let label = non_empty_string(obj.get("label")).unwrap_or_else(|| "(no label)".to_string());
    let start = non_empty_string(obj.get("start")).unwrap_or_else(|| "?".to_string());
    let end = non_empty_string(obj.get("end")).unwrap_or_else(|| "?".to_string());
    let confirmed = obj
        .get("confirmed")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    Some(format!("{label}: {start} .. {end} (confirmed={confirmed})"))
}

fn is_not_proposal_envelope_file(report: &ProposalEnvelopeValidationReport) -> bool {
    report.errors.iter().any(|error| {
        error.contains("parse")
            || error.contains("Trip Proposal Envelope ではありません")
            || error.contains("trip validate-export")
            || error.contains("proposal object が必要")
            || error.contains("JSON object ではありません")
    })
}

pub fn print_proposal_show(view: &ProposalEnvelopeView) {
    let report = &view.report;
    let details = &view.details;

    println!("Trip Proposal Envelope");
    println!("File: {}", report.file);
    println!();
    println!(
        "Title                 : {}",
        report.summary.title.as_deref().unwrap_or("-")
    );
    println!(
        "Destination           : {}",
        report.summary.destination.as_deref().unwrap_or("-")
    );
    println!(
        "Date policy           : {}",
        report.summary.date_policy.as_deref().unwrap_or("-")
    );
    println!(
        "Created at            : {}",
        report.summary.created_at.as_deref().unwrap_or("-")
    );
    println!(
        "Valid until           : {}",
        report.summary.valid_until.as_deref().unwrap_or("-")
    );
    println!(
        "Expired               : {}",
        if details.expired { "yes" } else { "no" }
    );
    println!(
        "Source                : {}",
        report.summary.source.as_deref().unwrap_or("-")
    );
    println!(
        "Provider              : {}",
        report.summary.provider.as_deref().unwrap_or("-")
    );
    println!();
    println!("Candidate days        : {}", details.candidate_days_count);
    println!(
        "Candidate itineraries : {}",
        details.candidate_itineraries_count
    );
    println!("Missing fields        : {}", details.missing_fields_count);
    println!("Assumptions           : {}", details.assumptions_count);
    println!("Hint warnings         : {}", details.hint_warnings_count);
    println!();
    println!(
        "Validation            : {}",
        if report.valid { "valid" } else { "invalid" }
    );
    println!("Blocking errors       : {}", report.errors.len());
    println!("Validation warnings   : {}", report.warnings.len());

    if !report.errors.is_empty() {
        println!();
        println!("Blocking errors:");
        for error in &report.errors {
            println!("  - {error}");
        }
    }
}

pub fn print_proposal_inspect(view: &ProposalEnvelopeView) {
    print_proposal_show(view);

    let report = &view.report;
    let details = &view.details;

    println!();
    println!("--- Inspect details ---");
    println!();
    println!("Metadata:");
    println!(
        "  Proposal ID : {}",
        details.proposal_id.as_deref().unwrap_or("-")
    );
    println!(
        "  Created at  : {}",
        report.summary.created_at.as_deref().unwrap_or("-")
    );
    println!(
        "  Valid until : {}",
        report.summary.valid_until.as_deref().unwrap_or("-")
    );
    println!(
        "  Source      : {}",
        report.summary.source.as_deref().unwrap_or("-")
    );
    println!(
        "  Provider    : {}",
        report.summary.provider.as_deref().unwrap_or("-")
    );

    println!();
    println!("Date policy:");
    println!(
        "  Policy      : {}",
        report.summary.date_policy.as_deref().unwrap_or("-")
    );
    println!(
        "  Start date  : {}",
        details.start_date.as_deref().unwrap_or("-")
    );
    println!(
        "  End date    : {}",
        details.end_date.as_deref().unwrap_or("-")
    );
    print_string_list("  Date ranges", &details.candidate_date_ranges_preview);

    if let Some(notes) = &details.notes {
        println!();
        println!("Notes:");
        println!("  {notes}");
    }

    println!();
    println!("Materialize hints:");
    print_string_list("  Required decisions", &details.required_decisions);
    print_string_list("  Missing fields", &details.missing_fields);
    print_string_list("  Assumptions", &details.assumptions);
    print_string_list("  Warnings", &details.hint_warnings);

    println!();
    print_string_list("Candidate days", &details.candidate_days_preview);
    print_string_list(
        "Candidate itineraries",
        &details.candidate_itineraries_preview,
    );

    println!();
    println!("Validation warnings:");
    if report.warnings.is_empty() {
        println!("  なし");
    } else {
        for warning in &report.warnings {
            println!("  - {warning}");
        }
    }
}

fn print_string_list(title: &str, items: &[String]) {
    println!("{title}:");
    if items.is_empty() {
        println!("  なし");
    } else {
        for item in items {
            println!("  - {item}");
        }
    }
}

pub fn run_proposal_show(path: &str) -> Result<()> {
    let view = load_proposal_envelope_view(path)?;
    print_proposal_show(&view);
    if is_not_proposal_envelope_file(&view.report) {
        anyhow::bail!("Trip Proposal Envelope として読めないファイルです");
    }
    Ok(())
}

pub fn run_proposal_inspect(path: &str) -> Result<()> {
    let view = load_proposal_envelope_view(path)?;
    print_proposal_inspect(&view);
    if is_not_proposal_envelope_file(&view.report) {
        anyhow::bail!("Trip Proposal Envelope として読めないファイルです");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const VALID_ENVELOPE: &str = r#"{
      "metadata": {
        "proposal_id": "prop-test-01",
        "created_at": "2026-03-01T09:00:00Z",
        "source": "ai",
        "provider": "test-model"
      },
      "proposal": {
        "title": "Okinawa family trip (draft)",
        "destination": "Okinawa, Japan",
        "date_policy": "flexible_dates",
        "candidate_days": [{ "label": "Day 1", "summary": "Arrival" }],
        "notes": "Dates not confirmed."
      },
      "materialize_hints": {
        "missing_fields": ["hotel booking"],
        "assumptions": ["Family of five"],
        "warnings": []
      }
    }"#;

    #[test]
    fn valid_envelope_passes() {
        let report = analyze_proposal_envelope_json("test.json", VALID_ENVELOPE);
        assert!(report.valid, "errors: {:?}", report.errors);
        assert_eq!(
            report.summary.title.as_deref(),
            Some("Okinawa family trip (draft)")
        );
    }

    #[test]
    fn json_parse_error_fails() {
        let report = analyze_proposal_envelope_json("test.json", "{not json");
        assert!(!report.valid);
        assert!(report.errors.iter().any(|e| e.contains("parse")));
    }

    #[test]
    fn missing_title_fails() {
        let json = r#"{"proposal":{"destination":"X","date_policy":"undated","notes":"n"}}"#;
        let report = analyze_proposal_envelope_json("test.json", json);
        assert!(!report.valid);
        assert!(report.errors.iter().any(|e| e.contains("title")));
    }

    #[test]
    fn missing_destination_fails() {
        let json = r#"{"proposal":{"title":"T","date_policy":"undated","notes":"n"}}"#;
        let report = analyze_proposal_envelope_json("test.json", json);
        assert!(!report.valid);
        assert!(report.errors.iter().any(|e| e.contains("destination")));
    }

    #[test]
    fn missing_date_policy_fails() {
        let json = r#"{"proposal":{"title":"T","destination":"D","notes":"n"}}"#;
        let report = analyze_proposal_envelope_json("test.json", json);
        assert!(!report.valid);
        assert!(report.errors.iter().any(|e| e.contains("date_policy")));
    }

    #[test]
    fn unknown_date_policy_fails() {
        let json =
            r#"{"proposal":{"title":"T","destination":"D","date_policy":"maybe","notes":"n"}}"#;
        let report = analyze_proposal_envelope_json("test.json", json);
        assert!(!report.valid);
        assert!(report
            .errors
            .iter()
            .any(|e| e.contains("date_policy") && e.contains("maybe")));
    }

    #[test]
    fn schema_v8_trip_like_fails() {
        let json = r#"{"schema_version":8,"trip":{"name":"T","start_date":"2026-01-01","end_date":"2026-01-02"},"days":[]}"#;
        let report = analyze_proposal_envelope_json("test.json", json);
        assert!(!report.valid);
        assert!(report.errors.iter().any(|e| e.contains("schema_version")));
    }

    #[test]
    fn stale_valid_until_warns_but_passes() {
        let json = r#"{
          "metadata": {
            "created_at": "2024-01-01T00:00:00Z",
            "valid_until": "2024-06-01T00:00:00Z",
            "source": "ai"
          },
          "proposal": {
            "title": "Old trip",
            "destination": "Somewhere",
            "date_policy": "undated",
            "notes": "Still readable"
          }
        }"#;
        let report = analyze_proposal_envelope_json("test.json", json);
        assert!(report.valid, "errors: {:?}", report.errors);
        assert!(report.warnings.iter().any(|w| w.contains("valid_until")));
    }

    #[test]
    fn fixed_dates_without_dates_fails() {
        let json = r#"{"proposal":{"title":"T","destination":"D","date_policy":"fixed_dates","notes":"n"}}"#;
        let report = analyze_proposal_envelope_json("test.json", json);
        assert!(!report.valid);
        assert!(report.errors.iter().any(|e| e.contains("fixed_dates")));
    }

    #[test]
    fn nearly_empty_body_fails() {
        let json = r#"{"proposal":{"title":"T","destination":"D","date_policy":"undated"}}"#;
        let report = analyze_proposal_envelope_json("test.json", json);
        assert!(!report.valid);
        assert!(report.errors.iter().any(|e| e.contains("空に近い")));
    }
}
