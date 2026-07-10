mod common;

use std::path::PathBuf;

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/proposals")
        .join(name)
}

fn run_cli(args: &[&str]) -> std::process::Output {
    common::run_cli(args)
}

#[test]
fn cli_proposal_materialize_dry_run_writes_valid_trip_export() {
    let workspace = common::TestWorkspace::new();
    let envelope = fixture_path("materialize-ready-envelope.json");
    let output_path = workspace.path().join("materialized-trip.json");

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
}

#[test]
fn cli_proposal_materialize_dry_run_and_confirm_together_fails() {
    let envelope = fixture_path("materialize-ready-envelope.json");
    let output = run_cli(&[
        "proposal",
        "materialize",
        envelope.to_str().unwrap(),
        "--dry-run",
        "--confirm",
    ]);
    assert!(!output.status.success());
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        combined.contains("併用")
            || combined.contains("cannot be used with")
            || combined.contains("conflicts"),
        "expected mutual exclusion error, got: {combined}"
    );
}

#[test]
fn cli_proposal_materialize_requires_dry_run_or_confirm_flag() {
    let envelope = fixture_path("materialize-ready-envelope.json");
    let output = run_cli(&["proposal", "materialize", envelope.to_str().unwrap()]);
    assert!(!output.status.success());
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(combined.contains("--dry-run") || combined.contains("--confirm"));
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
    let workspace = common::TestWorkspace::new();
    let output_path = workspace.path().join("candidate-trip.json");

    let output = common::run_cli_in(
        workspace.path(),
        &[
            "proposal",
            "materialize",
            envelope.to_str().unwrap(),
            "--dry-run",
            "--output",
            output_path.to_str().unwrap(),
        ],
    );

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn run_cli_in(cwd: &std::path::Path, args: &[&str]) -> std::process::Output {
    common::run_cli_in(cwd, args)
}

#[test]
fn cli_proposal_materialize_confirm_saves_trip_to_db() {
    let workspace = common::TestWorkspace::new();
    let dir = workspace.path();
    let envelope = fixture_path("materialize-ready-envelope.json");

    let output = run_cli_in(
        &dir,
        &[
            "proposal",
            "materialize",
            envelope.to_str().unwrap(),
            "--confirm",
        ],
    );
    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Materialize confirm result:"));
    assert!(stdout.contains("trip_id: 1"));
    assert!(stdout.contains("旅行をインポートしました"));
    assert!(stdout.contains("Okinawa weekend draft"));

    let list = run_cli_in(&dir, &["trip", "list"]);
    assert!(list.status.success());
    let list_stdout = String::from_utf8_lossy(&list.stdout);
    assert!(list_stdout.contains("Okinawa weekend draft"));

    let show = run_cli_in(&dir, &["trip", "show", "1"]);
    assert!(show.status.success());
    let show_stdout = String::from_utf8_lossy(&show.stdout);
    assert!(show_stdout.contains("Okinawa weekend draft"));
}

#[test]
fn cli_proposal_materialize_dry_run_without_confirm_does_not_save_trip() {
    let workspace = common::TestWorkspace::new();
    let dir = workspace.path();
    let envelope = fixture_path("materialize-ready-envelope.json");

    let output = run_cli_in(
        &dir,
        &[
            "proposal",
            "materialize",
            envelope.to_str().unwrap(),
            "--dry-run",
        ],
    );
    assert!(output.status.success());

    let list = run_cli_in(&dir, &["trip", "list"]);
    assert!(list.status.success());
    let list_stdout = String::from_utf8_lossy(&list.stdout);
    assert!(!list_stdout.contains("Okinawa weekend draft"));
}

#[test]
fn cli_proposal_materialize_confirm_json_includes_trip_id() {
    let workspace = common::TestWorkspace::new();
    let dir = workspace.path();
    let envelope = fixture_path("materialize-ready-envelope.json");

    let output = run_cli_in(
        &dir,
        &[
            "proposal",
            "materialize",
            envelope.to_str().unwrap(),
            "--confirm",
            "--json",
        ],
    );
    assert!(output.status.success());
    let parsed: serde_json::Value =
        serde_json::from_str(&String::from_utf8_lossy(&output.stdout)).expect("json report");
    assert_eq!(parsed["valid"], true);
    assert_eq!(parsed["confirm"], true);
    assert_eq!(parsed["trip_id"], 1);
}
