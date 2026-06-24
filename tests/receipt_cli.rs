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
    let dir = std::env::temp_dir().join(format!("caglla-cli-receipt-{n}"));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn setup_trip_with_itinerary(dir: &std::path::Path) -> i64 {
    assert!(run_cli(dir, &["db", "reset"]).status.success());
    assert!(run_cli(
        dir,
        &[
            "trip",
            "add",
            "Receipt Trip",
            "--start",
            "2026-04-26",
            "--end",
            "2026-04-29",
        ],
    )
    .status
    .success());
    assert!(
        run_cli(dir, &["itinerary", "add", "1", "--day", "1", "Shop"],)
            .status
            .success()
    );
    1
}

#[test]
fn cli_receipt_add_list_show_update_link_ignore_delete() {
    let dir = temp_workdir();
    setup_trip_with_itinerary(&dir);

    assert!(run_cli(
        &dir,
        &[
            "receipt",
            "add",
            "--trip",
            "1",
            "--day",
            "1",
            "--amount",
            "1700",
            "--currency",
            "JPY",
            "--memo",
            "これなんだっけ？",
        ],
    )
    .status
    .success());

    let list = run_cli(&dir, &["receipt", "list", "--trip", "1"]);
    assert!(list.status.success());
    let stdout = String::from_utf8_lossy(&list.stdout);
    assert!(stdout.contains("unreviewed"));
    assert!(stdout.contains("1,700") || stdout.contains("1700"));

    let unreviewed = run_cli(&dir, &["receipt", "list", "--trip", "1", "--unreviewed"]);
    assert!(unreviewed.status.success());
    assert!(
        String::from_utf8_lossy(&unreviewed.stdout).contains("1,700")
            || String::from_utf8_lossy(&unreviewed.stdout).contains("1700")
    );

    let show: serde_json::Value =
        serde_json::from_slice(&run_cli(&dir, &["receipt", "show", "1", "--json"]).stdout).unwrap();
    assert_eq!(show["status"], "unreviewed");
    assert_eq!(show["amount"], 1700);
    assert_eq!(show["currency"], "JPY");
    assert_eq!(show["memo"], "これなんだっけ？");

    assert!(run_cli(
        &dir,
        &["receipt", "update", "1", "--memo", "おかんのお土産っぽい",],
    )
    .status
    .success());

    assert!(run_cli(&dir, &["receipt", "link", "1", "--day", "1"])
        .status
        .success());
    let linked = run_cli(&dir, &["receipt", "show", "1", "--json"]);
    let linked_json: serde_json::Value = serde_json::from_slice(&linked.stdout).unwrap();
    assert_eq!(linked_json["status"], "linked");

    assert!(run_cli(&dir, &["receipt", "link", "1", "--itinerary", "1"])
        .status
        .success());
    let linked_it: serde_json::Value =
        serde_json::from_slice(&run_cli(&dir, &["receipt", "show", "1", "--json"]).stdout).unwrap();
    assert_eq!(linked_it["itinerary_id"], 1);

    assert!(run_cli(
        &dir,
        &["receipt", "ignore", "1", "--memo", "旅行費用ではない"],
    )
    .status
    .success());
    let ignored: serde_json::Value =
        serde_json::from_slice(&run_cli(&dir, &["receipt", "show", "1", "--json"]).stdout).unwrap();
    assert_eq!(ignored["status"], "ignored");
    assert_eq!(ignored["amount"], 1700);

    let ignored_list = run_cli(
        &dir,
        &["receipt", "list", "--trip", "1", "--status", "ignored"],
    );
    assert!(ignored_list.status.success());
    assert!(String::from_utf8_lossy(&ignored_list.stdout).contains("ignored"));

    assert!(run_cli(&dir, &["receipt", "delete", "1"]).status.success());
    let empty = run_cli(&dir, &["receipt", "list", "--trip", "1"]);
    assert!(!String::from_utf8_lossy(&empty.stdout).contains("1700"));
}

