use std::fs;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

static TEST_DIR_COUNTER: AtomicU64 = AtomicU64::new(0);

fn temp_workdir() -> std::path::PathBuf {
    let n = TEST_DIR_COUNTER.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!("travel-ledger-cli-db-test-{n}"));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn run_cli(cwd: &std::path::Path, args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_travel-ledger-cli"))
        .current_dir(cwd)
        .args(args)
        .output()
        .expect("failed to run CLI")
}

fn normalize_path_str(path: &str) -> String {
    path.strip_prefix("/private").unwrap_or(path).to_string()
}

fn assert_same_resolved_db_path(actual: &str, expected: &std::path::Path) {
    assert_eq!(
        normalize_path_str(actual),
        normalize_path_str(&expected.to_string_lossy())
    );
}

#[test]
fn cli_db_path_does_not_create_db_file() {
    let dir = temp_workdir();
    let db_path = dir.join("caglla.db");

    let output = run_cli(&dir, &["db", "path"]);
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    assert_same_resolved_db_path(&stdout, &db_path);
    assert!(!db_path.exists(), "db path must not create SQLite file");
}

#[test]
fn cli_db_status_missing_db_does_not_create_file() {
    let dir = temp_workdir();
    let db_path = dir.join("caglla.db");

    let output = run_cli(&dir, &["db", "status"]);
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains(db_path.to_string_lossy().as_ref())
            || stdout.contains(&normalize_path_str(&db_path.to_string_lossy()))
    );
    assert!(stdout.contains("Exists                    : no"));
    assert!(stdout.contains("Path source               : default"));
    assert!(stdout.contains("Trip export schema version: 8"));
    assert!(!stdout.contains("File size (bytes)"));
    assert!(!stdout.contains("Table counts:"));
    assert!(!db_path.exists(), "db status must not create SQLite file");
}

#[test]
fn cli_db_status_json_missing_db_omits_optional_fields() {
    let dir = temp_workdir();
    let db_path = dir.join("caglla.db");

    let output = run_cli(&dir, &["db", "status", "--json"]);
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.trim_start().starts_with('{'),
        "expected JSON only stdout, got: {stdout}"
    );
    assert!(!stdout.contains("Path"));

    let parsed: serde_json::Value = serde_json::from_str(stdout.trim()).expect("valid JSON");
    assert_eq!(parsed["schema_version"], 2);
    assert_same_resolved_db_path(parsed["path"].as_str().unwrap(), &db_path);
    assert_eq!(parsed["path_source"], "default");
    assert_eq!(parsed["exists"], false);
    assert_eq!(parsed["trip_export_schema_version"], 8);
    assert!(parsed.get("file_size_bytes").is_none());
    assert!(parsed.get("table_counts").is_none());
    assert!(parsed.get("config_path").is_none());
    assert!(
        !db_path.exists(),
        "db status --json must not create SQLite file"
    );
}

#[test]
fn cli_db_status_json_existing_db_includes_table_counts() {
    let dir = temp_workdir();

    assert!(run_cli(&dir, &["db", "reset"]).status.success());
    assert!(run_cli(
        &dir,
        &[
            "trip",
            "add",
            "DB Status Trip",
            "--start",
            "2026-04-26",
            "--end",
            "2026-04-28",
        ]
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
            "--time",
            "09:00",
            "首里城",
        ]
    )
    .status
    .success());
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
        ]
    )
    .status
    .success());

    let output = run_cli(&dir, &["db", "status", "--json"]);
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value = serde_json::from_str(stdout.trim()).expect("valid JSON");
    assert_eq!(parsed["schema_version"], 2);
    assert_eq!(parsed["path_source"], "default");
    assert_eq!(parsed["exists"], true);
    assert_eq!(parsed["trip_export_schema_version"], 8);
    assert!(parsed["file_size_bytes"].as_u64().unwrap() > 0);
    assert_eq!(parsed["table_counts"]["trips"], 1);
    assert_eq!(parsed["table_counts"]["itinerary_items"], 1);
    assert_eq!(parsed["table_counts"]["estimates"], 1);
}

#[test]
fn cli_db_status_existing_db_shows_table_counts() {
    let dir = temp_workdir();

    assert!(run_cli(&dir, &["db", "reset"]).status.success());
    assert!(run_cli(
        &dir,
        &[
            "trip",
            "add",
            "Counts Trip",
            "--start",
            "2026-05-01",
            "--end",
            "2026-05-02",
        ]
    )
    .status
    .success());

    let output = run_cli(&dir, &["db", "status"]);
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Exists                    : yes"));
    assert!(stdout.contains("Path source               : default"));
    assert!(stdout.contains("File size (bytes)"));
    assert!(stdout.contains("Table counts:"));
    assert!(stdout.contains("trips                   : 1"));
}

