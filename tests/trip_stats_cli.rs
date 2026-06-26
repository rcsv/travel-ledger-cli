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
    let dir = std::env::temp_dir().join(format!("travel-ledger-cli-trip-stats-{n}"));
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
            "Stats Trip",
            "--start",
            "2026-04-26",
            "--end",
            "2026-04-29",
        ],
    )
    .status
    .success());
    assert!(
        run_cli(dir, &["itinerary", "add", "1", "--day", "1", "Aquarium"],)
            .status
            .success()
    );
}

#[test]
fn cli_trip_stats_shows_planned_total_with_estimates() {
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
            "2180",
            "--currency",
            "JPY",
            "--title",
            "入館料",
        ],
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
            "カフェ",
        ],
    )
    .status
    .success());

    let output = run_cli(&dir, &["trip", "stats", "1"]);
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Estimates: 2"));
    assert!(stdout.contains("Planned total:"));
    assert!(stdout.contains("JPY 7,180"));
}

#[test]
fn cli_trip_stats_multi_currency_estimates() {
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
            "10000",
            "--currency",
            "JPY",
        ],
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
            "12.50",
            "--currency",
            "USD",
        ],
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
            "5.00",
            "--currency",
            "USD",
        ],
    )
    .status
    .success());

    let output = run_cli(&dir, &["trip", "stats", "1"]);
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("JPY 10,000"));
    assert!(stdout.contains("USD 17.50"));
}

#[test]
fn cli_trip_stats_without_estimates_still_works() {
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
            "1200",
            "--currency",
            "JPY",
        ],
    )
    .status
    .success());

    let output = run_cli(&dir, &["trip", "stats", "1"]);
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!stdout.contains("Planned total:"));
    assert!(!stdout.contains("Estimates:"));
    assert!(stdout.contains("Expenses: 1"));
    assert!(stdout.contains("Actual total:"));
    assert!(stdout.contains("JPY 1,200"));
}

#[test]
fn cli_trip_stats_json_includes_estimate_totals() {
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
            "2500",
            "--currency",
            "JPY",
        ],
    )
    .status
    .success());

    let output = run_cli(&dir, &["trip", "stats", "1", "--json"]);
    assert!(output.status.success());
    let parsed: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("stats json should parse");
    assert_eq!(parsed["estimate_count"], 1);
    assert_eq!(parsed["estimate_totals"]["JPY"], 2500);
    assert_eq!(parsed["expense_count"], 0);
    assert!(parsed.get("difference_totals").is_none());
}

#[test]
fn cli_trip_stats_shows_difference_with_estimates_and_expenses() {
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
            "180000",
            "--currency",
            "JPY",
        ],
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
            "172500",
            "--currency",
            "JPY",
        ],
    )
    .status
    .success());

    let output = run_cli(&dir, &["trip", "stats", "1"]);
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Planned total:"));
    assert!(stdout.contains("Actual total:"));
    assert!(stdout.contains("Difference:"));
    assert!(stdout.contains("JPY -7,500"));
}

#[test]
fn cli_trip_stats_json_includes_difference_totals() {
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
            "10000",
            "--currency",
            "JPY",
        ],
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
            "0.50",
            "--currency",
            "USD",
        ],
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
            "9500",
            "--currency",
            "JPY",
        ],
    )
    .status
    .success());

    let output = run_cli(&dir, &["trip", "stats", "1", "--json"]);
    assert!(output.status.success());
    let parsed: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("stats json should parse");
    assert_eq!(parsed["difference_totals"]["JPY"], -500);
    assert_eq!(parsed["difference_totals"]["USD"], -50);
}

#[test]
fn cli_trip_stats_estimate_only_omits_difference() {
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
            "2500",
            "--currency",
            "JPY",
        ],
    )
    .status
    .success());

    let output = run_cli(&dir, &["trip", "stats", "1", "--json"]);
    assert!(output.status.success());
    let parsed: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("stats json should parse");
    assert!(parsed.get("difference_totals").is_none());

    let human = run_cli(&dir, &["trip", "stats", "1"]);
    let stdout = String::from_utf8_lossy(&human.stdout);
    assert!(!stdout.contains("Difference:"));
}

#[test]
fn cli_trip_stats_expense_only_omits_difference() {
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
            "1200",
            "--currency",
            "JPY",
        ],
    )
    .status
    .success());

    let output = run_cli(&dir, &["trip", "stats", "1", "--json"]);
    assert!(output.status.success());
    let parsed: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("stats json should parse");
    assert!(parsed.get("difference_totals").is_none());

    let human = run_cli(&dir, &["trip", "stats", "1"]);
    let stdout = String::from_utf8_lossy(&human.stdout);
    assert!(!stdout.contains("Difference:"));
}
