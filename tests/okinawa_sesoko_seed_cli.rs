use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

static TEST_DIR_COUNTER: AtomicU64 = AtomicU64::new(0);

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn run_cli(cwd: &std::path::Path, args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_caglla-cli"))
        .current_dir(cwd)
        .args(args)
        .output()
        .expect("failed to run CLI")
}

fn temp_workdir() -> std::path::PathBuf {
    let n = TEST_DIR_COUNTER.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!("caglla-cli-okinawa-sesoko-{n}"));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn normalize_export_v3(value: &serde_json::Value) -> serde_json::Value {
    serde_json::json!({
        "schema_version": value["schema_version"],
        "trip": {
            "name": value["trip"]["name"],
            "start_date": value["trip"]["start_date"],
            "end_date": value["trip"]["end_date"],
        },
        "days": value["days"],
        "checklist_items": value["checklist_items"]
            .as_array()
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .map(|item| {
                serde_json::json!({
                    "title": item["title"],
                    "is_done": item["is_done"],
                    "sort_order": item["sort_order"],
                })
            })
            .collect::<Vec<_>>(),
        "notes": value.get("notes").cloned().unwrap_or(serde_json::json!([])),
        "participants": value
            .get("participants")
            .cloned()
            .unwrap_or(serde_json::json!([])),
    })
}

#[test]
fn okinawa_sesoko_expected_export_structure() {
    let expected_path = repo_root().join("samples/okinawa_sesoko_2026/expected-export-v3.json");
    let expected: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&expected_path).unwrap()).unwrap();
    assert_eq!(expected["schema_version"], 7);
    assert_eq!(expected["trip"]["name"], "沖縄 瀬底 4日間");
    let itinerary_count: usize = expected["days"]
        .as_array()
        .unwrap()
        .iter()
        .map(|day| day["itineraries"].as_array().map(|v| v.len()).unwrap_or(0))
        .sum();
    assert_eq!(itinerary_count, 58);
    let expense_count: usize = expected["days"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|day| day["itineraries"].as_array().cloned().unwrap_or_default())
        .map(|it| it["expenses"].as_array().map(|v| v.len()).unwrap_or(0))
        .sum();
    assert_eq!(expense_count, 49);
}

#[test]
fn cli_okinawa_sesoko_seed_export_matches_expected() {
    let dir = temp_workdir();
    let root = repo_root();
    let seed_script = root.join("samples/okinawa_sesoko_2026/seed.sh");

    let seed = Command::new("bash")
        .current_dir(&root)
        .env("CAGLLA_SAMPLE_WORKDIR", &dir)
        .arg(&seed_script)
        .output()
        .expect("failed to run seed.sh");
    assert!(
        seed.status.success(),
        "seed stderr: {}",
        String::from_utf8_lossy(&seed.stderr)
    );

    let export_path = dir.join("okinawa-export.json");
    assert!(run_cli(
        &dir,
        &[
            "trip",
            "export",
            "1",
            "--output",
            export_path.to_str().unwrap(),
        ]
    )
    .status
    .success());

    let exported: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&export_path).unwrap()).unwrap();
    let expected: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(root.join("samples/okinawa_sesoko_2026/expected-export-v3.json"))
            .unwrap(),
    )
    .unwrap();

    assert_eq!(
        normalize_export_v3(&exported),
        normalize_export_v3(&expected)
    );

    let validate = run_cli(
        &dir,
        &["trip", "validate-export", export_path.to_str().unwrap()],
    );
    assert!(validate.status.success());
    let stdout = String::from_utf8_lossy(&validate.stdout);
    assert!(stdout.contains("有効な export ファイル"));

    let stats = run_cli(&dir, &["trip", "stats", "1"]);
    assert!(stats.status.success());
    let stats_stdout = String::from_utf8_lossy(&stats.stdout);
    assert!(stats_stdout.contains("Expenses: 49"));
    assert!(stats_stdout.contains("561,780"));
}
