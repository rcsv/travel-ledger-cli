mod common;

fn setup_trip_with_itinerary(dir: &std::path::Path) {
    assert!(common::run_cli_in(dir, &["db", "reset"]).status.success());
    assert!(common::run_cli_in(
        dir,
        &[
            "trip",
            "add",
            "Expense Trip",
            "--start",
            "2026-04-26",
            "--end",
            "2026-04-29",
        ],
    )
    .status
    .success());
    assert!(
        common::run_cli_in(dir, &["itinerary", "add", "1", "--day", "1", "Lunch"],)
            .status
            .success()
    );
}

fn setup_trip_with_itinerary_and_participants(dir: &std::path::Path) -> (i64, i64) {
    setup_trip_with_itinerary(dir);
    assert!(common::run_cli_in(
        dir,
        &[
            "participant",
            "add",
            "--trip",
            "1",
            "--name",
            "Alice",
            "--self"
        ],
    )
    .status
    .success());
    assert!(
        common::run_cli_in(dir, &["participant", "add", "--trip", "1", "--name", "Bob"],)
            .status
            .success()
    );
    (1, 1) // trip_id, itinerary_id
}

#[test]
fn cli_expense_add_with_paid_by_participant_and_beneficiaries() {
    let workspace = common::TestWorkspace::new();
    let dir = workspace.path();
    setup_trip_with_itinerary_and_participants(&dir);

    let output = common::run_cli_in(
        &dir,
        &[
            "expense",
            "add",
            "--itinerary",
            "1",
            "--amount",
            "4000",
            "--currency",
            "JPY",
            "--title",
            "Dinner",
            "--paid-by-participant",
            "Alice",
            "--beneficiary",
            "Alice",
            "--beneficiary",
            "Bob",
        ],
    );
    assert!(output.status.success());
    let show: serde_json::Value = serde_json::from_slice(
        &common::run_cli_in(&dir, &["expense", "show", "1", "--json"]).stdout,
    )
    .unwrap();
    assert_eq!(show["paid_by_participant_name"], "Alice");
    assert_eq!(show["shared"], true);
    assert_eq!(show["beneficiaries"].as_array().unwrap().len(), 2);
}

#[test]
fn cli_expense_add_shared_with_all() {
    let workspace = common::TestWorkspace::new();
    let dir = workspace.path();
    setup_trip_with_itinerary_and_participants(&dir);

    let output = common::run_cli_in(
        &dir,
        &[
            "expense",
            "add",
            "--itinerary",
            "1",
            "--amount",
            "3000",
            "--currency",
            "JPY",
            "--shared-with",
            "all",
        ],
    );
    assert!(output.status.success());
    let show: serde_json::Value = serde_json::from_slice(
        &common::run_cli_in(&dir, &["expense", "show", "1", "--json"]).stdout,
    )
    .unwrap();
    assert_eq!(show["beneficiaries"].as_array().unwrap().len(), 2);
}

#[test]
fn cli_expense_rejects_structured_without_participants() {
    let workspace = common::TestWorkspace::new();
    let dir = workspace.path();
    setup_trip_with_itinerary(&dir);

    let output = common::run_cli_in(
        &dir,
        &[
            "expense",
            "add",
            "--itinerary",
            "1",
            "--amount",
            "1000",
            "--currency",
            "JPY",
            "--paid-by-participant",
            "Alice",
        ],
    );
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr)
        .contains("no participants registered for this trip"));
}

#[test]
fn cli_expense_rejects_shared_with_and_beneficiary_on_add() {
    let workspace = common::TestWorkspace::new();
    let dir = workspace.path();
    setup_trip_with_itinerary_and_participants(&dir);

    let output = common::run_cli_in(
        &dir,
        &[
            "expense",
            "add",
            "--itinerary",
            "1",
            "--amount",
            "1000",
            "--currency",
            "JPY",
            "--shared-with",
            "all",
            "--beneficiary",
            "Alice",
        ],
    );
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr)
        .contains("cannot combine --shared-with and --beneficiary"));
}

#[test]
fn cli_expense_rejects_shared_with_and_beneficiary_on_update() {
    let workspace = common::TestWorkspace::new();
    let dir = workspace.path();
    setup_trip_with_itinerary_and_participants(&dir);
    assert!(common::run_cli_in(
        &dir,
        &[
            "expense",
            "add",
            "--itinerary",
            "1",
            "--amount",
            "1000",
            "--currency",
            "JPY",
        ],
    )
    .status
    .success());

    let output = common::run_cli_in(
        &dir,
        &[
            "expense",
            "update",
            "1",
            "--shared-with",
            "all",
            "--beneficiary",
            "Bob",
        ],
    );
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr)
        .contains("cannot combine --shared-with and --beneficiary"));
}

#[test]
fn cli_expense_update_clear_beneficiaries() {
    let workspace = common::TestWorkspace::new();
    let dir = workspace.path();
    setup_trip_with_itinerary_and_participants(&dir);
    assert!(common::run_cli_in(
        &dir,
        &[
            "expense",
            "add",
            "--itinerary",
            "1",
            "--amount",
            "1000",
            "--currency",
            "JPY",
            "--beneficiary",
            "Alice",
        ],
    )
    .status
    .success());

    let update = common::run_cli_in(&dir, &["expense", "update", "1", "--clear-beneficiaries"]);
    assert!(update.status.success());
    let show: serde_json::Value = serde_json::from_slice(
        &common::run_cli_in(&dir, &["expense", "show", "1", "--json"]).stdout,
    )
    .unwrap();
    assert_eq!(show["shared"], false);
    assert!(show["beneficiaries"].as_array().unwrap().is_empty());
}

