mod common;

use std::path::PathBuf;
use std::process::Command;

fn fixture_path(name: &str) -> PathBuf {
    common::manifest_dir()
        .join("tests/fixtures/fragments")
        .join(name)
}

fn run_cli_in(cwd: &std::path::Path, args: &[&str]) -> std::process::Output {
    common::run_cli_in(cwd, args)
}

fn query_itinerary_day_columns(db_path: &std::path::Path, itinerary_id: i64) -> (i64, i64) {
    let sql = format!("SELECT day_id, day FROM itinerary_items WHERE id = {itinerary_id};");
    let output = Command::new("sqlite3")
        .arg(db_path)
        .arg(&sql)
        .output()
        .expect("sqlite3 required for raw itinerary_items column check");
    assert!(
        output.status.success(),
        "sqlite3 query failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let line = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let mut parts = line.split('|');
    let day_id: i64 = parts
        .next()
        .expect("day_id column")
        .parse()
        .expect("day_id parse");
    let day: i64 = parts
        .next()
        .expect("day column")
        .parse()
        .expect("day parse");
    (day_id, day)
}

fn query_day_id_for_trip_day(db_path: &std::path::Path, trip_id: i64, day_number: i64) -> i64 {
    let sql =
        format!("SELECT id FROM days WHERE trip_id = {trip_id} AND day_number = {day_number};");
    let output = Command::new("sqlite3")
        .arg(db_path)
        .arg(&sql)
        .output()
        .expect("sqlite3 required for days.id lookup");
    assert!(
        output.status.success(),
        "sqlite3 query failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout)
        .trim()
        .parse()
        .expect("days.id parse")
}

#[test]
fn cli_fragment_apply_dry_run_writes_preview_and_keeps_db_unchanged() {
    let workspace = common::TestWorkspace::new();
    let dir = workspace.path();
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
}

#[test]
fn cli_fragment_apply_requires_dry_run_or_confirm_flag() {
    let workspace = common::TestWorkspace::new();
    let dir = workspace.path();
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
}

#[test]
fn cli_fragment_apply_dry_run_and_confirm_together_fails() {
    let workspace = common::TestWorkspace::new();
    let dir = workspace.path();
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
}

#[test]
fn cli_fragment_apply_confirm_inserts_itinerary() {
    let workspace = common::TestWorkspace::new();
    let dir = workspace.path();
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
}

#[test]
fn cli_fragment_apply_confirm_unsupported_intent_fails_without_db_write() {
    let workspace = common::TestWorkspace::new();
    let dir = workspace.path();
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
}

#[test]
fn cli_fragment_apply_confirm_non_day_target_fails_without_db_write() {
    let workspace = common::TestWorkspace::new();
    let dir = workspace.path();
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
}

#[test]
fn cli_fragment_apply_confirm_required_decisions_block_db_write() {
    let workspace = common::TestWorkspace::new();
    let dir = workspace.path();
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
}

#[test]
fn cli_fragment_apply_unresolved_target_fails() {
    let workspace = common::TestWorkspace::new();
    let dir = workspace.path();
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
            "--json",
        ],
    );
    assert!(!output.status.success());
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(combined.contains("unresolved"));
}

#[test]
fn cli_fragment_apply_missing_trip_fails() {
    let workspace = common::TestWorkspace::new();
    let dir = workspace.path();
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
}

#[test]
fn cli_fragment_apply_json_gate_report() {
    let workspace = common::TestWorkspace::new();
    let dir = workspace.path();
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
}

#[test]
fn cli_fragment_apply_confirm_writes_expanded_itinerary_fields() {
    let workspace = common::TestWorkspace::new();
    let dir = workspace.path();
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
}

#[test]
fn cli_fragment_apply_invalid_category_blocks_without_db_write() {
    let workspace = common::TestWorkspace::new();
    let dir = workspace.path();
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
}

