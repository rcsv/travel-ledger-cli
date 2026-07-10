use anyhow::{Context, Result};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::output::json::print_json;

pub const FRAGMENT_VALIDATION_REPORT_SCHEMA_VERSION: i32 = 1;

const VALID_INTENTS: &[&str] = &[
    "add",
    "add_note",
    "add_expense",
    "add_estimate",
    "update_estimate",
    "add_reservation",
    "update_itinerary",
    "delete_itinerary",
    "reorder_itinerary",
    "move_itinerary",
    "enrich",
    "replace_candidate",
    "reorder_hint",
    "warning",
];

const VALID_TARGET_TYPES: &[&str] = &["trip", "day", "itinerary", "unresolved"];

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProposalFragmentSummary {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fragment_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_summary: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub intent: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub valid_until: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
    pub conflicts_count: usize,
    pub missing_fields_count: usize,
    pub assumptions_count: usize,
    pub warnings_count: usize,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProposalFragmentValidationReport {
    pub schema_version: i32,
    pub file: String,
    pub valid: bool,
    pub document_kind: String,
    pub summary: ProposalFragmentSummary,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

impl ProposalFragmentValidationReport {
    fn new(file: impl Into<String>) -> Self {
        Self {
            schema_version: FRAGMENT_VALIDATION_REPORT_SCHEMA_VERSION,
            file: file.into(),
            valid: false,
            document_kind: "proposal_fragment".to_string(),
            summary: ProposalFragmentSummary {
                fragment_id: None,
                target_type: None,
                target_summary: None,
                intent: None,
                created_at: None,
                valid_until: None,
                source: None,
                provider: None,
                conflicts_count: 0,
                missing_fields_count: 0,
                assumptions_count: 0,
                warnings_count: 0,
            },
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }
}

pub fn analyze_proposal_fragment(path: &str) -> Result<ProposalFragmentValidationReport> {
    let json = std::fs::read_to_string(path)
        .with_context(|| format!("ファイル '{path}' を読み込めませんでした"))?;
    Ok(analyze_proposal_fragment_json(path, &json))
}

pub fn analyze_proposal_fragment_json(path: &str, json: &str) -> ProposalFragmentValidationReport {
    let mut report = ProposalFragmentValidationReport::new(path);

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
    collect_envelope_like_errors(root_obj, &mut report.errors);

    let target = match root_obj.get("target") {
        Some(Value::Object(obj)) => obj,
        Some(_) => {
            report
                .errors
                .push("target は object である必要があります".to_string());
            report.valid = report.errors.is_empty();
            return report;
        }
        None => {
            report
                .errors
                .push("Proposal Fragment には target object が必要です".to_string());
            report.valid = report.errors.is_empty();
            return report;
        }
    };

    let fragment_body = match root_obj.get("fragment") {
        Some(Value::Object(obj)) => obj,
        Some(_) => {
            report
                .errors
                .push("fragment は object である必要があります".to_string());
            report.valid = report.errors.is_empty();
            return report;
        }
        None => {
            report
                .errors
                .push("Proposal Fragment には fragment object が必要です".to_string());
            report.valid = report.errors.is_empty();
            return report;
        }
    };

    let metadata = root_obj
        .get("metadata")
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default();

    let adoption_hints = root_obj.get("adoption_hints").and_then(Value::as_object);

    validate_target_fields(target, &mut report);
    validate_fragment_fields(fragment_body, &mut report);
    fill_fragment_summary(
        target,
        fragment_body,
        &metadata,
        adoption_hints,
        &mut report,
    );
    collect_fragment_metadata_warnings(&metadata, &mut report.warnings);
    collect_adoption_hint_warnings(adoption_hints, &mut report.warnings);
    collect_target_warnings(target, &mut report.warnings);

    report.summary.warnings_count = report.warnings.len();
    report.valid = report.errors.is_empty();
    report
}

fn collect_trip_export_like_errors(
    root: &serde_json::Map<String, Value>,
    errors: &mut Vec<String>,
) {
    if root.contains_key("schema_version") {
        errors.push(
            "schema_version が含まれています — これは schema v8 Trip export の可能性が高く、Proposal Fragment ではありません（trip validate-export を使用してください）".to_string(),
        );
    }

    if let Some(trip) = root.get("trip") {
        if trip.is_object() {
            errors.push(
                "トップレベル trip object が含まれています — schema v8 Trip export の可能性が高く、Proposal Fragment ではありません（trip validate-export を使用してください）".to_string(),
            );
        }
    }

    if root.contains_key("itinerary_items") {
        errors.push(
            "itinerary_items が含まれています — schema v8 Trip export（legacy flat）の可能性が高く、Proposal Fragment ではありません".to_string(),
        );
    }

    if root.contains_key("days")
        && root.get("days").and_then(Value::as_array).is_some()
        && !root.contains_key("fragment")
    {
        errors.push(
            "days 配列のみの export 形式です — schema v8 Trip export の可能性が高く、Proposal Fragment ではありません".to_string(),
        );
    }
}

fn collect_envelope_like_errors(root: &serde_json::Map<String, Value>, errors: &mut Vec<String>) {
    if root.contains_key("proposal") {
        errors.push(
            "proposal object が含まれています — Trip Proposal Envelope の可能性が高く、Proposal Fragment ではありません（proposal validate を使用してください）".to_string(),
        );
    }

    if root.contains_key("materialize_hints") && !root.contains_key("fragment") {
        errors.push(
            "materialize_hints が含まれています — Trip Proposal Envelope の可能性が高く、Proposal Fragment ではありません（proposal validate を使用してください）".to_string(),
        );
    }

    if let Some(proposal) = root.get("proposal").and_then(Value::as_object) {
        if proposal.contains_key("date_policy") || proposal.contains_key("destination") {
            errors.push(
                "Trip Proposal Envelope の特徴（date_policy / destination 等）が含まれています — Proposal Fragment ではありません（proposal validate を使用してください）".to_string(),
            );
        }
    }
}

fn non_empty_string(value: Option<&Value>) -> Option<String> {
    value
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
}

fn validate_target_fields(
    target: &serde_json::Map<String, Value>,
    report: &mut ProposalFragmentValidationReport,
) {
    let target_type = non_empty_string(target.get("target_type"));
    match &target_type {
        None => report
            .errors
            .push("target.target_type が必要です".to_string()),
        Some(kind) if !VALID_TARGET_TYPES.contains(&kind.as_str()) => {
            report.errors.push(format!(
                "target.target_type が不明です: {kind}（trip / day / itinerary / unresolved のいずれか）"
            ));
        }
        _ => {}
    }
}

fn validate_fragment_fields(
    fragment: &serde_json::Map<String, Value>,
    report: &mut ProposalFragmentValidationReport,
) {
    let intent = non_empty_string(fragment.get("intent"));
    match &intent {
        None => report.errors.push("fragment.intent が必要です".to_string()),
        Some(value) if !VALID_INTENTS.contains(&value.as_str()) => {
            report.errors.push(format!(
                "fragment.intent が想定範囲外です: {value}（add / add_note / add_expense / add_estimate / update_estimate / add_reservation / update_itinerary / delete_itinerary / reorder_itinerary / move_itinerary / enrich / replace_candidate / reorder_hint / warning のいずれか）"
            ));
        }
        _ => {}
    }

    if fragment_body_nearly_empty(fragment, intent.as_deref()) {
        report.errors.push(
            "fragment body が空に近いです（candidate_content または notes が必要）".to_string(),
        );
    }
}

fn fragment_body_nearly_empty(
    fragment: &serde_json::Map<String, Value>,
    intent: Option<&str>,
) -> bool {
    if intent == Some("add_note") {
        return add_note_body_from_fragment(fragment).is_none();
    }
    if intent == Some("add_expense") {
        return add_expense_body_nearly_empty(fragment);
    }
    if intent == Some("add_estimate") {
        return add_estimate_body_nearly_empty(fragment);
    }
    if intent == Some("update_estimate") {
        return update_estimate_body_nearly_empty(fragment);
    }
    if intent == Some("add_reservation") {
        return add_reservation_body_nearly_empty(fragment);
    }
    if intent == Some("update_itinerary") {
        return update_itinerary_body_nearly_empty(fragment);
    }
    if intent == Some("delete_itinerary") {
        return false;
    }
    let has_notes = non_empty_string(fragment.get("notes")).is_some();
    let candidate = fragment.get("candidate_content");
    let has_candidate = candidate
        .and_then(Value::as_object)
        .is_some_and(|obj| !obj.is_empty())
        || non_empty_string(candidate).is_some();
    !has_notes && !has_candidate
}

fn add_note_body_from_fragment(fragment: &serde_json::Map<String, Value>) -> Option<String> {
    if let Some(candidate) = fragment.get("candidate_content") {
        if let Some(obj) = candidate.as_object() {
            if let Some(body) = non_empty_string(obj.get("body")) {
                return Some(body);
            }
        } else if let Some(text) = non_empty_string(Some(candidate)) {
            return Some(text);
        }
    }
    non_empty_string(fragment.get("notes"))
}

fn add_expense_body_nearly_empty(fragment: &serde_json::Map<String, Value>) -> bool {
    let Some(candidate) = fragment.get("candidate_content").and_then(Value::as_object) else {
        return true;
    };
    let has_amount = candidate
        .get("amount")
        .is_some_and(|value| !value.is_null());
    let has_currency = non_empty_string(candidate.get("currency")).is_some();
    !(has_amount && has_currency)
}

fn add_estimate_body_nearly_empty(fragment: &serde_json::Map<String, Value>) -> bool {
    add_expense_body_nearly_empty(fragment)
}

fn update_estimate_body_nearly_empty(fragment: &serde_json::Map<String, Value>) -> bool {
    let Some(candidate) = fragment.get("candidate_content").and_then(Value::as_object) else {
        return true;
    };
    if candidate.is_empty() {
        return true;
    }
    // estimate_id のみでも fragment body は有効。更新フィールド 0 件は dry-run で拒否する。
    candidate.get("estimate_id").is_none()
        && !candidate
            .keys()
            .any(|key| is_update_estimate_field_key(key.as_str()))
}

fn is_update_estimate_field_key(key: &str) -> bool {
    matches!(
        key,
        "amount"
            | "currency"
            | "title"
            | "note"
            | "sort_order"
            | "clear_title"
            | "clear_note"
            | "expected_amount"
            | "expected_currency"
            | "expected_title"
            | "expected_note"
            | "expected_sort_order"
    )
}

fn update_itinerary_body_nearly_empty(fragment: &serde_json::Map<String, Value>) -> bool {
    let Some(candidate) = fragment.get("candidate_content").and_then(Value::as_object) else {
        return true;
    };
    !candidate
        .keys()
        .any(|key| is_update_itinerary_field_key(key.as_str()))
}

fn is_update_itinerary_field_key(key: &str) -> bool {
    matches!(
        key,
        "title"
            | "note"
            | "location"
            | "category"
            | "start_time"
            | "time"
            | "duration_minutes"
            | "duration"
            | "travel_minutes"
            | "travel_time_minutes"
            | "travel"
    )
}

fn add_reservation_body_nearly_empty(fragment: &serde_json::Map<String, Value>) -> bool {
    let Some(candidate) = fragment.get("candidate_content").and_then(Value::as_object) else {
        return true;
    };
    let has_reservation_type = non_empty_string(candidate.get("reservation_type"))
        .or_else(|| non_empty_string(candidate.get("type")))
        .is_some();
    let has_provider_name = non_empty_string(candidate.get("provider"))
        .or_else(|| non_empty_string(candidate.get("provider_name")))
        .is_some();
    !(has_reservation_type && has_provider_name)
}

fn target_summary_string(target: &serde_json::Map<String, Value>) -> Option<String> {
    let mut parts = Vec::new();
    if let Some(trip_ref) = non_empty_string(target.get("trip_reference")) {
        parts.push(format!("trip={trip_ref}"));
    }
    if let Some(day_ref) = target.get("day_reference") {
        parts.push(format!("day={day_ref}"));
    }
    if let Some(it_ref) = target.get("itinerary_reference") {
        parts.push(format!("itinerary={it_ref}"));
    }
    if parts.is_empty() {
        non_empty_string(target.get("summary"))
    } else {
        Some(parts.join(", "))
    }
}

fn fill_fragment_summary(
    target: &serde_json::Map<String, Value>,
    fragment: &serde_json::Map<String, Value>,
    metadata: &serde_json::Map<String, Value>,
    adoption_hints: Option<&serde_json::Map<String, Value>>,
    report: &mut ProposalFragmentValidationReport,
) {
    report.summary.fragment_id = non_empty_string(metadata.get("fragment_id"));
    report.summary.target_type = non_empty_string(target.get("target_type"));
    report.summary.target_summary = target_summary_string(target);
    report.summary.intent = non_empty_string(fragment.get("intent"));
    report.summary.created_at = non_empty_string(metadata.get("created_at"));
    report.summary.valid_until = non_empty_string(metadata.get("valid_until"));
    report.summary.source = non_empty_string(metadata.get("source"));
    report.summary.provider = non_empty_string(metadata.get("provider"));

    if let Some(hints) = adoption_hints {
        report.summary.conflicts_count = string_array(hints.get("conflicts")).len();
        report.summary.missing_fields_count = string_array(hints.get("missing_fields")).len();
        report.summary.assumptions_count = string_array(hints.get("assumptions")).len();
    }
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

fn collect_fragment_metadata_warnings(
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
    } else {
        warnings.push("metadata.valid_until がありません".to_string());
        if let Some(created_str) = &created_at {
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
}

fn collect_adoption_hint_warnings(
    hints: Option<&serde_json::Map<String, Value>>,
    warnings: &mut Vec<String>,
) {
    let Some(hints) = hints else {
        return;
    };

    if string_array(hints.get("missing_fields"))
        .iter()
        .any(|s| !s.is_empty())
    {
        warnings.push("adoption_hints.missing_fields があります".to_string());
    }

    if string_array(hints.get("assumptions"))
        .iter()
        .any(|s| !s.is_empty())
    {
        warnings.push("adoption_hints.assumptions があります".to_string());
    }

    if string_array(hints.get("warnings"))
        .iter()
        .any(|s| !s.is_empty())
    {
        warnings.push("adoption_hints.warnings があります".to_string());
    }

    if string_array(hints.get("conflicts"))
        .iter()
        .any(|s| !s.is_empty())
    {
        warnings.push("adoption_hints.conflicts があります".to_string());
    }

    if hints.contains_key("required_decisions") {
        let decisions = string_array(hints.get("required_decisions"));
        if !decisions.is_empty() {
            warnings.push("adoption_hints.required_decisions があります".to_string());
        }
    }
}

fn collect_target_warnings(target: &serde_json::Map<String, Value>, warnings: &mut Vec<String>) {
    let target_type = non_empty_string(target.get("target_type"));

    if target_type.as_deref() == Some("unresolved") {
        warnings.push("target が unresolved です".to_string());
    }

    if target
        .get("unresolved_target_hints")
        .and_then(Value::as_array)
        .is_some_and(|a| !a.is_empty())
    {
        warnings.push("target.unresolved_target_hints があります".to_string());
    }

    let has_trip =
        non_empty_string(target.get("trip_reference")).is_some() || target.get("trip_id").is_some();
    let has_day = target.get("day_reference").is_some() || target.get("day_id").is_some();

    if matches!(target_type.as_deref(), Some("day" | "itinerary")) && !has_trip {
        warnings.push("target reference が曖昧です（trip 参照がありません）".to_string());
    }

    if target_type.as_deref() == Some("itinerary") && !has_day && !has_trip {
        warnings.push("target reference が曖昧です（itinerary 参照の文脈が不足）".to_string());
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

pub fn print_proposal_fragment_validation_report(report: &ProposalFragmentValidationReport) {
    println!("Fragment file: {}", report.file);
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
        "  Fragment ID     : {}",
        report.summary.fragment_id.as_deref().unwrap_or("-")
    );
    println!(
        "  Target type     : {}",
        report.summary.target_type.as_deref().unwrap_or("-")
    );
    println!(
        "  Target summary  : {}",
        report.summary.target_summary.as_deref().unwrap_or("-")
    );
    println!(
        "  Intent          : {}",
        report.summary.intent.as_deref().unwrap_or("-")
    );
    println!(
        "  Created at      : {}",
        report.summary.created_at.as_deref().unwrap_or("-")
    );
    println!(
        "  Valid until     : {}",
        report.summary.valid_until.as_deref().unwrap_or("-")
    );
    println!(
        "  Source          : {}",
        report.summary.source.as_deref().unwrap_or("-")
    );
    println!(
        "  Provider        : {}",
        report.summary.provider.as_deref().unwrap_or("-")
    );
    println!("  Conflicts       : {}", report.summary.conflicts_count);
    println!(
        "  Missing fields  : {}",
        report.summary.missing_fields_count
    );
    println!("  Assumptions     : {}", report.summary.assumptions_count);
    println!("  Warnings        : {}", report.summary.warnings_count);

    println!();
    println!("Result:");
    if report.valid {
        println!("  有効な Proposal Fragment ファイル");
    } else {
        println!("  無効な Proposal Fragment ファイル");
    }
}

pub fn run_fragment_validate(path: &str, json: bool) -> Result<()> {
    let report = analyze_proposal_fragment(path)?;
    if json {
        print_json(&report)?;
    } else {
        print_proposal_fragment_validation_report(&report);
    }
    if !report.valid {
        anyhow::bail!("無効な Proposal Fragment ファイルです");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const VALID_FRAGMENT: &str = r#"{
      "metadata": {
        "fragment_id": "frag-test-01",
        "created_at": "2026-03-15T14:00:00Z",
        "source": "ai",
        "provider": "test-model"
      },
      "target": {
        "target_type": "day",
        "trip_reference": "Example Trip",
        "day_reference": 2
      },
      "fragment": {
        "intent": "add",
        "candidate_content": { "title": "Lunch candidate" },
        "notes": "Hours not verified."
      },
      "adoption_hints": {
        "missing_fields": ["opening hours"],
        "conflicts": [],
        "warnings": []
      }
    }"#;

    #[test]
    fn valid_fragment_passes() {
        let report = analyze_proposal_fragment_json("test.json", VALID_FRAGMENT);
        assert!(report.valid, "errors: {:?}", report.errors);
        assert_eq!(report.summary.intent.as_deref(), Some("add"));
    }

    #[test]
    fn json_parse_error_fails() {
        let report = analyze_proposal_fragment_json("test.json", "{bad");
        assert!(!report.valid);
    }

    #[test]
    fn missing_target_fails() {
        let json = r#"{"fragment":{"intent":"add","candidate_content":{"title":"x"},"notes":"n"}}"#;
        let report = analyze_proposal_fragment_json("test.json", json);
        assert!(!report.valid);
        assert!(report.errors.iter().any(|e| e.contains("target")));
    }

    #[test]
    fn missing_intent_fails() {
        let json =
            r#"{"target":{"target_type":"day"},"fragment":{"candidate_content":{"title":"x"}}}"#;
        let report = analyze_proposal_fragment_json("test.json", json);
        assert!(!report.valid);
        assert!(report.errors.iter().any(|e| e.contains("intent")));
    }

    #[test]
    fn unknown_target_type_fails() {
        let json = r#"{"target":{"target_type":"planet"},"fragment":{"intent":"add","notes":"n"}}"#;
        let report = analyze_proposal_fragment_json("test.json", json);
        assert!(!report.valid);
        assert!(report.errors.iter().any(|e| e.contains("target_type")));
    }

    #[test]
    fn unknown_intent_fails() {
        let json = r#"{"target":{"target_type":"day"},"fragment":{"intent":"maybe","notes":"n"}}"#;
        let report = analyze_proposal_fragment_json("test.json", json);
        assert!(!report.valid);
        assert!(report.errors.iter().any(|e| e.contains("intent")));
    }

    #[test]
    fn unresolved_target_warns_but_passes() {
        let json = r#"{
          "metadata": {"created_at": "2026-01-01T00:00:00Z", "source": "ai"},
          "target": {"target_type": "unresolved", "unresolved_target_hints": ["pick a day"]},
          "fragment": {"intent": "add", "notes": "TBD"}
        }"#;
        let report = analyze_proposal_fragment_json("test.json", json);
        assert!(report.valid, "errors: {:?}", report.errors);
        assert!(report.warnings.iter().any(|w| w.contains("unresolved")));
    }

    #[test]
    fn schema_v8_trip_fails() {
        let json = r#"{"schema_version":8,"trip":{"name":"T"},"days":[]}"#;
        let report = analyze_proposal_fragment_json("test.json", json);
        assert!(!report.valid);
        assert!(report.errors.iter().any(|e| e.contains("schema_version")));
    }

    #[test]
    fn envelope_fails() {
        let json =
            r#"{"proposal":{"title":"T","destination":"D","date_policy":"undated","notes":"n"}}"#;
        let report = analyze_proposal_fragment_json("test.json", json);
        assert!(!report.valid);
        assert!(report.errors.iter().any(|e| e.contains("Envelope")));
    }
}
