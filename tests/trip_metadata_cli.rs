mod common;

use std::process::Command;

fn reset_and_add_trip(dir: &std::path::Path) {
    assert!(common::run_cli_in(dir, &["db", "reset"]).status.success());
    assert!(common::run_cli_in(
        dir,
        &[
            "trip",
            "add",
            "Base Trip",
            "--start",
            "2026-06-01",
            "--end",
            "2026-06-03",
        ],
    )
    .status
    .success());
}

fn create_legacy_db_without_trip_metadata(dir: &std::path::Path) {
    let db_path = dir.join("caglla.db");
    let sql = r"
CREATE TABLE trips (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    name        TEXT NOT NULL,
    start_date  TEXT,
    end_date    TEXT,
    summary     TEXT,
    created_at  TEXT NOT NULL,
    updated_at  TEXT NOT NULL
);
INSERT INTO trips (name, start_date, end_date, created_at, updated_at)
VALUES ('Legacy Trip', '2026-01-01', '2026-01-03', 't', 't');
";
    let status = Command::new("sqlite3")
        .arg(&db_path)
        .arg(sql)
        .status()
        .expect("sqlite3 required for legacy DB simulation");
    assert!(status.success(), "failed to create legacy caglla.db");
}

#[test]
fn cli_trip_add_show_metadata_and_json_contract() {
    let workspace = common::TestWorkspace::new();
    let dir = workspace.path();
    reset_and_add_trip(&dir);

    let add = common::run_cli_in(
        &dir,
        &[
            "trip",
            "add",
            "Okinawa Trip",
            "--start",
            "2026-07-01",
            "--end",
            "2026-07-05",
            "--main-destination",
            "Okinawa",
            "--main-destination-country-code",
            "jp",
            "--default-currency",
            "jpy",
        ],
    );
    assert!(
        add.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&add.stderr)
    );

    let show = common::run_cli_in(&dir, &["trip", "show", "2"]);
    assert!(show.status.success());
    let show_stdout = String::from_utf8_lossy(&show.stdout);
    assert!(show_stdout.contains("Okinawa"));
    assert!(show_stdout.contains("JP"));
    assert!(show_stdout.contains("JPY"));

    let show_json = common::run_cli_in(&dir, &["trip", "show", "2", "--json"]);
    assert!(show_json.status.success());
    let trip: serde_json::Value = serde_json::from_slice(&show_json.stdout).unwrap();
    assert_eq!(trip["main_destination"], "Okinawa");
    assert_eq!(trip["main_destination_country_code"], "JP");
    assert_eq!(trip["default_currency"], "JPY");
}

#[test]
fn cli_trip_add_without_metadata_remains_compatible() {
    let workspace = common::TestWorkspace::new();
    let dir = workspace.path();
    reset_and_add_trip(&dir);

    let show_json = common::run_cli_in(&dir, &["trip", "show", "1", "--json"]);
    assert!(show_json.status.success());
    let trip: serde_json::Value = serde_json::from_slice(&show_json.stdout).unwrap();
    assert!(trip["main_destination"].is_null());
    assert!(trip["main_destination_country_code"].is_null());
    assert!(trip["default_currency"].is_null());
}

#[test]
fn cli_trip_update_set_clear_and_unrelated_preserve() {
    let workspace = common::TestWorkspace::new();
    let dir = workspace.path();
    reset_and_add_trip(&dir);

    assert!(common::run_cli_in(
        &dir,
        &[
            "trip",
            "update",
            "1",
            "--main-destination",
            "Hawaii",
            "--main-destination-country-code",
            "US",
            "--default-currency",
            "USD",
        ],
    )
    .status
    .success());

    assert!(
        common::run_cli_in(&dir, &["trip", "update", "1", "--clear-default-currency"],)
            .status
            .success()
    );

    assert!(
        common::run_cli_in(&dir, &["trip", "update", "1", "--name", "Renamed Trip"])
            .status
            .success()
    );

    let trip: serde_json::Value =
        serde_json::from_slice(&common::run_cli_in(&dir, &["trip", "show", "1", "--json"]).stdout)
            .unwrap();
    assert_eq!(trip["name"], "Renamed Trip");
    assert_eq!(trip["main_destination"], "Hawaii");
    assert_eq!(trip["main_destination_country_code"], "US");
    assert!(trip["default_currency"].is_null());
}

#[test]
fn cli_trip_update_set_clear_conflict_rejects() {
    let workspace = common::TestWorkspace::new();
    let dir = workspace.path();
    reset_and_add_trip(&dir);

    let output = common::run_cli_in(
        &dir,
        &[
            "trip",
            "update",
            "1",
            "--main-destination",
            "X",
            "--clear-main-destination",
        ],
    );
    assert!(!output.status.success());

    let trip: serde_json::Value =
        serde_json::from_slice(&common::run_cli_in(&dir, &["trip", "show", "1", "--json"]).stdout)
            .unwrap();
    assert!(trip["main_destination"].is_null());
}

#[test]
fn cli_trip_add_rejects_invalid_country_and_currency() {
    let workspace = common::TestWorkspace::new();
    let dir = workspace.path();
    reset_and_add_trip(&dir);

    let bad_country = common::run_cli_in(
        &dir,
        &[
            "trip",
            "add",
            "Bad Country",
            "--start",
            "2026-06-01",
            "--end",
            "2026-06-03",
            "--main-destination-country-code",
            "ZZ",
        ],
    );
    assert!(!bad_country.status.success());

    let bad_currency = common::run_cli_in(
        &dir,
        &[
            "trip",
            "add",
            "Bad Currency",
            "--start",
            "2026-06-01",
            "--end",
            "2026-06-03",
            "--default-currency",
            "ZZZ",
        ],
    );
    assert!(!bad_currency.status.success());

    let list_json = common::run_cli_in(&dir, &["trip", "list", "--json"]);
    let trips: serde_json::Value = serde_json::from_slice(&list_json.stdout).unwrap();
    assert_eq!(trips.as_array().unwrap().len(), 1);
}

#[test]
fn cli_legacy_db_migration_leaves_metadata_null() {
    let workspace = common::TestWorkspace::new();
    let dir = workspace.path();
    create_legacy_db_without_trip_metadata(&dir);

    let show = common::run_cli_in(&dir, &["trip", "show", "1", "--json"]);
    assert!(
        show.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&show.stderr)
    );
    let trip: serde_json::Value = serde_json::from_slice(&show.stdout).unwrap();
    assert_eq!(trip["name"], "Legacy Trip");
    assert!(trip["main_destination"].is_null());
    assert!(trip["main_destination_country_code"].is_null());
    assert!(trip["default_currency"].is_null());
}

#[test]
fn cli_trip_list_plain_text_unchanged_without_metadata_columns() {
    let workspace = common::TestWorkspace::new();
    let dir = workspace.path();
    reset_and_add_trip(&dir);

    let list = common::run_cli_in(&dir, &["trip", "list"]);
    assert!(list.status.success());
    let stdout = String::from_utf8_lossy(&list.stdout);
    assert!(stdout.contains("ID"));
    assert!(stdout.contains("Base Trip"));
    assert!(!stdout.contains("代表目的地"));
    assert!(!stdout.contains("main_destination"));
}