#[test]
fn cli_fragment_apply_ordering_hint_warns_and_confirm_appends_to_day_end() {
    let workspace = common::TestWorkspace::new();
    let dir = workspace.path();
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

fn seed_trip_two_days_with_itineraries(dir: &std::path::Path) {
    assert!(run_cli_in(
        dir,
        &[
            "trip",
            "add",
            "Okinawa Two Days",
            "--start",
            "2026-05-01",
            "--end",
            "2026-05-02",
        ],
    )
    .status
    .success());

    assert!(
        run_cli_in(dir, &["itinerary", "add", "1", "--day", "1", "Breakfast"])
            .status
            .success()
    );
    assert!(
        run_cli_in(dir, &["itinerary", "add", "1", "--day", "1", "Aquarium"])
            .status
            .success()
    );
    assert!(
        run_cli_in(dir, &["itinerary", "add", "1", "--day", "1", "Dinner"])
            .status
            .success()
    );
    assert!(run_cli_in(
        dir,
        &["itinerary", "add", "1", "--day", "2", "Museum (Day 2)"]
    )
    .status
    .success());
}

fn seed_trip_two_days_with_itineraries_for_move(dir: &std::path::Path) {
    assert!(run_cli_in(
        dir,
        &[
            "trip",
            "add",
            "Okinawa Two Days (move)",
            "--start",
            "2026-05-01",
            "--end",
            "2026-05-02",
        ],
    )
    .status
    .success());

    assert!(
        run_cli_in(dir, &["itinerary", "add", "1", "--day", "1", "Breakfast"])
            .status
            .success()
    );
    assert!(
        run_cli_in(dir, &["itinerary", "add", "1", "--day", "1", "Aquarium"])
            .status
            .success()
    );
    assert!(
        run_cli_in(dir, &["itinerary", "add", "1", "--day", "1", "Dinner"])
            .status
            .success()
    );
    assert!(run_cli_in(
        dir,
        &["itinerary", "add", "1", "--day", "2", "Museum (Day 2)"]
    )
    .status
    .success());
    assert!(run_cli_in(
        dir,
        &["itinerary", "add", "1", "--day", "2", "Beach (Day 2)"]
    )
    .status
    .success());
}

fn seed_trip_single_source_itinerary_for_move(dir: &std::path::Path) {
    assert!(run_cli_in(
        dir,
        &[
            "trip",
            "add",
            "Okinawa Single Source (move)",
            "--start",
            "2026-05-01",
            "--end",
            "2026-05-02",
        ],
    )
    .status
    .success());

    assert!(
        run_cli_in(dir, &["itinerary", "add", "1", "--day", "1", "Aquarium"])
            .status
            .success()
    );
    assert!(run_cli_in(
        dir,
        &["itinerary", "add", "1", "--day", "2", "Museum (Day 2)"]
    )
    .status
    .success());
    assert!(run_cli_in(
        dir,
        &["itinerary", "add", "1", "--day", "2", "Beach (Day 2)"]
    )
    .status
    .success());
}

fn seed_trip_day_with_ambiguous_number_selector(dir: &std::path::Path) {
    // Day 1 に id=2 が存在する状態で、別の itinerary に sort_order=2 を与え「数値 selector が id と sort_order に一致」するケースを作る
    assert!(run_cli_in(
        dir,
        &[
            "trip",
            "add",
            "Ambiguous Selector Trip",
            "--start",
            "2026-05-01",
            "--end",
            "2026-05-01",
        ],
    )
    .status
    .success());
    assert!(
        run_cli_in(dir, &["itinerary", "add", "1", "--day", "1", "One"])
            .status
            .success()
    );
    assert!(
        run_cli_in(dir, &["itinerary", "add", "1", "--day", "1", "Two (id=2)"])
            .status
            .success()
    );
    assert!(run_cli_in(
        dir,
        &[
            "itinerary",
            "add",
            "1",
            "--day",
            "1",
            "--order",
            "2",
            "SortOrder=2 (ambiguous)",
        ],
    )
    .status
    .success());
}

fn seed_trip_two_days_with_ambiguous_move_number_selector(dir: &std::path::Path) {
    assert!(run_cli_in(
        dir,
        &[
            "trip",
            "add",
            "Ambiguous Move Selector Trip",
            "--start",
            "2026-05-01",
            "--end",
            "2026-05-02",
        ],
    )
    .status
    .success());
    assert!(
        run_cli_in(dir, &["itinerary", "add", "1", "--day", "1", "One"])
            .status
            .success()
    );
    assert!(
        run_cli_in(dir, &["itinerary", "add", "1", "--day", "1", "Two (id=2)"])
            .status
            .success()
    );
    // Day 2 に sort_order=2 を作り、after_destination_order の「2」が id と sort_order の双方に一致する状態にする
    assert!(run_cli_in(
        dir,
        &[
            "itinerary",
            "add",
            "1",
            "--day",
            "2",
            "--order",
            "2",
            "Dest sort_order=2",
        ],
    )
    .status
    .success());
    assert!(
        run_cli_in(dir, &["itinerary", "add", "1", "--day", "2", "Dest other"])
            .status
            .success()
    );
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
    let workspace = common::TestWorkspace::new();
    let dir = workspace.path();
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
}

#[test]
fn cli_fragment_apply_add_note_day_dry_run_preview() {
    let workspace = common::TestWorkspace::new();
    let dir = workspace.path();
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
}

#[test]
fn cli_fragment_apply_add_note_itinerary_dry_run_preview() {
    let workspace = common::TestWorkspace::new();
    let dir = workspace.path();
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
}

#[test]
fn cli_fragment_apply_add_note_required_decisions_invalid_without_db_write() {
    let workspace = common::TestWorkspace::new();
    let dir = workspace.path();
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
}

#[test]
fn cli_fragment_apply_add_note_trip_confirm_inserts_note() {
    let workspace = common::TestWorkspace::new();
    let dir = workspace.path();
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
}

#[test]
fn cli_fragment_apply_add_note_day_confirm_inserts_note() {
    let workspace = common::TestWorkspace::new();
    let dir = workspace.path();
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
}

#[test]
fn cli_fragment_apply_add_note_itinerary_confirm_inserts_note() {
    let workspace = common::TestWorkspace::new();
    let dir = workspace.path();
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
}

#[test]
fn cli_fragment_apply_add_note_confirm_required_decisions_block_db_write() {
    let workspace = common::TestWorkspace::new();
    let dir = workspace.path();
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
}

fn itinerary_expense_count(dir: &std::path::Path, itinerary_id: &str) -> usize {
    let output = run_cli_in(
        dir,
        &["expense", "list", "--itinerary", itinerary_id, "--json"],
    );
    assert!(output.status.success());
    let parsed: serde_json::Value =
        serde_json::from_str(&String::from_utf8_lossy(&output.stdout)).unwrap();
    parsed["expenses"]
        .as_array()
        .map(|items| items.len())
        .unwrap_or(0)
}

fn itinerary_reservation_count(dir: &std::path::Path, itinerary_id: &str) -> usize {
    let output = run_cli_in(
        dir,
        &["reservation", "list", "--itinerary", itinerary_id, "--json"],
    );
    assert!(output.status.success());
    let parsed: serde_json::Value =
        serde_json::from_str(&String::from_utf8_lossy(&output.stdout)).unwrap();
    parsed["reservations"]
        .as_array()
        .map(|items| items.len())
        .unwrap_or(0)
}

fn itinerary_estimate_count(dir: &std::path::Path, itinerary_id: &str) -> usize {
    let output = run_cli_in(
        dir,
        &["estimate", "list", "--itinerary", itinerary_id, "--json"],
    );
    assert!(output.status.success());
    let parsed: serde_json::Value =
        serde_json::from_str(&String::from_utf8_lossy(&output.stdout)).unwrap();
    parsed["estimates"]
        .as_array()
        .map(|items| items.len())
        .unwrap_or(0)
}

fn trip_estimate_count(dir: &std::path::Path, trip_id: &str) -> usize {
    let output = run_cli_in(dir, &["estimate", "list", "--trip", trip_id, "--json"]);
    assert!(output.status.success());
    let parsed: serde_json::Value =
        serde_json::from_str(&String::from_utf8_lossy(&output.stdout)).unwrap();
    parsed["estimates"]
        .as_array()
        .map(|items| items.len())
        .unwrap_or(0)
}

fn first_itinerary_id(dir: &std::path::Path) -> i64 {
    let output = run_cli_in(dir, &["itinerary", "list", "1", "--json"]);
    assert!(output.status.success());
    let items: Vec<serde_json::Value> =
        serde_json::from_str(&String::from_utf8_lossy(&output.stdout)).unwrap();
    items[0]["id"].as_i64().expect("itinerary id")
}

fn assert_add_estimate_invalid_dry_run(
    dir: &std::path::Path,
    fragment_path: &std::path::Path,
    preview_path: Option<&std::path::Path>,
    error_hint: &str,
) {
    let itinerary_id = first_itinerary_id(dir);
    let before_estimates = itinerary_estimate_count(dir, &itinerary_id.to_string());
    let before_expenses = itinerary_expense_count(dir, &itinerary_id.to_string());

    let mut args = vec![
        "fragment",
        "apply",
        fragment_path.to_str().unwrap(),
        "--dry-run",
        "--trip",
        "1",
        "--json",
    ];
    let output_path_owned;
    if let Some(preview_path) = preview_path {
        output_path_owned = preview_path.to_str().unwrap().to_string();
        args.push("--output");
        args.push(&output_path_owned);
    }

    let output = run_cli_in(dir, &args);
    assert!(!output.status.success());
    let parsed: serde_json::Value =
        serde_json::from_str(&String::from_utf8_lossy(&output.stdout)).expect("json report");
    assert_eq!(parsed["valid"], false);
    let errors = parsed["errors"]
        .as_array()
        .expect("errors array")
        .iter()
        .map(|value| value.as_str().unwrap_or_default())
        .collect::<Vec<_>>()
        .join(" ");
    assert!(
        errors.contains(error_hint),
        "expected error containing {error_hint:?}, got: {errors}"
    );
    if let Some(preview_path) = preview_path {
        assert!(!preview_path.exists());
    }
    assert_eq!(
        itinerary_estimate_count(dir, &itinerary_id.to_string()),
        before_estimates
    );
    assert_eq!(
        itinerary_expense_count(dir, &itinerary_id.to_string()),
        before_expenses
    );
}

fn assert_add_estimate_invalid_confirm(dir: &std::path::Path, fragment_path: &std::path::Path) {
    let itinerary_id = first_itinerary_id(dir);
    let before_estimates = itinerary_estimate_count(dir, &itinerary_id.to_string());
    let before_expenses = itinerary_expense_count(dir, &itinerary_id.to_string());

    let output = run_cli_in(
        dir,
        &[
            "fragment",
            "apply",
            fragment_path.to_str().unwrap(),
            "--confirm",
            "--trip",
            "1",
            "--json",
        ],
    );
    assert!(!output.status.success());
    let parsed: serde_json::Value =
        serde_json::from_str(&String::from_utf8_lossy(&output.stdout)).expect("json report");
    assert_eq!(parsed["valid"], false);
    assert!(parsed.get("inserted_estimate_id").is_none());
    assert_eq!(
        itinerary_estimate_count(dir, &itinerary_id.to_string()),
        before_estimates
    );
    assert_eq!(
        itinerary_expense_count(dir, &itinerary_id.to_string()),
        before_expenses
    );
}

#[test]
fn cli_fragment_apply_add_expense_itinerary_dry_run_preview_keeps_db_unchanged() {
    let workspace = common::TestWorkspace::new();
    let dir = workspace.path();
    let fragment = fixture_path("apply-add-expense-itinerary-fragment.json");
    let preview_path = dir.join("add-expense-preview.json");
    seed_trip_with_itinerary(&dir);

    let before = itinerary_expense_count(&dir, "1");
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
    assert_eq!(parsed["preview"]["action"], "add_expense");
    assert_eq!(parsed["preview"]["expenses_after"], 1);
    assert_eq!(parsed["preview"]["expense_preview"]["amount"], 500);
    assert_eq!(parsed["preview"]["expense_preview"]["currency"], "JPY");

    let preview_json = std::fs::read_to_string(&preview_path).expect("preview json");
    let export: serde_json::Value = serde_json::from_str(&preview_json).expect("parse preview");
    let day = export["days"]
        .as_array()
        .unwrap()
        .iter()
        .find(|day| day["day_number"] == 1)
        .expect("day 1");
    let itinerary = day["itineraries"]
        .as_array()
        .unwrap()
        .iter()
        .find(|item| item["title"] == "Morning temple")
        .expect("itinerary");
    let expenses = itinerary["expenses"].as_array().expect("expenses");
    assert_eq!(expenses.len(), 1);
    assert_eq!(expenses[0]["amount"], 500);
    assert_eq!(expenses[0]["currency"], "JPY");
    assert_eq!(expenses[0]["title"], "Temple admission");

    assert!(run_cli_in(
        &dir,
        &["trip", "validate-export", preview_path.to_str().unwrap()],
    )
    .status
    .success());

    assert_eq!(itinerary_expense_count(&dir, "1"), before);
}

#[test]
fn cli_fragment_apply_add_expense_invalid_currency_blocks_without_db_write() {
    let workspace = common::TestWorkspace::new();
    let dir = workspace.path();
    let fragment = fixture_path("apply-add-expense-invalid-currency-fragment.json");
    seed_trip_with_itinerary(&dir);

    let before = itinerary_expense_count(&dir, "1");
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
    assert_eq!(itinerary_expense_count(&dir, "1"), before);
}

#[test]
fn cli_fragment_apply_add_expense_trip_target_fails_without_db_write() {
    let workspace = common::TestWorkspace::new();
    let dir = workspace.path();
    let fragment = fixture_path("apply-add-expense-trip-target-fragment.json");
    seed_trip_with_itinerary(&dir);

    let before = itinerary_expense_count(&dir, "1");
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
    assert!(combined.contains("itinerary target"));
    assert_eq!(itinerary_expense_count(&dir, "1"), before);
}

#[test]
fn cli_fragment_apply_add_expense_required_decisions_invalid_without_db_write() {
    let workspace = common::TestWorkspace::new();
    let dir = workspace.path();
    let fragment = fixture_path("apply-add-expense-required-decisions-fragment.json");
    seed_trip_with_itinerary(&dir);

    let before = itinerary_expense_count(&dir, "1");
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
    assert_eq!(itinerary_expense_count(&dir, "1"), before);
}

#[test]
fn cli_fragment_apply_add_expense_itinerary_confirm_inserts_expense() {
    let workspace = common::TestWorkspace::new();
    let dir = workspace.path();
    let fragment = fixture_path("apply-add-expense-itinerary-fragment.json");
    seed_trip_with_itinerary(&dir);

    let before = itinerary_expense_count(&dir, "1");
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
    assert_eq!(parsed["preview"]["action"], "add_expense");
    let inserted_id = parsed["inserted_expense_id"]
        .as_i64()
        .expect("inserted_expense_id");

    let show = run_cli_in(
        &dir,
        &["expense", "show", &inserted_id.to_string(), "--json"],
    );
    assert!(show.status.success());
    let expense: serde_json::Value =
        serde_json::from_str(&String::from_utf8_lossy(&show.stdout)).unwrap();
    assert_eq!(expense["amount"], 500);
    assert_eq!(expense["currency"], "JPY");
    assert_eq!(expense["title"], "Temple admission");
    assert_eq!(expense["note"], "Cash only at gate.");

    assert_eq!(itinerary_expense_count(&dir, "1"), before + 1);
    let list = run_cli_in(&dir, &["expense", "list", "--itinerary", "1", "--json"]);
    assert!(list.status.success());
    let list_parsed: serde_json::Value =
        serde_json::from_str(&String::from_utf8_lossy(&list.stdout)).unwrap();
    let expenses = list_parsed["expenses"].as_array().expect("expenses");
    assert_eq!(expenses.len(), 1);
    assert_eq!(expenses[0]["id"], inserted_id);
}

#[test]
fn cli_fragment_apply_add_expense_confirm_required_decisions_block_db_write() {
    let workspace = common::TestWorkspace::new();
    let dir = workspace.path();
    let fragment = fixture_path("apply-add-expense-required-decisions-fragment.json");
    seed_trip_with_itinerary(&dir);

    let before = itinerary_expense_count(&dir, "1");
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
    assert_eq!(itinerary_expense_count(&dir, "1"), before);
}

#[test]
fn cli_fragment_apply_add_expense_trip_target_confirm_blocks_db_write() {
    let workspace = common::TestWorkspace::new();
    let dir = workspace.path();
    let fragment = fixture_path("apply-add-expense-trip-target-fragment.json");
    seed_trip_with_itinerary(&dir);

    let before = itinerary_expense_count(&dir, "1");
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
    assert_eq!(itinerary_expense_count(&dir, "1"), before);
}

#[test]
fn cli_fragment_apply_add_reservation_itinerary_dry_run_preview_keeps_db_unchanged() {
    let workspace = common::TestWorkspace::new();
    let dir = workspace.path();
    let fragment = fixture_path("apply-add-reservation-itinerary-fragment.json");
    let preview_path = dir.join("add-reservation-preview.json");
    seed_trip_with_itinerary(&dir);

    let before_reservations = itinerary_reservation_count(&dir, "1");
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
    assert_eq!(parsed["preview"]["action"], "add_reservation");
    assert_eq!(parsed["preview"]["reservations_after"], 1);
    assert_eq!(
        parsed["preview"]["reservation_preview"]["reservation_type"],
        "ticket"
    );
    assert_eq!(
        parsed["preview"]["reservation_preview"]["provider_name"],
        "Temple office"
    );
    assert_eq!(
        parsed["preview"]["reservation_preview"]["remark"],
        "Bring printed QR code."
    );

    let preview_json = std::fs::read_to_string(&preview_path).expect("preview json");
    let export: serde_json::Value = serde_json::from_str(&preview_json).expect("parse preview");
    let day = export["days"]
        .as_array()
        .unwrap()
        .iter()
        .find(|day| day["day_number"] == 1)
        .expect("day 1");
    let itinerary = day["itineraries"]
        .as_array()
        .unwrap()
        .iter()
        .find(|item| item["title"] == "Morning temple")
        .expect("itinerary");
    let reservations = itinerary["reservations"].as_array().expect("reservations");
    assert_eq!(reservations.len(), 1);
    assert_eq!(reservations[0]["reservation_type"], "ticket");
    assert_eq!(reservations[0]["provider_name"], "Temple office");
    assert_eq!(reservations[0]["confirmation_code"], "TCK-12345");
    assert_eq!(
        reservations[0]["reservation_site_url"],
        "https://example.invalid/reservation/TCK-12345"
    );
    assert_eq!(reservations[0]["remark"], "Bring printed QR code.");

    assert!(run_cli_in(
        &dir,
        &["trip", "validate-export", preview_path.to_str().unwrap()],
    )
    .status
    .success());

    assert_eq!(itinerary_reservation_count(&dir, "1"), before_reservations);
    assert_eq!(note_count(&dir, "1"), before_notes);
}

#[test]
fn cli_fragment_apply_add_reservation_itinerary_confirm_inserts_reservation() {
    let workspace = common::TestWorkspace::new();
    let dir = workspace.path();
    let fragment = fixture_path("apply-add-reservation-itinerary-fragment.json");
    seed_trip_with_itinerary(&dir);

    let before_reservations = itinerary_reservation_count(&dir, "1");
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
    assert_eq!(parsed["preview"]["action"], "add_reservation");
    let inserted_id = parsed["inserted_reservation_id"]
        .as_i64()
        .expect("inserted_reservation_id");

    assert_eq!(
        itinerary_reservation_count(&dir, "1"),
        before_reservations + 1
    );
    assert_eq!(note_count(&dir, "1"), before_notes);

    let show = run_cli_in(
        &dir,
        &["reservation", "show", &inserted_id.to_string(), "--json"],
    );
    assert!(show.status.success());
    let reservation: serde_json::Value =
        serde_json::from_str(&String::from_utf8_lossy(&show.stdout)).unwrap();
    assert_eq!(reservation["id"], inserted_id);
    assert_eq!(reservation["itinerary_id"], 1);
    assert_eq!(reservation["reservation_type"], "ticket");
    assert_eq!(reservation["provider_name"], "Temple office");
    assert_eq!(reservation["confirmation_code"], "TCK-12345");
    assert_eq!(
        reservation["reservation_site_url"],
        "https://example.invalid/reservation/TCK-12345"
    );
    assert_eq!(reservation["remark"], "Bring printed QR code.");
    assert_eq!(reservation["start_at"], "2026-05-01T09:00:00+09:00");
    assert_eq!(reservation["end_at"], "2026-05-01T10:00:00+09:00");

    let list = run_cli_in(&dir, &["reservation", "list", "--itinerary", "1", "--json"]);
    assert!(list.status.success());
    let list_parsed: serde_json::Value =
        serde_json::from_str(&String::from_utf8_lossy(&list.stdout)).unwrap();
    let reservations = list_parsed["reservations"]
        .as_array()
        .expect("reservations");
    assert_eq!(reservations.len(), before_reservations + 1);
    assert_eq!(reservations[0]["id"], inserted_id);
    assert_eq!(reservations[0]["remark"], "Bring printed QR code.");
}

#[test]
fn cli_fragment_apply_add_reservation_invalid_type_blocks_without_db_write() {
    let workspace = common::TestWorkspace::new();
    let dir = workspace.path();
    let fragment = fixture_path("apply-add-reservation-invalid-type-fragment.json");
    seed_trip_with_itinerary(&dir);

    let before = itinerary_reservation_count(&dir, "1");
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
    assert_eq!(itinerary_reservation_count(&dir, "1"), before);
}

#[test]
fn cli_fragment_apply_add_reservation_invalid_type_confirm_blocks_db_write() {
    let workspace = common::TestWorkspace::new();
    let dir = workspace.path();
    let fragment = fixture_path("apply-add-reservation-invalid-type-fragment.json");
    seed_trip_with_itinerary(&dir);

    let before = itinerary_reservation_count(&dir, "1");
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
    assert!(!output.status.success());
    assert_eq!(itinerary_reservation_count(&dir, "1"), before);
}

#[test]
fn cli_fragment_apply_add_reservation_trip_target_fails_without_db_write() {
    let workspace = common::TestWorkspace::new();
    let dir = workspace.path();
    let fragment = fixture_path("apply-add-reservation-trip-target-fragment.json");
    seed_trip_with_itinerary(&dir);

    let before = itinerary_reservation_count(&dir, "1");
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
    assert_eq!(itinerary_reservation_count(&dir, "1"), before);
}

#[test]
fn cli_fragment_apply_add_reservation_trip_target_confirm_blocks_db_write() {
    let workspace = common::TestWorkspace::new();
    let dir = workspace.path();
    let fragment = fixture_path("apply-add-reservation-trip-target-fragment.json");
    seed_trip_with_itinerary(&dir);

    let before = itinerary_reservation_count(&dir, "1");
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
    assert!(!output.status.success());
    assert_eq!(itinerary_reservation_count(&dir, "1"), before);
}

#[test]
fn cli_fragment_apply_add_reservation_required_decisions_blocks_without_db_write() {
    let workspace = common::TestWorkspace::new();
    let dir = workspace.path();
    let fragment = fixture_path("apply-add-reservation-required-decisions-fragment.json");
    seed_trip_with_itinerary(&dir);

    let before = itinerary_reservation_count(&dir, "1");
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
    assert_eq!(itinerary_reservation_count(&dir, "1"), before);
}

#[test]
fn cli_fragment_apply_add_reservation_required_decisions_confirm_blocks_db_write() {
    let workspace = common::TestWorkspace::new();
    let dir = workspace.path();
    let fragment = fixture_path("apply-add-reservation-required-decisions-fragment.json");
    seed_trip_with_itinerary(&dir);

    let before = itinerary_reservation_count(&dir, "1");
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
    assert!(!output.status.success());
    assert_eq!(itinerary_reservation_count(&dir, "1"), before);
}

#[test]
fn cli_fragment_apply_add_reservation_missing_provider_confirm_blocks_db_write() {
    let workspace = common::TestWorkspace::new();
    let dir = workspace.path();
    seed_trip_with_itinerary(&dir);
    let fragment = dir.join("add-reservation-missing-provider.json");
    std::fs::write(
        &fragment,
        r#"{
  "metadata": {
    "fragment_id": "frag-add-reservation-missing-provider",
    "created_at": "2026-03-15T14:00:00Z",
    "source": "manual",
    "provider": "fixture"
  },
  "target": {
    "target_type": "itinerary",
    "day_reference": 1,
    "itinerary_reference": "Morning temple"
  },
  "fragment": {
    "intent": "add_reservation",
    "candidate_content": {
      "reservation_type": "ticket"
    }
  },
  "adoption_hints": {
    "required_decisions": []
  }
}"#,
    )
    .expect("write fragment");

    let before = itinerary_reservation_count(&dir, "1");
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
    assert!(!output.status.success());
    assert_eq!(itinerary_reservation_count(&dir, "1"), before);
}

#[test]
fn cli_fragment_apply_add_reservation_memo_confirm_maps_to_remark_only() {
    let workspace = common::TestWorkspace::new();
    let dir = workspace.path();
    seed_trip_with_itinerary(&dir);
    let fragment = dir.join("add-reservation-memo.json");
    std::fs::write(
        &fragment,
        r#"{
  "metadata": {
    "fragment_id": "frag-add-reservation-memo",
    "created_at": "2026-03-15T14:00:00Z",
    "source": "manual",
    "provider": "fixture"
  },
  "target": {
    "target_type": "itinerary",
    "day_reference": 1,
    "itinerary_reference": "Morning temple"
  },
  "fragment": {
    "intent": "add_reservation",
    "candidate_content": {
      "reservation_type": "activity",
      "provider_name": "Garden desk",
      "memo": "Ask for the east gate entrance."
    }
  },
  "adoption_hints": {
    "required_decisions": []
  }
}"#,
    )
    .expect("write fragment");

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
    let inserted_id = parsed["inserted_reservation_id"]
        .as_i64()
        .expect("inserted_reservation_id");

    let show = run_cli_in(
        &dir,
        &["reservation", "show", &inserted_id.to_string(), "--json"],
    );
    assert!(show.status.success());
    let reservation: serde_json::Value =
        serde_json::from_str(&String::from_utf8_lossy(&show.stdout)).unwrap();
    assert_eq!(reservation["remark"], "Ask for the east gate entrance.");
    assert_eq!(note_count(&dir, "1"), before_notes);
}

