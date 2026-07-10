mod common;

fn setup_trip(dir: &std::path::Path) {
    assert!(common::run_cli_in(dir, &["db", "reset"]).status.success());
    assert!(common::run_cli_in(
        dir,
        &[
            "trip",
            "add",
            "Okinawa Family Trip",
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
fn cli_day_list_shows_days_with_dates() {
    let workspace = common::TestWorkspace::new();
    let dir = workspace.path();
    setup_trip(&dir);

    let output = common::run_cli_in(&dir, &["day", "list", "1"]);
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Trip: Okinawa Family Trip"));
    assert!(stdout.contains("Day 1  2026-04-26"));
    assert!(stdout.contains("Day 4  2026-04-29"));
}

#[test]
fn cli_day_show_lists_itineraries_for_day() {
    let workspace = common::TestWorkspace::new();
    let dir = workspace.path();
    setup_trip(&dir);
    assert!(common::run_cli_in(
        &dir,
        &[
            "itinerary",
            "add",
            "1",
            "--day",
            "2",
            "--time",
            "09:00",
            "美ら海水族館",
        ],
    )
    .status
    .success());
    assert!(common::run_cli_in(
        &dir,
        &[
            "itinerary",
            "add",
            "1",
            "--day",
            "2",
            "--time",
            "13:00",
            "海邦丸"
        ],
    )
    .status
    .success());

    let output = common::run_cli_in(&dir, &["day", "show", "1", "2"]);
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Day 2"));
    assert!(stdout.contains("Date: 2026-04-27"));
    assert!(stdout.contains("Itineraries:"));
    assert!(stdout.contains("- 09:00 美ら海水族館"));
    assert!(stdout.contains("- 13:00 海邦丸"));
}

#[test]
fn cli_day_show_empty_day() {
    let workspace = common::TestWorkspace::new();
    let dir = workspace.path();
    setup_trip(&dir);

    let output = common::run_cli_in(&dir, &["day", "show", "1", "3"]);
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Day 3"));
    assert!(stdout.contains("Date: 2026-04-28"));
    assert!(stdout.contains("Itineraries:"));
    assert!(!stdout.contains("- "));
}

#[test]
fn cli_day_show_rejects_invalid_day_number() {
    let workspace = common::TestWorkspace::new();
    let dir = workspace.path();
    setup_trip(&dir);

    let output = common::run_cli_in(&dir, &["day", "show", "1", "99"]);
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Day not found: trip 1 day 99"));
}

#[test]
fn cli_day_swap_exchanges_itineraries() {
    let workspace = common::TestWorkspace::new();
    let dir = workspace.path();
    setup_trip(&dir);
    assert!(common::run_cli_in(
        &dir,
        &[
            "itinerary",
            "add",
            "1",
            "--day",
            "2",
            "--time",
            "09:00",
            "Day2 Plan"
        ],
    )
    .status
    .success());
    assert!(common::run_cli_in(
        &dir,
        &[
            "itinerary",
            "add",
            "1",
            "--day",
            "3",
            "--time",
            "10:00",
            "Day3 Plan"
        ],
    )
    .status
    .success());

    let output = common::run_cli_in(&dir, &["day", "swap", "1", "2", "3"]);
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Day 2 と Day 3 の計画内容を入れ替えました"));

    let day2 = common::run_cli_in(&dir, &["day", "show", "1", "2"]);
    let day3 = common::run_cli_in(&dir, &["day", "show", "1", "3"]);
    assert!(String::from_utf8_lossy(&day2.stdout).contains("Day3 Plan"));
    assert!(String::from_utf8_lossy(&day3.stdout).contains("Day2 Plan"));
}

#[test]
fn cli_day_swap_exchanges_plan_payload() {
    let workspace = common::TestWorkspace::new();
    let dir = workspace.path();
    setup_trip(&dir);
    assert!(common::run_cli_in(
        &dir,
        &[
            "day",
            "update",
            "1",
            "2",
            "--summary",
            "美ら海水族館を中心に回る",
        ],
    )
    .status
    .success());
    assert!(common::run_cli_in(
        &dir,
        &[
            "day",
            "update",
            "1",
            "3",
            "--summary",
            "瀬底ビーチでゆっくりする",
        ],
    )
    .status
    .success());
    assert!(common::run_cli_in(
        &dir,
        &[
            "note",
            "add",
            "--trip",
            "1",
            "--day",
            "2",
            "--body",
            "午後は無理しない",
        ],
    )
    .status
    .success());
    assert!(common::run_cli_in(
        &dir,
        &[
            "note",
            "add",
            "--trip",
            "1",
            "--day",
            "3",
            "--body",
            "天気が悪ければ室内案",
        ],
    )
    .status
    .success());
    assert!(
        common::run_cli_in(&dir, &["note", "add", "--trip", "1", "--body", "trip note"],)
            .status
            .success()
    );
    assert!(common::run_cli_in(
        &dir,
        &[
            "itinerary",
            "add",
            "1",
            "--day",
            "2",
            "--time",
            "09:00",
            "美ら海水族館",
        ],
    )
    .status
    .success());
    assert!(common::run_cli_in(
        &dir,
        &[
            "itinerary",
            "add",
            "1",
            "--day",
            "3",
            "--time",
            "10:00",
            "瀬底ビーチ",
        ],
    )
    .status
    .success());

    assert!(common::run_cli_in(&dir, &["day", "swap", "1", "2", "3"])
        .status
        .success());

    let day2 = common::run_cli_in(&dir, &["day", "show", "1", "2", "--json"]);
    let day3 = common::run_cli_in(&dir, &["day", "show", "1", "3", "--json"]);
    let day2_json: serde_json::Value = serde_json::from_slice(&day2.stdout).unwrap();
    let day3_json: serde_json::Value = serde_json::from_slice(&day3.stdout).unwrap();
    assert_eq!(day2_json["date"], "2026-04-27");
    assert_eq!(day3_json["date"], "2026-04-28");
    assert_eq!(day2_json["summary"], "瀬底ビーチでゆっくりする");
    assert_eq!(day3_json["summary"], "美ら海水族館を中心に回る");
    assert_eq!(day2_json["itineraries"][0]["title"], "瀬底ビーチ");
    assert_eq!(day3_json["itineraries"][0]["title"], "美ら海水族館");

    let day2_notes = common::run_cli_in(
        &dir,
        &["note", "list", "--trip", "1", "--day", "2", "--json"],
    );
    let day3_notes = common::run_cli_in(
        &dir,
        &["note", "list", "--trip", "1", "--day", "3", "--json"],
    );
    let day2_notes_json: serde_json::Value = serde_json::from_slice(&day2_notes.stdout).unwrap();
    let day3_notes_json: serde_json::Value = serde_json::from_slice(&day3_notes.stdout).unwrap();
    assert_eq!(day2_notes_json["notes"][0]["body"], "天気が悪ければ室内案");
    assert_eq!(day3_notes_json["notes"][0]["body"], "午後は無理しない");

    let trip_notes = common::run_cli_in(&dir, &["note", "list", "--trip", "1", "--json"]);
    let trip_notes_json: serde_json::Value = serde_json::from_slice(&trip_notes.stdout).unwrap();
    assert_eq!(trip_notes_json["notes"][0]["body"], "trip note");
}

#[test]
fn cli_day_swap_rejects_same_day() {
    let workspace = common::TestWorkspace::new();
    let dir = workspace.path();
    setup_trip(&dir);
    assert!(
        common::run_cli_in(&dir, &["itinerary", "add", "1", "--day", "2", "Plan"],)
            .status
            .success()
    );

    let output = common::run_cli_in(&dir, &["day", "swap", "1", "2", "2"]);
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("同じ Day"));
}

#[test]
fn cli_day_list_json() {
    let workspace = common::TestWorkspace::new();
    let dir = workspace.path();
    setup_trip(&dir);

    let output = common::run_cli_in(&dir, &["day", "list", "1", "--json"]);
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let parsed: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("stdout should be valid JSON");
    assert_eq!(parsed["trip_name"], "Okinawa Family Trip");
    assert_eq!(parsed["days"].as_array().unwrap().len(), 4);
    assert_eq!(parsed["days"][1]["date"], "2026-04-27");
}

#[test]
fn cli_day_show_json() {
    let workspace = common::TestWorkspace::new();
    let dir = workspace.path();
    setup_trip(&dir);
    assert!(common::run_cli_in(
        &dir,
        &[
            "itinerary",
            "add",
            "1",
            "--day",
            "2",
            "--time",
            "09:00",
            "Museum"
        ],
    )
    .status
    .success());

    let output = common::run_cli_in(&dir, &["day", "show", "1", "2", "--json"]);
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let parsed: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("stdout should be valid JSON");
    assert_eq!(parsed["day_number"], 2);
    assert_eq!(parsed["date"], "2026-04-27");
    assert_eq!(parsed["itineraries"].as_array().unwrap().len(), 1);
}
