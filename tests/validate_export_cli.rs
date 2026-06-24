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
    assert!(stdout.contains("✓ schema_version"));
    assert!(stdout.contains("✗ checklist_items"));
    assert!(stdout.contains("有効な export ファイル"));
    assert!(stdout.contains("schema_version がありません（旧形式 v1）"));
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
    assert!(!parsed["errors"].as_array().unwrap().is_empty());
}

#[test]
fn cli_validate_export_missing_file_exits_with_error() {
    let dir = temp_workdir();
    let output = run_cli(&dir, &["trip", "validate-export", "missing-export.json"]);
    assert!(!output.status.success());
}

fn write_v3_export(dir: &std::path::Path, filename: &str, days_json: &str) -> std::path::PathBuf {
    let export_path = dir.join(filename);
    let json = format!(
        r#"{{
  "schema_version": 3,
  "trip": {{
    "id": 1,
    "name": "Expense Validate Trip",
    "start_date": "2026-04-26",
    "end_date": "2026-04-29",
    "created_at": "2026-01-01 00:00:00",
    "updated_at": "2026-01-01 00:00:00"
  }},
  "days": {days_json},
  "checklist_items": [],
  "notes": []
}}"#
    );
    fs::write(&export_path, json).unwrap();
    export_path
}

#[test]
fn cli_validate_export_v3_expense_invalid_currency_fails() {
    let dir = temp_workdir();
    let export_path = write_v3_export(
        &dir,
        "invalid-currency.json",
        r#"[
    {
      "day_number": 1,
      "itineraries": [
        {
          "title": "Lunch",
          "sort_order": 0,
          "expenses": [
            { "amount": 1000, "currency": "JP", "sort_order": 0 }
          ]
        }
      ]
    }
  ]"#,
    );

    let output = run_cli(
        &dir,
        &["trip", "validate-export", export_path.to_str().unwrap()],
    );
    assert!(!output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("無効な export ファイル"));
    assert!(stdout.contains("currency"));
}

#[test]
fn cli_validate_export_v3_expense_empty_currency_fails() {
    let dir = temp_workdir();
    let export_path = write_v3_export(
        &dir,
        "empty-currency.json",
        r#"[
    {
      "day_number": 1,
      "itineraries": [
        {
          "title": "Lunch",
          "sort_order": 0,
          "expenses": [
            { "amount": 1000, "currency": "", "sort_order": 0 }
          ]
        }
      ]
    }
  ]"#,
    );

    let output = run_cli(
        &dir,
        &["trip", "validate-export", export_path.to_str().unwrap()],
    );
    assert!(!output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("currency"));
    assert!(stdout.contains("必須"));
}

#[test]
fn cli_validate_export_v3_expense_invalid_date_fails() {
    let dir = temp_workdir();
    let export_path = write_v3_export(
        &dir,
        "invalid-date.json",
        r#"[
    {
      "day_number": 1,
      "itineraries": [
        {
          "title": "Lunch",
          "sort_order": 0,
          "expenses": [
            {
              "amount": 1000,
              "currency": "JPY",
              "expense_date": "2026/04/26",
              "sort_order": 0
            }
          ]
        }
      ]
    }
  ]"#,
    );

    let output = run_cli(
        &dir,
        &["trip", "validate-export", export_path.to_str().unwrap()],
    );
    assert!(!output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("expense_date"));
}

#[test]
fn cli_validate_export_v3_expense_valid_nested_structure_succeeds() {
    let dir = temp_workdir();
    let export_path = write_v3_export(
        &dir,
        "valid-expenses.json",
        r#"[
    {
      "day_number": 2,
      "itineraries": [
        {
          "title": "Aquarium",
          "sort_order": 0,
          "start_time": "09:00",
          "expenses": [
            {
              "title": "入館料",
              "amount": 2500,
              "currency": "JPY",
              "sort_order": 0
            },
            {
              "title": "駐車場",
              "amount": 500,
              "currency": "JPY",
              "sort_order": 1
            }
          ]
        }
      ]
    }
  ]"#,
    );

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
    assert_eq!(parsed["export_schema_version"], 3);
    let expenses_check = parsed["checks"]
        .as_array()
        .unwrap()
        .iter()
        .find(|check| check["id"] == "expenses")
        .expect("expenses check");
    assert_eq!(expenses_check["passed"], true);
}

