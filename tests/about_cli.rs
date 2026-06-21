use std::fs;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

static TEST_DIR_COUNTER: AtomicU64 = AtomicU64::new(0);

fn run_cli_in_temp_dir(args: &[&str]) -> (std::process::Output, std::path::PathBuf) {
    let n = TEST_DIR_COUNTER.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!("caglla-cli-about-test-{n}"));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_caglla-cli"))
        .current_dir(&dir)
        .args(args)
        .output()
        .expect("failed to run CLI");

    (output, dir.join("caglla.db"))
}

#[test]
fn cli_about_prints_english_overview() {
    let (output, db_path) = run_cli_in_temp_dir(&["--about"]);

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Caglla.Travel CLI"));
    assert!(stdout.contains("local-first travel planning CLI"));
    assert!(stdout.contains("Itinerary is not a venue"));
    assert!(stdout.contains("caglla.db"));
    assert!(stdout.contains("License: MIT"));
    assert!(
        !db_path.exists(),
        "about path should not create/open SQLite DB file"
    );
}

#[test]
fn cli_about_does_not_require_subcommand() {
    let (output, db_path) = run_cli_in_temp_dir(&["--about", "trip", "list"]);

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Caglla.Travel CLI"));
    assert!(
        !db_path.exists(),
        "about path should not create/open SQLite DB file"
    );
}
