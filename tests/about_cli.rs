mod common;

#[test]
fn cli_about_prints_english_overview() {
    let workspace = common::TestWorkspace::new();
    let output = common::run_cli_in(workspace.path(), &["--about"]);
    let db_path = workspace.path().join("caglla.db");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Caglla.Travel CLI"));
    assert!(stdout.contains("local-first travel planning CLI"));
    assert!(stdout.contains("Itinerary is not a venue"));
    assert!(stdout.contains("caglla.db"));
    assert!(stdout.contains("License: MIT"));
    assert!(
        !db_path.exists(),
        "about path should not create/open SQLite DB file"
    );
}

#[test]
fn cli_about_does_not_require_subcommand() {
    let workspace = common::TestWorkspace::new();
    let output = common::run_cli_in(workspace.path(), &["--about", "trip", "list"]);
    let db_path = workspace.path().join("caglla.db");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Caglla.Travel CLI"));
    assert!(
        !db_path.exists(),
        "about path should not create/open SQLite DB file"
    );
}
