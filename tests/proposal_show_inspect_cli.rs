use std::path::PathBuf;
use std::process::Command;

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/proposals")
        .join(name)
}

fn run_cli(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_travel-ledger-cli"))
        .args(args)
        .output()
        .expect("failed to run CLI")
}

#[test]
fn cli_proposal_show_valid_envelope_succeeds() {
    let path = fixture_path("valid-envelope.json");
    let output = run_cli(&["proposal", "show", path.to_str().unwrap()]);
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Trip Proposal Envelope"));
    assert!(stdout.contains("Weekend city break (draft)"));
    assert!(stdout.contains("Kyoto, Japan"));
    assert!(stdout.contains("flexible_dates"));
}

#[test]
fn cli_proposal_inspect_valid_envelope_succeeds() {
    let path = fixture_path("valid-envelope.json");
    let output = run_cli(&["proposal", "inspect", path.to_str().unwrap()]);
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("--- Inspect details ---"));
    assert!(stdout.contains("Missing fields"));
    assert!(stdout.contains("hotel confirmation"));
    assert!(stdout.contains("Assumptions"));
    assert!(stdout.contains("Two adults"));
}

#[test]
fn cli_proposal_show_invalid_envelope_shows_blocking_error_without_panic() {
    let path = fixture_path("invalid-envelope-missing-title.json");
    let output = run_cli(&["proposal", "show", path.to_str().unwrap()]);
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Blocking errors"));
    assert!(stdout.contains("title"));
    assert!(stdout.contains("invalid"));
}

#[test]
fn cli_proposal_inspect_schema_v8_trip_is_not_proposal() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("docs/public/examples/schema-v8-minimal-trip.json");
    let output = run_cli(&["proposal", "inspect", path.to_str().unwrap()]);
    assert!(!output.status.success());
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(combined.contains("schema_version") || combined.contains("trip validate-export"));
}

#[test]
fn cli_proposal_show_non_normative_example_succeeds() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("docs/public/examples-non-normative/trip-proposal-envelope.example.json");
    let output = run_cli(&["proposal", "show", path.to_str().unwrap()]);
    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Okinawa family trip (draft)"));
}