#[test]
fn cli_validate_export_v4_multiple_self_fails() {
    let dir = temp_workdir();
    let export_path = dir.join("multiple-self.json");
    fs::write(
        &export_path,
        r#"{
  "schema_version": 4,
  "trip": {
    "id": 1,
    "name": "Multiple Self Trip",
    "start_date": "2026-01-01",
    "end_date": "2026-01-03",
    "created_at": "2026-01-01 00:00:00",
    "updated_at": "2026-01-01 00:00:00"
  },
  "days": [
    {
      "day_number": 1,
      "itineraries": [
        { "title": "Sightseeing", "sort_order": 0 }
      ]
    }
  ],
  "checklist_items": [],
  "notes": [],
  "participants": [
    { "name": "A", "sort_order": 0, "is_self": true },
    { "name": "B", "sort_order": 1, "is_self": true }
  ]
}"#,
    )
    .unwrap();

    let output = run_cli(
        &dir,
        &["trip", "validate-export", export_path.to_str().unwrap()],
    );
    assert!(!output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("無効な export ファイル"));
    assert!(stdout.contains("is_self"));

    let json_output = run_cli(
        &dir,
        &[
            "trip",
            "validate-export",
            export_path.to_str().unwrap(),
            "--json",
        ],
    );
    assert!(!json_output.status.success());
    let parsed: serde_json::Value = serde_json::from_slice(&json_output.stdout).unwrap();
    assert_eq!(parsed["valid"], false);
    let errors = parsed["errors"].as_array().unwrap();
    assert!(
        errors
            .iter()
            .any(|e| e.as_str().unwrap().contains("is_self")),
        "expected multiple self validation error, got {errors:?}"
    );
}

#[test]
fn cli_validate_export_v6_invalid_estimate_currency_fails() {
    let dir = temp_workdir();
    let export_path = dir.join("invalid-estimate.json");
    fs::write(
        &export_path,
        r#"{
  "schema_version": 7,
  "trip": {
    "id": 1,
    "name": "Estimate Trip",
    "start_date": "2026-01-01",
    "end_date": "2026-01-03",
    "created_at": "2026-01-01 00:00:00",
    "updated_at": "2026-01-01 00:00:00"
  },
  "days": [
    {
      "day_number": 1,
      "itineraries": [
        {
          "title": "Hotel",
          "sort_order": 0,
          "estimates": [
            {
              "amount": 1000,
              "currency": "",
              "sort_order": 0
            }
          ]
        }
      ]
    }
  ],
  "checklist_items": [],
  "notes": [],
  "participants": []
}"#,
    )
    .unwrap();

    let output = run_cli(
        &dir,
        &["trip", "validate-export", export_path.to_str().unwrap()],
    );
    assert!(!output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("currency"));
}

#[test]
fn cli_validate_export_v5_import_skips_estimate_checks() {
    let dir = temp_workdir();
    let export_path = dir.join("v5-no-estimates.json");
    fs::write(
        &export_path,
        r#"{
  "schema_version": 5,
  "trip": {
    "id": 1,
    "name": "Legacy v5 Trip",
    "start_date": "2026-01-01",
    "end_date": "2026-01-03",
    "created_at": "2026-01-01 00:00:00",
    "updated_at": "2026-01-01 00:00:00"
  },
  "days": [
    {
      "day_number": 1,
      "itineraries": [
        {
          "title": "Sightseeing",
          "sort_order": 0
        }
      ]
    }
  ],
  "checklist_items": [],
  "notes": [],
  "participants": []
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
    let estimates_check = parsed["checks"]
        .as_array()
        .unwrap()
        .iter()
        .find(|check| check["id"] == "estimates")
        .expect("estimates check");
    assert_eq!(estimates_check["passed"], true);
}
