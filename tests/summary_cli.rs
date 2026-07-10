mod common;

use std::fs;
use std::path::Path;
use std::process::Command;

fn setup_trip(dir: &Path) {
    assert!(common::run_cli_in(dir, &["db", "reset"]).status.success());
    assert!(common::run_cli_in(
        dir,
        &[
            "trip",
            "add",
            "Summary Trip",
            "--start",
            "2026-05-01",
            "--end",
            "2026-05-03",
        ],
    )
    .status
    .success());
}

fn create_legacy_db_without_trip_summary(dir: &Path) {
    let db_path = dir.join("caglla.db");
    let sql = r"
CREATE TABLE trips (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    name        TEXT NOT NULL,
    start_date  TEXT,
    end_date    TEXT,
    created_at  TEXT NOT NULL,
    updated_at  TEXT NOT NULL
);
";
    let status = Command::new("sqlite3")
        .arg(&db_path)
        .arg(sql)
        .status()
        .expect("sqlite3 required for legacy DB simulation");
    assert!(status.success(), "failed to create legacy caglla.db");
}

#[test]
fn cli_trip_add_show_update_and_clear_summary() {
    let workspace = common::TestWorkspace::new();
    let dir = workspace.path();
    assert!(common::run_cli_in(&dir, &["db", "reset"]).status.success());

    let add = common::run_cli_in(
        &dir,
        &[
            "trip",
            "add",
            "Beach Trip",
            "--start",
            "2026-06-01",
            "--end",
            "2026-06-03",
            "--summary",
            "  Relax by the sea  ",
        ],
    );
    assert!(
        add.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&add.stderr)
    );
    let add_stdout = String::from_utf8_lossy(&add.stdout);
    assert!(add_stdout.contains("Relax by the sea"));

    let show = common::run_cli_in(&dir, &["trip", "show", "1"]);
    assert!(show.status.success());
    let show_stdout = String::from_utf8_lossy(&show.stdout);
    assert!(show_stdout.contains("Relax by the sea"));

    let show_json = common::run_cli_in(&dir, &["trip", "show", "1", "--json"]);
    assert!(show_json.status.success());
    let trip: serde_json::Value =
        serde_json::from_slice(&show_json.stdout).expect("trip show --json");
    assert_eq!(trip["summary"], "Relax by the sea");

    assert!(common::run_cli_in(
        &dir,
        &["trip", "update", "1", "--summary", "Updated overview",],
    )
    .status
    .success());

    let after_update = common::run_cli_in(&dir, &["trip", "show", "1", "--json"]);
    let trip: serde_json::Value = serde_json::from_slice(&after_update.stdout).unwrap();
    assert_eq!(trip["summary"], "Updated overview");

    assert!(
        common::run_cli_in(&dir, &["trip", "update", "1", "--clear-summary"])
            .status
            .success()
    );
    let cleared = common::run_cli_in(&dir, &["trip", "show", "1", "--json"]);
    let trip: serde_json::Value = serde_json::from_slice(&cleared.stdout).unwrap();
    assert!(trip["summary"].is_null());
}

#[test]
fn cli_day_update_show_json_includes_summary() {
    let workspace = common::TestWorkspace::new();
    let dir = workspace.path();
    setup_trip(&dir);

    assert!(common::run_cli_in(
        &dir,
        &["day", "update", "1", "2", "--summary", "  Day two theme  ",],
    )
    .status
    .success());

    let show_json = common::run_cli_in(&dir, &["day", "show", "1", "2", "--json"]);
    assert!(
        show_json.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&show_json.stderr)
    );
    let day: serde_json::Value = serde_json::from_slice(&show_json.stdout).unwrap();
    assert_eq!(day["day_number"], 2);
    assert_eq!(day["summary"], "Day two theme");

    assert!(
        common::run_cli_in(&dir, &["day", "update", "1", "2", "--clear-summary"])
            .status
            .success()
    );
    let cleared = common::run_cli_in(&dir, &["day", "show", "1", "2", "--json"]);
    let day: serde_json::Value = serde_json::from_slice(&cleared.stdout).unwrap();
    assert!(day["summary"].is_null());
}