#[test]
fn cli_fragment_apply_add_estimate_itinerary_dry_run_preview_keeps_db_unchanged() {
    let workspace = common::TestWorkspace::new();
    let dir = workspace.path();
    let fragment = fixture_path("apply-add-estimate-valid.json");
    let preview_path = dir.join("add-estimate-preview.json");
    seed_trip_with_itinerary(&dir);
    let itinerary_id = first_itinerary_id(&dir);

    let before_estimates = itinerary_estimate_count(&dir, &itinerary_id.to_string());
    let before_expenses = itinerary_expense_count(&dir, &itinerary_id.to_string());
    let before_trip_estimates = trip_estimate_count(&dir, "1");
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
    assert_eq!(parsed["preview"]["action"], "add_estimate");
    assert_eq!(parsed["preview"]["estimates_before"], before_trip_estimates);
    assert_eq!(
        parsed["preview"]["estimates_after"],
        before_trip_estimates + 1
    );
    assert_eq!(parsed["preview"]["estimate_preview"]["amount"], 14000);
    assert_eq!(parsed["preview"]["estimate_preview"]["currency"], "JPY");
    assert_eq!(
        parsed["preview"]["estimate_preview"]["title"],
        "Aquarium tickets"
    );
    assert_eq!(
        parsed["preview"]["estimate_preview"]["note"],
        "Estimated total for five people"
    );
    assert_eq!(parsed["preview"]["estimate_preview"]["sort_order"], 0);
    assert_eq!(
        parsed["preview"]["estimate_preview"]["target_itinerary_id"],
        itinerary_id
    );
    assert_eq!(
        parsed["preview"]["estimate_preview"]["target_itinerary_title"],
        "Morning temple"
    );
    assert!(parsed["preview"]["expense_preview"].is_null());

    let preview_json = std::fs::read_to_string(&preview_path).expect("preview json");
    let export: serde_json::Value = serde_json::from_str(&preview_json).expect("parse preview");
    let day = export["days"]
        .as_array()
        .unwrap()
        .iter()
        .find(|day| day["day_number"] == 1)
        .expect("day 1");
    let itinerary = day["itineraries"]
        .as_array()
        .unwrap()
        .iter()
        .find(|item| item["title"] == "Morning temple")
        .expect("itinerary");
    let estimates = itinerary["estimates"].as_array().expect("estimates");
    assert_eq!(estimates.len(), 1);
    assert_eq!(estimates[0]["amount"], 14000);
    assert_eq!(estimates[0]["currency"], "JPY");
    assert_eq!(estimates[0]["title"], "Aquarium tickets");
    assert_eq!(estimates[0]["note"], "Estimated total for five people");
    assert_eq!(estimates[0]["sort_order"], 0);
    assert!(itinerary["expenses"].as_array().unwrap().is_empty());
    assert!(parsed.get("inserted_estimate_id").is_none());

    assert!(run_cli_in(
        &dir,
        &["trip", "validate-export", preview_path.to_str().unwrap()],
    )
    .status
    .success());

    assert_eq!(
        itinerary_estimate_count(&dir, &itinerary_id.to_string()),
        before_estimates
    );
    assert_eq!(
        itinerary_expense_count(&dir, &itinerary_id.to_string()),
        before_expenses
    );
}

#[test]
fn cli_fragment_apply_add_estimate_minor_units_dry_run_normalizes_amount() {
    let workspace = common::TestWorkspace::new();
    let dir = workspace.path();
    let fragment = fixture_path("apply-add-estimate-valid-minor-units.json");
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
            "--json",
        ],
    );
    assert!(output.status.success());
    let parsed: serde_json::Value =
        serde_json::from_str(&String::from_utf8_lossy(&output.stdout)).unwrap();
    assert_eq!(parsed["preview"]["estimate_preview"]["amount"], 1250);
    assert_eq!(parsed["preview"]["estimate_preview"]["currency"], "USD");
    assert_eq!(parsed["preview"]["estimate_preview"]["sort_order"], 0);
}

#[test]
fn cli_fragment_apply_add_estimate_missing_amount_blocks_without_db_write() {
    let workspace = common::TestWorkspace::new();
    let dir = workspace.path();
    let fragment = fixture_path("apply-add-estimate-missing-amount.json");
    seed_trip_with_itinerary(&dir);
    let itinerary_id = first_itinerary_id(&dir);
    let before = itinerary_estimate_count(&dir, &itinerary_id.to_string());

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
    assert_eq!(
        itinerary_estimate_count(&dir, &itinerary_id.to_string()),
        before
    );
}

#[test]
fn cli_fragment_apply_add_estimate_invalid_currency_blocks_without_db_write() {
    let workspace = common::TestWorkspace::new();
    let dir = workspace.path();
    let fragment = fixture_path("apply-add-estimate-invalid-currency.json");
    seed_trip_with_itinerary(&dir);
    let itinerary_id = first_itinerary_id(&dir);
    let before = itinerary_estimate_count(&dir, &itinerary_id.to_string());

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
    assert_eq!(
        itinerary_estimate_count(&dir, &itinerary_id.to_string()),
        before
    );
}

#[test]
fn cli_fragment_apply_add_estimate_negative_amount_blocks_without_db_write() {
    let workspace = common::TestWorkspace::new();
    let dir = workspace.path();
    let fragment = fixture_path("apply-add-estimate-negative-amount.json");
    seed_trip_with_itinerary(&dir);
    let itinerary_id = first_itinerary_id(&dir);
    let before = itinerary_estimate_count(&dir, &itinerary_id.to_string());

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
    assert_eq!(
        itinerary_estimate_count(&dir, &itinerary_id.to_string()),
        before
    );
}

#[test]
fn cli_fragment_apply_add_estimate_invalid_sort_order_blocks_without_db_write() {
    let workspace = common::TestWorkspace::new();
    let dir = workspace.path();
    let fragment = fixture_path("apply-add-estimate-invalid-sort-order.json");
    seed_trip_with_itinerary(&dir);
    let itinerary_id = first_itinerary_id(&dir);
    let before = itinerary_estimate_count(&dir, &itinerary_id.to_string());

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
    assert_eq!(
        itinerary_estimate_count(&dir, &itinerary_id.to_string()),
        before
    );
}

#[test]
fn cli_fragment_apply_add_estimate_unsupported_target_blocks_without_db_write() {
    let workspace = common::TestWorkspace::new();
    let dir = workspace.path();
    let fragment = fixture_path("apply-add-estimate-unsupported-target.json");
    seed_trip_with_itinerary(&dir);
    let itinerary_id = first_itinerary_id(&dir);
    let before = itinerary_estimate_count(&dir, &itinerary_id.to_string());

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
    assert!(combined.contains("itinerary target"));
    assert_eq!(
        itinerary_estimate_count(&dir, &itinerary_id.to_string()),
        before
    );
}

#[test]
fn cli_fragment_apply_add_estimate_required_decisions_invalid_without_db_write() {
    let workspace = common::TestWorkspace::new();
    let dir = workspace.path();
    let fragment = fixture_path("apply-add-estimate-required-decisions.json");
    let preview_path = dir.join("add-estimate-required-decisions-preview.json");
    seed_trip_with_itinerary(&dir);
    let itinerary_id = first_itinerary_id(&dir);
    let before = itinerary_estimate_count(&dir, &itinerary_id.to_string());

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
            "--json",
        ],
    );
    assert!(!output.status.success());
    assert!(!preview_path.exists());
    assert_eq!(
        itinerary_estimate_count(&dir, &itinerary_id.to_string()),
        before
    );
}

#[test]
fn cli_fragment_apply_add_estimate_itinerary_confirm_inserts_estimate() {
    let workspace = common::TestWorkspace::new();
    let dir = workspace.path();
    let fragment = fixture_path("apply-add-estimate-valid.json");
    seed_trip_with_itinerary(&dir);
    let itinerary_id = first_itinerary_id(&dir);

    let before_estimates = itinerary_estimate_count(&dir, &itinerary_id.to_string());
    let before_expenses = itinerary_expense_count(&dir, &itinerary_id.to_string());
    let before_trip_estimates = trip_estimate_count(&dir, "1");

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
    assert_eq!(parsed["confirm"], true);
    assert_eq!(parsed["valid"], true);
    assert_eq!(parsed["preview"]["action"], "add_estimate");
    let inserted_id = parsed["inserted_estimate_id"]
        .as_i64()
        .expect("inserted_estimate_id");

    let show = run_cli_in(
        &dir,
        &["estimate", "show", &inserted_id.to_string(), "--json"],
    );
    assert!(show.status.success());
    let estimate: serde_json::Value =
        serde_json::from_str(&String::from_utf8_lossy(&show.stdout)).unwrap();
    assert_eq!(estimate["itinerary_id"], itinerary_id);
    assert_eq!(estimate["amount"], 14000);
    assert_eq!(estimate["currency"], "JPY");
    assert_eq!(estimate["title"], "Aquarium tickets");
    assert_eq!(estimate["note"], "Estimated total for five people");
    assert_eq!(estimate["sort_order"], 0);

    assert_eq!(
        itinerary_estimate_count(&dir, &itinerary_id.to_string()),
        before_estimates + 1
    );
    assert_eq!(
        itinerary_expense_count(&dir, &itinerary_id.to_string()),
        before_expenses
    );
    assert_eq!(trip_estimate_count(&dir, "1"), before_trip_estimates + 1);

    let list = run_cli_in(
        &dir,
        &[
            "estimate",
            "list",
            "--itinerary",
            &itinerary_id.to_string(),
            "--json",
        ],
    );
    assert!(list.status.success());
    let list_parsed: serde_json::Value =
        serde_json::from_str(&String::from_utf8_lossy(&list.stdout)).unwrap();
    let estimates = list_parsed["estimates"].as_array().expect("estimates");
    assert_eq!(estimates.len(), 1);
    assert_eq!(estimates[0]["id"], inserted_id);
}

#[test]
fn cli_fragment_apply_add_estimate_minor_units_confirm_stores_1250_not_125000() {
    let workspace = common::TestWorkspace::new();
    let dir = workspace.path();
    let fragment = fixture_path("apply-add-estimate-valid-minor-units.json");
    seed_trip_with_itinerary(&dir);
    let itinerary_id = first_itinerary_id(&dir);
    let before_estimates = itinerary_estimate_count(&dir, &itinerary_id.to_string());
    let before_expenses = itinerary_expense_count(&dir, &itinerary_id.to_string());

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
    let inserted_id = parsed["inserted_estimate_id"]
        .as_i64()
        .expect("inserted_estimate_id");

    let show = run_cli_in(
        &dir,
        &["estimate", "show", &inserted_id.to_string(), "--json"],
    );
    assert!(show.status.success());
    let estimate: serde_json::Value =
        serde_json::from_str(&String::from_utf8_lossy(&show.stdout)).unwrap();
    assert_eq!(estimate["amount"], 1250);
    assert_ne!(estimate["amount"].as_i64().unwrap(), 125000);
    assert_eq!(estimate["currency"], "USD");
    assert_eq!(estimate["title"], "Admission estimate");
    assert_eq!(estimate["sort_order"], 0);

    assert_eq!(
        itinerary_estimate_count(&dir, &itinerary_id.to_string()),
        before_estimates + 1
    );
    assert_eq!(
        itinerary_expense_count(&dir, &itinerary_id.to_string()),
        before_expenses
    );
}

#[test]
fn cli_fragment_apply_add_estimate_itinerary_id_selector_confirm_succeeds() {
    let workspace = common::TestWorkspace::new();
    let dir = workspace.path();
    seed_trip_with_itinerary(&dir);
    let itinerary_id = first_itinerary_id(&dir);

    let fragment_path = dir.join("add-estimate-id-selector-confirm.json");
    std::fs::write(
        &fragment_path,
        format!(
            r#"{{
  "target": {{
    "target_type": "itinerary",
    "day_reference": 1,
    "itinerary_reference": {itinerary_id}
  }},
  "fragment": {{
    "intent": "add_estimate",
    "candidate_content": {{
      "amount": "2500",
      "currency": "JPY"
    }}
  }},
  "adoption_hints": {{ "required_decisions": [] }}
}}"#
        ),
    )
    .unwrap();

    let before = itinerary_estimate_count(&dir, &itinerary_id.to_string());
    let output = run_cli_in(
        &dir,
        &[
            "fragment",
            "apply",
            fragment_path.to_str().unwrap(),
            "--confirm",
            "--trip",
            "1",
            "--json",
        ],
    );
    assert!(output.status.success());
    let parsed: serde_json::Value =
        serde_json::from_str(&String::from_utf8_lossy(&output.stdout)).unwrap();
    let inserted_id = parsed["inserted_estimate_id"].as_i64().unwrap();
    let show = run_cli_in(
        &dir,
        &["estimate", "show", &inserted_id.to_string(), "--json"],
    );
    assert!(show.status.success());
    let estimate: serde_json::Value =
        serde_json::from_str(&String::from_utf8_lossy(&show.stdout)).unwrap();
    assert_eq!(estimate["itinerary_id"], itinerary_id);
    assert_eq!(estimate["amount"], 2500);
    assert_eq!(
        itinerary_estimate_count(&dir, &itinerary_id.to_string()),
        before + 1
    );
}

#[test]
fn cli_fragment_apply_add_estimate_minimal_optional_fields_confirm_succeeds() {
    let workspace = common::TestWorkspace::new();
    let dir = workspace.path();
    seed_trip_with_itinerary(&dir);
    let itinerary_id = first_itinerary_id(&dir);

    let fragment_path = dir.join("add-estimate-minimal-confirm.json");
    std::fs::write(
        &fragment_path,
        r#"{
  "target": {
    "target_type": "itinerary",
    "day_reference": 1,
    "itinerary_reference": "Morning temple"
  },
  "fragment": {
    "intent": "add_estimate",
    "candidate_content": {
      "amount": "3000",
      "currency": "JPY"
    }
  },
  "adoption_hints": { "required_decisions": [] }
}"#,
    )
    .unwrap();

    let output = run_cli_in(
        &dir,
        &[
            "fragment",
            "apply",
            fragment_path.to_str().unwrap(),
            "--confirm",
            "--trip",
            "1",
            "--json",
        ],
    );
    assert!(output.status.success());
    let parsed: serde_json::Value =
        serde_json::from_str(&String::from_utf8_lossy(&output.stdout)).unwrap();
    let inserted_id = parsed["inserted_estimate_id"].as_i64().unwrap();
    let show = run_cli_in(
        &dir,
        &["estimate", "show", &inserted_id.to_string(), "--json"],
    );
    assert!(show.status.success());
    let estimate: serde_json::Value =
        serde_json::from_str(&String::from_utf8_lossy(&show.stdout)).unwrap();
    assert_eq!(estimate["itinerary_id"], itinerary_id);
    assert_eq!(estimate["amount"], 3000);
    assert!(estimate["title"].is_null());
    assert!(estimate["note"].is_null());
    assert_eq!(estimate["sort_order"], 0);
}

#[test]
fn cli_fragment_apply_add_estimate_missing_amount_confirm_blocks_without_db_write() {
    let workspace = common::TestWorkspace::new();
    let dir = workspace.path();
    let fragment = fixture_path("apply-add-estimate-missing-amount.json");
    seed_trip_with_itinerary(&dir);
    assert_add_estimate_invalid_confirm(&dir, &fragment);
}

#[test]
fn cli_fragment_apply_add_estimate_negative_amount_confirm_blocks_without_db_write() {
    let workspace = common::TestWorkspace::new();
    let dir = workspace.path();
    let fragment = fixture_path("apply-add-estimate-negative-amount.json");
    seed_trip_with_itinerary(&dir);
    assert_add_estimate_invalid_confirm(&dir, &fragment);
}

#[test]
fn cli_fragment_apply_add_estimate_invalid_currency_confirm_blocks_without_db_write() {
    let workspace = common::TestWorkspace::new();
    let dir = workspace.path();
    let fragment = fixture_path("apply-add-estimate-invalid-currency.json");
    seed_trip_with_itinerary(&dir);
    assert_add_estimate_invalid_confirm(&dir, &fragment);
}

#[test]
fn cli_fragment_apply_add_estimate_invalid_sort_order_confirm_blocks_without_db_write() {
    let workspace = common::TestWorkspace::new();
    let dir = workspace.path();
    let fragment = fixture_path("apply-add-estimate-invalid-sort-order.json");
    seed_trip_with_itinerary(&dir);
    assert_add_estimate_invalid_confirm(&dir, &fragment);
}

