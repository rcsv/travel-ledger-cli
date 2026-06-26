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
    let dir = std::env::temp_dir().join(format!("travel-ledger-cli-itinerary-{n}"));
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
            "Okinawa Trip",
            "--start",
            "2026-04-26",
            "--end",
            "2026-04-28",
        ],
    )
    .status
    .success());
}

#[test]
fn cli_itinerary_add_appends_to_day_end() {
    let dir = temp_workdir();
    setup_trip(&dir);

    assert!(
        run_cli(&dir, &["itinerary", "add", "1", "--day", "1", "First"],)
            .status
            .success()
    );
    let second = run_cli(&dir, &["itinerary", "add", "1", "--day", "1", "Second"]);
    assert!(second.status.success());
    let stdout = String::from_utf8_lossy(&second.stdout);
    assert!(stdout.contains("並び順  : 2000"));

    let list = run_cli(&dir, &["itinerary", "list", "1"]);
    assert!(list.status.success());
    let list_out = String::from_utf8_lossy(&list.stdout);
    assert!(list_out.contains("1000"));
    assert!(list_out.contains("2000"));
}

#[test]
fn cli_itinerary_add_respects_explicit_order() {
    let dir = temp_workdir();
    setup_trip(&dir);

    let output = run_cli(
        &dir,
        &[
            "itinerary",
            "add",
            "1",
            "--day",
            "1",
            "--order",
            "42",
            "Custom",
        ],
    );
    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("並び順  : 42"));
}

#[test]
fn cli_itinerary_add_after_inserts_midpoint() {
    let dir = temp_workdir();
    setup_trip(&dir);

    assert!(run_cli(
        &dir,
        &[
            "itinerary",
            "add",
            "1",
            "--day",
            "1",
            "--order",
            "1000",
            "空港",
        ],
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
            "--order",
            "2000",
            "搭乗",
        ],
    )
    .status
    .success());
    let wifi = run_cli(
        &dir,
        &[
            "itinerary",
            "add",
            "1",
            "--day",
            "1",
            "--after",
            "1",
            "Wi-Fi",
        ],
    );
    assert!(wifi.status.success());
    assert!(String::from_utf8_lossy(&wifi.stdout).contains("並び順  : 1500"));

    let list = run_cli(&dir, &["itinerary", "list", "1", "--json"]);
    assert!(list.status.success());
    let parsed: serde_json::Value = serde_json::from_slice(&list.stdout).expect("valid json");
    let titles: Vec<_> = parsed
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v["title"].as_str().unwrap())
        .collect();
    assert_eq!(titles, vec!["空港", "Wi-Fi", "搭乗"]);
}

#[test]
fn cli_itinerary_add_before_inserts_midpoint() {
    let dir = temp_workdir();
    setup_trip(&dir);

    assert!(run_cli(
        &dir,
        &[
            "itinerary",
            "add",
            "1",
            "--day",
            "1",
            "--order",
            "1000",
            "空港",
        ],
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
            "--order",
            "2000",
            "搭乗",
        ],
    )
    .status
    .success());
    let wifi = run_cli(
        &dir,
        &[
            "itinerary",
            "add",
            "1",
            "--day",
            "1",
            "--before",
            "2",
            "Wi-Fi",
        ],
    );
    assert!(wifi.status.success());
    assert!(String::from_utf8_lossy(&wifi.stdout).contains("並び順  : 1500"));
}

#[test]
fn cli_itinerary_add_rejects_after_from_other_day() {
    let dir = temp_workdir();
    setup_trip(&dir);

    assert!(
        run_cli(&dir, &["itinerary", "add", "1", "--day", "2", "Day2 item"],)
            .status
            .success()
    );
    let output = run_cli(
        &dir,
        &[
            "itinerary",
            "add",
            "1",
            "--day",
            "1",
            "--after",
            "1",
            "Wi-Fi",
        ],
    );
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Day 1"));
}

