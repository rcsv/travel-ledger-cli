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
    let dir = std::env::temp_dir().join(format!("caglla-cli-note-{n}"));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn setup_trip(dir: &std::path::Path) {
    assert!(run_cli(dir, &["db", "reset"]).status.success());
    assert!(run_cli(
        dir,
        &[
            "trip",
            "add",
            "Note Trip",
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
fn cli_note_add_trip_note() {
    let dir = temp_workdir();
    setup_trip(&dir);

    let output = run_cli(
        &dir,
        &[
            "note",
            "add",
            "--trip",
            "1",
            "--title",
            "全体メモ",
            "--body",
            "旅の方針",
        ],
    );
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Note を追加しました"));
    assert!(stdout.contains("全体メモ"));
    assert!(stdout.contains("旅の方針"));
}

#[test]
fn cli_note_add_day_note() {
    let dir = temp_workdir();
    setup_trip(&dir);

    let output = run_cli(
        &dir,
        &[
            "note",
            "add",
            "--trip",
            "1",
            "--day",
            "2",
            "--body",
            "2日目メモ",
        ],
    );
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let list = run_cli(
        &dir,
        &["note", "list", "--trip", "1", "--day", "2", "--json"],
    );
    assert!(list.status.success());
    let parsed: serde_json::Value = serde_json::from_slice(&list.stdout).unwrap();
    assert_eq!(parsed["owner_type"], "day");
    assert_eq!(parsed["notes"].as_array().unwrap().len(), 1);
    assert_eq!(parsed["notes"][0]["body"], "2日目メモ");
}

#[test]
fn cli_note_add_itinerary_note() {
    let dir = temp_workdir();
    setup_trip(&dir);
    assert!(
        run_cli(&dir, &["itinerary", "add", "1", "--day", "1", "首里城"],)
            .status
            .success()
    );

    let output = run_cli(
        &dir,
        &[
            "note",
            "add",
            "--itinerary",
            "1",
            "--title",
            "駐車場",
            "--body",
            "北側P",
        ],
    );
    assert!(output.status.success());
    let list = run_cli(&dir, &["note", "list", "--itinerary", "1"]);
    assert!(String::from_utf8_lossy(&list.stdout).contains("北側P"));
}

#[test]
fn cli_note_list_json_trip_only() {
    let dir = temp_workdir();
    setup_trip(&dir);
    assert!(
        run_cli(&dir, &["note", "add", "--trip", "1", "--body", "trip note"],)
            .status
            .success()
    );
    assert!(run_cli(
        &dir,
        &["note", "add", "--trip", "1", "--day", "2", "--body", "day note",],
    )
    .status
    .success());

    let output = run_cli(&dir, &["note", "list", "--trip", "1", "--json"]);
    let parsed: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(parsed["owner_type"], "trip");
    assert_eq!(parsed["notes"].as_array().unwrap().len(), 1);
    assert_eq!(parsed["notes"][0]["body"], "trip note");
}

#[test]
fn cli_note_show_json() {
    let dir = temp_workdir();
    setup_trip(&dir);
    assert!(
        run_cli(&dir, &["note", "add", "--trip", "1", "--body", "show me"],)
            .status
            .success()
    );

    let output = run_cli(&dir, &["note", "show", "1", "--json"]);
    assert!(output.status.success());
    let parsed: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(parsed["id"], 1);
    assert_eq!(parsed["body"], "show me");
}

#[test]
fn cli_note_update_and_delete() {
    let dir = temp_workdir();
    setup_trip(&dir);
    assert!(
        run_cli(&dir, &["note", "add", "--trip", "1", "--body", "before"],)
            .status
            .success()
    );

    let update = run_cli(&dir, &["note", "update", "1", "--body", "after"]);
    assert!(update.status.success());
    let show = run_cli(&dir, &["note", "show", "1"]);
    assert!(String::from_utf8_lossy(&show.stdout).contains("after"));

    let delete = run_cli(&dir, &["note", "delete", "1"]);
    assert!(delete.status.success());
    let list = run_cli(&dir, &["note", "list", "--trip", "1", "--json"]);
    let parsed: serde_json::Value = serde_json::from_slice(&list.stdout).unwrap();
    assert_eq!(parsed["notes"].as_array().unwrap().len(), 0);
}

#[test]
fn cli_note_add_rejects_empty_body() {
    let dir = temp_workdir();
    setup_trip(&dir);

    let output = run_cli(&dir, &["note", "add", "--trip", "1", "--body", ""]);
    assert!(!output.status.success());
}

#[test]
fn cli_note_add_rejects_conflicting_owner_flags() {
    let dir = temp_workdir();
    setup_trip(&dir);

    let output = run_cli(
        &dir,
        &[
            "note",
            "add",
            "--trip",
            "1",
            "--itinerary",
            "1",
            "--body",
            "x",
        ],
    );
    assert!(!output.status.success());
}

#[test]
fn cli_trip_delete_removes_all_notes() {
    let dir = temp_workdir();
    setup_trip(&dir);
    assert!(
        run_cli(&dir, &["itinerary", "add", "1", "--day", "2", "Plan"],)
            .status
            .success()
    );
    assert!(
        run_cli(&dir, &["note", "add", "--trip", "1", "--body", "trip note"],)
            .status
            .success()
    );
    assert!(run_cli(
        &dir,
        &["note", "add", "--trip", "1", "--day", "2", "--body", "day note"],
    )
    .status
    .success());
    assert!(run_cli(
        &dir,
        &[
            "note",
            "add",
            "--itinerary",
            "1",
            "--body",
            "itinerary note"
        ],
    )
    .status
    .success());

    let output = run_cli(&dir, &["trip", "delete", "1"]);
    assert!(output.status.success());

    let list = run_cli(&dir, &["note", "list", "--trip", "1"]);
    assert!(!list.status.success());
}

#[test]
fn cli_trip_update_shrink_failure_preserves_trip_and_day_notes() {
    let dir = temp_workdir();
    setup_trip(&dir);
    assert!(
        run_cli(&dir, &["itinerary", "add", "1", "--day", "3", "Busy"],)
            .status
            .success()
    );
    assert!(run_cli(
        &dir,
        &["note", "add", "--trip", "1", "--day", "4", "--body", "day4"],
    )
    .status
    .success());

    let output = run_cli(&dir, &["trip", "update", "1", "--end", "2026-04-27"]);
    assert!(!output.status.success());

    let show = run_cli(&dir, &["trip", "show", "1"]);
    assert!(String::from_utf8_lossy(&show.stdout).contains("2026-04-29"));
    let list = run_cli(
        &dir,
        &["note", "list", "--trip", "1", "--day", "4", "--json"],
    );
    assert!(list.status.success());
    let parsed: serde_json::Value = serde_json::from_slice(&list.stdout).unwrap();
    assert_eq!(parsed["notes"].as_array().unwrap().len(), 1);
}