#[test]
fn cli_fragment_apply_add_estimate_required_decisions_confirm_blocks_without_db_write() {
    let workspace = common::TestWorkspace::new();
    let dir = workspace.path();
    let fragment = fixture_path("apply-add-estimate-required-decisions.json");
    seed_trip_with_itinerary(&dir);
    assert_add_estimate_invalid_confirm(&dir, &fragment);
}

#[test]
fn cli_fragment_apply_add_estimate_unsupported_target_confirm_blocks_without_db_write() {
    let workspace = common::TestWorkspace::new();
    let dir = workspace.path();
    let fragment = fixture_path("apply-add-estimate-unsupported-target.json");
    seed_trip_with_itinerary(&dir);
    assert_add_estimate_invalid_confirm(&dir, &fragment);
}

#[test]
fn cli_fragment_apply_add_estimate_ambiguous_target_confirm_blocks_without_db_write() {
    let workspace = common::TestWorkspace::new();
    let dir = workspace.path();
    seed_trip_with_itinerary(&dir);
    assert!(run_cli_in(
        &dir,
        &["itinerary", "add", "1", "--day", "1", "Morning temple"],
    )
    .status
    .success());

    let fragment_path = dir.join("ambiguous-title-estimate-confirm.json");
    std::fs::write(
        &fragment_path,
        r#"{
  "target": {
    "target_type": "itinerary",
    "day_reference": 1,
    "itinerary_reference": "Morning temple"
  },
  "fragment": {
    "intent": "add_estimate",
    "candidate_content": {
      "amount": "1000",
      "currency": "JPY"
    }
  },
  "adoption_hints": { "required_decisions": [] }
}"#,
    )
    .unwrap();
    assert_add_estimate_invalid_confirm(&dir, &fragment_path);
}

#[test]
fn cli_fragment_apply_add_estimate_invalid_amount_confirm_blocks_without_db_write() {
    let workspace = common::TestWorkspace::new();
    let dir = workspace.path();
    seed_trip_with_itinerary(&dir);
    let fragment_path = dir.join("add-estimate-invalid-amount-confirm.json");
    std::fs::write(
        &fragment_path,
        r#"{
  "target": {
    "target_type": "itinerary",
    "day_reference": 1,
    "itinerary_reference": "Morning temple"
  },
  "fragment": {
    "intent": "add_estimate",
    "candidate_content": {
      "amount": "not-a-number",
      "currency": "JPY"
    }
  },
  "adoption_hints": { "required_decisions": [] }
}"#,
    )
    .unwrap();
    assert_add_estimate_invalid_confirm(&dir, &fragment_path);
}

#[test]
fn cli_fragment_apply_add_estimate_ambiguous_title_target_blocks_without_db_write() {
    let workspace = common::TestWorkspace::new();
    let dir = workspace.path();
    seed_trip_with_itinerary(&dir);
    assert!(run_cli_in(
        &dir,
        &["itinerary", "add", "1", "--day", "1", "Morning temple"],
    )
    .status
    .success());

    let fragment_path = dir.join("ambiguous-title-estimate.json");
    std::fs::write(
        &fragment_path,
        r#"{
  "target": {
    "target_type": "itinerary",
    "day_reference": 1,
    "itinerary_reference": "Morning temple"
  },
  "fragment": {
    "intent": "add_estimate",
    "candidate_content": {
      "amount": "1000",
      "currency": "JPY"
    }
  },
  "adoption_hints": { "required_decisions": [] }
}"#,
    )
    .unwrap();

    let itinerary_id = first_itinerary_id(&dir);
    let before = itinerary_estimate_count(&dir, &itinerary_id.to_string());
    let output = run_cli_in(
        &dir,
        &[
            "fragment",
            "apply",
            fragment_path.to_str().unwrap(),
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
    assert!(combined.contains("曖昧"));
    assert_eq!(
        itinerary_estimate_count(&dir, &itinerary_id.to_string()),
        before
    );
}

#[test]
fn cli_fragment_apply_add_estimate_itinerary_id_selector_dry_run_succeeds() {
    let workspace = common::TestWorkspace::new();
    let dir = workspace.path();
    seed_trip_with_itinerary(&dir);
    let itinerary_id = first_itinerary_id(&dir);

    let fragment_path = dir.join("add-estimate-id-selector.json");
    std::fs::write(
        &fragment_path,
        format!(
            r#"{{
  "target": {{
    "target_type": "itinerary",
    "day_reference": 1,
    "itinerary_reference": {itinerary_id}
  }},
  "fragment": {{
    "intent": "add_estimate",
    "candidate_content": {{
      "amount": "2500",
      "currency": "JPY"
    }}
  }},
  "adoption_hints": {{ "required_decisions": [] }}
}}"#
        ),
    )
    .unwrap();

    let output = run_cli_in(
        &dir,
        &[
            "fragment",
            "apply",
            fragment_path.to_str().unwrap(),
            "--dry-run",
            "--trip",
            "1",
            "--json",
        ],
    );
    assert!(output.status.success());
    let parsed: serde_json::Value =
        serde_json::from_str(&String::from_utf8_lossy(&output.stdout)).unwrap();
    assert_eq!(
        parsed["preview"]["estimate_preview"]["target_itinerary_id"],
        itinerary_id
    );
}

#[test]
fn cli_fragment_apply_add_estimate_minimal_optional_fields_dry_run_succeeds() {
    let workspace = common::TestWorkspace::new();
    let dir = workspace.path();
    seed_trip_with_itinerary(&dir);

    let fragment_path = dir.join("add-estimate-minimal.json");
    std::fs::write(
        &fragment_path,
        r#"{
  "target": {
    "target_type": "itinerary",
    "day_reference": 1,
    "itinerary_reference": "Morning temple"
  },
  "fragment": {
    "intent": "add_estimate",
    "candidate_content": {
      "amount": "3000",
      "currency": "JPY"
    }
  },
  "adoption_hints": { "required_decisions": [] }
}"#,
    )
    .unwrap();

    let output = run_cli_in(
        &dir,
        &[
            "fragment",
            "apply",
            fragment_path.to_str().unwrap(),
            "--dry-run",
            "--trip",
            "1",
            "--json",
        ],
    );
    assert!(output.status.success());
    let parsed: serde_json::Value =
        serde_json::from_str(&String::from_utf8_lossy(&output.stdout)).unwrap();
    assert_eq!(parsed["preview"]["estimate_preview"]["amount"], 3000);
    assert!(parsed["preview"]["estimate_preview"]["title"].is_null());
    assert!(parsed["preview"]["estimate_preview"]["note"].is_null());
    assert_eq!(parsed["preview"]["estimate_preview"]["sort_order"], 0);
}

#[test]
fn cli_fragment_apply_add_estimate_empty_optional_text_normalizes_to_null() {
    let workspace = common::TestWorkspace::new();
    let dir = workspace.path();
    seed_trip_with_itinerary(&dir);

    let fragment_path = dir.join("add-estimate-empty-optional.json");
    std::fs::write(
        &fragment_path,
        r#"{
  "target": {
    "target_type": "itinerary",
    "day_reference": 1,
    "itinerary_reference": "Morning temple"
  },
  "fragment": {
    "intent": "add_estimate",
    "candidate_content": {
      "amount": "3000",
      "currency": "JPY",
      "title": "",
      "note": "   "
    }
  },
  "adoption_hints": { "required_decisions": [] }
}"#,
    )
    .unwrap();

    let output = run_cli_in(
        &dir,
        &[
            "fragment",
            "apply",
            fragment_path.to_str().unwrap(),
            "--dry-run",
            "--trip",
            "1",
            "--json",
        ],
    );
    assert!(output.status.success());
    let parsed: serde_json::Value =
        serde_json::from_str(&String::from_utf8_lossy(&output.stdout)).unwrap();
    assert!(parsed["preview"]["estimate_preview"]["title"].is_null());
    assert!(parsed["preview"]["estimate_preview"]["note"].is_null());
}

#[test]
fn cli_fragment_apply_add_estimate_lowercase_currency_normalizes() {
    let workspace = common::TestWorkspace::new();
    let dir = workspace.path();
    seed_trip_with_itinerary(&dir);

    let fragment_path = dir.join("add-estimate-lowercase-currency.json");
    std::fs::write(
        &fragment_path,
        r#"{
  "target": {
    "target_type": "itinerary",
    "day_reference": 1,
    "itinerary_reference": "Morning temple"
  },
  "fragment": {
    "intent": "add_estimate",
    "candidate_content": {
      "amount": "1000",
      "currency": "jpy"
    }
  },
  "adoption_hints": { "required_decisions": [] }
}"#,
    )
    .unwrap();

    let output = run_cli_in(
        &dir,
        &[
            "fragment",
            "apply",
            fragment_path.to_str().unwrap(),
            "--dry-run",
            "--trip",
            "1",
            "--json",
        ],
    );
    assert!(output.status.success());
    let parsed: serde_json::Value =
        serde_json::from_str(&String::from_utf8_lossy(&output.stdout)).unwrap();
    assert_eq!(parsed["preview"]["estimate_preview"]["currency"], "JPY");
}

#[test]
fn cli_fragment_apply_add_estimate_non_numeric_amount_blocks_without_db_write() {
    let workspace = common::TestWorkspace::new();
    let dir = workspace.path();
    seed_trip_with_itinerary(&dir);
    let fragment_path = dir.join("add-estimate-non-numeric-amount.json");
    std::fs::write(
        &fragment_path,
        r#"{
  "target": { "target_type": "itinerary", "day_reference": 1, "itinerary_reference": "Morning temple" },
  "fragment": {
    "intent": "add_estimate",
    "candidate_content": { "amount": "abc", "currency": "JPY" }
  },
  "adoption_hints": { "required_decisions": [] }
}"#,
    )
    .unwrap();
    assert_add_estimate_invalid_dry_run(
        &dir,
        &fragment_path,
        Some(&dir.join("preview.json")),
        "amount",
    );
}

#[test]
fn cli_fragment_apply_add_estimate_excessive_decimal_precision_blocks_without_db_write() {
    let workspace = common::TestWorkspace::new();
    let dir = workspace.path();
    seed_trip_with_itinerary(&dir);
    let fragment_path = dir.join("add-estimate-excessive-decimal.json");
    std::fs::write(
        &fragment_path,
        r#"{
  "target": { "target_type": "itinerary", "day_reference": 1, "itinerary_reference": "Morning temple" },
  "fragment": {
    "intent": "add_estimate",
    "candidate_content": { "amount": "12.555", "currency": "USD" }
  },
  "adoption_hints": { "required_decisions": [] }
}"#,
    )
    .unwrap();
    assert_add_estimate_invalid_dry_run(
        &dir,
        &fragment_path,
        Some(&dir.join("preview.json")),
        "小数桁",
    );
}

#[test]
fn cli_fragment_apply_add_estimate_negative_string_amount_blocks_without_db_write() {
    let workspace = common::TestWorkspace::new();
    let dir = workspace.path();
    seed_trip_with_itinerary(&dir);
    let fragment_path = dir.join("add-estimate-negative-string-amount.json");
    std::fs::write(
        &fragment_path,
        r#"{
  "target": { "target_type": "itinerary", "day_reference": 1, "itinerary_reference": "Morning temple" },
  "fragment": {
    "intent": "add_estimate",
    "candidate_content": { "amount": "-100", "currency": "JPY" }
  },
  "adoption_hints": { "required_decisions": [] }
}"#,
    )
    .unwrap();
    assert_add_estimate_invalid_dry_run(
        &dir,
        &fragment_path,
        Some(&dir.join("preview.json")),
        "0 以上",
    );
}

#[test]
fn cli_fragment_apply_add_estimate_missing_currency_blocks_without_db_write() {
    let workspace = common::TestWorkspace::new();
    let dir = workspace.path();
    seed_trip_with_itinerary(&dir);
    let fragment_path = dir.join("add-estimate-missing-currency.json");
    std::fs::write(
        &fragment_path,
        r#"{
  "target": { "target_type": "itinerary", "day_reference": 1, "itinerary_reference": "Morning temple" },
  "fragment": {
    "intent": "add_estimate",
    "candidate_content": { "amount": "1000" }
  },
  "adoption_hints": { "required_decisions": [] }
}"#,
    )
    .unwrap();
    assert_add_estimate_invalid_dry_run(
        &dir,
        &fragment_path,
        Some(&dir.join("preview.json")),
        "fragment body が空に近い",
    );
}

#[test]
fn cli_fragment_apply_add_estimate_invalid_currency_type_blocks_without_db_write() {
    let workspace = common::TestWorkspace::new();
    let dir = workspace.path();
    seed_trip_with_itinerary(&dir);
    let fragment_path = dir.join("add-estimate-invalid-currency-type.json");
    std::fs::write(
        &fragment_path,
        r#"{
  "target": { "target_type": "itinerary", "day_reference": 1, "itinerary_reference": "Morning temple" },
  "fragment": {
    "intent": "add_estimate",
    "candidate_content": { "amount": "1000", "currency": 840 }
  },
  "adoption_hints": { "required_decisions": [] }
}"#,
    )
    .unwrap();
    assert_add_estimate_invalid_dry_run(
        &dir,
        &fragment_path,
        Some(&dir.join("preview.json")),
        "fragment body が空に近い",
    );
}

#[test]
fn cli_fragment_apply_add_estimate_invalid_currency_format_blocks_without_db_write() {
    let workspace = common::TestWorkspace::new();
    let dir = workspace.path();
    seed_trip_with_itinerary(&dir);
    let fragment_path = dir.join("add-estimate-invalid-currency-format.json");
    std::fs::write(
        &fragment_path,
        r#"{
  "target": { "target_type": "itinerary", "day_reference": 1, "itinerary_reference": "Morning temple" },
  "fragment": {
    "intent": "add_estimate",
    "candidate_content": { "amount": "1000", "currency": "12" }
  },
  "adoption_hints": { "required_decisions": [] }
}"#,
    )
    .unwrap();
    assert_add_estimate_invalid_dry_run(
        &dir,
        &fragment_path,
        Some(&dir.join("preview.json")),
        "currency",
    );
}

#[test]
fn cli_fragment_apply_add_estimate_non_string_title_blocks_without_db_write() {
    let workspace = common::TestWorkspace::new();
    let dir = workspace.path();
    seed_trip_with_itinerary(&dir);
    let fragment_path = dir.join("add-estimate-non-string-title.json");
    std::fs::write(
        &fragment_path,
        r#"{
  "target": { "target_type": "itinerary", "day_reference": 1, "itinerary_reference": "Morning temple" },
  "fragment": {
    "intent": "add_estimate",
    "candidate_content": { "amount": "1000", "currency": "JPY", "title": 42 }
  },
  "adoption_hints": { "required_decisions": [] }
}"#,
    )
    .unwrap();
    assert_add_estimate_invalid_dry_run(
        &dir,
        &fragment_path,
        Some(&dir.join("preview.json")),
        "title は文字列",
    );
}

#[test]
fn cli_fragment_apply_add_estimate_non_string_note_blocks_without_db_write() {
    let workspace = common::TestWorkspace::new();
    let dir = workspace.path();
    seed_trip_with_itinerary(&dir);
    let fragment_path = dir.join("add-estimate-non-string-note.json");
    std::fs::write(
        &fragment_path,
        r#"{
  "target": { "target_type": "itinerary", "day_reference": 1, "itinerary_reference": "Morning temple" },
  "fragment": {
    "intent": "add_estimate",
    "candidate_content": { "amount": "1000", "currency": "JPY", "note": 42 }
  },
  "adoption_hints": { "required_decisions": [] }
}"#,
    )
    .unwrap();
    assert_add_estimate_invalid_dry_run(
        &dir,
        &fragment_path,
        Some(&dir.join("preview.json")),
        "note は文字列",
    );
}