#[test]
fn cli_db_path_with_cli_db_flag() {
    let dir = temp_workdir();
    let db_path = dir.join("a.db");

    let output = run_cli(&dir, &["--db", db_path.to_str().unwrap(), "db", "path"]);
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    assert_same_resolved_db_path(&stdout, &db_path);
    assert!(!db_path.exists());
}

#[test]
fn cli_db_path_with_caglla_db_env() {
    let dir = temp_workdir();
    let db_path = dir.join("b.db");

    let output = Command::new(env!("CARGO_BIN_EXE_travel-ledger-cli"))
        .current_dir(&dir)
        .env("CAGLLA_DB", "./b.db")
        .args(["db", "path"])
        .output()
        .expect("failed to run CLI");
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    assert_same_resolved_db_path(&stdout, &db_path);
    assert!(!db_path.exists());
}

#[test]
fn cli_db_path_with_caglla_toml() {
    let dir = temp_workdir();
    let db_path = dir.join("from-config.db");
    fs::write(
        dir.join("caglla.toml"),
        "[database]\npath = \"./from-config.db\"\n",
    )
    .unwrap();

    let output = run_cli(&dir, &["db", "path"]);
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    assert_same_resolved_db_path(&stdout, &db_path);
    assert!(!db_path.exists());
}

#[test]
fn cli_trip_add_uses_selected_db_only() {
    let dir = temp_workdir();
    let alt_db = dir.join("alt.db");
    let default_db = dir.join("caglla.db");

    let output = run_cli(
        &dir,
        &[
            "--db",
            alt_db.to_str().unwrap(),
            "trip",
            "add",
            "Alt Trip",
            "--start",
            "2026-06-01",
            "--end",
            "2026-06-02",
        ],
    );
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(alt_db.exists(), "selected DB should be created");
    assert!(
        !default_db.exists(),
        "default caglla.db must not be created"
    );
}

#[test]
fn cli_trip_list_accepts_trailing_db_flag() {
    let dir = temp_workdir();
    let alt_db = dir.join("trailing.db");

    assert!(run_cli(
        &dir,
        &[
            "--db",
            alt_db.to_str().unwrap(),
            "trip",
            "add",
            "Trailing DB Trip",
            "--start",
            "2026-06-10",
            "--end",
            "2026-06-11",
        ],
    )
    .status
    .success());

    let output = run_cli(&dir, &["trip", "list", "--db", alt_db.to_str().unwrap()]);
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Trailing DB Trip"));
}

fn create_legacy_days_db_without_summary(dir: &std::path::Path) {
    let db_path = dir.join("caglla.db");
    let sql = r"
CREATE TABLE trips (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    name        TEXT NOT NULL,
    start_date  TEXT,
    end_date    TEXT,
    created_at  TEXT NOT NULL,
    updated_at  TEXT NOT NULL
);
CREATE TABLE days (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    trip_id     INTEGER NOT NULL,
    day_number  INTEGER NOT NULL,
    title       TEXT NOT NULL DEFAULT '',
    created_at  TEXT NOT NULL,
    updated_at  TEXT NOT NULL,
    UNIQUE(trip_id, day_number)
);
INSERT INTO trips (name, start_date, end_date, created_at, updated_at)
VALUES ('Legacy Trip', '2026-01-01', '2026-01-03', '2026-01-01 00:00:00', '2026-01-01 00:00:00');
";
    let status = Command::new("sqlite3")
        .arg(&db_path)
        .arg(sql)
        .status()
        .expect("sqlite3 required for legacy DB simulation");
    assert!(
        status.success(),
        "failed to create legacy caglla.db without days.summary"
    );
}

#[test]
fn cli_db_status_json_legacy_days_without_summary_column() {
    let dir = temp_workdir();
    create_legacy_days_db_without_summary(&dir);

    let output = run_cli(&dir, &["db", "status", "--json"]);
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value = serde_json::from_str(stdout.trim()).expect("valid JSON");
    assert_eq!(parsed["schema_version"], 2);
    assert_eq!(parsed["exists"], true);
    assert_eq!(parsed["path_source"], "default");
    assert_eq!(parsed["table_counts"]["trips"], 1);
    assert_eq!(parsed["table_counts"]["days"], 3);
}

