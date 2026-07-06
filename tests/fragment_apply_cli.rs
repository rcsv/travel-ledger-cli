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
fn cli_fragment_apply_confirm_writes_expanded_itinerary_fields() {
    let dir = temp_workdir();
    let fragment = fixture_path("apply-expanded-fragment.json");
    let preview_path = dir.join("apply-preview.json");
    let export_path = dir.join("trip-after-confirm.json");

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

    let dry_run = run_cli_in(
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
    assert!(dry_run.status.success(), "dry-run failed");

    let confirm = run_cli_in(
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
    assert!(confirm.status.success(), "confirm failed");

    let list = run_cli_in(&dir, &["itinerary", "list", "1"]);
    assert!(list.status.success());
    let list_stdout = String::from_utf8_lossy(&list.stdout);
    assert!(list_stdout.contains("Afternoon temple visit"));
    assert!(list_stdout.contains("14:30"));
    assert!(list_stdout.contains("90分"));
    assert!(list_stdout.contains("20分"));

    let day_show = run_cli_in(&dir, &["day", "show", "1", "1"]);
    assert!(day_show.status.success());
    let day_stdout = String::from_utf8_lossy(&day_show.stdout);
    assert!(day_stdout.contains("Afternoon temple visit"));

    assert!(run_cli_in(
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
    assert!(run_cli_in(
        &dir,
        &["trip", "validate-export", export_path.to_str().unwrap(),],
    )
    .status
    .success());

    let preview: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&preview_path).unwrap()).unwrap();
    let exported: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&export_path).unwrap()).unwrap();
    let preview_day = preview["days"]
        .as_array()
        .unwrap()
        .iter()
        .find(|day| day["day_number"] == 1)
        .expect("preview day 1");
    let export_day = exported["days"]
        .as_array()
        .unwrap()
        .iter()
        .find(|day| day["day_number"] == 1)
        .expect("export day 1");
    let preview_item = preview_day["itineraries"]
        .as_array()
        .unwrap()
        .iter()
        .find(|item| item["title"] == "Afternoon temple visit")
        .expect("preview item");
    let export_item = export_day["itineraries"]
        .as_array()
        .unwrap()
        .iter()
        .find(|item| item["title"] == "Afternoon temple visit")
        .expect("export item");
    assert_eq!(preview_item["category"], "activity");
    assert_eq!(export_item["category"], "activity");
    assert_eq!(preview_item["start_time"], "14:30");
    assert_eq!(export_item["start_time"], "14:30");
    assert_eq!(preview_item["duration_minutes"], 90);
    assert_eq!(export_item["duration_minutes"], 90);
    assert_eq!(preview_item["travel_minutes"], 20);
    assert_eq!(export_item["travel_minutes"], 20);
    assert_eq!(preview_item["location"], "Kiyomizu area");
    assert_eq!(export_item["location"], "Kiyomizu area");
    assert_eq!(preview_item["sort_order"], export_item["sort_order"]);

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn cli_fragment_apply_invalid_category_blocks_without_db_write() {
    let dir = temp_workdir();
    let fragment = fixture_path("apply-invalid-category-fragment.json");
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

    let before = run_cli_in(&dir, &["itinerary", "list", "1"]);
    assert!(before.status.success());

    for args in [
        [
            "fragment",
            "apply",
            fragment.to_str().unwrap(),
            "--dry-run",
            "--trip",
            "1",
        ],
        [
            "fragment",
            "apply",
            fragment.to_str().unwrap(),
            "--confirm",
            "--trip",
            "1",
        ],
    ] {
        let output = run_cli_in(&dir, &args);
        assert!(!output.status.success());
        let combined = format!(
            "{}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(combined.contains("category"));
    }

    let after = run_cli_in(&dir, &["itinerary", "list", "1"]);
    assert!(after.status.success());
    assert_eq!(
        String::from_utf8_lossy(&before.stdout),
        String::from_utf8_lossy(&after.stdout)
    );

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn cli_fragment_apply_ordering_hint_warns_and_confirm_appends_to_day_end() {
    let dir = temp_workdir();
    let fragment = fixture_path("apply-ordering-hint-fragment.json");
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

    let dry_run = run_cli_in(
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
    assert!(dry_run.status.success());
    let dry_stdout = String::from_utf8_lossy(&dry_run.stdout);
    assert!(dry_stdout.contains("ordering_hint"));

    let confirm = run_cli_in(
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
    assert!(confirm.status.success());

    let list = run_cli_in(&dir, &["itinerary", "list", "1", "--json"]);
    assert!(list.status.success());
    let parsed: serde_json::Value =
        serde_json::from_str(&String::from_utf8_lossy(&list.stdout)).unwrap();
    let evening = parsed
        .as_array()
        .unwrap()
        .iter()
        .find(|item| item["title"] == "Evening walk")
        .expect("evening walk");
    assert_eq!(evening["sort_order"], 2000);

    let _ = std::fs::remove_dir_all(&dir);
}

fn seed_trip_with_itinerary(dir: &std::path::Path) {
    assert!(run_cli_in(
        dir,
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
        dir,
        &["itinerary", "add", "1", "--day", "1", "Morning temple"],
    )
    .status
    .success());
}

fn note_count(dir: &std::path::Path, trip_id: &str) -> usize {
    let output = run_cli_in(dir, &["note", "list", "--trip", trip_id, "--json"]);
    assert!(output.status.success());
    let parsed: serde_json::Value =
        serde_json::from_str(&String::from_utf8_lossy(&output.stdout)).unwrap();
    parsed["notes"]
        .as_array()
        .map(|items| items.len())
        .unwrap_or(0)
}

#[test]
fn cli_fragment_apply_add_note_trip_dry_run_preview_keeps_db_unchanged() {
    let dir = temp_workdir();
    let fragment = fixture_path("apply-add-note-trip-fragment.json");
    let preview_path = dir.join("add-note-trip-preview.json");
    seed_trip_with_itinerary(&dir);

    let before_notes = note_count(&dir, "1");
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
    assert!(stdout.contains("add_note"));
    assert!(stdout.contains("notes_after: 1"));

    let preview_json = std::fs::read_to_string(&preview_path).expect("preview json");
    let parsed: serde_json::Value = serde_json::from_str(&preview_json).expect("parse preview");
    let notes = parsed["notes"].as_array().expect("notes array");
    assert_eq!(notes.len(), 1);
    assert_eq!(notes[0]["owner_type"], "trip");
    assert_eq!(notes[0]["body"], "Book JR tickets before departure week.");

    assert!(run_cli_in(
        &dir,
        &["trip", "validate-export", preview_path.to_str().unwrap()],
    )
    .status
    .success());

    assert_eq!(note_count(&dir, "1"), before_notes);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn cli_fragment_apply_add_note_day_dry_run_preview() {
    let dir = temp_workdir();
    let fragment = fixture_path("apply-add-note-day-fragment.json");
    let preview_path = dir.join("add-note-day-preview.json");
    seed_trip_with_itinerary(&dir);

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
    assert!(output.status.success());

    let preview_json = std::fs::read_to_string(&preview_path).expect("preview json");
    let parsed: serde_json::Value = serde_json::from_str(&preview_json).expect("parse preview");
    let notes = parsed["notes"].as_array().expect("notes array");
    assert_eq!(notes.len(), 1);
    assert_eq!(notes[0]["owner_type"], "day");
    assert_eq!(notes[0]["day_number"], 1);
    assert_eq!(notes[0]["body"], "Temple opens at 06:00 — arrive early.");

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn cli_fragment_apply_add_note_itinerary_dry_run_preview() {
    let dir = temp_workdir();
    let fragment = fixture_path("apply-add-note-itinerary-fragment.json");
    let preview_path = dir.join("add-note-itinerary-preview.json");
    seed_trip_with_itinerary(&dir);

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
    assert!(output.status.success());

    let preview_json = std::fs::read_to_string(&preview_path).expect("preview json");
    let parsed: serde_json::Value = serde_json::from_str(&preview_json).expect("parse preview");
    let notes = parsed["notes"].as_array().expect("notes array");
    assert_eq!(notes.len(), 1);
    assert_eq!(notes[0]["owner_type"], "itinerary");
    assert_eq!(notes[0]["itinerary_key"]["title"], "Morning temple");
    assert_eq!(
        notes[0]["body"],
        "Photography allowed in outer garden only."
    );

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn cli_fragment_apply_add_note_required_decisions_invalid_without_db_write() {
    let dir = temp_workdir();
    let fragment = fixture_path("apply-add-note-required-decisions-fragment.json");
    seed_trip_with_itinerary(&dir);

    let before_notes = note_count(&dir, "1");
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
    assert!(!output.status.success());
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(combined.contains("required decisions"));
    assert_eq!(note_count(&dir, "1"), before_notes);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn cli_fragment_apply_add_note_trip_confirm_inserts_note() {
    let dir = temp_workdir();
    let fragment = fixture_path("apply-add-note-trip-fragment.json");
    seed_trip_with_itinerary(&dir);

    let before_notes = note_count(&dir, "1");
    let output = run_cli_in(
        &dir,
        &[
            "fragment",
            "apply",
            fragment.to_str().unwrap(),
            "--confirm",
            "--trip",
            "1",
            "--json",
        ],
    );
    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let parsed: serde_json::Value =
        serde_json::from_str(&String::from_utf8_lossy(&output.stdout)).expect("json report");
    assert_eq!(parsed["valid"], true);
    assert_eq!(parsed["confirm"], true);
    assert_eq!(parsed["preview"]["action"], "add_note");
    let inserted_id = parsed["inserted_note_id"]
        .as_i64()
        .expect("inserted_note_id");
    assert!(inserted_id > 0);

    assert_eq!(note_count(&dir, "1"), before_notes + 1);
    let show = run_cli_in(&dir, &["note", "show", &inserted_id.to_string(), "--json"]);
    assert!(show.status.success());
    let note: serde_json::Value =
        serde_json::from_str(&String::from_utf8_lossy(&show.stdout)).unwrap();
    assert_eq!(note["owner_type"], "trip");
    assert_eq!(note["body"], "Book JR tickets before departure week.");

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn cli_fragment_apply_add_note_day_confirm_inserts_note() {
    let dir = temp_workdir();
    let fragment = fixture_path("apply-add-note-day-fragment.json");
    seed_trip_with_itinerary(&dir);

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
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Note を DB に追加しました"));

    let list = run_cli_in(
        &dir,
        &["note", "list", "--trip", "1", "--day", "1", "--json"],
    );
    assert!(list.status.success());
    let parsed: serde_json::Value =
        serde_json::from_str(&String::from_utf8_lossy(&list.stdout)).unwrap();
    assert_eq!(parsed["owner_type"], "day");
    assert_eq!(parsed["notes"].as_array().unwrap().len(), 1);
    assert_eq!(
        parsed["notes"][0]["body"],
        "Temple opens at 06:00 — arrive early."
    );

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn cli_fragment_apply_add_note_itinerary_confirm_inserts_note() {
    let dir = temp_workdir();
    let fragment = fixture_path("apply-add-note-itinerary-fragment.json");
    seed_trip_with_itinerary(&dir);

    let output = run_cli_in(
        &dir,
        &[
            "fragment",
            "apply",
            fragment.to_str().unwrap(),
            "--confirm",
            "--trip",
            "1",
            "--json",
        ],
    );
    assert!(output.status.success());
    let parsed: serde_json::Value =
        serde_json::from_str(&String::from_utf8_lossy(&output.stdout)).expect("json report");
    let inserted_id = parsed["inserted_note_id"]
        .as_i64()
        .expect("inserted_note_id");

    let show = run_cli_in(&dir, &["note", "show", &inserted_id.to_string(), "--json"]);
    assert!(show.status.success());
    let note: serde_json::Value =
        serde_json::from_str(&String::from_utf8_lossy(&show.stdout)).unwrap();
    assert_eq!(note["owner_type"], "itinerary");
    assert_eq!(note["body"], "Photography allowed in outer garden only.");

    let itinerary_list = run_cli_in(&dir, &["itinerary", "list", "1", "--json"]);
    assert!(itinerary_list.status.success());
    let items: Vec<serde_json::Value> =
        serde_json::from_str(&String::from_utf8_lossy(&itinerary_list.stdout)).unwrap();
    let itinerary_id = items[0]["id"].as_i64().expect("itinerary id");
    let list = run_cli_in(
        &dir,
        &[
            "note",
            "list",
            "--itinerary",
            &itinerary_id.to_string(),
            "--json",
        ],
    );
    assert!(list.status.success());
    let list_parsed: serde_json::Value =
        serde_json::from_str(&String::from_utf8_lossy(&list.stdout)).unwrap();
    assert_eq!(list_parsed["notes"].as_array().unwrap().len(), 1);

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn cli_fragment_apply_add_note_confirm_required_decisions_block_db_write() {
    let dir = temp_workdir();
    let fragment = fixture_path("apply-add-note-required-decisions-fragment.json");
    seed_trip_with_itinerary(&dir);

    let before_notes = note_count(&dir, "1");
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
    assert_eq!(note_count(&dir, "1"), before_notes);
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