#[test]
fn cli_fragment_apply_update_itinerary_basic_dry_run_preview_keeps_db_unchanged() {
    let workspace = common::TestWorkspace::new();
    let dir = workspace.path();
    let fragment = fixture_path("apply-update-itinerary-basic-fragment.json");
    let preview_path = dir.join("update-itinerary-preview.json");
    seed_trip_with_itinerary(&dir);

    let before_expenses = itinerary_expense_count(&dir, "1");
    let before_reservations = itinerary_reservation_count(&dir, "1");
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
    assert_eq!(parsed["preview"]["action"], "update_itinerary");
    let changes = parsed["preview"]["itinerary_field_changes"]
        .as_array()
        .expect("itinerary_field_changes");
    assert_eq!(changes.len(), 3);
    assert_eq!(changes[0]["field"], "title");
    assert_eq!(changes[0]["before"], "Morning temple");
    assert_eq!(changes[0]["after"], "Morning temple visit");
    assert_eq!(changes[1]["field"], "note");
    assert_eq!(changes[1]["before"], "-");
    assert_eq!(changes[1]["after"], "Arrive 15 minutes early.");
    assert_eq!(changes[2]["field"], "category");
    assert_eq!(changes[2]["before"], "-");
    assert_eq!(changes[2]["after"], "museum");

    let preview_json = std::fs::read_to_string(&preview_path).expect("preview json");
    let export: serde_json::Value = serde_json::from_str(&preview_json).expect("parse preview");
    let day = export["days"]
        .as_array()
        .unwrap()
        .iter()
        .find(|day| day["day_number"] == 1)
        .expect("day 1");
    let itinerary = day["itineraries"]
        .as_array()
        .unwrap()
        .iter()
        .find(|item| item["title"] == "Morning temple visit")
        .expect("updated itinerary");
    assert_eq!(itinerary["category"], "museum");
    assert_eq!(itinerary["note"], "Arrive 15 minutes early.");

    assert!(run_cli_in(
        &dir,
        &["trip", "validate-export", preview_path.to_str().unwrap()],
    )
    .status
    .success());

    let list = run_cli_in(&dir, &["itinerary", "list", "1"]);
    assert!(list.status.success());
    let list_stdout = String::from_utf8_lossy(&list.stdout);
    assert!(list_stdout.contains("Morning temple"));
    assert!(!list_stdout.contains("Morning temple visit"));
    assert_eq!(itinerary_expense_count(&dir, "1"), before_expenses);
    assert_eq!(itinerary_reservation_count(&dir, "1"), before_reservations);
    assert_eq!(note_count(&dir, "1"), before_notes);
}

#[test]
fn cli_fragment_apply_update_itinerary_invalid_category_blocks_without_db_write() {
    let workspace = common::TestWorkspace::new();
    let dir = workspace.path();
    let fragment = fixture_path("apply-update-itinerary-invalid-category-fragment.json");
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
            "--json",
        ],
    );
    assert!(!output.status.success());
    let combined = format!(
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(combined.contains("category"));
    let list = run_cli_in(&dir, &["itinerary", "list", "1"]);
    assert!(list.status.success());
    assert!(String::from_utf8_lossy(&list.stdout).contains("Morning temple"));
}

#[test]
fn cli_fragment_apply_update_itinerary_trip_target_blocks_without_db_write() {
    let workspace = common::TestWorkspace::new();
    let dir = workspace.path();
    let fragment = fixture_path("apply-update-itinerary-trip-target-fragment.json");
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
        ],
    );
    assert!(!output.status.success());
    let combined = format!(
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(combined.contains("itinerary target"));
}

#[test]
fn cli_fragment_apply_update_itinerary_required_decisions_blocks_without_db_write() {
    let workspace = common::TestWorkspace::new();
    let dir = workspace.path();
    let fragment = fixture_path("apply-update-itinerary-required-decisions-fragment.json");
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
        ],
    );
    assert!(!output.status.success());
    let combined = format!(
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(combined.contains("required decisions"));
}

#[test]
fn cli_fragment_apply_update_itinerary_conflict_blocks_without_db_write() {
    let workspace = common::TestWorkspace::new();
    let dir = workspace.path();
    let fragment = fixture_path("apply-update-itinerary-conflict-fragment.json");
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
            "--json",
        ],
    );
    assert!(!output.status.success());
    let combined = format!(
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(combined.contains("baseline mismatch"));
    let parsed: serde_json::Value =
        serde_json::from_str(&String::from_utf8_lossy(&output.stdout)).expect("json report");
    let required = parsed["required_decisions"]
        .as_array()
        .expect("required_decisions");
    assert!(required
        .iter()
        .any(|item| item.as_str()
            == Some("Category may have been updated since fragment was authored")));
}

#[test]
fn cli_fragment_apply_update_itinerary_noop_blocks_without_db_write() {
    let workspace = common::TestWorkspace::new();
    let dir = workspace.path();
    let fragment = fixture_path("apply-update-itinerary-noop-fragment.json");
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
        ],
    );
    assert!(!output.status.success());
    let combined = format!(
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(combined.contains("no-op"));
}

#[test]
fn cli_fragment_apply_update_itinerary_basic_confirm_updates_itinerary() {
    let workspace = common::TestWorkspace::new();
    let dir = workspace.path();
    let fragment = fixture_path("apply-update-itinerary-basic-fragment.json");
    seed_trip_with_itinerary(&dir);

    let before_expenses = itinerary_expense_count(&dir, "1");
    let before_reservations = itinerary_reservation_count(&dir, "1");
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
    assert_eq!(parsed["preview"]["action"], "update_itinerary");
    assert_eq!(parsed["updated_itinerary_id"], 1);
    let changes = parsed["preview"]["itinerary_field_changes"]
        .as_array()
        .expect("itinerary_field_changes");
    assert_eq!(changes.len(), 3);

    let show = run_cli_in(&dir, &["itinerary", "show", "1", "--json"]);
    assert!(show.status.success());
    let item: serde_json::Value =
        serde_json::from_str(&String::from_utf8_lossy(&show.stdout)).unwrap();
    assert_eq!(item["title"], "Morning temple visit");
    assert_eq!(item["category"], "museum");
    assert_eq!(item["note"], "Arrive 15 minutes early.");

    let export_path = dir.join("trip-export-after-confirm.json");
    let export = run_cli_in(
        &dir,
        &[
            "trip",
            "export",
            "1",
            "--output",
            export_path.to_str().unwrap(),
        ],
    );
    assert!(
        export.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&export.stdout),
        String::from_utf8_lossy(&export.stderr)
    );
    let export_json: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&export_path).expect("export json")).unwrap();
    let itinerary = export_json["days"]
        .as_array()
        .unwrap()
        .iter()
        .find(|day| day["day_number"] == 1)
        .unwrap()["itineraries"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["title"] == "Morning temple visit")
        .expect("exported itinerary");
    assert_eq!(itinerary["category"], "museum");
    assert_eq!(itinerary["note"], "Arrive 15 minutes early.");

    assert_eq!(itinerary_expense_count(&dir, "1"), before_expenses);
    assert_eq!(itinerary_reservation_count(&dir, "1"), before_reservations);
    assert_eq!(note_count(&dir, "1"), before_notes);
}

#[test]
fn cli_fragment_apply_update_itinerary_trip_target_confirm_blocks_db_write() {
    let workspace = common::TestWorkspace::new();
    let dir = workspace.path();
    let fragment = fixture_path("apply-update-itinerary-trip-target-fragment.json");
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
    assert!(!output.status.success());
    let list = run_cli_in(&dir, &["itinerary", "list", "1"]);
    assert!(list.status.success());
    assert!(String::from_utf8_lossy(&list.stdout).contains("Morning temple"));
}

#[test]
fn cli_fragment_apply_update_itinerary_day_target_confirm_blocks_db_write() {
    let workspace = common::TestWorkspace::new();
    let dir = workspace.path();
    let fragment = fixture_path("apply-update-itinerary-day-target-fragment.json");
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
    assert!(!output.status.success());
    let list = run_cli_in(&dir, &["itinerary", "list", "1"]);
    assert!(list.status.success());
    assert!(String::from_utf8_lossy(&list.stdout).contains("Morning temple"));
}

#[test]
fn cli_fragment_apply_update_itinerary_required_decisions_confirm_blocks_db_write() {
    let workspace = common::TestWorkspace::new();
    let dir = workspace.path();
    let fragment = fixture_path("apply-update-itinerary-required-decisions-fragment.json");
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
    assert!(!output.status.success());
    let list = run_cli_in(&dir, &["itinerary", "list", "1"]);
    assert!(list.status.success());
    assert!(String::from_utf8_lossy(&list.stdout).contains("Morning temple"));
}

#[test]
fn cli_fragment_apply_update_itinerary_conflict_confirm_blocks_db_write() {
    let workspace = common::TestWorkspace::new();
    let dir = workspace.path();
    let fragment = fixture_path("apply-update-itinerary-conflict-fragment.json");
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
    assert!(!output.status.success());
    let combined = format!(
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(combined.contains("baseline mismatch"));
    let list = run_cli_in(&dir, &["itinerary", "list", "1"]);
    assert!(list.status.success());
    assert!(String::from_utf8_lossy(&list.stdout).contains("Morning temple"));
}

#[test]
fn cli_fragment_apply_update_itinerary_noop_confirm_blocks_db_write() {
    let workspace = common::TestWorkspace::new();
    let dir = workspace.path();
    let fragment = fixture_path("apply-update-itinerary-noop-fragment.json");
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
    assert!(!output.status.success());
    let list = run_cli_in(&dir, &["itinerary", "list", "1"]);
    assert!(list.status.success());
    assert!(String::from_utf8_lossy(&list.stdout).contains("Morning temple"));
}

#[test]
fn cli_fragment_apply_update_itinerary_invalid_category_confirm_blocks_db_write() {
    let workspace = common::TestWorkspace::new();
    let dir = workspace.path();
    let fragment = fixture_path("apply-update-itinerary-invalid-category-fragment.json");
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
    assert!(!output.status.success());
    let list = run_cli_in(&dir, &["itinerary", "list", "1"]);
    assert!(list.status.success());
    assert!(String::from_utf8_lossy(&list.stdout).contains("Morning temple"));
}

#[test]
fn cli_fragment_apply_update_itinerary_invalid_time_confirm_blocks_db_write() {
    let workspace = common::TestWorkspace::new();
    let dir = workspace.path();
    seed_trip_with_itinerary(&dir);
    let fragment = dir.join("apply-update-itinerary-invalid-time.json");
    std::fs::write(
        &fragment,
        r#"{
  "metadata": {
    "fragment_id": "frag-update-itinerary-invalid-time",
    "created_at": "2026-03-15T14:00:00Z",
    "source": "manual",
    "provider": "fixture"
  },
  "target": {
    "target_type": "itinerary",
    "day_reference": 1,
    "itinerary_reference": "Morning temple"
  },
  "fragment": {
    "intent": "update_itinerary",
    "candidate_content": {
      "start_time": "25:99"
    }
  },
  "adoption_hints": {
    "required_decisions": []
  }
}"#,
    )
    .expect("write fragment");

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
}

#[test]
fn cli_fragment_apply_update_itinerary_negative_duration_confirm_blocks_db_write() {
    let workspace = common::TestWorkspace::new();
    let dir = workspace.path();
    seed_trip_with_itinerary(&dir);
    let fragment = dir.join("apply-update-itinerary-negative-duration.json");
    std::fs::write(
        &fragment,
        r#"{
  "metadata": {
    "fragment_id": "frag-update-itinerary-negative-duration",
    "created_at": "2026-03-15T14:00:00Z",
    "source": "manual",
    "provider": "fixture"
  },
  "target": {
    "target_type": "itinerary",
    "day_reference": 1,
    "itinerary_reference": "Morning temple"
  },
  "fragment": {
    "intent": "update_itinerary",
    "candidate_content": {
      "duration_minutes": -10
    }
  },
  "adoption_hints": {
    "required_decisions": []
  }
}"#,
    )
    .expect("write fragment");

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
}

#[test]
fn cli_fragment_apply_delete_itinerary_basic_dry_run_preview_keeps_db_unchanged() {
    let workspace = common::TestWorkspace::new();
    let dir = workspace.path();
    let fragment = fixture_path("apply-delete-itinerary-basic-fragment.json");
    let preview_path = dir.join("delete-itinerary-preview.json");
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
    assert_eq!(parsed["preview"]["action"], "delete_itinerary");
    assert_eq!(parsed["preview"]["itineraries_before"], 1);
    assert_eq!(parsed["preview"]["itineraries_after"], 0);
    let delete_preview = &parsed["preview"]["delete_preview"];
    assert_eq!(delete_preview["target_type"], "itinerary");
    assert_eq!(delete_preview["itinerary_id"], 1);
    assert_eq!(delete_preview["title"], "Morning temple");
    assert_eq!(delete_preview["day_number"], 1);
    assert_eq!(delete_preview["blocking_children"]["expenses"], 0);
    assert_eq!(delete_preview["blocking_children"]["estimates"], 0);
    assert_eq!(delete_preview["blocking_children"]["reservations"], 0);
    assert_eq!(delete_preview["blocking_children"]["notes"], 0);

    let preview_json = std::fs::read_to_string(&preview_path).expect("preview json");
    let export: serde_json::Value = serde_json::from_str(&preview_json).expect("parse preview");
    let day = export["days"]
        .as_array()
        .unwrap()
        .iter()
        .find(|day| day["day_number"] == 1)
        .expect("day 1");
    assert!(day["itineraries"].as_array().unwrap().is_empty());

    assert!(run_cli_in(
        &dir,
        &["trip", "validate-export", preview_path.to_str().unwrap()],
    )
    .status
    .success());

    let list = run_cli_in(&dir, &["itinerary", "list", "1"]);
    assert!(list.status.success());
    assert!(String::from_utf8_lossy(&list.stdout).contains("Morning temple"));
}