#[test]
fn cli_db_use_then_db_path_uses_config_source() {
    let dir = temp_workdir();
    let db_path = dir.join("data").join("app.db");

    let use_output = run_cli(&dir, &["db", "use", "./data/app.db"]);
    assert!(
        use_output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&use_output.stderr)
    );

    let stdout = String::from_utf8_lossy(&use_output.stdout);
    assert!(stdout.contains("Database path saved to config"));
    assert!(stdout.contains("./data/app.db"));
    assert!(!db_path.exists(), "db use must not create SQLite file");

    let path_output = run_cli(&dir, &["db", "path"]);
    assert!(path_output.status.success());
    let path_stdout = String::from_utf8_lossy(&path_output.stdout)
        .trim()
        .to_string();
    assert_same_resolved_db_path(&path_stdout, &db_path);

    let status_output = run_cli(&dir, &["db", "status", "--json"]);
    assert!(status_output.status.success());
    let parsed: serde_json::Value =
        serde_json::from_str(String::from_utf8_lossy(&status_output.stdout).trim()).unwrap();
    assert_eq!(parsed["path_source"], "config");
    assert!(parsed["config_path"].is_string());
}

#[test]
fn cli_db_use_clear_reverts_to_default_db_path() {
    let dir = temp_workdir();
    let default_db = dir.join("caglla.db");

    assert!(run_cli(&dir, &["db", "use", "./data/app.db"])
        .status
        .success());
    assert!(dir.join("caglla.toml").exists());

    let clear_output = run_cli(&dir, &["db", "use", "--clear"]);
    assert!(
        clear_output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&clear_output.stderr)
    );
    let stdout = String::from_utf8_lossy(&clear_output.stdout);
    assert!(stdout.contains("Database path cleared from config"));
    assert!(stdout.contains("./caglla.db") || stdout.contains("Default"));

    let path_output = run_cli(&dir, &["db", "path"]);
    assert!(path_output.status.success());
    let path_stdout = String::from_utf8_lossy(&path_output.stdout)
        .trim()
        .to_string();
    assert_same_resolved_db_path(&path_stdout, &default_db);

    let status_output = run_cli(&dir, &["db", "status", "--json"]);
    let parsed: serde_json::Value =
        serde_json::from_str(String::from_utf8_lossy(&status_output.stdout).trim()).unwrap();
    assert_eq!(parsed["path_source"], "default");
}

#[test]
fn cli_db_use_config_is_overridden_by_cli_db_flag() {
    let dir = temp_workdir();
    let config_db = dir.join("from-config.db");
    let cli_db = dir.join("from-cli.db");

    assert!(run_cli(&dir, &["db", "use", "./from-config.db"])
        .status
        .success());

    let output = run_cli(&dir, &["--db", cli_db.to_str().unwrap(), "db", "path"]);
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    assert_same_resolved_db_path(&stdout, &cli_db);
    assert_ne!(
        normalize_path_str(&stdout),
        normalize_path_str(&config_db.to_string_lossy())
    );
}

#[test]
fn cli_db_use_config_is_overridden_by_caglla_db_env() {
    let dir = temp_workdir();
    let config_db = dir.join("from-config.db");
    let env_db = dir.join("from-env.db");

    assert!(run_cli(&dir, &["db", "use", "./from-config.db"])
        .status
        .success());

    let output = Command::new(env!("CARGO_BIN_EXE_travel-ledger-cli"))
        .current_dir(&dir)
        .env("CAGLLA_DB", "./from-env.db")
        .args(["db", "path"])
        .output()
        .expect("failed to run CLI");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    assert_same_resolved_db_path(&stdout, &env_db);
    assert_ne!(
        normalize_path_str(&stdout),
        normalize_path_str(&config_db.to_string_lossy())
    );
}

#[test]
fn cli_db_use_invalid_toml_does_not_corrupt_config() {
    let dir = temp_workdir();
    let invalid = "not = [valid";
    fs::write(dir.join("caglla.toml"), invalid).unwrap();

    let output = run_cli(&dir, &["db", "use", "./app.db"]);
    assert!(!output.status.success());
    let contents = fs::read_to_string(dir.join("caglla.toml")).unwrap();
    assert_eq!(contents, invalid);
}

#[test]
fn cli_db_use_missing_db_prints_note() {
    let dir = temp_workdir();

    let output = run_cli(&dir, &["db", "use", "./missing.db"]);
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("database file does not exist yet"));
    assert!(!dir.join("missing.db").exists());
}
