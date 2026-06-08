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
    let dir = std::env::temp_dir().join(format!("caglla-cli-export-roundtrip-{n}"));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn cli_export_import_reexport_roundtrip_with_checklist() {
    let dir = temp_workdir();
    assert!(run_cli(&dir, &["db", "reset"]).status.success());

    assert!(run_cli(
        &dir,
        &[
            "trip",
            "add",
            "Roundtrip CLI Trip",
            "--start",
            "2026-08-01",
            "--end",
            "2026-08-03",
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
            "--duration",
            "90",
            "Shuri Castle",
        ]
    )
    .status
    .success());

    assert!(run_cli(&dir, &["checklist", "add", "1", "Passport"])
        .status
        .success());
    assert!(run_cli(&dir, &["checklist", "add", "1", "Charger"])
        .status
        .success());
    assert!(run_cli(&dir, &["checklist", "check", "2"]).status.success());

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

    let import_output = run_cli(&dir, &["trip", "import", export_path.to_str().unwrap()]);
    assert!(import_output.status.success(), "{:?}", import_output.stderr);

    let reexport_path = dir.join("trip-reexport.json");
    assert!(run_cli(
        &dir,
        &[
            "trip",
            "export",
            "1",
            "--output",
            reexport_path.to_str().unwrap(),
        ]
    )
    .status
    .success());

    let before: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&export_path).unwrap()).unwrap();
    let after: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&reexport_path).unwrap()).unwrap();

    assert_eq!(
        comparable_export_json(&before),
        comparable_export_json(&after)
    );
}

#[test]
fn cli_export_import_reexport_roundtrip_with_notes() {
    let dir = temp_workdir();
    assert!(run_cli(&dir, &["db", "reset"]).status.success());

    assert!(run_cli(
        &dir,
        &[
            "trip",
            "add",
            "Note Roundtrip Trip",
            "--start",
            "2026-08-01",
            "--end",
            "2026-08-03",
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
            "--time",
            "09:00",
            "--order",
            "3",
            "Aquarium",
        ]
    )
    .status
    .success());

    assert!(run_cli(
        &dir,
        &[
            "note",
            "add",
            "--trip",
            "1",
            "--title",
            "Trip memo",
            "--body",
            "trip note",
        ]
    )
    .status
    .success());
    assert!(run_cli(
        &dir,
        &[
            "note", "add", "--trip", "1", "--day", "2", "--title", "Day memo", "--body",
            "day note",
        ]
    )
    .status
    .success());
    assert!(run_cli(
        &dir,
        &[
            "note",
            "add",
            "--itinerary",
            "1",
            "--title",
            "Itinerary memo",
            "--body",
            "itinerary note",
        ]
    )
    .status
    .success());

    let export_path = dir.join("trip-export-notes.json");
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

    let exported: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&export_path).unwrap()).unwrap();
    assert_eq!(exported["schema_version"], 2);
    assert_eq!(exported["notes"].as_array().unwrap().len(), 3);

    assert!(run_cli(&dir, &["db", "reset"]).status.success());

    let import_output = run_cli(&dir, &["trip", "import", export_path.to_str().unwrap()]);
    assert!(import_output.status.success(), "{:?}", import_output.stderr);
    let import_stdout = String::from_utf8_lossy(&import_output.stdout);
    assert!(import_stdout.contains("Note           : 3 件"));

    let reexport_path = dir.join("trip-reexport-notes.json");
    assert!(run_cli(
        &dir,
        &[
            "trip",
            "export",
            "1",
            "--output",
            reexport_path.to_str().unwrap(),
        ]
    )
    .status
    .success());

    let before: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&export_path).unwrap()).unwrap();
    let after: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&reexport_path).unwrap()).unwrap();

    assert_eq!(
        comparable_export_json(&before),
        comparable_export_json(&after)
    );
}

fn comparable_export_json(value: &serde_json::Value) -> serde_json::Value {
    let trip = &value["trip"];
    let itinerary = value["itinerary_items"]
        .as_array()
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .map(|item| {
            serde_json::json!({
                "day": item["day"],
                "title": item["title"],
                "note": item["note"],
                "start_time": item["start_time"],
                "sort_order": item["sort_order"],
                "duration_minutes": item["duration_minutes"],
                "travel_minutes": item["travel_minutes"],
                "location": item["location"],
                "category": item["category"],
            })
        })
        .collect::<Vec<_>>();

    let checklist = value
        .get("checklist_items")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .map(|item| {
            serde_json::json!({
                "title": item["title"],
                "is_done": item["is_done"],
                "sort_order": item["sort_order"],
            })
        })
        .collect::<Vec<_>>();

    let notes = value
        .get("notes")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();

    serde_json::json!({
        "trip_name": trip["name"],
        "trip_start_date": trip["start_date"],
        "trip_end_date": trip["end_date"],
        "itinerary_items": itinerary,
        "checklist_items": checklist,
        "notes": notes,
    })
}