#[test]
fn cli_fragment_apply_delete_itinerary_expense_blocks_without_db_write() {
    let workspace = common::TestWorkspace::new();
    let dir = workspace.path();
    let fragment = fixture_path("apply-delete-itinerary-basic-fragment.json");
    seed_trip_with_itinerary(&dir);
    assert!(run_cli_in(
        &dir,
        &[
            "expense",
            "add",
            "--itinerary",
            "1",
            "--amount",
            "500",
            "--currency",
            "JPY",
            "--title",
            "Temple admission",
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
            "--json",
        ],
    );
    assert!(!output.status.success());
    let combined = format!(
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(combined.contains("blocking child"));
    assert!(combined.contains("expenses: 1"));
    let parsed: serde_json::Value =
        serde_json::from_str(&String::from_utf8_lossy(&output.stdout)).expect("json report");
    assert_eq!(parsed["preview"]["action"], "delete_itinerary");
    assert_eq!(
        parsed["preview"]["delete_preview"]["blocking_children"]["expenses"],
        1
    );
    let list = run_cli_in(&dir, &["itinerary", "list", "1"]);
    assert!(list.status.success());
    assert!(String::from_utf8_lossy(&list.stdout).contains("Morning temple"));
}

#[test]
fn cli_fragment_apply_delete_itinerary_reservation_blocks_without_db_write() {
    let workspace = common::TestWorkspace::new();
    let dir = workspace.path();
    let fragment = fixture_path("apply-delete-itinerary-basic-fragment.json");
    seed_trip_with_itinerary(&dir);
    assert!(run_cli_in(
        &dir,
        &[
            "reservation",
            "add",
            "--itinerary",
            "1",
            "--reservation-type",
            "hotel",
            "--provider",
            "Example Hotel",
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
            "--json",
        ],
    );
    assert!(!output.status.success());
    let combined = format!(
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(combined.contains("blocking child"));
    assert!(combined.contains("reservations: 1"));
    let parsed: serde_json::Value =
        serde_json::from_str(&String::from_utf8_lossy(&output.stdout)).expect("json report");
    assert_eq!(parsed["preview"]["action"], "delete_itinerary");
    assert_eq!(
        parsed["preview"]["delete_preview"]["blocking_children"]["reservations"],
        1
    );
}

#[test]
fn cli_fragment_apply_delete_itinerary_note_blocks_without_db_write() {
    let workspace = common::TestWorkspace::new();
    let dir = workspace.path();
    let fragment = fixture_path("apply-delete-itinerary-basic-fragment.json");
    let note_fragment = fixture_path("apply-add-note-itinerary-fragment.json");
    seed_trip_with_itinerary(&dir);
    assert!(run_cli_in(
        &dir,
        &[
            "fragment",
            "apply",
            note_fragment.to_str().unwrap(),
            "--confirm",
            "--trip",
            "1",
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
            "--json",
        ],
    );
    assert!(!output.status.success());
    let combined = format!(
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(combined.contains("blocking child"));
    assert!(combined.contains("notes: 1"));
    let parsed: serde_json::Value =
        serde_json::from_str(&String::from_utf8_lossy(&output.stdout)).expect("json report");
    assert_eq!(parsed["preview"]["action"], "delete_itinerary");
    assert_eq!(
        parsed["preview"]["delete_preview"]["blocking_children"]["notes"],
        1
    );
}

#[test]
fn cli_fragment_apply_delete_itinerary_trip_target_blocks_without_db_write() {
    let workspace = common::TestWorkspace::new();
    let dir = workspace.path();
    let fragment = fixture_path("apply-delete-itinerary-trip-target-fragment.json");
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
        ],
    );
    assert!(!output.status.success());
    let combined = format!(
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(combined.contains("itinerary target"));
}

#[test]
fn cli_fragment_apply_delete_itinerary_day_target_blocks_without_db_write() {
    let workspace = common::TestWorkspace::new();
    let dir = workspace.path();
    let fragment = fixture_path("apply-delete-itinerary-day-target-fragment.json");
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
        ],
    );
    assert!(!output.status.success());
    let combined = format!(
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(combined.contains("itinerary target"));
}

#[test]
fn cli_fragment_apply_delete_itinerary_required_decisions_blocks_without_db_write() {
    let workspace = common::TestWorkspace::new();
    let dir = workspace.path();
    let fragment = fixture_path("apply-delete-itinerary-required-decisions-fragment.json");
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
        ],
    );
    assert!(!output.status.success());
    let combined = format!(
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(combined.contains("required decisions"));
}

#[test]
fn cli_fragment_apply_delete_itinerary_conflict_blocks_without_db_write() {
    let workspace = common::TestWorkspace::new();
    let dir = workspace.path();
    let fragment = fixture_path("apply-delete-itinerary-conflict-fragment.json");
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
            "--json",
        ],
    );
    assert!(!output.status.success());
    let combined = format!(
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(combined.contains("required decisions"));
    let parsed: serde_json::Value =
        serde_json::from_str(&String::from_utf8_lossy(&output.stdout)).expect("json report");
    let required = parsed["required_decisions"]
        .as_array()
        .expect("required_decisions");
    assert!(!required.is_empty());
}

#[test]
fn cli_fragment_apply_delete_itinerary_basic_confirm_deletes_itinerary() {
    let workspace = common::TestWorkspace::new();
    let dir = workspace.path();
    let fragment = fixture_path("apply-delete-itinerary-basic-fragment.json");
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
    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let parsed: serde_json::Value =
        serde_json::from_str(&String::from_utf8_lossy(&output.stdout)).expect("json report");
    assert_eq!(parsed["preview"]["action"], "delete_itinerary");
    assert_eq!(parsed["deleted_itinerary_id"], 1);
    assert!(parsed.get("updated_itinerary_id").is_none());
    assert!(parsed.get("inserted_itinerary_id").is_none());

    let show = run_cli_in(&dir, &["itinerary", "show", "1"]);
    assert!(!show.status.success());

    let export_path = dir.join("trip-export-after-delete.json");
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
    let export_json: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&export_path).expect("export json")).unwrap();
    let day = export_json["days"]
        .as_array()
        .unwrap()
        .iter()
        .find(|day| day["day_number"] == 1)
        .expect("day 1");
    assert!(day["itineraries"].as_array().unwrap().is_empty());

    let list = run_cli_in(&dir, &["itinerary", "list", "1"]);
    assert!(list.status.success());
    assert!(!String::from_utf8_lossy(&list.stdout).contains("Morning temple"));
}

#[test]
fn cli_fragment_apply_reorder_itinerary_valid_same_day_dry_run_preview_keeps_db_unchanged() {
    let workspace = common::TestWorkspace::new();
    let dir = workspace.path();
    let fragment = fixture_path("apply-reorder-itinerary-basic-fragment.json");
    let preview_path = dir.join("reorder-itinerary-preview.json");
    seed_trip_two_days_with_itineraries(&dir);

    let before_day = run_cli_in(&dir, &["day", "show", "1", "1"]);
    assert!(before_day.status.success());
    let before_stdout = String::from_utf8_lossy(&before_day.stdout).to_string();

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
    assert_eq!(parsed["preview"]["action"], "reorder_itinerary");
    assert_eq!(parsed["preview"]["reorder_preview"]["day_number"], 1);
    let changes = parsed["preview"]["reorder_preview"]["itinerary_order_changes"]
        .as_array()
        .expect("changes array");
    assert_eq!(changes.len(), 3);
    for item in changes {
        assert!(item["itinerary_id"].as_i64().unwrap() >= 1);
        assert!(item["title"].as_str().unwrap().len() >= 1);
        assert!(item["before_sort_order"].as_i64().is_some());
        assert!(item["after_sort_order"].as_i64().is_some());
    }

    let preview_json = std::fs::read_to_string(&preview_path).expect("preview json");
    let export: serde_json::Value = serde_json::from_str(&preview_json).expect("parse preview");
    let day = export["days"]
        .as_array()
        .unwrap()
        .iter()
        .find(|day| day["day_number"] == 1)
        .expect("day 1");
    let titles: Vec<String> = day["itineraries"]
        .as_array()
        .unwrap()
        .iter()
        .map(|it| it["title"].as_str().unwrap().to_string())
        .collect();
    assert_eq!(titles, vec!["Aquarium", "Breakfast", "Dinner"]);

    assert!(run_cli_in(
        &dir,
        &["trip", "validate-export", preview_path.to_str().unwrap()],
    )
    .status
    .success());

    let after_day = run_cli_in(&dir, &["day", "show", "1", "1"]);
    assert!(after_day.status.success());
    let after_stdout = String::from_utf8_lossy(&after_day.stdout).to_string();
    assert_eq!(before_stdout, after_stdout);
}

#[test]
fn cli_fragment_apply_reorder_itinerary_id_selector_is_supported_and_keeps_db_unchanged() {
    let workspace = common::TestWorkspace::new();
    let dir = workspace.path();
    let fragment = fixture_path("apply-reorder-itinerary-id-fragment.json");
    let preview_path = dir.join("reorder-itinerary-id-preview.json");
    seed_trip_two_days_with_itineraries(&dir);

    let before_day = run_cli_in(&dir, &["day", "show", "1", "1"]);
    assert!(before_day.status.success());
    let before_stdout = String::from_utf8_lossy(&before_day.stdout).to_string();

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
            "--json",
        ],
    );
    assert!(output.status.success());
    let parsed: serde_json::Value =
        serde_json::from_str(&String::from_utf8_lossy(&output.stdout)).expect("json report");
    assert_eq!(parsed["valid"], true);
    assert_eq!(parsed["preview"]["action"], "reorder_itinerary");
    assert_eq!(parsed["preview"]["reorder_preview"]["day_number"], 1);

    let preview_json = std::fs::read_to_string(&preview_path).expect("preview json");
    let export: serde_json::Value = serde_json::from_str(&preview_json).expect("parse preview");
    let day = export["days"]
        .as_array()
        .unwrap()
        .iter()
        .find(|day| day["day_number"] == 1)
        .expect("day 1");
    let titles: Vec<String> = day["itineraries"]
        .as_array()
        .unwrap()
        .iter()
        .map(|it| it["title"].as_str().unwrap().to_string())
        .collect();
    assert_eq!(titles, vec!["Aquarium", "Breakfast", "Dinner"]);

    let after_day = run_cli_in(&dir, &["day", "show", "1", "1"]);
    assert!(after_day.status.success());
    let after_stdout = String::from_utf8_lossy(&after_day.stdout).to_string();
    assert_eq!(before_stdout, after_stdout);
}

#[test]
fn cli_fragment_apply_reorder_itinerary_missing_expected_order_is_invalid_and_keeps_db_unchanged() {
    let workspace = common::TestWorkspace::new();
    let dir = workspace.path();
    let fragment = fixture_path("apply-reorder-itinerary-missing-expected-order-fragment.json");
    seed_trip_two_days_with_itineraries(&dir);

    let before_day = run_cli_in(&dir, &["day", "show", "1", "1"]);
    assert!(before_day.status.success());
    let before_stdout = String::from_utf8_lossy(&before_day.stdout).to_string();

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
    assert!(combined.contains("expected_order"));

    let after_day = run_cli_in(&dir, &["day", "show", "1", "1"]);
    assert!(after_day.status.success());
    let after_stdout = String::from_utf8_lossy(&after_day.stdout).to_string();
    assert_eq!(before_stdout, after_stdout);
}

#[test]
fn cli_fragment_apply_reorder_itinerary_cross_day_is_rejected_and_keeps_db_unchanged() {
    let workspace = common::TestWorkspace::new();
    let dir = workspace.path();
    let fragment = fixture_path("apply-reorder-itinerary-cross-day-fragment.json");
    seed_trip_two_days_with_itineraries(&dir);

    let before_day = run_cli_in(&dir, &["day", "show", "1", "1"]);
    assert!(before_day.status.success());
    let before_stdout = String::from_utf8_lossy(&before_day.stdout).to_string();

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
    assert!(combined.contains("cross-day"));

    let after_day = run_cli_in(&dir, &["day", "show", "1", "1"]);
    assert!(after_day.status.success());
    let after_stdout = String::from_utf8_lossy(&after_day.stdout).to_string();
    assert_eq!(before_stdout, after_stdout);
}

#[test]
fn cli_fragment_apply_reorder_itinerary_duplicate_is_rejected_and_keeps_db_unchanged() {
    let workspace = common::TestWorkspace::new();
    let dir = workspace.path();
    let fragment = fixture_path("apply-reorder-itinerary-duplicate-fragment.json");
    seed_trip_two_days_with_itineraries(&dir);

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
    assert!(combined.contains("重複"));
}

#[test]
fn cli_fragment_apply_reorder_itinerary_unknown_is_rejected_and_keeps_db_unchanged() {
    let workspace = common::TestWorkspace::new();
    let dir = workspace.path();
    let fragment = fixture_path("apply-reorder-itinerary-unknown-fragment.json");
    seed_trip_two_days_with_itineraries(&dir);

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
    assert!(combined.contains("見つかりません"));
}

#[test]
fn cli_fragment_apply_reorder_itinerary_confirm_is_supported() {
    let workspace = common::TestWorkspace::new();
    let dir = workspace.path();
    let fragment = fixture_path("apply-reorder-itinerary-basic-fragment.json");
    seed_trip_two_days_with_itineraries(&dir);

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
    assert_eq!(parsed["preview"]["action"], "reorder_itinerary");
    assert!(parsed["reordered_itineraries"].as_u64().unwrap() >= 1);
}

#[test]
fn cli_fragment_apply_reorder_itinerary_confirm_updates_sort_order_in_db() {
    let workspace = common::TestWorkspace::new();
    let dir = workspace.path();
    let fragment = fixture_path("apply-reorder-itinerary-basic-fragment.json");
    seed_trip_two_days_with_itineraries(&dir);

    let before = run_cli_in(&dir, &["itinerary", "list", "1", "--json"]);
    assert!(before.status.success());
    let before_json: serde_json::Value =
        serde_json::from_str(&String::from_utf8_lossy(&before.stdout)).unwrap();
    let before_titles: Vec<String> = before_json
        .as_array()
        .unwrap()
        .iter()
        .filter(|it| it["day"] == 1)
        .map(|it| it["title"].as_str().unwrap().to_string())
        .collect();
    assert_eq!(before_titles, vec!["Breakfast", "Aquarium", "Dinner"]);

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
    assert_eq!(parsed["preview"]["action"], "reorder_itinerary");
    assert_eq!(parsed["preview"]["reorder_preview"]["day_number"], 1);
    assert!(parsed["reordered_itineraries"].as_u64().unwrap() >= 1);

    let after = run_cli_in(&dir, &["itinerary", "list", "1", "--json"]);
    assert!(after.status.success());
    let after_json: serde_json::Value =
        serde_json::from_str(&String::from_utf8_lossy(&after.stdout)).unwrap();
    let after_day1: Vec<&serde_json::Value> = after_json
        .as_array()
        .unwrap()
        .iter()
        .filter(|it| it["day"] == 1)
        .collect();
    let after_titles: Vec<String> = after_day1
        .iter()
        .map(|it| it["title"].as_str().unwrap().to_string())
        .collect();
    assert_eq!(after_titles, vec!["Aquarium", "Breakfast", "Dinner"]);

    // sparse slot を維持していること（1000/2000/3000 を並べ替え）
    let orders: Vec<i64> = after_day1
        .iter()
        .map(|it| it["sort_order"].as_i64().unwrap())
        .collect();
    assert_eq!(orders, vec![1000, 2000, 3000]);
}

#[test]
fn cli_fragment_apply_reorder_itinerary_confirm_id_selector_succeeds() {
    let workspace = common::TestWorkspace::new();
    let dir = workspace.path();
    let fragment = fixture_path("apply-reorder-itinerary-id-fragment.json");
    seed_trip_two_days_with_itineraries(&dir);

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

    let after = run_cli_in(&dir, &["itinerary", "list", "1", "--json"]);
    assert!(after.status.success());
    let after_json: serde_json::Value =
        serde_json::from_str(&String::from_utf8_lossy(&after.stdout)).unwrap();
    let after_titles: Vec<String> = after_json
        .as_array()
        .unwrap()
        .iter()
        .filter(|it| it["day"] == 1)
        .map(|it| it["title"].as_str().unwrap().to_string())
        .collect();
    assert_eq!(after_titles, vec!["Aquarium", "Breakfast", "Dinner"]);
}

#[test]
fn cli_fragment_apply_reorder_itinerary_confirm_missing_after_order_blocks_db_write() {
    let workspace = common::TestWorkspace::new();
    let dir = workspace.path();
    let fragment = fixture_path("apply-reorder-itinerary-missing-after-order-fragment.json");
    seed_trip_two_days_with_itineraries(&dir);

    let before_day = run_cli_in(&dir, &["day", "show", "1", "1"]);
    assert!(before_day.status.success());
    let before_stdout = String::from_utf8_lossy(&before_day.stdout).to_string();

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
    assert!(!output.status.success());

    let after_day = run_cli_in(&dir, &["day", "show", "1", "1"]);
    assert!(after_day.status.success());
    let after_stdout = String::from_utf8_lossy(&after_day.stdout).to_string();
    assert_eq!(before_stdout, after_stdout);
}

