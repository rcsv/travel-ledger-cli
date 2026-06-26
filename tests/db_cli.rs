use std::fs;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

static TEST_DIR_COUNTER: AtomicU64 = AtomicU64::new(0);

fn temp_workdir() -> std::path::PathBuf {
    let n = TEST_DIR_COUNTER.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!("travel-ledger-cli-db-test-{n}"));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn run_cli(cwd: &std::path::Path, args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_travel-ledger-cli"))
        .current_dir(cwd)
        .args(args)
        .output()
        .expect("failed to run CLI")
}

fn normalize_path_str(path: &str) -> String {
    path.strip_prefix("/private").unwrap_or(path).to_string()
}

fn assert_same_resolved_db_path(actual: &str, expected: &std::path::Path) {
    assert_eq!(
        normalize_path_str(actual),
        normalize_path_str(&expected.to_string_lossy())
    );
}

#[test]
fn cli_db_path_does_not_create_db_file() {
    let dir = temp_workdir();
    let db_path = dir.join("caglla.db");

    let output = run_cli(&dir, &["db", "path"]);
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    assert_same_resolved_db_path(&stdout, &db_path);
    assert!(!db_path.exists(), "db path must not create SQLite file");
}

#[test]
fn cli_db_status_missing_db_does_not_create_file() {
    let dir = temp_workdir();
    let db_path = dir.join("caglla.db");

    let output = run_cli(&dir, &["db", "status"]);
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains(db_path.to_string_lossy().as_ref())
            || stdout.contains(&normalize_path_str(&db_path.to_string_lossy()))
    );
    assert!(stdout.contains("Exists                    : no"));
    assert!(stdout.contains("Trip export schema version: 8"));
    assert!(!stdout.contains("File size (bytes)"));
    assert!(!stdout.contains("Table counts:"));
    assert!(!db_path.exists(), "db status must not create SQLite file");
}

#[test]
fn cli_db_status_json_missing_db_omits_optional_fields() {
    let dir = temp_workdir();
    let db_path = dir.join("caglla.db");

    let output = run_cli(&dir, &["db", "status", "--json"]);
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.trim_start().starts_with('{'),
        "expected JSON only stdout, got: {stdout}"
    );
    assert!(!stdout.contains("Path"));

    let parsed: serde_json::Value = serde_json::from_str(stdout.trim()).expect("valid JSON");
    assert_eq!(parsed["schema_version"], 1);
    assert_same_resolved_db_path(parsed["path"].as_str().unwrap(), &db_path);
    assert_eq!(parsed["exists"], false);
    assert_eq!(parsed["trip_export_schema_version"], 8);
    assert!(parsed.get("file_size_bytes").is_none());
    assert!(parsed.get("table_counts").is_none());
    assert!(
        !db_path.exists(),
        "db status --json must not create SQLite file"
    );
}

#[test]
fn cli_db_status_json_existing_db_includes_table_counts() {
    let dir = temp_workdir();

    assert!(run_cli(&dir, &["db", "reset"]).status.success());
    assert!(run_cli(
        &dir,
        &[
            "trip",
            "add",
            "DB Status Trip",
            "--start",
            "2026-04-26",
            "--end",
            "2026-04-28",
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
            "1",
            "--time",
            "09:00",
            "首里城",
        ]
    )
    .status
    .success());
    assert!(run_cli(
        &dir,
        &[
            "estimate",
            "add",
            "--itinerary",
            "1",
            "--amount",
            "1400",
            "--currency",
            "JPY",
        ]
    )
    .status
    .success());

    let output = run_cli(&dir, &["db", "status", "--json"]);
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value = serde_json::from_str(stdout.trim()).expect("valid JSON");
    assert_eq!(parsed["schema_version"], 1);
    assert_eq!(parsed["exists"], true);
    assert_eq!(parsed["trip_export_schema_version"], 8);
    assert!(parsed["file_size_bytes"].as_u64().unwrap() > 0);
    assert_eq!(parsed["table_counts"]["trips"], 1);
    assert_eq!(parsed["table_counts"]["itinerary_items"], 1);
    assert_eq!(parsed["table_counts"]["estimates"], 1);
}

#[test]
fn cli_db_status_existing_db_shows_table_counts() {
    let dir = temp_workdir();

    assert!(run_cli(&dir, &["db", "reset"]).status.success());
    assert!(run_cli(
        &dir,
        &[
            "trip",
            "add",
            "Counts Trip",
            "--start",
            "2026-05-01",
            "--end",
            "2026-05-02",
        ]
    )
    .status
    .success());

    let output = run_cli(&dir, &["db", "status"]);
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Exists                    : yes"));
    assert!(stdout.contains("File size (bytes)"));
    assert!(stdout.contains("Table counts:"));
    assert!(stdout.contains("trips                   : 1"));
}
