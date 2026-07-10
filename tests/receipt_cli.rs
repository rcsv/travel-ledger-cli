mod common;

use std::fs;
fn setup_trip(dir: &std::path::Path) {
    assert!(common::run_cli_in(dir, &["db", "reset"]).status.success());
    assert!(common::run_cli_in(
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
}

#[test]
fn cli_receipt_add_list_show_update_ignore_delete() {
    let workspace = common::TestWorkspace::new();
    let dir = workspace.path();
    setup_trip(&dir);

    assert!(common::run_cli_in(
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

    let list = common::run_cli_in(&dir, &["receipt", "list", "--trip", "1"]);
    assert!(list.status.success());
    let stdout = String::from_utf8_lossy(&list.stdout);
    assert!(stdout.contains("unreviewed"));
    assert!(stdout.contains("1,700") || stdout.contains("1700"));

    let unreviewed = common::run_cli_in(&dir, &["receipt", "list", "--trip", "1", "--unreviewed"]);
    assert!(unreviewed.status.success());
    assert!(
        String::from_utf8_lossy(&unreviewed.stdout).contains("1,700")
            || String::from_utf8_lossy(&unreviewed.stdout).contains("1700")
    );

    let show: serde_json::Value = serde_json::from_slice(
        &common::run_cli_in(&dir, &["receipt", "show", "1", "--json"]).stdout,
    )
    .unwrap();
    assert_eq!(show["status"], "unreviewed");
    assert_eq!(show["amount"], 1700);
    assert_eq!(show["currency"], "JPY");
    assert_eq!(show["memo"], "これなんだっけ？");
    assert!(show.get("itinerary_id").is_none());
    assert!(show.get("linked_expense_id").is_none());

    assert!(common::run_cli_in(
        &dir,
        &["receipt", "update", "1", "--memo", "おかんのお土産っぽい",],
    )
    .status
    .success());

    assert!(common::run_cli_in(
        &dir,
        &["receipt", "ignore", "1", "--memo", "旅行費用ではない"],
    )
    .status
    .success());
    let ignored: serde_json::Value = serde_json::from_slice(
        &common::run_cli_in(&dir, &["receipt", "show", "1", "--json"]).stdout,
    )
    .unwrap();
    assert_eq!(ignored["status"], "ignored");
    assert_eq!(ignored["amount"], 1700);

    let ignored_list = common::run_cli_in(
        &dir,
        &[
            "receipt",
            "list",
            "--trip",
            "1",
            "--trashed",
            "--status",
            "ignored",
        ],
    );
    assert!(ignored_list.status.success());
    assert!(String::from_utf8_lossy(&ignored_list.stdout).contains("ignored"));

    assert!(common::run_cli_in(&dir, &["receipt", "delete", "1"])
        .status
        .success());
    let empty = common::run_cli_in(&dir, &["receipt", "list", "--trip", "1"]);
    assert!(!String::from_utf8_lossy(&empty.stdout).contains("1,700"));
}

#[test]
fn cli_receipt_link_command_removed() {
    let workspace = common::TestWorkspace::new();
    let dir = workspace.path();
    setup_trip(&dir);
    assert!(
        common::run_cli_in(&dir, &["receipt", "add", "--trip", "1", "--memo", "inbox",],)
            .status
            .success()
    );

    let output = common::run_cli_in(&dir, &["receipt", "link", "1", "--day", "1"]);
    assert!(!output.status.success());
}

#[test]
fn cli_receipt_validation_amount_currency_pair() {
    let workspace = common::TestWorkspace::new();
    let dir = workspace.path();
    setup_trip(&dir);

    let amount_only = common::run_cli_in(
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

    let currency_only = common::run_cli_in(
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
    let workspace = common::TestWorkspace::new();
    let dir = workspace.path();
    setup_trip(&dir);

    assert!(common::run_cli_in(
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

    let list = common::run_cli_in(&dir, &["receipt", "list", "--trip", "1"]);
    let stdout = String::from_utf8_lossy(&list.stdout);
    assert!(stdout.contains("12.50 USD"));
}

#[test]
fn cli_receipt_export_v8_trip_level_simplified() {
    let workspace = common::TestWorkspace::new();
    let dir = workspace.path();
    setup_trip(&dir);

    assert!(common::run_cli_in(
        &dir,
        &["receipt", "add", "--trip", "1", "--memo", "inbox item",],
    )
    .status
    .success());

    let export_path = dir.join("trip-export.json");
    assert!(common::run_cli_in(
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
    assert_eq!(exported["schema_version"], 8);
    assert!(exported["receipts"].as_array().unwrap().len() >= 1);
    let receipt = &exported["receipts"][0];
    assert!(receipt.get("itinerary_ref").is_none());
    assert!(receipt.get("linked_expense_ref").is_none());
    assert!(exported.get("image_path").is_none());
    let first_day = &exported["days"][0];
    let first_it = &first_day["itineraries"][0];
    assert!(first_it.get("receipts").is_none());
}

#[test]
fn cli_receipt_v6_import_still_works() {
    let workspace = common::TestWorkspace::new();
    let dir = workspace.path();
    assert!(common::run_cli_in(&dir, &["db", "reset"]).status.success());
    assert!(common::run_cli_in(
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
        common::run_cli_in(&dir, &["itinerary", "add", "1", "--day", "1", "Breakfast"])
            .status
            .success()
    );

    let export_path = dir.join("v6-export.json");
    assert!(common::run_cli_in(
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

    assert!(common::run_cli_in(&dir, &["db", "reset"]).status.success());
    let import = common::run_cli_in(&dir, &["trip", "import", export_path.to_str().unwrap()]);
    assert!(import.status.success(), "{:?}", import.stderr);

    let list = common::run_cli_in(&dir, &["receipt", "list", "--trip", "1"]);
    assert!(list.status.success());
}

#[test]
fn cli_receipt_does_not_affect_trip_stats() {
    let workspace = common::TestWorkspace::new();
    let dir = workspace.path();
    setup_trip(&dir);

    assert!(
        common::run_cli_in(&dir, &["itinerary", "add", "1", "--day", "1", "Breakfast"])
            .status
            .success()
    );
    assert!(common::run_cli_in(
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
    assert!(common::run_cli_in(
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
        serde_json::from_slice(&common::run_cli_in(&dir, &["trip", "stats", "1", "--json"]).stdout)
            .unwrap();
    assert_eq!(stats_json["expense_count"], 1);
    assert_eq!(stats_json["expense_totals"]["JPY"], 500);
    assert!(stats_json.get("receipt_count").is_none());
}