#[test]
fn cli_fragment_apply_reorder_itinerary_confirm_baseline_mismatch_blocks_db_write() {
    let workspace = common::TestWorkspace::new();
    let dir = workspace.path();
    let fragment = fixture_path("apply-reorder-itinerary-baseline-mismatch-fragment.json");
    seed_trip_two_days_with_itineraries(&dir);

    let before_day = run_cli_in(&dir, &["day", "show", "1", "1"]);
    assert!(before_day.status.success());
    let before_stdout = String::from_utf8_lossy(&before_day.stdout).to_string();

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
    assert!(!output.status.success());
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(combined.contains("baseline mismatch") || combined.contains("TOCTOU"));

    let after_day = run_cli_in(&dir, &["day", "show", "1", "1"]);
    assert!(after_day.status.success());
    let after_stdout = String::from_utf8_lossy(&after_day.stdout).to_string();
    assert_eq!(before_stdout, after_stdout);
}

#[test]
fn cli_fragment_apply_reorder_itinerary_confirm_set_mismatch_blocks_db_write() {
    let workspace = common::TestWorkspace::new();
    let dir = workspace.path();
    let fragment = fixture_path("apply-reorder-itinerary-set-mismatch-fragment.json");
    seed_trip_two_days_with_itineraries(&dir);

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
    assert!(!output.status.success());
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(combined.contains("集合") || combined.contains("same itinerary"));
}

#[test]
fn cli_fragment_apply_reorder_itinerary_confirm_noop_blocks_db_write() {
    let workspace = common::TestWorkspace::new();
    let dir = workspace.path();
    let fragment = fixture_path("apply-reorder-itinerary-noop-fragment.json");
    seed_trip_two_days_with_itineraries(&dir);

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
    assert!(!output.status.success());
}

#[test]
fn cli_fragment_apply_reorder_itinerary_confirm_ambiguous_number_selector_is_rejected() {
    let workspace = common::TestWorkspace::new();
    let dir = workspace.path();
    let fragment = fixture_path("apply-reorder-itinerary-ambiguous-number-fragment.json");
    seed_trip_day_with_ambiguous_number_selector(&dir);

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
    assert!(!output.status.success());
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(combined.contains("曖昧"));
}

#[test]
fn cli_fragment_apply_move_itinerary_valid_cross_day_dry_run_preview_keeps_db_unchanged() {
    let workspace = common::TestWorkspace::new();
    let dir = workspace.path();
    let fragment = fixture_path("apply-move-itinerary-basic-fragment.json");
    let preview_path = dir.join("move-itinerary-preview.json");
    seed_trip_two_days_with_itineraries_for_move(&dir);

    let before_day1 = run_cli_in(&dir, &["day", "show", "1", "1"]);
    assert!(before_day1.status.success());
    let before_day1_stdout = String::from_utf8_lossy(&before_day1.stdout).to_string();
    let before_day2 = run_cli_in(&dir, &["day", "show", "1", "2"]);
    assert!(before_day2.status.success());
    let before_day2_stdout = String::from_utf8_lossy(&before_day2.stdout).to_string();

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
    assert_eq!(parsed["preview"]["action"], "move_itinerary");
    assert_eq!(parsed["preview"]["move_preview"]["from_day_number"], 1);
    assert_eq!(parsed["preview"]["move_preview"]["to_day_number"], 2);
    assert!(
        parsed["preview"]["move_preview"]["itinerary_id"]
            .as_i64()
            .unwrap()
            >= 1
    );
    assert!(
        parsed["preview"]["move_preview"]["title"]
            .as_str()
            .unwrap()
            .len()
            >= 1
    );

    let changes_source = parsed["preview"]["move_preview"]["source_order_changes"]
        .as_array()
        .expect("source changes");
    let changes_dest = parsed["preview"]["move_preview"]["destination_order_changes"]
        .as_array()
        .expect("dest changes");
    assert!(!changes_source.is_empty());
    assert!(!changes_dest.is_empty());

    let preview_json = std::fs::read_to_string(&preview_path).expect("preview json");
    let export: serde_json::Value = serde_json::from_str(&preview_json).expect("parse preview");
    let day1 = export["days"]
        .as_array()
        .unwrap()
        .iter()
        .find(|day| day["day_number"] == 1)
        .expect("day 1");
    let day2 = export["days"]
        .as_array()
        .unwrap()
        .iter()
        .find(|day| day["day_number"] == 2)
        .expect("day 2");
    let titles_day1: Vec<String> = day1["itineraries"]
        .as_array()
        .unwrap()
        .iter()
        .map(|it| it["title"].as_str().unwrap().to_string())
        .collect();
    let titles_day2: Vec<String> = day2["itineraries"]
        .as_array()
        .unwrap()
        .iter()
        .map(|it| it["title"].as_str().unwrap().to_string())
        .collect();
    assert_eq!(titles_day1, vec!["Breakfast", "Dinner"]);
    assert_eq!(
        titles_day2,
        vec!["Museum (Day 2)", "Aquarium", "Beach (Day 2)"]
    );

    assert!(run_cli_in(
        &dir,
        &["trip", "validate-export", preview_path.to_str().unwrap()],
    )
    .status
    .success());

    let after_day1 = run_cli_in(&dir, &["day", "show", "1", "1"]);
    assert!(after_day1.status.success());
    assert_eq!(
        before_day1_stdout,
        String::from_utf8_lossy(&after_day1.stdout).to_string()
    );
    let after_day2 = run_cli_in(&dir, &["day", "show", "1", "2"]);
    assert!(after_day2.status.success());
    assert_eq!(
        before_day2_stdout,
        String::from_utf8_lossy(&after_day2.stdout).to_string()
    );
}

#[test]
fn cli_fragment_apply_move_itinerary_id_selector_is_supported_and_keeps_db_unchanged() {
    let workspace = common::TestWorkspace::new();
    let dir = workspace.path();
    let fragment = fixture_path("apply-move-itinerary-id-fragment.json");
    let preview_path = dir.join("move-itinerary-id-preview.json");
    seed_trip_two_days_with_itineraries_for_move(&dir);

    let before_day1 = run_cli_in(&dir, &["day", "show", "1", "1"]);
    assert!(before_day1.status.success());
    let before_day1_stdout = String::from_utf8_lossy(&before_day1.stdout).to_string();
    let before_day2 = run_cli_in(&dir, &["day", "show", "1", "2"]);
    assert!(before_day2.status.success());
    let before_day2_stdout = String::from_utf8_lossy(&before_day2.stdout).to_string();

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
            "--json",
        ],
    );
    assert!(output.status.success());
    let parsed: serde_json::Value =
        serde_json::from_str(&String::from_utf8_lossy(&output.stdout)).expect("json report");
    assert_eq!(parsed["valid"], true);
    assert_eq!(parsed["preview"]["action"], "move_itinerary");

    let after_day1 = run_cli_in(&dir, &["day", "show", "1", "1"]);
    assert!(after_day1.status.success());
    assert_eq!(
        before_day1_stdout,
        String::from_utf8_lossy(&after_day1.stdout).to_string()
    );
    let after_day2 = run_cli_in(&dir, &["day", "show", "1", "2"]);
    assert!(after_day2.status.success());
    assert_eq!(
        before_day2_stdout,
        String::from_utf8_lossy(&after_day2.stdout).to_string()
    );
}

#[test]
fn cli_fragment_apply_move_itinerary_missing_expected_source_order_is_invalid_and_keeps_db_unchanged(
) {
    let workspace = common::TestWorkspace::new();
    let dir = workspace.path();
    let fragment = fixture_path("apply-move-itinerary-missing-expected-source-order-fragment.json");
    seed_trip_two_days_with_itineraries_for_move(&dir);

    let before_day1 = run_cli_in(&dir, &["day", "show", "1", "1"]);
    assert!(before_day1.status.success());
    let before_day1_stdout = String::from_utf8_lossy(&before_day1.stdout).to_string();
    let before_day2 = run_cli_in(&dir, &["day", "show", "1", "2"]);
    assert!(before_day2.status.success());
    let before_day2_stdout = String::from_utf8_lossy(&before_day2.stdout).to_string();

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
    assert!(combined.contains("expected_source_order"));

    let after_day1 = run_cli_in(&dir, &["day", "show", "1", "1"]);
    assert!(after_day1.status.success());
    assert_eq!(
        before_day1_stdout,
        String::from_utf8_lossy(&after_day1.stdout).to_string()
    );
    let after_day2 = run_cli_in(&dir, &["day", "show", "1", "2"]);
    assert!(after_day2.status.success());
    assert_eq!(
        before_day2_stdout,
        String::from_utf8_lossy(&after_day2.stdout).to_string()
    );
}

#[test]
fn cli_fragment_apply_move_itinerary_same_day_is_rejected_and_keeps_db_unchanged() {
    let workspace = common::TestWorkspace::new();
    let dir = workspace.path();
    let fragment = fixture_path("apply-move-itinerary-same-day-fragment.json");
    seed_trip_two_days_with_itineraries_for_move(&dir);

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
    assert!(combined.contains("reorder_itinerary"));
}

#[test]
fn cli_fragment_apply_move_itinerary_confirm_is_supported() {
    let workspace = common::TestWorkspace::new();
    let dir = workspace.path();
    let fragment = fixture_path("apply-move-itinerary-basic-fragment.json");
    seed_trip_two_days_with_itineraries_for_move(&dir);

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
    assert_eq!(parsed["preview"]["action"], "move_itinerary");
    assert!(parsed["moved_itinerary_id"].as_i64().unwrap() >= 1);
    assert!(parsed["moved_itinerary_updated_rows"].as_u64().unwrap() >= 1);
}

#[test]
fn cli_fragment_apply_move_itinerary_confirm_updates_db() {
    let workspace = common::TestWorkspace::new();
    let dir = workspace.path();
    let fragment = fixture_path("apply-move-itinerary-basic-fragment.json");
    seed_trip_two_days_with_itineraries_for_move(&dir);

    let before = run_cli_in(&dir, &["itinerary", "list", "1", "--json"]);
    assert!(before.status.success());
    let before_json: serde_json::Value =
        serde_json::from_str(&String::from_utf8_lossy(&before.stdout)).unwrap();
    assert_eq!(before_json.as_array().unwrap().len(), 5);

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

    let after = run_cli_in(&dir, &["itinerary", "list", "1", "--json"]);
    assert!(after.status.success());
    let after_json: serde_json::Value =
        serde_json::from_str(&String::from_utf8_lossy(&after.stdout)).unwrap();
    assert_eq!(after_json.as_array().unwrap().len(), 5);

    let day1_titles: Vec<String> = after_json
        .as_array()
        .unwrap()
        .iter()
        .filter(|it| it["day"] == 1)
        .map(|it| it["title"].as_str().unwrap().to_string())
        .collect();
    let day2_titles: Vec<String> = after_json
        .as_array()
        .unwrap()
        .iter()
        .filter(|it| it["day"] == 2)
        .map(|it| it["title"].as_str().unwrap().to_string())
        .collect();
    assert_eq!(day1_titles, vec!["Breakfast", "Dinner"]);
    assert_eq!(
        day2_titles,
        vec!["Museum (Day 2)", "Aquarium", "Beach (Day 2)"]
    );

    let aquarium = after_json
        .as_array()
        .unwrap()
        .iter()
        .find(|it| it["title"] == "Aquarium")
        .expect("aquarium");
    assert_eq!(aquarium["day"], 2);

    let day1_orders: Vec<i64> = after_json
        .as_array()
        .unwrap()
        .iter()
        .filter(|it| it["day"] == 1)
        .map(|it| it["sort_order"].as_i64().unwrap())
        .collect();
    assert_eq!(day1_orders, vec![1000, 2000]);
}

#[test]
fn cli_fragment_apply_move_itinerary_confirm_id_selector_succeeds() {
    let workspace = common::TestWorkspace::new();
    let dir = workspace.path();
    let fragment = fixture_path("apply-move-itinerary-id-fragment.json");
    seed_trip_two_days_with_itineraries_for_move(&dir);

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

    let after = run_cli_in(&dir, &["itinerary", "list", "1", "--json"]);
    assert!(after.status.success());
    let after_json: serde_json::Value =
        serde_json::from_str(&String::from_utf8_lossy(&after.stdout)).unwrap();
    let day2_titles: Vec<String> = after_json
        .as_array()
        .unwrap()
        .iter()
        .filter(|it| it["day"] == 2)
        .map(|it| it["title"].as_str().unwrap().to_string())
        .collect();
    assert_eq!(
        day2_titles,
        vec!["Museum (Day 2)", "Aquarium", "Beach (Day 2)"]
    );
}

#[test]
fn cli_fragment_apply_move_itinerary_confirm_empty_source_day_succeeds() {
    let workspace = common::TestWorkspace::new();
    let dir = workspace.path();
    let fragment = fixture_path("apply-move-itinerary-empty-source-fragment.json");
    seed_trip_single_source_itinerary_for_move(&dir);

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

    let after = run_cli_in(&dir, &["itinerary", "list", "1", "--json"]);
    assert!(after.status.success());
    let after_json: serde_json::Value =
        serde_json::from_str(&String::from_utf8_lossy(&after.stdout)).unwrap();
    let day1_count = after_json
        .as_array()
        .unwrap()
        .iter()
        .filter(|it| it["day"] == 1)
        .count();
    assert_eq!(day1_count, 0);
    let day2_titles: Vec<String> = after_json
        .as_array()
        .unwrap()
        .iter()
        .filter(|it| it["day"] == 2)
        .map(|it| it["title"].as_str().unwrap().to_string())
        .collect();
    assert_eq!(
        day2_titles,
        vec!["Museum (Day 2)", "Aquarium", "Beach (Day 2)"]
    );
}

#[test]
fn cli_fragment_apply_move_itinerary_confirm_same_day_is_rejected_and_keeps_db_unchanged() {
    let workspace = common::TestWorkspace::new();
    let dir = workspace.path();
    let fragment = fixture_path("apply-move-itinerary-same-day-fragment.json");
    seed_trip_two_days_with_itineraries_for_move(&dir);

    let before_day1 = run_cli_in(&dir, &["day", "show", "1", "1"]);
    assert!(before_day1.status.success());
    let before_day1_stdout = String::from_utf8_lossy(&before_day1.stdout).to_string();

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
    assert!(!output.status.success());
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(combined.contains("reorder_itinerary"));

    let after_day1 = run_cli_in(&dir, &["day", "show", "1", "1"]);
    assert!(after_day1.status.success());
    assert_eq!(
        before_day1_stdout,
        String::from_utf8_lossy(&after_day1.stdout).to_string()
    );
}

#[test]
fn cli_fragment_apply_move_itinerary_confirm_baseline_mismatch_source_blocks_db_write() {
    let workspace = common::TestWorkspace::new();
    let dir = workspace.path();
    let fragment = fixture_path("apply-move-itinerary-baseline-mismatch-source-fragment.json");
    seed_trip_two_days_with_itineraries_for_move(&dir);

    let before_day1 = run_cli_in(&dir, &["day", "show", "1", "1"]);
    assert!(before_day1.status.success());
    let before_stdout = String::from_utf8_lossy(&before_day1.stdout).to_string();

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
    assert!(!output.status.success());
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(combined.contains("baseline mismatch") || combined.contains("TOCTOU"));

    let after_day1 = run_cli_in(&dir, &["day", "show", "1", "1"]);
    assert!(after_day1.status.success());
    assert_eq!(
        before_stdout,
        String::from_utf8_lossy(&after_day1.stdout).to_string()
    );
}

