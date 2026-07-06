use std::path::PathBuf;
use std::process::Command;

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/fragments")
        .join(name)
}

fn temp_workdir() -> std::path::PathBuf {
    static COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let n = COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!("caglla-fragment-apply-{n}"));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("create temp dir");
    dir
}

fn run_cli_in(cwd: &std::path::Path, args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_travel-ledger-cli"))
        .current_dir(cwd)
        .args(args)
        .output()
        .expect("failed to run CLI")
}

#[test]
fn cli_fragment_apply_dry_run_writes_preview_and_keeps_db_unchanged() {
    let dir = temp_workdir();
    let fragment = fixture_path("apply-ready-fragment.json");
    let preview_path = dir.join("apply-preview.json");

    assert!(run_cli_in(
        &dir,
        &[
            "trip",
            "add",
            "Kyoto Weekend",
            "--start",
            "2026-05-01",
            "--end",
            "2026-05-01",
        ],
    )
    .status
    .success());
    assert!(run_cli_in(
        &dir,
        &["itinerary", "add", "1", "--day", "1", "Morning temple"],
    )
    .status
    .success());

    let before = run_cli_in(&dir, &["itinerary", "list", "1"]);
    assert!(before.status.success());
    let before_stdout = String::from_utf8_lossy(&before.stdout);
    assert_eq!(before_stdout.matches("Morning temple").count(), 1);

    let output = run_cli_in(
        &dir,
        &[
            "fragment",
            "apply",
            fragment.to_str().unwrap(),
            "--dry-run",
            "--trip",
            "1",
            "--output",
            preview_path.to_str().unwrap(),
        ],
    );
    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("apply preview"));
    assert!(stdout.contains("add_itinerary"));
    assert!(stdout.contains("itineraries_after: 2"));

    let preview_json = std::fs::read_to_string(&preview_path).expect("preview json");
    let parsed: serde_json::Value = serde_json::from_str(&preview_json).expect("parse preview");
    assert_eq!(parsed["schema_version"], 8);

    let validate = run_cli_in(
        &dir,
        &["trip", "validate-export", preview_path.to_str().unwrap()],
    );
    assert!(
        validate.status.success(),
        "validate-export stderr: {}",
        String::from_utf8_lossy(&validate.stderr)
    );

    let after = run_cli_in(&dir, &["itinerary", "list", "1"]);
    assert!(after.status.success());
    let after_stdout = String::from_utf8_lossy(&after.stdout);
    assert_eq!(after_stdout.matches("Morning temple").count(), 1);
    assert!(!after_stdout.contains("Lunch at local cafe"));

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn cli_fragment_apply_requires_dry_run_or_confirm_flag() {
    let dir = temp_workdir();
    let fragment = fixture_path("apply-ready-fragment.json");
    let output = run_cli_in(
        &dir,
        &[
            "fragment",
            "apply",
            fragment.to_str().unwrap(),
            "--trip",
            "1",
        ],
    );
    assert!(!output.status.success());
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(combined.contains("--dry-run") || combined.contains("--confirm"));
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn cli_fragment_apply_dry_run_and_confirm_together_fails() {
    let dir = temp_workdir();
    let fragment = fixture_path("apply-ready-fragment.json");
    let output = run_cli_in(
        &dir,
        &[
            "fragment",
            "apply",
            fragment.to_str().unwrap(),
            "--dry-run",
            "--confirm",
            "--trip",
            "1",
        ],
    );
    assert!(!output.status.success());
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(combined.contains("--dry-run") && combined.contains("--confirm"));
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn cli_fragment_apply_confirm_inserts_itinerary() {
    let dir = temp_workdir();
    let fragment = fixture_path("apply-ready-fragment.json");

    assert!(run_cli_in(
        &dir,
        &[
            "trip",
            "add",
            "Kyoto Weekend",
            "--start",
            "2026-05-01",
            "--end",
            "2026-05-01",
        ],
    )
    .status
    .success());
    assert!(run_cli_in(
        &dir,
        &["itinerary", "add", "1", "--day", "1", "Morning temple"],
    )
    .status
    .success());

    let before = run_cli_in(&dir, &["itinerary", "list", "1"]);
    assert!(before.status.success());
    let before_stdout = String::from_utf8_lossy(&before.stdout);
    assert_eq!(before_stdout.matches("Morning temple").count(), 1);
    assert!(!before_stdout.contains("Lunch at local cafe"));

    let output = run_cli_in(
        &dir,
        &[
            "fragment",
            "apply",
            fragment.to_str().unwrap(),
            "--confirm",
            "--trip",
            "1",
        ],
    );
    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("fragment apply --confirm"));
    assert!(stdout.contains("Lunch at local cafe"));

    let list = run_cli_in(&dir, &["itinerary", "list", "1"]);
    assert!(list.status.success());
    let list_stdout = String::from_utf8_lossy(&list.stdout);
    assert_eq!(list_stdout.matches("Morning temple").count(), 1);
    assert_eq!(list_stdout.matches("Lunch at local cafe").count(), 1);

    let day_show = run_cli_in(&dir, &["day", "show", "1", "1"]);
    assert!(day_show.status.success());
    let day_stdout = String::from_utf8_lossy(&day_show.stdout);
    assert!(day_stdout.contains("Lunch at local cafe"));

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn cli_fragment_apply_confirm_unsupported_intent_fails_without_db_write() {
    let dir = temp_workdir();
    let fragment = fixture_path("apply-confirm-enrich-fragment.json");
    assert!(run_cli_in(
        &dir,
        &[
            "trip",
            "add",
            "Trip",
            "--start",
            "2026-05-01",
            "--end",
            "2026-05-01",
        ],
    )
    .status
    .success());

    let output = run_cli_in(
        &dir,
        &[
            "fragment",
            "apply",
            fragment.to_str().unwrap(),
            "--confirm",
            "--trip",
            "1",
        ],
    );
    assert!(!output.status.success());
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(combined.contains("add_itinerary"));

    let list = run_cli_in(&dir, &["itinerary", "list", "1"]);
    assert!(list.status.success());
    assert!(!String::from_utf8_lossy(&list.stdout).contains("Afternoon focus"));
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn cli_fragment_apply_confirm_non_day_target_fails_without_db_write() {
    let dir = temp_workdir();
    let fragment = fixture_path("apply-confirm-itinerary-target-fragment.json");
    assert!(run_cli_in(
        &dir,
        &[
            "trip",
            "add",
            "Trip",
            "--start",
            "2026-05-01",
            "--end",
            "2026-05-01",
        ],
    )
    .status
    .success());
    assert!(run_cli_in(
        &dir,
        &["itinerary", "add", "1", "--day", "1", "Morning temple"],
    )
    .status
    .success());

    let output = run_cli_in(
        &dir,
        &[
            "fragment",
            "apply",
            fragment.to_str().unwrap(),
            "--confirm",
            "--trip",
            "1",
        ],
    );
    assert!(!output.status.success());
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(combined.contains("Day target"));

    let list = run_cli_in(&dir, &["itinerary", "list", "1"]);
    assert!(list.status.success());
    let list_stdout = String::from_utf8_lossy(&list.stdout);
    assert_eq!(list_stdout.matches("Morning temple").count(), 1);
    assert!(!list_stdout.contains("After temple lunch"));
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn cli_fragment_apply_confirm_required_decisions_block_db_write() {
    let dir = temp_workdir();
    let fragment = fixture_path("apply-confirm-required-decisions-fragment.json");
    assert!(run_cli_in(
        &dir,
        &[
            "trip",
            "add",
            "Trip",
            "--start",
            "2026-05-01",
            "--end",
            "2026-05-01",
        ],
    )
    .status
    .success());

    let output = run_cli_in(
        &dir,
        &[
            "fragment",
            "apply",
            fragment.to_str().unwrap(),
            "--confirm",
            "--trip",
            "1",
        ],
    );
    assert!(!output.status.success());
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(combined.contains("required decisions"));

    let list = run_cli_in(&dir, &["itinerary", "list", "1"]);
    assert!(list.status.success());
    assert!(
        String::from_utf8_lossy(&list.stdout)
            .matches("Dinner reservation candidate")
            .count()
            == 0
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn cli_fragment_apply_unresolved_target_fails() {
    let dir = temp_workdir();
    let fragment = fixture_path("warn-fragment-unresolved-target.json");
    assert!(run_cli_in(
        &dir,
        &[
            "trip",
            "add",
            "Trip",
            "--start",
            "2026-05-01",
            "--end",
            "2026-05-01",
        ],
    )
    .status
    .success());

    let output = run_cli_in(
        &dir,
        &[
            "fragment",
            "apply",
            fragment.to_str().unwrap(),
            "--dry-run",
            "--trip",
            "1",
        ],
    );
    assert!(!output.status.success());
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(combined.contains("unresolved"));
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn cli_fragment_apply_missing_trip_fails() {
    let dir = temp_workdir();
    let fragment = fixture_path("apply-ready-fragment.json");
    let output = run_cli_in(
        &dir,
        &[
            "fragment",
            "apply",
            fragment.to_str().unwrap(),
            "--dry-run",
            "--trip",
            "999",
        ],
    );
    assert!(!output.status.success());
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn cli_fragment_apply_json_gate_report() {
    let dir = temp_workdir();
    let fragment = fixture_path("apply-ready-fragment.json");
    assert!(run_cli_in(
        &dir,
        &[
            "trip",
            "add",
            "Kyoto Weekend",
            "--start",
            "2026-05-01",
            "--end",
            "2026-05-01",
        ],
    )
    .status
    .success());
    assert!(run_cli_in(
        &dir,
        &["itinerary", "add", "1", "--day", "1", "Morning temple"],
    )
    .status
    .success());

    let output = run_cli_in(
        &dir,
        &[
            "fragment",
            "apply",
            fragment.to_str().unwrap(),
            "--dry-run",
            "--trip",
            "1",
            "--json",
        ],
    );
    assert!(output.status.success());
    let parsed: serde_json::Value =
        serde_json::from_str(&String::from_utf8_lossy(&output.stdout)).expect("json report");
    assert_eq!(parsed["valid"], true);
    assert_eq!(parsed["dry_run"], true);
    assert_eq!(parsed["trip_id"], 1);
    assert_eq!(parsed["trip_export_valid"], true);

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn cli_fragment_validate_remains_file_only() {
    let fragment = fixture_path("valid-fragment.json");
    let temp_dir = std::env::temp_dir().join(format!(
        "caglla-fragment-validate-isolated-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&temp_dir).expect("create temp dir");
    let output = Command::new(env!("CARGO_BIN_EXE_travel-ledger-cli"))
        .current_dir(&temp_dir)
        .args(["fragment", "validate", fragment.to_str().unwrap()])
        .output()
        .expect("failed to run CLI");
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let _ = std::fs::remove_dir_all(&temp_dir);
}
