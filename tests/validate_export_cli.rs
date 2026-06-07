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
    let dir = std::env::temp_dir().join(format!("caglla-cli-validate-export-{n}"));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn cli_validate_export_current_format_succeeds() {
    let dir = temp_workdir();
    assert!(run_cli(&dir, &["db", "reset"]).status.success());
    assert!(run_cli(
        &dir,
        &[
            "trip",
            "add",
            "Validate Export Trip",
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
    assert!(run_cli(&dir, &["checklist", "add", "1", "Passport"])
        .status
        .success());

    let export_path = dir.join("backup.json");
    assert!(run_cli(
        &dir,
        &[
            "trip",
            "export",
            "1",
            "--output",
            export_path.to_str().unwrap(),
        ]
    )
    .status
    .success());

    let output = run_cli(
        &dir,
        &["trip", "validate-export", export_path.to_str().unwrap()],
    );
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Metadata:"));
    assert!(stdout.contains("Generator : caglla-cli"));
    assert!(stdout.contains("Version   :"));
    assert!(stdout.contains("Exported  :"));
    assert!(stdout.contains("Warnings:"));
    assert!(stdout.contains("なし"));
    assert!(stdout.contains("有効な export ファイル"));
}

#[test]
fn cli_validate_export_json_includes_errors_array() {
    let dir = temp_workdir();
    let export_path = dir.join("legacy.json");
    fs::write(
        &export_path,
        r#"{
            "trip": {
                "id": 1,
                "name": "Legacy Trip",
                "start_date": "2026-01-01",
                "end_date": "2026-01-03",
                "created_at": "2026-01-01 00:00:00",
                "updated_at": "2026-01-01 00:00:00"
            },
            "itinerary_items": []
        }"#,
    )
    .unwrap();

    let output = run_cli(
        &dir,
        &[
            "trip",
            "validate-export",
            export_path.to_str().unwrap(),
            "--json",
        ],
    );
    assert!(output.status.success());
    let parsed: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(parsed["valid"], true);
    assert_eq!(parsed["errors"], serde_json::json!([]));
    assert_eq!(parsed["generator"], serde_json::Value::Null);
    assert_eq!(parsed["generator_version"], serde_json::Value::Null);
    assert_eq!(parsed["exported_at"], serde_json::Value::Null);
    assert!(parsed["warnings"].as_array().unwrap().len() >= 2);
}

#[test]
fn cli_validate_export_legacy_text_output_is_valid_with_warnings() {
    let dir = temp_workdir();
    let export_path = dir.join("legacy.json");
    fs::write(
        &export_path,
        r#"{
            "trip": {
                "id": 1,
                "name": "Legacy Trip",
                "start_date": "2026-01-01",
                "end_date": "2026-01-03",
                "created_at": "2026-01-01 00:00:00",
                "updated_at": "2026-01-01 00:00:00"
            },
            "itinerary_items": []
        }"#,
    )
    .unwrap();

    let output = run_cli(
        &dir,
        &["trip", "validate-export", export_path.to_str().unwrap()],
    );
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("✗ schema_version"));
    assert!(stdout.contains("✗ checklist_items"));
    assert!(stdout.contains("有効な export ファイル"));
    assert!(stdout.contains("schema_version がありません（旧形式）"));
    assert!(stdout.contains("Metadata:"));
    assert!(stdout.contains("Generator : 不明"));
    assert!(stdout.contains("Version   : 不明"));
    assert!(stdout.contains("Exported  : 不明"));
}

#[test]
fn cli_validate_export_json_includes_generator_metadata() {
    let dir = temp_workdir();
    assert!(run_cli(&dir, &["db", "reset"]).status.success());
    assert!(run_cli(
        &dir,
        &[
            "trip",
            "add",
            "JSON Metadata Trip",
            "--start",
            "2026-01-01",
            "--end",
            "2026-01-03",
        ]
    )
    .status
    .success());

    let export_path = dir.join("backup.json");
    assert!(run_cli(
        &dir,
        &[
            "trip",
            "export",
            "1",
            "--output",
            export_path.to_str().unwrap(),
        ]
    )
    .status
    .success());

    let output = run_cli(
        &dir,
        &[
            "trip",
            "validate-export",
            export_path.to_str().unwrap(),
            "--json",
        ],
    );
    assert!(output.status.success());
    let parsed: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(parsed["generator"], "caglla-cli");
    assert!(parsed["generator_version"].is_string());
    assert!(parsed["exported_at"].is_string());
}

#[test]
fn cli_validate_export_invalid_json_exits_with_error() {
    let dir = temp_workdir();
    let export_path = dir.join("broken.json");
    fs::write(&export_path, "not json").unwrap();

    let output = run_cli(
        &dir,
        &["trip", "validate-export", export_path.to_str().unwrap()],
    );
    assert!(!output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("✗ JSON形式"));
    assert!(stdout.contains("Errors:"));
    assert!(stdout.contains("JSON形式が不正です"));
    assert!(stdout.contains("無効な export ファイル"));
}

#[test]
fn cli_validate_export_invalid_json_json_output_includes_errors() {
    let dir = temp_workdir();
    let export_path = dir.join("broken.json");
    fs::write(&export_path, "not json").unwrap();

    let output = run_cli(
        &dir,
        &[
            "trip",
            "validate-export",
            export_path.to_str().unwrap(),
            "--json",
        ],
    );
    assert!(!output.status.success());
    let parsed: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(parsed["valid"], false);
    assert!(parsed["errors"].as_array().unwrap().len() >= 1);
}

#[test]
fn cli_validate_export_missing_file_exits_with_error() {
    let dir = temp_workdir();
    let output = run_cli(&dir, &["trip", "validate-export", "missing-export.json"]);
    assert!(!output.status.success());
}
