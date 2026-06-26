use std::fs;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

static TEST_DIR_COUNTER: AtomicU64 = AtomicU64::new(0);

fn run_cli(cwd: &std::path::Path, args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_travel-ledger-cli"))
        .current_dir(cwd)
        .args(args)
        .output()
        .expect("failed to run CLI")
}

fn temp_workdir() -> std::path::PathBuf {
    let n = TEST_DIR_COUNTER.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!("travel-ledger-cli-estimate-{n}"));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn setup_trip_with_itinerary(dir: &std::path::Path) {
    assert!(run_cli(dir, &["db", "reset"]).status.success());
    assert!(run_cli(
        dir,
        &[
            "trip",
            "add",
            "Estimate Trip",
            "--start",
            "2026-04-26",
            "--end",
            "2026-04-29",
        ],
    )
    .status
    .success());
    assert!(
        run_cli(dir, &["itinerary", "add", "1", "--day", "1", "Breakfast"],)
            .status
            .success()
    );
}

#[test]
fn cli_estimate_add_list_show_update_delete() {
    let dir = temp_workdir();
    setup_trip_with_itinerary(&dir);

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
            "Hotel breakfast",
        ],
    )
    .status
    .success());

    let list = run_cli(&dir, &["estimate", "list", "--itinerary", "1"]);
    assert!(list.status.success());
    let stdout = String::from_utf8_lossy(&list.stdout);
    assert!(stdout.contains("Hotel breakfast"));

    let show: serde_json::Value =
        serde_json::from_slice(&run_cli(&dir, &["estimate", "show", "1", "--json"]).stdout)
            .unwrap();
    assert_eq!(show["amount"], 14000);
    assert_eq!(show["currency"], "JPY");
    assert_eq!(show["title"], "Hotel breakfast");

    assert!(run_cli(
        &dir,
        &["estimate", "update", "1", "--amount", "15000", "--note", "5 people",],
    )
    .status
    .success());

    let updated: serde_json::Value =
        serde_json::from_slice(&run_cli(&dir, &["estimate", "show", "1", "--json"]).stdout)
            .unwrap();
    assert_eq!(updated["amount"], 15000);
    assert_eq!(updated["note"], "5 people");

    assert!(run_cli(&dir, &["estimate", "delete", "1"]).status.success());
    assert!(!run_cli(&dir, &["estimate", "show", "1"]).status.success());
}

#[test]
fn cli_estimate_list_trip_and_json() {
    let dir = temp_workdir();
    setup_trip_with_itinerary(&dir);
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
        ],
    )
    .status
    .success());

    let list = run_cli(&dir, &["estimate", "list", "--trip", "1"]);
    assert!(list.status.success());

    let json: serde_json::Value = serde_json::from_slice(
        &run_cli(&dir, &["estimate", "list", "--trip", "1", "--json"]).stdout,
    )
    .unwrap();
    assert_eq!(json["trip_id"], 1);
    assert_eq!(json["estimates"].as_array().unwrap().len(), 1);
}

#[test]
fn cli_estimate_list_invalid_target() {
    let dir = temp_workdir();
    setup_trip_with_itinerary(&dir);

    assert!(!run_cli(&dir, &["estimate", "list"]).status.success());
    assert!(!run_cli(
        &dir,
        &["estimate", "list", "--trip", "1", "--itinerary", "1"],
    )
    .status
    .success());
}

#[test]
fn cli_estimate_update_clear_title_and_note() {
    let dir = temp_workdir();
    setup_trip_with_itinerary(&dir);
    assert!(run_cli(
        &dir,
        &[
            "estimate",
            "add",
            "--itinerary",
            "1",
            "--amount",
            "1000",
            "--currency",
            "JPY",
            "--title",
            "Lunch",
            "--note",
            "memo",
        ],
    )
    .status
    .success());

    assert!(run_cli(&dir, &["estimate", "update", "1", "--clear-title"])
        .status
        .success());
    let show: serde_json::Value =
        serde_json::from_slice(&run_cli(&dir, &["estimate", "show", "1", "--json"]).stdout)
            .unwrap();
    assert!(show["title"].is_null());

    assert!(run_cli(&dir, &["estimate", "update", "1", "--clear-note"])
        .status
        .success());
    let show: serde_json::Value =
        serde_json::from_slice(&run_cli(&dir, &["estimate", "show", "1", "--json"]).stdout)
            .unwrap();
    assert!(show["note"].is_null());
}

#[test]
fn cli_estimate_update_no_fields_rejects() {
    let dir = temp_workdir();
    setup_trip_with_itinerary(&dir);
    assert!(run_cli(
        &dir,
        &[
            "estimate",
            "add",
            "--itinerary",
            "1",
            "--amount",
            "1000",
            "--currency",
            "JPY",
        ],
    )
    .status
    .success());
    assert!(!run_cli(&dir, &["estimate", "update", "1"]).status.success());
}

#[test]
fn cli_estimate_usd_decimal_amount() {
    let dir = temp_workdir();
    setup_trip_with_itinerary(&dir);
    assert!(run_cli(
        &dir,
        &[
            "estimate",
            "add",
            "--itinerary",
            "1",
            "--amount",
            "12.50",
            "--currency",
            "USD",
        ],
    )
    .status
    .success());
    let show: serde_json::Value =
        serde_json::from_slice(&run_cli(&dir, &["estimate", "show", "1", "--json"]).stdout)
            .unwrap();
    assert_eq!(show["amount"], 1250);
    assert_eq!(show["currency"], "USD");
}

#[test]
fn cli_estimate_cascade_itinerary_delete() {
    let dir = temp_workdir();
    setup_trip_with_itinerary(&dir);
    assert!(run_cli(
        &dir,
        &[
            "estimate",
            "add",
            "--itinerary",
            "1",
            "--amount",
            "1000",
            "--currency",
            "JPY",
        ],
    )
    .status
    .success());
    assert!(run_cli(&dir, &["itinerary", "delete", "1"])
        .status
        .success());
    assert!(!run_cli(&dir, &["estimate", "show", "1"]).status.success());
}

#[test]
fn cli_estimate_update_currency_without_amount_rejects() {
    let dir = temp_workdir();
    setup_trip_with_itinerary(&dir);
    assert!(run_cli(
        &dir,
        &[
            "estimate",
            "add",
            "--itinerary",
            "1",
            "--amount",
            "1000",
            "--currency",
            "JPY",
        ],
    )
    .status
    .success());
    assert!(
        !run_cli(&dir, &["estimate", "update", "1", "--currency", "USD"],)
            .status
            .success()
    );
}
