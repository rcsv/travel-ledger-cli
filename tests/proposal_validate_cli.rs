use std::path::PathBuf;
use std::process::Command;

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/proposals")
        .join(name)
}

fn run_cli(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_travel-ledger-cli"))
        .args(args)
        .output()
        .expect("failed to run CLI")
}

#[test]
fn cli_proposal_validate_valid_envelope_passes() {
    let path = fixture_path("valid-envelope.json");
    let output = run_cli(&["proposal", "validate", path.to_str().unwrap()]);
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Validation result:"));
    assert!(stdout.contains("valid"));
    assert!(stdout.contains("Weekend city break (draft)"));
}

#[test]
fn cli_proposal_validate_missing_title_fails() {
    let path = fixture_path("invalid-envelope-missing-title.json");
    let output = run_cli(&["proposal", "validate", path.to_str().unwrap()]);
    assert!(!output.status.success());
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(combined.contains("title"));
}

#[test]
fn cli_proposal_validate_stale_warns_but_passes() {
    let path = fixture_path("warn-envelope-stale.json");
    let output = run_cli(&["proposal", "validate", path.to_str().unwrap()]);
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("valid_until"));
}

#[test]
fn cli_proposal_validate_schema_v8_trip_fails() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("docs/public/examples/schema-v8-minimal-trip.json");
    let output = run_cli(&["proposal", "validate", path.to_str().unwrap()]);
    assert!(!output.status.success());
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(combined.contains("schema_version") || combined.contains("trip"));
}

#[test]
fn cli_proposal_validate_json_output() {
    let path = fixture_path("valid-envelope.json");
    let output = run_cli(&["proposal", "validate", path.to_str().unwrap(), "--json"]);
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value = serde_json::from_str(&stdout).expect("json output");
    assert_eq!(parsed["valid"], true);
    assert_eq!(parsed["document_kind"], "trip_proposal_envelope");
    assert_eq!(parsed["summary"]["title"], "Weekend city break (draft)");
}

#[test]
fn cli_proposal_validate_non_normative_example_passes() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("docs/public/examples-non-normative/trip-proposal-envelope.example.json");
    let output = run_cli(&["proposal", "validate", path.to_str().unwrap()]);
    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}
