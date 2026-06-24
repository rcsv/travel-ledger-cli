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
    assert_eq!(exported["schema_version"], 7);
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

#[test]
fn cli_export_import_reexport_roundtrip_with_expenses() {
    let dir = temp_workdir();
    assert!(run_cli(&dir, &["db", "reset"]).status.success());

    assert!(run_cli(
        &dir,
        &[
            "trip",
            "add",
            "Expense Roundtrip Trip",
            "--start",
            "2026-04-26",
            "--end",
            "2026-04-29",
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
            "0",
            "Aquarium",
        ]
    )
    .status
    .success());

    assert!(run_cli(
        &dir,
        &[
            "expense",
            "add",
            "--itinerary",
            "1",
            "--amount",
            "2500",
            "--currency",
            "JPY",
            "--title",
            "入館料",
        ]
    )
    .status
    .success());
    assert!(run_cli(
        &dir,
        &[
            "expense",
            "add",
            "--itinerary",
            "1",
            "--amount",
            "500",
            "--currency",
            "JPY",
            "--title",
            "駐車場",
        ]
    )
    .status
    .success());

    let export_path = dir.join("trip-export-expenses.json");
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
    assert_eq!(exported["schema_version"], 7);
    assert_eq!(
        exported["days"][1]["itineraries"][0]["expenses"]
            .as_array()
            .unwrap()
            .len(),
        2
    );

    assert!(run_cli(&dir, &["db", "reset"]).status.success());

    let import_output = run_cli(&dir, &["trip", "import", export_path.to_str().unwrap()]);
    assert!(import_output.status.success(), "{:?}", import_output.stderr);
    let import_stdout = String::from_utf8_lossy(&import_output.stdout);
    assert!(import_stdout.contains("Expense"));

    let reexport_path = dir.join("trip-reexport-expenses.json");
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
fn cli_export_import_reexport_roundtrip_with_estimates() {
    let dir = temp_workdir();
    assert!(run_cli(&dir, &["db", "reset"]).status.success());

    assert!(run_cli(
        &dir,
        &[
            "trip",
            "add",
            "Estimate Roundtrip Trip",
            "--start",
            "2026-05-01",
            "--end",
            "2026-05-03",
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
            "08:00",
            "--order",
            "0",
            "Hotel",
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
            "14000",
            "--currency",
            "JPY",
            "--title",
            "ホテル朝食",
            "--note",
            "5人分",
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
            "5000",
            "--currency",
            "JPY",
            "--title",
            "駐車場",
        ]
    )
    .status
    .success());

    let export_path = dir.join("trip-export-estimates.json");
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
    assert_eq!(exported["schema_version"], 7);
    assert_eq!(
        exported["days"][0]["itineraries"][0]["estimates"]
            .as_array()
            .unwrap()
            .len(),
        2
    );

    assert!(run_cli(&dir, &["db", "reset"]).status.success());

    let import_output = run_cli(&dir, &["trip", "import", export_path.to_str().unwrap()]);
    assert!(import_output.status.success(), "{:?}", import_output.stderr);

    let list = run_cli(&dir, &["estimate", "list", "--trip", "1", "--json"]);
    assert!(list.status.success());
    let list_json: serde_json::Value = serde_json::from_slice(&list.stdout).unwrap();
    assert_eq!(list_json["estimates"].as_array().unwrap().len(), 2);

    let reexport_path = dir.join("trip-reexport-estimates.json");
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

fn comparable_estimate_json(est: &serde_json::Value) -> serde_json::Value {
    serde_json::json!({
        "title": est["title"],
        "amount": est["amount"],
        "currency": est["currency"],
        "note": est["note"],
        "sort_order": est["sort_order"],
    })
}

fn comparable_expense_json(exp: &serde_json::Value) -> serde_json::Value {
    serde_json::json!({
        "title": exp["title"],
        "amount": exp["amount"],
        "currency": exp["currency"],
        "paid_by_name": exp["paid_by_name"],
        "expense_date": exp["expense_date"],
        "note": exp["note"],
        "sort_order": exp["sort_order"],
    })
}

fn comparable_itinerary_json(
    day: &serde_json::Value,
    item: &serde_json::Value,
) -> serde_json::Value {
    let expenses = item["expenses"]
        .as_array()
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .map(|exp| comparable_expense_json(&exp))
        .collect::<Vec<_>>();
    let estimates = item["estimates"]
        .as_array()
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .map(|est| comparable_estimate_json(&est))
        .collect::<Vec<_>>();

    serde_json::json!({
        "day": day["day_number"],
        "title": item["title"],
        "note": item["note"],
        "start_time": item["start_time"],
        "sort_order": item["sort_order"],
        "duration_minutes": item["duration_minutes"],
        "travel_minutes": item["travel_minutes"],
        "location": item["location"],
        "category": item["category"],
        "expenses": expenses,
        "estimates": estimates,
    })
}

fn comparable_export_json(value: &serde_json::Value) -> serde_json::Value {
    let trip = &value["trip"];
    let schema_version = value.get("schema_version").and_then(|v| v.as_i64());

    let itinerary = if matches!(
        schema_version,
        Some(3) | Some(4) | Some(5) | Some(6) | Some(7)
    ) {
        value["days"]
            .as_array()
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .flat_map(|day| {
                day["itineraries"]
                    .as_array()
                    .cloned()
                    .unwrap_or_default()
                    .into_iter()
                    .map(move |item| comparable_itinerary_json(&day, &item))
            })
            .collect::<Vec<_>>()
    } else {
        value["itinerary_items"]
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
                    "expenses": [],
                })
            })
            .collect::<Vec<_>>()
    };

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

    let participants = value
        .get("participants")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .map(|item| {
            serde_json::json!({
                "name": item["name"],
                "sort_order": item["sort_order"],
                "is_self": item["is_self"],
            })
        })
        .collect::<Vec<_>>();

    let receipts = value
        .get("receipts")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .map(|item| {
            serde_json::json!({
                "day_ref": item.get("day_ref"),
                "itinerary_ref": item.get("itinerary_ref"),
                "amount": item.get("amount"),
                "currency": item.get("currency"),
                "occurred_date": item.get("occurred_date"),
                "memo": item.get("memo"),
                "status": item.get("status"),
            })
        })
        .collect::<Vec<_>>();

    serde_json::json!({
        "trip_name": trip["name"],
        "trip_start_date": trip["start_date"],
        "trip_end_date": trip["end_date"],
        "itinerary_items": itinerary,
        "checklist_items": checklist,
        "notes": notes,
        "participants": participants,
        "receipts": receipts,
    })
}
