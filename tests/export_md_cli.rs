use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

static TEST_DIR_COUNTER: AtomicU64 = AtomicU64::new(0);

fn temp_workdir() -> PathBuf {
    let n = TEST_DIR_COUNTER.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!("caglla-cli-export-md-test-{n}"));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn run_cli(dir: &PathBuf, args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_caglla-cli"))
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