#[test]
fn cli_receipt_validation_amount_currency_pair() {
    let dir = temp_workdir();
    setup_trip_with_itinerary(&dir);

    let amount_only = run_cli(
        &dir,
        &[
            "receipt",
            "add",
            "--trip",
            "1",
            "--amount",
            "100",
            "--memo",
            "no currency",
        ],
    );
    assert!(!amount_only.status.success());

    let currency_only = run_cli(
        &dir,
        &[
            "receipt",
            "add",
            "--trip",
            "1",
            "--currency",
            "JPY",
            "--memo",
            "no amount",
        ],
    );
    assert!(!currency_only.status.success());
}

#[test]
fn cli_receipt_list_uses_shared_amount_formatter() {
    let dir = temp_workdir();
    setup_trip_with_itinerary(&dir);

    assert!(run_cli(
        &dir,
        &[
            "receipt",
            "add",
            "--trip",
            "1",
            "--amount",
            "12.50",
            "--currency",
            "USD",
            "--memo",
            "coffee",
        ],
    )
    .status
    .success());

    let list = run_cli(&dir, &["receipt", "list", "--trip", "1"]);
    let stdout = String::from_utf8_lossy(&list.stdout);
    assert!(stdout.contains("12.50 USD"));
}

#[test]
fn cli_receipt_export_v7_trip_level_not_under_itinerary() {
    let dir = temp_workdir();
    setup_trip_with_itinerary(&dir);

    assert!(run_cli(
        &dir,
        &["receipt", "add", "--trip", "1", "--memo", "inbox item",],
    )
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
        ],
    )
    .status
    .success());

    let exported: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&export_path).unwrap()).unwrap();
    assert_eq!(exported["schema_version"], 7);
    assert!(exported["receipts"].as_array().unwrap().len() >= 1);
    let first_day = &exported["days"][0];
    let first_it = &first_day["itineraries"][0];
    assert!(first_it.get("receipts").is_none());
    assert!(exported.get("image_path").is_none());
    assert!(exported["receipts"][0].get("image_path").is_none());
}

#[test]
fn cli_receipt_v6_import_still_works() {
    let dir = temp_workdir();
    assert!(run_cli(&dir, &["db", "reset"]).status.success());
    assert!(run_cli(
        &dir,
        &[
            "trip",
            "add",
            "V6 Trip",
            "--start",
            "2026-04-26",
            "--end",
            "2026-04-28",
        ],
    )
    .status
    .success());
    assert!(
        run_cli(&dir, &["itinerary", "add", "1", "--day", "1", "Breakfast"])
            .status
            .success()
    );

    let export_path = dir.join("v6-export.json");
    assert!(run_cli(
        &dir,
        &[
            "trip",
            "export",
            "1",
            "--output",
            export_path.to_str().unwrap(),
        ],
    )
    .status
    .success());

    let mut exported: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&export_path).unwrap()).unwrap();
    exported["schema_version"] = serde_json::json!(6);
    exported.as_object_mut().unwrap().remove("receipts");
    fs::write(
        &export_path,
        serde_json::to_string_pretty(&exported).unwrap(),
    )
    .unwrap();

    assert!(run_cli(&dir, &["db", "reset"]).status.success());
    let import = run_cli(&dir, &["trip", "import", export_path.to_str().unwrap()]);
    assert!(import.status.success(), "{:?}", import.stderr);

    let list = run_cli(&dir, &["receipt", "list", "--trip", "1"]);
    assert!(list.status.success());
}

#[test]
fn cli_receipt_does_not_affect_trip_stats() {
    let dir = temp_workdir();
    setup_trip_with_itinerary(&dir);

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
        ],
    )
    .status
    .success());
    assert!(run_cli(
        &dir,
        &[
            "receipt",
            "add",
            "--trip",
            "1",
            "--amount",
            "9999",
            "--currency",
            "JPY",
            "--memo",
            "not actual",
        ],
    )
    .status
    .success());

    let stats_json: serde_json::Value =
        serde_json::from_slice(&run_cli(&dir, &["trip", "stats", "1", "--json"]).stdout).unwrap();
    assert_eq!(stats_json["expense_count"], 1);
    assert_eq!(stats_json["expense_totals"]["JPY"], 500);
    assert!(stats_json.get("receipt_count").is_none());
}