#[test]
fn cli_export_import_roundtrip_preserves_summaries() {
    let workspace = common::TestWorkspace::new();
    let dir = workspace.path();
    assert!(common::run_cli_in(&dir, &["db", "reset"]).status.success());
    assert!(common::run_cli_in(
        &dir,
        &[
            "trip",
            "add",
            "Export Summary Trip",
            "--start",
            "2026-07-01",
            "--end",
            "2026-07-02",
            "--summary",
            "Trip-level note",
        ],
    )
    .status
    .success());
    assert!(common::run_cli_in(
        &dir,
        &["day", "update", "1", "1", "--summary", "First day focus"],
    )
    .status
    .success());

    let export_path = dir.join("trip-export-summary.json");
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
    assert_eq!(exported["trip"]["summary"], "Trip-level note");
    assert_eq!(exported["days"][0]["summary"], "First day focus");

    assert!(common::run_cli_in(&dir, &["db", "reset"]).status.success());
    let import = common::run_cli_in(&dir, &["trip", "import", export_path.to_str().unwrap()]);
    assert!(
        import.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&import.stderr)
    );

    let reexport_path = dir.join("trip-reexport-summary.json");
    assert!(common::run_cli_in(
        &dir,
        &[
            "trip",
            "export",
            "1",
            "--output",
            reexport_path.to_str().unwrap(),
        ],
    )
    .status
    .success());

    let after: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&reexport_path).unwrap()).unwrap();
    assert_eq!(after["trip"]["summary"], "Trip-level note");
    assert_eq!(after["days"][0]["summary"], "First day focus");

    let show = common::run_cli_in(&dir, &["trip", "show", "1", "--json"]);
    let trip: serde_json::Value = serde_json::from_slice(&show.stdout).unwrap();
    assert_eq!(trip["summary"], "Trip-level note");
}

#[test]
fn cli_export_md_includes_trip_summary_when_set() {
    let workspace = common::TestWorkspace::new();
    let dir = workspace.path();
    assert!(common::run_cli_in(&dir, &["db", "reset"]).status.success());
    assert!(common::run_cli_in(
        &dir,
        &[
            "trip",
            "add",
            "Markdown Summary Trip",
            "--start",
            "2026-08-01",
            "--end",
            "2026-08-02",
            "--summary",
            "Pack light and enjoy.",
        ],
    )
    .status
    .success());
    assert!(common::run_cli_in(
        &dir,
        &["day", "update", "1", "1", "--summary", "Arrival day"],
    )
    .status
    .success());
    assert!(common::run_cli_in(
        &dir,
        &[
            "itinerary",
            "add",
            "1",
            "--day",
            "1",
            "--time",
            "10:00",
            "Check in"
        ],
    )
    .status
    .success());

    let output = common::run_cli_in(&dir, &["trip", "export-md", "1"]);
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let md = String::from_utf8_lossy(&output.stdout);
    assert!(md.contains("# Markdown Summary Trip"));
    assert!(md.contains("Pack light and enjoy."));
    assert!(md.contains("## Day 1"));
    assert!(md.contains("Arrival day"));
}

#[test]
fn cli_legacy_db_migration_then_reset_and_summary() {
    let workspace = common::TestWorkspace::new();
    let dir = workspace.path();
    create_legacy_db_without_trip_summary(&dir);

    let add = common::run_cli_in(
        &dir,
        &[
            "trip",
            "add",
            "Migrated Trip",
            "--start",
            "2026-09-01",
            "--end",
            "2026-09-02",
            "--summary",
            "After migration",
        ],
    );
    assert!(
        add.status.success(),
        "legacy DB should migrate: {}",
        String::from_utf8_lossy(&add.stderr)
    );

    let show = common::run_cli_in(&dir, &["trip", "show", "1", "--json"]);
    assert!(show.status.success());
    let trip: serde_json::Value = serde_json::from_slice(&show.stdout).unwrap();
    assert_eq!(trip["summary"], "After migration");

    assert!(common::run_cli_in(&dir, &["db", "reset"]).status.success());
    assert!(common::run_cli_in(
        &dir,
        &[
            "trip",
            "add",
            "Fresh Trip",
            "--start",
            "2026-10-01",
            "--end",
            "2026-10-02",
            "--summary",
            "Post-reset summary",
        ],
    )
    .status
    .success());

    let fresh = common::run_cli_in(&dir, &["trip", "show", "1", "--json"]);
    let trip: serde_json::Value = serde_json::from_slice(&fresh.stdout).unwrap();
    assert_eq!(trip["name"], "Fresh Trip");
    assert_eq!(trip["summary"], "Post-reset summary");
}
