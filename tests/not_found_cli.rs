use std::fs;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

static TEST_DIR_COUNTER: AtomicU64 = AtomicU64::new(0);

fn run_cli(args: &[&str]) -> std::process::Output {
    let n = TEST_DIR_COUNTER.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!("travel-ledger-cli-not-found-cli-test-{n}"));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();

    Command::new(env!("CARGO_BIN_EXE_travel-ledger-cli"))
        .current_dir(&dir)
        .args(args)
        .output()
        .expect("failed to run CLI")
}

fn assert_not_found(output: &std::process::Output, expected: &str) {
    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains(expected),
        "expected stderr to contain '{expected}', got: {stderr}"
    );
    assert!(
        !stderr.contains("Query returned no rows"),
        "stderr leaked internal error: {stderr}"
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.trim().is_empty(),
        "stdout should be empty, got: {stdout}"
    );
}

#[test]
fn cli_trip_show_not_found() {
    let output = run_cli(&["trip", "show", "9999"]);
    assert_not_found(&output, "Trip not found: 9999");
}

#[test]
fn cli_trip_show_not_found_json() {
    let output = run_cli(&["trip", "show", "9999", "--json"]);
    assert_not_found(&output, "Trip not found: 9999");
}

#[test]
fn cli_itinerary_list_not_found() {
    let output = run_cli(&["itinerary", "list", "9999"]);
    assert_not_found(&output, "Trip not found: 9999");
}

#[test]
fn cli_itinerary_show_not_found() {
    let output = run_cli(&["itinerary", "show", "9999"]);
    assert_not_found(&output, "Itinerary not found: 9999");
}

#[test]
fn cli_itinerary_show_not_found_json() {
    let output = run_cli(&["itinerary", "show", "9999", "--json"]);
    assert_not_found(&output, "Itinerary not found: 9999");
}

#[test]
fn cli_checklist_show_not_found() {
    let output = run_cli(&["checklist", "show", "9999"]);
    assert_not_found(&output, "Checklist item not found: 9999");
}

#[test]
fn cli_checklist_list_not_found_json() {
    let output = run_cli(&["checklist", "list", "9999", "--json"]);
    assert_not_found(&output, "Trip not found: 9999");
}

#[test]
fn cli_trip_stats_not_found_json() {
    let output = run_cli(&["trip", "stats", "9999", "--json"]);
    assert_not_found(&output, "Trip not found: 9999");
}

#[test]
fn cli_trip_doctor_not_found() {
    let output = run_cli(&["trip", "doctor", "9999"]);
    assert_not_found(&output, "Trip not found: 9999");
}

#[test]
fn cli_day_list_not_found() {
    let output = run_cli(&["day", "list", "9999"]);
    assert_not_found(&output, "Trip not found: 9999");
}

#[test]
fn cli_day_show_not_found_json() {
    let output = run_cli(&["day", "show", "9999", "1", "--json"]);
    assert_not_found(&output, "Trip not found: 9999");
}

#[test]
fn cli_note_show_not_found() {
    let output = run_cli(&["note", "show", "9999"]);
    assert_not_found(&output, "Note not found: 9999");
}

#[test]
fn cli_note_show_not_found_json() {
    let output = run_cli(&["note", "show", "9999", "--json"]);
    assert_not_found(&output, "Note not found: 9999");
}

#[test]
fn cli_participant_show_not_found() {
    let output = run_cli(&["participant", "show", "9999"]);
    assert_not_found(&output, "participant not found: 9999");
}

#[test]
fn cli_participant_list_not_found() {
    let output = run_cli(&["participant", "list", "--trip", "9999"]);
    assert_not_found(&output, "Trip not found: 9999");
}
