mod common;

use std::fs;
fn setup_trip_with_itinerary(dir: &std::path::Path) {
    assert!(common::run_cli_in(dir, &["db", "reset"]).status.success());
    assert!(common::run_cli_in(
        dir,
        &[
            "trip",
            "add",
            "Reservation Trip",
            "--start",
            "2026-04-26",
            "--end",
            "2026-04-29",
        ],
    )
    .status
    .success());
    assert!(common::run_cli_in(
        dir,
        &[
            "itinerary",
            "add",
            "1",
            "--day",
            "1",
            "--time",
            "16:40",
            "Check-in",
        ],
    )
    .status
    .success());
}

#[test]
fn cli_reservation_add_and_show() {
    let workspace = common::TestWorkspace::new();
    let dir = workspace.path();
    setup_trip_with_itinerary(&dir);

    let output = common::run_cli_in(
        &dir,
        &[
            "reservation",
            "add",
            "--itinerary",
            "1",
            "--reservation-type",
            "hotel",
            "--provider",
            "Hilton Sesoko Resort",
            "--confirmation",
            "ABC123",
            "--site-url",
            "https://example.com/booking",
            "--remark",
            "Twin room",
            "--start-at",
            "2026-04-26T16:40",
            "--end-at",
            "2026-04-29T10:00",
        ],
    );
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Reservation を追加しました"));
    assert!(stdout.contains("ABC123"));
    assert!(stdout.contains("Hilton Sesoko Resort"));

    let show = common::run_cli_in(&dir, &["reservation", "show", "1", "--json"]);
    assert!(show.status.success());
    let json: serde_json::Value = serde_json::from_slice(&show.stdout).unwrap();
    assert_eq!(json["reservation_type"], "hotel");
    assert_eq!(json["provider_name"], "Hilton Sesoko Resort");
    assert_eq!(json["confirmation_code"], "ABC123");
}

#[test]
fn cli_reservation_list_by_itinerary_and_trip() {
    let workspace = common::TestWorkspace::new();
    let dir = workspace.path();
    setup_trip_with_itinerary(&dir);
    assert!(common::run_cli_in(
        &dir,
        &[
            "reservation",
            "add",
            "--itinerary",
            "1",
            "--reservation-type",
            "hotel",
            "--provider",
            "Hilton Sesoko Resort",
            "--confirmation",
            "ABC123",
        ],
    )
    .status
    .success());

    let list_itinerary = common::run_cli_in(&dir, &["reservation", "list", "--itinerary", "1"]);
    assert!(list_itinerary.status.success());
    assert!(String::from_utf8_lossy(&list_itinerary.stdout).contains("ABC123"));

    let list_trip = common::run_cli_in(&dir, &["reservation", "list", "--trip", "1"]);
    assert!(list_trip.status.success());
    let trip_stdout = String::from_utf8_lossy(&list_trip.stdout);
    assert!(trip_stdout.contains("Day 1"));
    assert!(trip_stdout.contains("Check-in"));
}

#[test]
fn cli_reservation_update_and_delete() {
    let workspace = common::TestWorkspace::new();
    let dir = workspace.path();
    setup_trip_with_itinerary(&dir);
    assert!(common::run_cli_in(
        &dir,
        &[
            "reservation",
            "add",
            "--itinerary",
            "1",
            "--reservation-type",
            "rental_car",
            "--provider",
            "KS Rent A Car",
            "--confirmation",
            "XYZ987",
        ],
    )
    .status
    .success());

    assert!(common::run_cli_in(
        &dir,
        &[
            "reservation",
            "update",
            "1",
            "--confirmation",
            "NEW999",
            "--remark",
            "ETC required",
        ],
    )
    .status
    .success());

    let show: serde_json::Value = serde_json::from_slice(
        &common::run_cli_in(&dir, &["reservation", "show", "1", "--json"]).stdout,
    )
    .unwrap();
    assert_eq!(show["confirmation_code"], "NEW999");
    assert_eq!(show["remark"], "ETC required");

    assert!(common::run_cli_in(&dir, &["reservation", "delete", "1"])
        .status
        .success());
    assert!(!common::run_cli_in(&dir, &["reservation", "show", "1"])
        .status
        .success());
}

#[test]
fn cli_reservation_export_import_roundtrip() {
    let workspace = common::TestWorkspace::new();
    let dir = workspace.path();
    setup_trip_with_itinerary(&dir);
    assert!(common::run_cli_in(
        &dir,
        &[
            "reservation",
            "add",
            "--itinerary",
            "1",
            "--reservation-type",
            "hotel",
            "--provider",
            "Hilton Sesoko Resort",
            "--confirmation",
            "ABC123",
        ],
    )
    .status
    .success());

    let export_path = dir.join("trip-export.json");
    assert!(common::run_cli_in(
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
        serde_json::from_str(&fs::read_to_string(&export_path).unwrap()).unwrap();
    let reservations = &export_json["days"][0]["itineraries"][0]["reservations"];
    assert!(reservations.is_array());
    assert_eq!(reservations[0]["provider_name"], "Hilton Sesoko Resort");

    assert!(common::run_cli_in(&dir, &["db", "reset"]).status.success());
    assert!(
        common::run_cli_in(&dir, &["trip", "import", export_path.to_str().unwrap(),],)
            .status
            .success()
    );

    let list = common::run_cli_in(&dir, &["reservation", "list", "--trip", "1", "--json"]);
    assert!(list.status.success());
    let list_json: serde_json::Value = serde_json::from_slice(&list.stdout).unwrap();
    assert_eq!(list_json["reservations"].as_array().unwrap().len(), 1);
}

#[test]
fn cli_reservation_validation_errors() {
    let workspace = common::TestWorkspace::new();
    let dir = workspace.path();
    setup_trip_with_itinerary(&dir);

    let missing_provider = common::run_cli_in(
        &dir,
        &[
            "reservation",
            "add",
            "--itinerary",
            "1",
            "--reservation-type",
            "hotel",
            "--provider",
            "   ",
        ],
    );
    assert!(!missing_provider.status.success());

    let invalid_type = common::run_cli_in(
        &dir,
        &[
            "reservation",
            "add",
            "--itinerary",
            "1",
            "--reservation-type",
            "cruise",
            "--provider",
            "Carrier",
        ],
    );
    assert!(!invalid_type.status.success());

    let missing_itinerary = common::run_cli_in(
        &dir,
        &[
            "reservation",
            "add",
            "--itinerary",
            "999",
            "--reservation-type",
            "hotel",
            "--provider",
            "Hilton",
        ],
    );
    assert!(!missing_itinerary.status.success());
}