#[test]
fn cli_fragment_apply_move_itinerary_confirm_baseline_mismatch_destination_blocks_db_write() {
    let workspace = common::TestWorkspace::new();
    let dir = workspace.path();
    let fragment = fixture_path("apply-move-itinerary-baseline-mismatch-destination-fragment.json");
    seed_trip_two_days_with_itineraries_for_move(&dir);

    let before_day2 = run_cli_in(&dir, &["day", "show", "1", "2"]);
    assert!(before_day2.status.success());
    let before_stdout = String::from_utf8_lossy(&before_day2.stdout).to_string();

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
    assert!(!output.status.success());
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(combined.contains("baseline mismatch") || combined.contains("TOCTOU"));

    let after_day2 = run_cli_in(&dir, &["day", "show", "1", "2"]);
    assert!(after_day2.status.success());
    assert_eq!(
        before_stdout,
        String::from_utf8_lossy(&after_day2.stdout).to_string()
    );
}

#[test]
fn cli_fragment_apply_move_itinerary_confirm_moved_in_after_source_blocks_db_write() {
    let workspace = common::TestWorkspace::new();
    let dir = workspace.path();
    let fragment = fixture_path("apply-move-itinerary-moved-in-after-source-fragment.json");
    seed_trip_two_days_with_itineraries_for_move(&dir);

    let before_day1 = run_cli_in(&dir, &["day", "show", "1", "1"]);
    assert!(before_day1.status.success());
    let before_stdout = String::from_utf8_lossy(&before_day1.stdout).to_string();

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
    assert!(!output.status.success());
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(combined.contains("after_source_order"));

    let after_day1 = run_cli_in(&dir, &["day", "show", "1", "1"]);
    assert!(after_day1.status.success());
    assert_eq!(
        before_stdout,
        String::from_utf8_lossy(&after_day1.stdout).to_string()
    );
}

#[test]
fn cli_fragment_apply_move_itinerary_confirm_duplicate_selector_blocks_db_write() {
    let workspace = common::TestWorkspace::new();
    let dir = workspace.path();
    let fragment = fixture_path("apply-move-itinerary-duplicate-selector-fragment.json");
    seed_trip_two_days_with_itineraries_for_move(&dir);

    let before_day2 = run_cli_in(&dir, &["day", "show", "1", "2"]);
    assert!(before_day2.status.success());
    let before_stdout = String::from_utf8_lossy(&before_day2.stdout).to_string();

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
    assert!(!output.status.success());
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(combined.contains("重複"));

    let after_day2 = run_cli_in(&dir, &["day", "show", "1", "2"]);
    assert!(after_day2.status.success());
    assert_eq!(
        before_stdout,
        String::from_utf8_lossy(&after_day2.stdout).to_string()
    );
}

#[test]
fn cli_fragment_apply_move_itinerary_confirm_ambiguous_number_selector_is_rejected() {
    let workspace = common::TestWorkspace::new();
    let dir = workspace.path();
    let fragment = fixture_path("apply-move-itinerary-ambiguous-number-fragment.json");
    seed_trip_two_days_with_ambiguous_move_number_selector(&dir);

    let before_day1 = run_cli_in(&dir, &["day", "show", "1", "1"]);
    assert!(before_day1.status.success());
    let before_day1_stdout = String::from_utf8_lossy(&before_day1.stdout).to_string();

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
    assert!(!output.status.success());
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(combined.contains("曖昧"));

    let after_day1 = run_cli_in(&dir, &["day", "show", "1", "1"]);
    assert!(after_day1.status.success());
    assert_eq!(
        before_day1_stdout,
        String::from_utf8_lossy(&after_day1.stdout).to_string()
    );
}

#[test]
fn cli_fragment_apply_move_itinerary_confirm_mutation_boundary() {
    let workspace = common::TestWorkspace::new();
    let dir = workspace.path();
    let fragment = fixture_path("apply-move-itinerary-basic-fragment.json");
    seed_trip_two_days_with_itineraries_for_move(&dir);

    let before = run_cli_in(&dir, &["itinerary", "list", "1", "--json"]);
    assert!(before.status.success());
    let before_json: serde_json::Value =
        serde_json::from_str(&String::from_utf8_lossy(&before.stdout)).unwrap();
    let aquarium_before = before_json
        .as_array()
        .unwrap()
        .iter()
        .find(|it| it["title"] == "Aquarium")
        .expect("aquarium before")
        .clone();
    let aquarium_id = aquarium_before["id"].as_i64().unwrap();
    let aquarium_title = aquarium_before["title"].as_str().unwrap().to_string();
    let before_count = before_json.as_array().unwrap().len();

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

    let after = run_cli_in(&dir, &["itinerary", "list", "1", "--json"]);
    assert!(after.status.success());
    let after_json: serde_json::Value =
        serde_json::from_str(&String::from_utf8_lossy(&after.stdout)).unwrap();
    assert_eq!(after_json.as_array().unwrap().len(), before_count);

    let aquarium_after = after_json
        .as_array()
        .unwrap()
        .iter()
        .find(|it| it["id"] == aquarium_id)
        .expect("aquarium after move");
    assert_eq!(aquarium_after["title"].as_str().unwrap(), aquarium_title);
    assert_eq!(aquarium_after["day"], 2);
    assert_ne!(
        aquarium_after["sort_order"].as_i64().unwrap(),
        aquarium_before["sort_order"].as_i64().unwrap()
    );

    let db_path = dir.join("caglla.db");
    let destination_day_id = query_day_id_for_trip_day(&db_path, 1, 2);
    let (raw_day_id, raw_day) = query_itinerary_day_columns(&db_path, aquarium_id);
    assert_eq!(raw_day_id, destination_day_id);
    assert_eq!(raw_day, 2);
}

#[test]
fn cli_fragment_apply_move_itinerary_ambiguous_number_selector_is_rejected_and_keeps_db_unchanged()
{
    let workspace = common::TestWorkspace::new();
    let dir = workspace.path();
    let fragment = fixture_path("apply-move-itinerary-ambiguous-number-fragment.json");
    seed_trip_two_days_with_ambiguous_move_number_selector(&dir);

    let before_day1 = run_cli_in(&dir, &["day", "show", "1", "1"]);
    assert!(before_day1.status.success());
    let before_day1_stdout = String::from_utf8_lossy(&before_day1.stdout).to_string();
    let before_day2 = run_cli_in(&dir, &["day", "show", "1", "2"]);
    assert!(before_day2.status.success());
    let before_day2_stdout = String::from_utf8_lossy(&before_day2.stdout).to_string();

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
    assert!(combined.contains("曖昧"));

    let after_day1 = run_cli_in(&dir, &["day", "show", "1", "1"]);
    assert!(after_day1.status.success());
    assert_eq!(
        before_day1_stdout,
        String::from_utf8_lossy(&after_day1.stdout).to_string()
    );
    let after_day2 = run_cli_in(&dir, &["day", "show", "1", "2"]);
    assert!(after_day2.status.success());
    assert_eq!(
        before_day2_stdout,
        String::from_utf8_lossy(&after_day2.stdout).to_string()
    );
}

#[test]
fn cli_fragment_apply_delete_itinerary_trip_target_confirm_blocks_db_write() {
    let workspace = common::TestWorkspace::new();
    let dir = workspace.path();
    let fragment = fixture_path("apply-delete-itinerary-trip-target-fragment.json");
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
    assert!(!output.status.success());
    let list = run_cli_in(&dir, &["itinerary", "list", "1"]);
    assert!(list.status.success());
    assert!(String::from_utf8_lossy(&list.stdout).contains("Morning temple"));
}

#[test]
fn cli_fragment_apply_delete_itinerary_day_target_confirm_blocks_db_write() {
    let workspace = common::TestWorkspace::new();
    let dir = workspace.path();
    let fragment = fixture_path("apply-delete-itinerary-day-target-fragment.json");
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
    assert!(!output.status.success());
    let list = run_cli_in(&dir, &["itinerary", "list", "1"]);
    assert!(list.status.success());
    assert!(String::from_utf8_lossy(&list.stdout).contains("Morning temple"));
}

#[test]
fn cli_fragment_apply_delete_itinerary_required_decisions_confirm_blocks_db_write() {
    let workspace = common::TestWorkspace::new();
    let dir = workspace.path();
    let fragment = fixture_path("apply-delete-itinerary-required-decisions-fragment.json");
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
    assert!(!output.status.success());
    let list = run_cli_in(&dir, &["itinerary", "list", "1"]);
    assert!(list.status.success());
    assert!(String::from_utf8_lossy(&list.stdout).contains("Morning temple"));
}

#[test]
fn cli_fragment_apply_delete_itinerary_conflict_confirm_blocks_db_write() {
    let workspace = common::TestWorkspace::new();
    let dir = workspace.path();
    let fragment = fixture_path("apply-delete-itinerary-conflict-fragment.json");
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
    assert!(!output.status.success());
    let list = run_cli_in(&dir, &["itinerary", "list", "1"]);
    assert!(list.status.success());
    assert!(String::from_utf8_lossy(&list.stdout).contains("Morning temple"));
}

#[test]
fn cli_fragment_apply_delete_itinerary_expense_confirm_blocks_db_write() {
    let workspace = common::TestWorkspace::new();
    let dir = workspace.path();
    let fragment = fixture_path("apply-delete-itinerary-basic-fragment.json");
    seed_trip_with_itinerary(&dir);
    assert!(run_cli_in(
        &dir,
        &[
            "expense",
            "add",
            "--itinerary",
            "1",
            "--amount",
            "500",
            "--currency",
            "JPY",
            "--title",
            "Temple admission",
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
            "--json",
        ],
    );
    assert!(!output.status.success());
    let parsed: serde_json::Value =
        serde_json::from_str(&String::from_utf8_lossy(&output.stdout)).expect("json report");
    assert_eq!(parsed["preview"]["action"], "delete_itinerary");
    assert_eq!(
        parsed["preview"]["delete_preview"]["blocking_children"]["expenses"],
        1
    );
    assert_eq!(itinerary_expense_count(&dir, "1"), 1);
    let list = run_cli_in(&dir, &["itinerary", "list", "1"]);
    assert!(list.status.success());
    assert!(String::from_utf8_lossy(&list.stdout).contains("Morning temple"));
}

#[test]
fn cli_fragment_apply_delete_itinerary_reservation_confirm_blocks_db_write() {
    let workspace = common::TestWorkspace::new();
    let dir = workspace.path();
    let fragment = fixture_path("apply-delete-itinerary-basic-fragment.json");
    seed_trip_with_itinerary(&dir);
    assert!(run_cli_in(
        &dir,
        &[
            "reservation",
            "add",
            "--itinerary",
            "1",
            "--reservation-type",
            "hotel",
            "--provider",
            "Example Hotel",
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
            "--json",
        ],
    );
    assert!(!output.status.success());
    let parsed: serde_json::Value =
        serde_json::from_str(&String::from_utf8_lossy(&output.stdout)).expect("json report");
    assert_eq!(parsed["preview"]["action"], "delete_itinerary");
    assert_eq!(
        parsed["preview"]["delete_preview"]["blocking_children"]["reservations"],
        1
    );
    assert_eq!(itinerary_reservation_count(&dir, "1"), 1);
}

#[test]
fn cli_fragment_apply_delete_itinerary_note_confirm_blocks_db_write() {
    let workspace = common::TestWorkspace::new();
    let dir = workspace.path();
    let fragment = fixture_path("apply-delete-itinerary-basic-fragment.json");
    let note_fragment = fixture_path("apply-add-note-itinerary-fragment.json");
    seed_trip_with_itinerary(&dir);
    assert!(run_cli_in(
        &dir,
        &[
            "fragment",
            "apply",
            note_fragment.to_str().unwrap(),
            "--confirm",
            "--trip",
            "1",
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
            "--json",
        ],
    );
    assert!(!output.status.success());
    let parsed: serde_json::Value =
        serde_json::from_str(&String::from_utf8_lossy(&output.stdout)).expect("json report");
    assert_eq!(parsed["preview"]["action"], "delete_itinerary");
    assert_eq!(
        parsed["preview"]["delete_preview"]["blocking_children"]["notes"],
        1
    );
    let list = run_cli_in(&dir, &["itinerary", "list", "1"]);
    assert!(list.status.success());
    assert!(String::from_utf8_lossy(&list.stdout).contains("Morning temple"));
}

#[test]
fn cli_fragment_apply_delete_itinerary_estimate_confirm_blocks_db_write() {
    let workspace = common::TestWorkspace::new();
    let dir = workspace.path();
    let fragment = fixture_path("apply-delete-itinerary-basic-fragment.json");
    seed_trip_with_itinerary(&dir);
    assert!(run_cli_in(
        &dir,
        &[
            "estimate",
            "add",
            "--itinerary",
            "1",
            "--amount",
            "1200",
            "--currency",
            "JPY",
            "--title",
            "Breakfast",
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
            "--json",
        ],
    );
    assert!(!output.status.success());
    let parsed: serde_json::Value =
        serde_json::from_str(&String::from_utf8_lossy(&output.stdout)).expect("json report");
    assert_eq!(parsed["preview"]["action"], "delete_itinerary");
    assert_eq!(
        parsed["preview"]["delete_preview"]["blocking_children"]["estimates"],
        1
    );
    let list = run_cli_in(&dir, &["itinerary", "list", "1"]);
    assert!(list.status.success());
    assert!(String::from_utf8_lossy(&list.stdout).contains("Morning temple"));
}

#[test]
fn cli_fragment_apply_delete_itinerary_unresolved_target_confirm_blocks_db_write() {
    let workspace = common::TestWorkspace::new();
    let dir = workspace.path();
    let fragment = fixture_path("apply-delete-itinerary-unresolved-target-fragment.json");
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
    assert!(!output.status.success());
    let list = run_cli_in(&dir, &["itinerary", "list", "1"]);
    assert!(list.status.success());
    assert!(String::from_utf8_lossy(&list.stdout).contains("Morning temple"));
}

#[test]
fn cli_fragment_apply_delete_itinerary_ambiguous_target_confirm_blocks_db_write() {
    let workspace = common::TestWorkspace::new();
    let dir = workspace.path();
    let fragment = fixture_path("apply-delete-itinerary-ambiguous-fragment.json");

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
    let list = run_cli_in(&dir, &["itinerary", "list", "1"]);
    assert!(list.status.success());
    assert_eq!(
        String::from_utf8_lossy(&list.stdout)
            .matches("Morning temple")
            .count(),
        2
    );
}

#[test]
fn cli_fragment_apply_delete_itinerary_not_found_confirm_blocks_db_write() {
    let workspace = common::TestWorkspace::new();
    let dir = workspace.path();
    let fragment = fixture_path("apply-delete-itinerary-not-found-fragment.json");
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
    assert!(!output.status.success());
    let list = run_cli_in(&dir, &["itinerary", "list", "1"]);
    assert!(list.status.success());
    assert!(String::from_utf8_lossy(&list.stdout).contains("Morning temple"));
}

#[test]
fn cli_fragment_apply_delete_itinerary_inline_note_confirm_succeeds() {
    let workspace = common::TestWorkspace::new();
    let dir = workspace.path();
    let fragment = fixture_path("apply-delete-itinerary-basic-fragment.json");
    seed_trip_with_itinerary(&dir);
    assert!(run_cli_in(
        &dir,
        &["itinerary", "update", "1", "--note", "Inline memo only"]
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
            "--json",
        ],
    );
    assert!(output.status.success());
    let parsed: serde_json::Value =
        serde_json::from_str(&String::from_utf8_lossy(&output.stdout)).expect("json report");
    assert_eq!(parsed["deleted_itinerary_id"], 1);
    let show = run_cli_in(&dir, &["itinerary", "show", "1"]);
    assert!(!show.status.success());
}

#[test]
fn cli_fragment_validate_remains_file_only() {
    let fragment = fixture_path("valid-fragment.json");
    let workspace = common::TestWorkspace::new();
    let output = common::run_cli_in(
        workspace.path(),
        &["fragment", "validate", fragment.to_str().unwrap()],
    );
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}