#[test]
fn cli_itinerary_add_rejects_order_with_after() {
    let dir = temp_workdir();
    setup_trip(&dir);

    assert!(
        run_cli(&dir, &["itinerary", "add", "1", "--day", "1", "Anchor"],)
            .status
            .success()
    );
    let output = run_cli(
        &dir,
        &[
            "itinerary",
            "add",
            "1",
            "--day",
            "1",
            "--order",
            "500",
            "--after",
            "1",
            "Wi-Fi",
        ],
    );
    assert!(!output.status.success());
}

#[test]
fn cli_itinerary_normalize_rebalances_day() {
    let dir = temp_workdir();
    setup_trip(&dir);

    assert!(
        run_cli(&dir, &["itinerary", "add", "1", "--day", "1", "A"],)
            .status
            .success()
    );
    assert!(
        run_cli(&dir, &["itinerary", "add", "1", "--day", "1", "B"],)
            .status
            .success()
    );
    assert!(
        run_cli(&dir, &["itinerary", "add", "1", "--day", "1", "C"],)
            .status
            .success()
    );

    let output = run_cli(&dir, &["itinerary", "normalize", "1", "--day", "1"]);
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("1000"));
    assert!(stdout.contains("2000"));
    assert!(stdout.contains("3000"));
}

#[test]
fn cli_itinerary_add_before_first_with_legacy_sort_order_zero() {
    let dir = temp_workdir();
    setup_trip(&dir);

    assert!(run_cli(
        &dir,
        &[
            "itinerary",
            "add",
            "1",
            "--day",
            "1",
            "--order",
            "0",
            "First",
        ],
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
            "--order",
            "0",
            "Second",
        ],
    )
    .status
    .success());

    let prep = run_cli(
        &dir,
        &[
            "itinerary",
            "add",
            "1",
            "--day",
            "1",
            "--before",
            "1",
            "Prep",
        ],
    );
    assert!(
        prep.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&prep.stderr)
    );

    let list = run_cli(&dir, &["itinerary", "list", "1", "--json"]);
    assert!(list.status.success());
    let parsed: serde_json::Value = serde_json::from_slice(&list.stdout).expect("valid json");
    let titles: Vec<_> = parsed
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v["title"].as_str().unwrap())
        .collect();
    assert_eq!(titles, vec!["Prep", "First", "Second"]);
}

#[test]
fn cli_itinerary_move_rejects_self_reference() {
    let dir = temp_workdir();
    setup_trip(&dir);

    assert!(
        run_cli(&dir, &["itinerary", "add", "1", "--day", "1", "Only"],)
            .status
            .success()
    );

    let after_self = run_cli(&dir, &["itinerary", "move", "1", "--after", "1"]);
    assert!(!after_self.status.success());
    assert!(String::from_utf8_lossy(&after_self.stderr).contains("自分自身"));

    let before_self = run_cli(&dir, &["itinerary", "move", "1", "--before", "1"]);
    assert!(!before_self.status.success());
    assert!(String::from_utf8_lossy(&before_self.stderr).contains("自分自身"));
}

fn setup_week_trip(dir: &std::path::Path) {
    assert!(run_cli(dir, &["db", "reset"]).status.success());
    assert!(run_cli(
        dir,
        &[
            "trip",
            "add",
            "Week Trip",
            "--start",
            "2026-04-26",
            "--end",
            "2026-05-02",
        ],
    )
    .status
    .success());
}

#[test]
fn cli_itinerary_replicate_copies_pattern_to_multiple_days() {
    let dir = temp_workdir();
    setup_week_trip(&dir);

    for (title, order) in [
        ("Hotel breakfast", "1000"),
        ("Leave hotel", "2000"),
        ("Return to hotel", "7000"),
        ("Lounge dinner", "8000"),
    ] {
        assert!(run_cli(
            &dir,
            &[
                "itinerary",
                "add",
                "1",
                "--day",
                "2",
                "--order",
                order,
                title,
            ],
        )
        .status
        .success());
    }

    let output = run_cli(
        &dir,
        &[
            "itinerary",
            "replicate",
            "--items",
            "1,2,3,4",
            "--to-days",
            "3-5",
        ],
    );
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Itineraries replicated."));
    assert!(stdout.contains("Source Day: 2"));
    assert!(stdout.contains("Day 3: 4 items"));
    assert!(stdout.contains("Total: 12 items"));
    assert!(stdout.contains("Day 5:"));

    let day3 = run_cli(&dir, &["itinerary", "list", "1"]);
    assert!(day3.status.success());
    let list_out = String::from_utf8_lossy(&day3.stdout);
    assert_eq!(list_out.matches("Hotel breakfast").count(), 4);
}

