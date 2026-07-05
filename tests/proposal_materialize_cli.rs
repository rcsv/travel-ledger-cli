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
fn cli_proposal_materialize_dry_run_writes_valid_trip_export() {
    let envelope = fixture_path("materialize-ready-envelope.json");
    let output_path = std::env::temp_dir().join(format!(
        "caglla-proposal-materialize-{}.json",
        std::process::id()
    ));

    let output = run_cli(&[
        "proposal",
        "materialize",
        envelope.to_str().unwrap(),
        "--dry-run",
        "--output",
        output_path.to_str().unwrap(),
    ]);
    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let trip_json = std::fs::read_to_string(&output_path).expect("read materialized trip json");
    let parsed: serde_json::Value = serde_json::from_str(&trip_json).expect("trip json parse");
    assert_eq!(parsed["schema_version"], 8);
    assert_eq!(parsed["trip"]["name"], "Okinawa weekend draft");

    let validate = run_cli(&["trip", "validate-export", output_path.to_str().unwrap()]);
    assert!(
        validate.status.success(),
        "validate-export stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&validate.stdout),
        String::from_utf8_lossy(&validate.stderr)
    );

    let _ = std::fs::remove_file(&output_path);
}

#[test]
fn cli_proposal_materialize_requires_dry_run_flag() {
    let envelope = fixture_path("materialize-ready-envelope.json");
    let output = run_cli(&["proposal", "materialize", envelope.to_str().unwrap()]);
    assert!(!output.status.success());
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(combined.contains("--dry-run"));
}

#[test]
fn cli_proposal_materialize_flexible_envelope_with_cli_dates() {
    let envelope = fixture_path("valid-envelope.json");
    let output = run_cli(&[
        "proposal",
        "materialize",
        envelope.to_str().unwrap(),
        "--dry-run",
        "--start",
        "2026-05-01",
        "--end",
        "2026-05-01",
        "--json",
    ]);
    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let parsed: serde_json::Value =
        serde_json::from_str(&String::from_utf8_lossy(&output.stdout)).expect("json report");
    assert_eq!(parsed["valid"], true);
    assert_eq!(parsed["dry_run"], true);
    assert_eq!(parsed["trip_export_valid"], true);
}

#[test]
fn cli_proposal_materialize_missing_dates_fails() {
    let envelope = fixture_path("valid-envelope.json");
    let output = run_cli(&[
        "proposal",
        "materialize",
        envelope.to_str().unwrap(),
        "--dry-run",
    ]);
    assert!(!output.status.success());
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(combined.contains("start_date") || combined.contains("end_date"));
}

#[test]
fn cli_proposal_materialize_runs_without_db() {
    let envelope = fixture_path("materialize-ready-envelope.json");
    let temp_dir = std::env::temp_dir().join(format!(
        "caglla-proposal-materialize-isolated-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&temp_dir).expect("create temp dir");
    let output_path = temp_dir.join("candidate-trip.json");

    let output = Command::new(env!("CARGO_BIN_EXE_travel-ledger-cli"))
        .current_dir(&temp_dir)
        .args([
            "proposal",
            "materialize",
            envelope.to_str().unwrap(),
            "--dry-run",
            "--output",
            output_path.to_str().unwrap(),
        ])
        .output()
        .expect("failed to run CLI");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let _ = std::fs::remove_dir_all(&temp_dir);
}
