mod common;

use std::path::PathBuf;

fn fixture_path(name: &str) -> PathBuf {
    common::manifest_dir()
        .join("tests/fixtures/fragments")
        .join(name)
}

fn run_cli(args: &[&str]) -> std::process::Output {
    common::run_cli(args)
}

#[test]
fn cli_fragment_validate_valid_fragment_passes() {
    let path = fixture_path("valid-fragment.json");
    let output = run_cli(&["fragment", "validate", path.to_str().unwrap()]);
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Validation result:"));
    assert!(stdout.contains("valid"));
    assert!(stdout.contains("frag-test-valid-01"));
}

#[test]
fn cli_fragment_validate_missing_target_fails() {
    let path = fixture_path("invalid-fragment-missing-target.json");
    let output = run_cli(&["fragment", "validate", path.to_str().unwrap()]);
    assert!(!output.status.success());
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(combined.contains("target"));
}

#[test]
fn cli_fragment_validate_missing_intent_fails() {
    let path = fixture_path("invalid-fragment-missing-intent.json");
    let output = run_cli(&["fragment", "validate", path.to_str().unwrap()]);
    assert!(!output.status.success());
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(combined.contains("intent"));
}

#[test]
fn cli_fragment_validate_unresolved_target_warns_but_passes() {
    let path = fixture_path("warn-fragment-unresolved-target.json");
    let output = run_cli(&["fragment", "validate", path.to_str().unwrap()]);
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("unresolved"));
}

#[test]
fn cli_fragment_validate_schema_v8_trip_fails() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("docs/public/examples/schema-v8-minimal-trip.json");
    let output = run_cli(&["fragment", "validate", path.to_str().unwrap()]);
    assert!(!output.status.success());
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(combined.contains("schema_version") || combined.contains("trip validate-export"));
}

#[test]
fn cli_fragment_validate_envelope_fails() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/proposals/valid-envelope.json");
    let output = run_cli(&["fragment", "validate", path.to_str().unwrap()]);
    assert!(!output.status.success());
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(combined.contains("Envelope") || combined.contains("proposal validate"));
}

#[test]
fn cli_fragment_validate_json_output() {
    let path = fixture_path("valid-fragment.json");
    let output = run_cli(&["fragment", "validate", path.to_str().unwrap(), "--json"]);
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value = serde_json::from_str(&stdout).expect("json output");
    assert_eq!(parsed["valid"], true);
    assert_eq!(parsed["document_kind"], "proposal_fragment");
}