#[test]
fn cli_itinerary_replicate_rejects_source_day_in_targets() {
    let dir = temp_workdir();
    setup_week_trip(&dir);
    assert!(
        run_cli(&dir, &["itinerary", "add", "1", "--day", "2", "Breakfast"])
            .status
            .success()
    );

    let output = run_cli(
        &dir,
        &["itinerary", "replicate", "--items", "1", "--to-days", "2-4"],
    );
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("source Day"));
}

#[test]
fn cli_itinerary_replicate_dry_run_does_not_create_items() {
    let dir = temp_workdir();
    setup_week_trip(&dir);
    assert!(
        run_cli(&dir, &["itinerary", "add", "1", "--day", "2", "Breakfast"])
            .status
            .success()
    );

    let output = run_cli(
        &dir,
        &[
            "itinerary",
            "replicate",
            "--items",
            "1",
            "--to-days",
            "3",
            "--dry-run",
        ],
    );
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Dry run"));
    assert!(stdout.contains("Would create:"));

    let list = run_cli(&dir, &["itinerary", "list", "1"]);
    assert!(list.status.success());
    assert_eq!(
        String::from_utf8_lossy(&list.stdout)
            .matches("Breakfast")
            .count(),
        1
    );
}

#[test]
fn cli_itinerary_replicate_copies_estimates() {
    let dir = temp_workdir();
    setup_week_trip(&dir);
    assert!(
        run_cli(&dir, &["itinerary", "add", "1", "--day", "2", "Breakfast"])
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
            "1400",
            "--currency",
            "JPY",
            "--title",
            "朝食代",
            "--note",
            "2名分",
            "--sort-order",
            "10",
        ],
    )
    .status
    .success());

    let output = run_cli(
        &dir,
        &["itinerary", "replicate", "--items", "1", "--to-days", "3,4"],
    );
    assert!(output.status.success());

    let list = run_cli(&dir, &["estimate", "list", "--trip", "1"]);
    assert!(
        list.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&list.stderr)
    );
    let stdout = String::from_utf8_lossy(&list.stdout);
    assert_eq!(stdout.matches("朝食代").count(), 3, "stdout: {stdout}");
    assert_eq!(stdout.matches("1,400").count(), 3);
    assert_eq!(
        stdout.matches("2名分").count(),
        0,
        "note is not shown in list view"
    );
}

#[test]
fn cli_itinerary_replicate_dry_run_does_not_create_estimates() {
    let dir = temp_workdir();
    setup_week_trip(&dir);
    assert!(
        run_cli(&dir, &["itinerary", "add", "1", "--day", "2", "Breakfast"])
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
            "1400",
            "--currency",
            "JPY",
        ],
    )
    .status
    .success());

    let before = run_cli(&dir, &["estimate", "list", "--trip", "1"]);
    assert!(before.status.success());
    let before_stdout = String::from_utf8_lossy(&before.stdout);
    assert_eq!(
        before_stdout.matches("1,400").count(),
        1,
        "before: {before_stdout}"
    );

    assert!(run_cli(
        &dir,
        &[
            "itinerary",
            "replicate",
            "--items",
            "1",
            "--to-days",
            "3",
            "--dry-run",
        ],
    )
    .status
    .success());

    let after = run_cli(&dir, &["estimate", "list", "--trip", "1"]);
    assert!(after.status.success());
    let after_stdout = String::from_utf8_lossy(&after.stdout);
    assert_eq!(
        after_stdout.matches("1,400").count(),
        1,
        "after: {after_stdout}"
    );
}
