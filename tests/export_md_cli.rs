use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

static TEST_DIR_COUNTER: AtomicU64 = AtomicU64::new(0);

fn temp_workdir() -> PathBuf {
    let n = TEST_DIR_COUNTER.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!("travel-ledger-cli-export-md-test-{n}"));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn run_cli(dir: &PathBuf, args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_travel-ledger-cli"))
        .current_dir(dir)
        .args(args)
        .output()
        .expect("failed to run CLI")
}

#[test]
fn cli_export_md_stdout_mode() {
    let dir = temp_workdir();
    assert!(run_cli(
        &dir,
        &[
            "trip",
            "add",
            "Okinawa Trip",
            "--start",
            "2026-01-01",
            "--end",
            "2026-01-03",
        ]
    )
    .status
    .success());

    let output = run_cli(&dir, &["trip", "export-md", "1"]);
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("# Okinawa Trip"));
    assert!(stdout.contains("## Overview"));
    assert!(
        !stdout.contains("Markdown exported:"),
        "stdout mode should not print export confirmation"
    );
}

#[test]
fn cli_export_md_output_file() {
    let dir = temp_workdir();
    assert!(run_cli(
        &dir,
        &[
            "trip",
            "add",
            "Okinawa Trip",
            "--start",
            "2026-01-01",
            "--end",
            "2026-01-03",
        ]
    )
    .status
    .success());

    let output = run_cli(&dir, &["trip", "export-md", "1", "--output", "okinawa.md"]);
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(stdout.trim(), "Markdown exported: okinawa.md");
    assert!(!stdout.contains("# Okinawa Trip"));

    let content = fs::read_to_string(dir.join("okinawa.md")).expect("output file should exist");
    assert!(content.contains("# Okinawa Trip"));
    assert!(content.contains("## Overview"));
}

#[test]
fn cli_export_md_output_overwrites_existing_file() {
    let dir = temp_workdir();
    assert!(run_cli(
        &dir,
        &[
            "trip",
            "add",
            "First Trip",
            "--start",
            "2026-01-01",
            "--end",
            "2026-01-03",
        ]
    )
    .status
    .success());
    fs::write(dir.join("trip.md"), "old content").unwrap();

    assert!(
        run_cli(&dir, &["trip", "export-md", "1", "--output", "trip.md"],)
            .status
            .success()
    );

    let content = fs::read_to_string(dir.join("trip.md")).unwrap();
    assert!(content.contains("# First Trip"));
    assert!(!content.contains("old content"));
}

#[test]
fn cli_export_md_includes_expenses_under_itinerary() {
    let dir = temp_workdir();
    assert!(run_cli(
        &dir,
        &[
            "trip",
            "add",
            "Expense MD Trip",
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
            "1",
            "--time",
            "09:00",
            "Aquarium"
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

    let output = run_cli(&dir, &["trip", "export-md", "1"]);
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Expenses:"));
    assert!(stdout.contains("- 入館料: 2,500 JPY"));
}

#[test]
fn cli_export_md_includes_participants_section() {
    let dir = temp_workdir();
    assert!(run_cli(&dir, &["db", "reset"]).status.success());
    assert!(run_cli(
        &dir,
        &[
            "trip",
            "add",
            "Participant MD Trip",
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
            "participant",
            "add",
            "--trip",
            "1",
            "--name",
            "ともさん",
            "--self",
            "--sort-order",
            "0",
        ]
    )
    .status
    .success());
    assert!(run_cli(
        &dir,
        &[
            "participant",
            "add",
            "--trip",
            "1",
            "--name",
            "妻",
            "--sort-order",
            "1",
        ]
    )
    .status
    .success());

    let output = run_cli(&dir, &["trip", "export-md", "1"]);
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("## Overview"));
    assert!(stdout.contains("## Participants"));
    assert!(stdout.contains("| Name | Self |"));
    assert!(stdout.contains("ともさん"));
    assert!(stdout.contains("妻"));
    assert!(stdout.contains("| yes |"));
    assert!(stdout.contains("| no |"));
    assert!(stdout.contains("Travelers: 2 (companions: 1)"));
    assert!(
        stdout.find("## Participants").unwrap() > stdout.find("## Overview").unwrap(),
        "Participants section should follow Overview"
    );
}

#[test]
fn cli_export_md_includes_estimates_under_itinerary() {
    let dir = temp_workdir();
    assert!(run_cli(
        &dir,
        &[
            "trip",
            "add",
            "Estimate MD Trip",
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
            "1",
            "--time",
            "09:00",
            "Aquarium"
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
            "2180",
            "--currency",
            "JPY",
            "--title",
            "入館料",
            "--note",
            "大人5名想定",
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
            "カフェ",
        ]
    )
    .status
    .success());

    let output = run_cli(&dir, &["trip", "export-md", "1"]);
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("予定費用:"));
    assert!(stdout.contains("| 入館料 | JPY 2,180 | 大人5名想定 |"));
    assert!(stdout.contains("| カフェ | JPY 5,000 |  |"));
    assert!(stdout.contains("- Planned total:"));
    assert!(stdout.contains("- JPY 7,180"));
}

#[test]
fn cli_export_md_omits_estimate_section_when_none() {
    let dir = temp_workdir();
    assert!(run_cli(
        &dir,
        &[
            "trip",
            "add",
            "No Estimate MD Trip",
            "--start",
            "2026-04-26",
            "--end",
            "2026-04-29",
        ]
    )
    .status
    .success());
    assert!(
        run_cli(&dir, &["itinerary", "add", "1", "--day", "1", "Museum"])
            .status
            .success()
    );

    let output = run_cli(&dir, &["trip", "export-md", "1"]);
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!stdout.contains("予定費用:"));
    assert!(!stdout.contains("- Planned total:"));
}

#[test]
fn cli_export_md_handles_null_title_and_note_estimates() {
    let dir = temp_workdir();
    assert!(run_cli(
        &dir,
        &[
            "trip",
            "add",
            "Null Estimate MD Trip",
            "--start",
            "2026-04-26",
            "--end",
            "2026-04-29",
        ]
    )
    .status
    .success());
    assert!(
        run_cli(&dir, &["itinerary", "add", "1", "--day", "1", "Lunch spot"])
            .status
            .success()
    );
    assert!(run_cli(
        &dir,
        &[
            "estimate",
            "add",
            "--itinerary",
            "1",
            "--amount",
            "1500",
            "--currency",
            "JPY",
        ]
    )
    .status
    .success());

    let output = run_cli(&dir, &["trip", "export-md", "1"]);
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("| - | JPY 1,500 |  |"));
}