#[test]
fn cli_expense_add_and_show() {
    let workspace = common::TestWorkspace::new();
    let dir = workspace.path();
    setup_trip_with_itinerary(&dir);

    let output = common::run_cli_in(
        &dir,
        &[
            "expense",
            "add",
            "--itinerary",
            "1",
            "--amount",
            "2200",
            "--currency",
            "JPY",
            "--title",
            "Lunch",
            "--paid-by-name",
            "Tomo",
            "--expense-date",
            "2026-04-27",
        ],
    );
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Expense を追加しました"));
    assert!(stdout.contains("2,200 JPY"));
    assert!(stdout.contains("Tomo"));
    assert!(stdout.contains("2026-04-27"));

    let show = common::run_cli_in(&dir, &["expense", "show", "1"]);
    assert!(show.status.success());
    let show_stdout = String::from_utf8_lossy(&show.stdout);
    assert!(show_stdout.contains("Lunch"));
}

#[test]
fn cli_expense_add_usd_decimal() {
    let workspace = common::TestWorkspace::new();
    let dir = workspace.path();
    setup_trip_with_itinerary(&dir);

    let output = common::run_cli_in(
        &dir,
        &[
            "expense",
            "add",
            "--itinerary",
            "1",
            "--amount",
            "12.50",
            "--currency",
            "usd",
            "--title",
            "Coffee",
        ],
    );
    assert!(output.status.success());
    let show: serde_json::Value = serde_json::from_slice(
        &common::run_cli_in(&dir, &["expense", "show", "1", "--json"]).stdout,
    )
    .unwrap();
    assert_eq!(show["amount"], 1250);
    assert_eq!(show["currency"], "USD");
}

#[test]
fn cli_expense_list_by_itinerary_and_trip() {
    let workspace = common::TestWorkspace::new();
    let dir = workspace.path();
    setup_trip_with_itinerary(&dir);
    assert!(common::run_cli_in(
        &dir,
        &[
            "expense",
            "add",
            "--itinerary",
            "1",
            "--amount",
            "100",
            "--currency",
            "JPY",
        ],
    )
    .status
    .success());

    let by_itinerary = common::run_cli_in(&dir, &["expense", "list", "--itinerary", "1"]);
    assert!(by_itinerary.status.success());
    assert!(String::from_utf8_lossy(&by_itinerary.stdout).contains("100 JPY"));

    let by_trip = common::run_cli_in(&dir, &["expense", "list", "--trip", "1"]);
    assert!(by_trip.status.success());
    assert!(String::from_utf8_lossy(&by_trip.stdout).contains("100 JPY"));
}

#[test]
fn cli_expense_update_and_delete() {
    let workspace = common::TestWorkspace::new();
    let dir = workspace.path();
    setup_trip_with_itinerary(&dir);
    assert!(common::run_cli_in(
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
        ],
    )
    .status
    .success());

    let update = common::run_cli_in(
        &dir,
        &[
            "expense", "update", "1", "--amount", "600", "--note", "updated",
        ],
    );
    assert!(update.status.success());
    let show: serde_json::Value = serde_json::from_slice(
        &common::run_cli_in(&dir, &["expense", "show", "1", "--json"]).stdout,
    )
    .unwrap();
    assert_eq!(show["amount"], 600);
    assert_eq!(show["note"], "updated");

    let delete = common::run_cli_in(&dir, &["expense", "delete", "1"]);
    assert!(delete.status.success());
    assert!(!common::run_cli_in(&dir, &["expense", "show", "1"])
        .status
        .success());
}

#[test]
fn cli_expense_list_json() {
    let workspace = common::TestWorkspace::new();
    let dir = workspace.path();
    setup_trip_with_itinerary(&dir);
    assert!(common::run_cli_in(
        &dir,
        &[
            "expense",
            "add",
            "--itinerary",
            "1",
            "--amount",
            "100",
            "--currency",
            "JPY",
        ],
    )
    .status
    .success());

    let output = common::run_cli_in(&dir, &["expense", "list", "--trip", "1", "--json"]);
    assert!(output.status.success());
    let parsed: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(parsed["trip_id"], 1);
    assert_eq!(parsed["expenses"].as_array().unwrap().len(), 1);
}

#[test]
fn cli_expense_rejects_invalid_currency() {
    let workspace = common::TestWorkspace::new();
    let dir = workspace.path();
    setup_trip_with_itinerary(&dir);

    let output = common::run_cli_in(
        &dir,
        &[
            "expense",
            "add",
            "--itinerary",
            "1",
            "--amount",
            "100",
            "--currency",
            "JP",
        ],
    );
    assert!(!output.status.success());
}

#[test]
fn cli_expense_cascade_on_itinerary_delete() {
    let workspace = common::TestWorkspace::new();
    let dir = workspace.path();
    setup_trip_with_itinerary(&dir);
    assert!(common::run_cli_in(
        &dir,
        &[
            "expense",
            "add",
            "--itinerary",
            "1",
            "--amount",
            "100",
            "--currency",
            "JPY",
        ],
    )
    .status
    .success());

    assert!(common::run_cli_in(&dir, &["itinerary", "delete", "1"])
        .status
        .success());
    let list = common::run_cli_in(&dir, &["expense", "list", "--trip", "1"]);
    assert!(list.status.success());
    assert!(String::from_utf8_lossy(&list.stdout).contains("（なし）"));
}
