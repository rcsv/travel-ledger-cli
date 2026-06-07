use std::fs;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

static TEST_DIR_COUNTER: AtomicU64 = AtomicU64::new(0);

fn run_cli(cwd: &std::path::Path, args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_caglla-cli"))
        .current_dir(cwd)
        .args(args)
        .output()
        .expect("failed to run CLI")
}

fn temp_workdir() -> std::path::PathBuf {
    let n = TEST_DIR_COUNTER.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!("caglla-cli-doctor-advisor-json-{n}"));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn cli_trip_doctor_json_envelope_and_codes() {
    let dir = temp_workdir();
    assert!(run_cli(&dir, &["db", "reset"]).status.success());
    assert!(run_cli(
        &dir,
        &[
            "trip",
            "add",
            "Doctor JSON Trip",
            "--start",
            "2026-01-01",
            "--end",
            "2026-01-03",
        ]
    )
    .status
    .success());
    assert!(run_cli(
        &dir,
        &[
            "itinerary",
            "add",
            "1",
            "--day",
            "2",
            "Activity without duration",
        ]
    )
    .status
    .success());

    let output = run_cli(&dir, &["trip", "doctor", "1", "--json"]);
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let parsed: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("doctor stdout must be valid JSON");

    assert_eq!(parsed["schema_version"], 1);
    assert_eq!(parsed["trip_id"], 1);
    assert!(parsed["issues"].is_array());
    assert!(parsed["issues"]
        .as_array()
        .unwrap()
        .iter()
        .any(|issue| issue["code"] == "missing_duration"));
    assert!(parsed["issues"]
        .as_array()
        .unwrap()
        .iter()
        .any(|issue| issue["target"]["type"] == "itinerary"));
}

#[test]
fn cli_trip_advisor_json_envelope_and_commands_flag() {
    let dir = temp_workdir();
    assert!(run_cli(&dir, &["db", "reset"]).status.success());
    assert!(run_cli(
        &dir,
        &[
            "trip",
            "add",
            "Advisor JSON Trip",
            "--start",
            "2026-01-01",
            "--end",
            "2026-01-03",
        ]
    )
    .status
    .success());
    assert!(run_cli(
        &dir,
        &["itinerary", "add", "1", "--day", "1", "Sightseeing"]
    )
    .status
    .success());

    let output = run_cli(&dir, &["trip", "advisor", "1", "--json", "--with-commands"]);
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let parsed: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("advisor stdout must be valid JSON");

    assert_eq!(parsed["schema_version"], 1);
    assert_eq!(parsed["trip_id"], 1);
    assert_eq!(parsed["with_commands"], true);
    assert!(parsed["issues"].is_array());
    assert!(!parsed["issues"].as_array().unwrap().is_empty());
    assert!(parsed["issues"][0]["advice"].is_array());
    assert!(parsed["issues"][0]["commands"].is_array());
    assert_eq!(parsed["issues"][0]["code"], "no_restaurant");
}
