mod common;

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
    let workspace = common::TestWorkspace::new();
    let output = common::run_cli_in(workspace.path(), &["trip", "show", "9999"]);
    assert_not_found(&output, "Trip not found: 9999");
}

#[test]
fn cli_trip_show_not_found_json() {
    let workspace = common::TestWorkspace::new();
    let output = common::run_cli_in(workspace.path(), &["trip", "show", "9999", "--json"]);
    assert_not_found(&output, "Trip not found: 9999");
}

#[test]
fn cli_itinerary_list_not_found() {
    let workspace = common::TestWorkspace::new();
    let output = common::run_cli_in(workspace.path(), &["itinerary", "list", "9999"]);
    assert_not_found(&output, "Trip not found: 9999");
}

#[test]
fn cli_itinerary_show_not_found() {
    let workspace = common::TestWorkspace::new();
    let output = common::run_cli_in(workspace.path(), &["itinerary", "show", "9999"]);
    assert_not_found(&output, "Itinerary not found: 9999");
}

#[test]
fn cli_itinerary_show_not_found_json() {
    let workspace = common::TestWorkspace::new();
    let output = common::run_cli_in(workspace.path(), &["itinerary", "show", "9999", "--json"]);
    assert_not_found(&output, "Itinerary not found: 9999");
}

#[test]
fn cli_checklist_show_not_found() {
    let workspace = common::TestWorkspace::new();
    let output = common::run_cli_in(workspace.path(), &["checklist", "show", "9999"]);
    assert_not_found(&output, "Checklist item not found: 9999");
}

#[test]
fn cli_checklist_list_not_found_json() {
    let workspace = common::TestWorkspace::new();
    let output = common::run_cli_in(workspace.path(), &["checklist", "list", "9999", "--json"]);
    assert_not_found(&output, "Trip not found: 9999");
}

#[test]
fn cli_trip_stats_not_found_json() {
    let workspace = common::TestWorkspace::new();
    let output = common::run_cli_in(workspace.path(), &["trip", "stats", "9999", "--json"]);
    assert_not_found(&output, "Trip not found: 9999");
}

#[test]
fn cli_trip_doctor_not_found() {
    let workspace = common::TestWorkspace::new();
    let output = common::run_cli_in(workspace.path(), &["trip", "doctor", "9999"]);
    assert_not_found(&output, "Trip not found: 9999");
}

#[test]
fn cli_day_list_not_found() {
    let workspace = common::TestWorkspace::new();
    let output = common::run_cli_in(workspace.path(), &["day", "list", "9999"]);
    assert_not_found(&output, "Trip not found: 9999");
}

#[test]
fn cli_day_show_not_found_json() {
    let workspace = common::TestWorkspace::new();
    let output = common::run_cli_in(workspace.path(), &["day", "show", "9999", "1", "--json"]);
    assert_not_found(&output, "Trip not found: 9999");
}

#[test]
fn cli_note_show_not_found() {
    let workspace = common::TestWorkspace::new();
    let output = common::run_cli_in(workspace.path(), &["note", "show", "9999"]);
    assert_not_found(&output, "Note not found: 9999");
}

#[test]
fn cli_note_show_not_found_json() {
    let workspace = common::TestWorkspace::new();
    let output = common::run_cli_in(workspace.path(), &["note", "show", "9999", "--json"]);
    assert_not_found(&output, "Note not found: 9999");
}

#[test]
fn cli_participant_show_not_found() {
    let workspace = common::TestWorkspace::new();
    let output = common::run_cli_in(workspace.path(), &["participant", "show", "9999"]);
    assert_not_found(&output, "participant not found: 9999");
}

#[test]
fn cli_participant_list_not_found() {
    let workspace = common::TestWorkspace::new();
    let output = common::run_cli_in(workspace.path(), &["participant", "list", "--trip", "9999"]);
    assert_not_found(&output, "Trip not found: 9999");
}
