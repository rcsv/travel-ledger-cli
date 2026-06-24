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
    let dir = std::env::temp_dir().join(format!("caglla-cli-trip-import-{n}"));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn cli_trip_import_prints_enhanced_summary() {
    let dir = temp_workdir();
    assert!(run_cli(&dir, &["db", "reset"]).status.success());
    assert!(run_cli(
        &dir,
        &[
            "trip",
            "add",
            "Import Summary Trip",
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

    let export_path = dir.join("trip-export.json");
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

    assert!(run_cli(&dir, &["db", "reset"]).status.success());

    let output = run_cli(&dir, &["trip", "import", export_path.to_str().unwrap()]);
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("旅行をインポートしました\n"));
    assert!(!stdout.contains("旅行をインポートしました (ID:"));
    assert!(stdout.contains("Trip:"));
    assert!(stdout.contains("Import Summary Trip (ID: 1)"));
    assert!(stdout.contains("Created:"));
    assert!(stdout.contains("日程           : 1 件"));
    assert!(stdout.contains("チェックリスト : 1 件"));
    assert!(stdout.contains("Note           : 0 件"));
    assert!(stdout.contains("Schema:"));
    assert!(stdout.contains("version 7"));
    assert!(stdout.contains("Export:"));
    assert!(stdout.contains("generator : caglla-cli"));
    assert!(stdout.contains("version   :"));
}

#[test]
fn cli_trip_import_legacy_schema_summary() {
    let dir = temp_workdir();
    let import_path = dir.join("legacy.json");
    fs::write(
        &import_path,
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

    let output = run_cli(&dir, &["trip", "import", import_path.to_str().unwrap()]);
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Legacy Trip (ID: 1)"));
    assert!(stdout.contains("未指定（旧形式）"));
    assert!(stdout.contains("Export:"));
    assert!(stdout.contains("generator : 不明"));
    assert!(stdout.contains("version   : 不明"));
}
