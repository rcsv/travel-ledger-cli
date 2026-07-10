mod common;

use std::fs;
use std::path::PathBuf;

fn repo_root() -> PathBuf {
    common::manifest_dir()
}

fn normalize_receipts(value: &serde_json::Value) -> Vec<serde_json::Value> {
    let mut receipts: Vec<serde_json::Value> = value
        .as_array()
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .map(|receipt| {
            serde_json::json!({
                "day_ref": receipt.get("day_ref"),
                "amount": receipt.get("amount"),
                "currency": receipt.get("currency"),
                "memo": receipt.get("memo"),
                "status": receipt.get("status"),
                "trashed": receipt
                    .get("trashed")
                    .and_then(|v| v.as_bool())
                    .or_else(|| {
                        receipt
                            .get("trashed_at")
                            .map(|v| !v.is_null())
                    })
                    .unwrap_or(false),
            })
        })
        .collect();
    receipts.sort_by(|a, b| {
        a["memo"]
            .as_str()
            .unwrap_or("")
            .cmp(b["memo"].as_str().unwrap_or(""))
    });
    receipts
}

fn normalize_export_v3(value: &serde_json::Value) -> serde_json::Value {
    serde_json::json!({
        "schema_version": value["schema_version"],
        "trip": {
            "name": value["trip"]["name"],
            "start_date": value["trip"]["start_date"],
            "end_date": value["trip"]["end_date"],
            "summary": value["trip"]["summary"],
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
        "receipts": normalize_receipts(value.get("receipts").unwrap_or(&serde_json::json!([]))),
    })
}

#[test]
fn okinawa_sesoko_expected_export_structure() {
    let expected_path = repo_root().join("samples/okinawa_sesoko_2026/expected-export-v3.json");
    let expected: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&expected_path).unwrap()).unwrap();
    assert_eq!(expected["schema_version"], 8);
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
    assert_eq!(
        expected["receipts"]
            .as_array()
            .map(|v| v.len())
            .unwrap_or(0),
        6
    );
    let trashed_count = expected["receipts"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|r| r["trashed"].as_bool() == Some(true))
        .count();
    assert_eq!(trashed_count, 1);
}

#[test]
fn cli_okinawa_sesoko_seed_export_matches_expected() {
    let workspace = common::TestWorkspace::new();
    let dir = workspace.path();
    let root = repo_root();
    let seed_script = root.join("samples/okinawa_sesoko_2026/seed.sh");

    let seed = common::run_seed_script(&workspace, &seed_script);
    common::assert_seed_success(&seed, &workspace, &seed_script);

    let export_path = dir.join("okinawa-export.json");
    assert!(common::run_cli_in(
        dir,
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

    let validate = common::run_cli_in(
        dir,
        &["trip", "validate-export", export_path.to_str().unwrap()],
    );
    assert!(validate.status.success());
    let stdout = String::from_utf8_lossy(&validate.stdout);
    assert!(stdout.contains("有効な export ファイル"));

    let stats = common::run_cli_in(dir, &["trip", "stats", "1"]);
    assert!(stats.status.success());
    let stats_stdout = String::from_utf8_lossy(&stats.stdout);
    assert!(stats_stdout.contains("Expenses: 49"));
    assert!(stats_stdout.contains("561,780"));

    let receipts = common::run_cli_in(dir, &["receipt", "list", "--trip", "1"]);
    assert!(receipts.status.success());
    let receipts_stdout = String::from_utf8_lossy(&receipts.stdout);
    assert!(receipts_stdout.contains("Pending Receipts:"));
    assert!(receipts_stdout.contains("15,980 JPY"));
    assert!(receipts_stdout.contains("美ら海水族館ショップ"));
    assert!(!receipts_stdout.contains("個人的な雑貨購入"));
}

fn normalize_export_md_timestamp(markdown: &str) -> String {
    markdown
        .lines()
        .map(|line| {
            if line.starts_with("Generated at: ") {
                "Generated at: TIMESTAMP"
            } else {
                line
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn cli_okinawa_sesoko_seed_export_md_matches_expected() {
    let workspace = common::TestWorkspace::new();
    let dir = workspace.path();
    let root = repo_root();
    let seed_script = root.join("samples/okinawa_sesoko_2026/seed.sh");

    let seed = common::run_seed_script(&workspace, &seed_script);
    common::assert_seed_success(&seed, &workspace, &seed_script);

    let output = common::run_cli_in(dir, &["trip", "export-md", "1"]);
    assert!(
        output.status.success(),
        "export-md stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let actual = normalize_export_md_timestamp(&String::from_utf8_lossy(&output.stdout));
    let expected_path = root.join("samples/okinawa_sesoko_2026/expected-export-md.md");
    let expected = normalize_export_md_timestamp(&fs::read_to_string(&expected_path).unwrap());

    assert_eq!(actual, expected);

    assert!(actual.contains("## Trip overview"));
    assert!(actual.contains("## Daily schedule"));
    assert!(actual.contains("## Reservations"));
    assert!(actual.contains("## Planned cost"));
    assert!(actual.contains("## Notes"));
    assert!(actual.contains("## Colophon"));
    assert!(!actual.contains("Expenses:"));
    assert!(!actual.contains("- Difference:"));
}
