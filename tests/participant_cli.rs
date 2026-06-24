use std::fs;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

static TEST_DIR_COUNTER: AtomicU64 = AtomicU64::new(0);

fn run_cli(cwd: &std::path::Path, args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_caglla-cli"))
        .current_dir(cwd)
        .args(args)
        .output()
        .expect("failed to run CLI")
}

fn temp_workdir() -> std::path::PathBuf {
    let n = TEST_DIR_COUNTER.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!("caglla-cli-participant-{n}"));
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
            "Participant Trip",
            "--start",
            "2026-08-01",
            "--end",
            "2026-08-03",
        ],
    )
    .status
    .success());
}

#[test]
fn cli_participant_add_list_show_update_delete() {
    let dir = temp_workdir();
    setup_trip(&dir);

    let add = run_cli(
        &dir,
        &[
            "participant",
            "add",
            "--trip",
            "1",
            "--name",
            "ともさん",
            "--self",
            "--sort-order",
            "0",
        ],
    );
    assert!(add.status.success(), "{:?}", add.stderr);
    let stdout = String::from_utf8_lossy(&add.stdout);
    assert!(stdout.contains("Participant を追加しました"));
    assert!(stdout.contains("(self)"));

    let add2 = run_cli(
        &dir,
        &[
            "participant",
            "add",
            "--trip",
            "1",
            "--name",
            "妻",
            "--sort-order",
            "1",
        ],
    );
    assert!(add2.status.success());

    let list = run_cli(&dir, &["participant", "list", "--trip", "1"]);
    assert!(list.status.success());
    let list_stdout = String::from_utf8_lossy(&list.stdout);
    assert!(list_stdout.contains("ともさん"));
    assert!(list_stdout.contains("妻"));
    assert!(list_stdout.contains("Participants: 2 (companions: 1)"));

    let show = run_cli(&dir, &["participant", "show", "2", "--json"]);
    assert!(show.status.success());
    let json: serde_json::Value = serde_json::from_slice(&show.stdout).unwrap();
    assert_eq!(json["name"], "妻");
    assert_eq!(json["is_self"], false);

    let update = run_cli(
        &dir,
        &["participant", "update", "2", "--name", "パートナー"],
    );
    assert!(update.status.success());

    let delete = run_cli(&dir, &["participant", "delete", "2"]);
    assert!(delete.status.success());

    let list_after = run_cli(&dir, &["participant", "list", "--trip", "1", "--json"]);
    assert!(list_after.status.success());
    let list_json: serde_json::Value = serde_json::from_slice(&list_after.stdout).unwrap();
    assert_eq!(list_json["counts"]["registered_count"], 1);
    assert_eq!(list_json["counts"]["companion_count"], 0);
    assert_eq!(list_json["counts"]["self_known"], true);
}

#[test]
fn cli_participant_self_conflict_on_add() {
    let dir = temp_workdir();
    setup_trip(&dir);

    assert!(run_cli(
        &dir,
        &[
            "participant",
            "add",
            "--trip",
            "1",
            "--name",
            "自分",
            "--self",
        ],
    )
    .status
    .success());
    let conflict = run_cli(
        &dir,
        &[
            "participant",
            "add",
            "--trip",
            "1",
            "--name",
            "別の自分",
            "--self",
        ],
    );
    assert!(!conflict.status.success());
    let stderr = String::from_utf8_lossy(&conflict.stderr);
    assert!(stderr.contains("self participant"));
}

#[test]
fn cli_participant_update_self_transfer() {
    let dir = temp_workdir();
    setup_trip(&dir);

    assert!(run_cli(
        &dir,
        &["participant", "add", "--trip", "1", "--name", "A", "--self",],
    )
    .status
    .success());
    assert!(
        run_cli(&dir, &["participant", "add", "--trip", "1", "--name", "B"],)
            .status
            .success()
    );

    let transfer = run_cli(&dir, &["participant", "update", "2", "--self"]);
    assert!(transfer.status.success());

    let show_a = run_cli(&dir, &["participant", "show", "1", "--json"]);
    let show_b = run_cli(&dir, &["participant", "show", "2", "--json"]);
    let a: serde_json::Value = serde_json::from_slice(&show_a.stdout).unwrap();
    let b: serde_json::Value = serde_json::from_slice(&show_b.stdout).unwrap();
    assert_eq!(a["is_self"], false);
    assert_eq!(b["is_self"], true);

    assert!(run_cli(&dir, &["participant", "update", "2", "--not-self"])
        .status
        .success());
    let list = run_cli(&dir, &["participant", "list", "--trip", "1", "--json"]);
    let list_json: serde_json::Value = serde_json::from_slice(&list.stdout).unwrap();
    assert_eq!(list_json["counts"]["self_known"], false);
}

#[test]
fn cli_participant_export_v4_roundtrip() {
    let dir = temp_workdir();
    setup_trip(&dir);
    assert!(run_cli(
        &dir,
        &[
            "participant",
            "add",
            "--trip",
            "1",
            "--name",
            "ともさん",
            "--self",
        ],
    )
    .status
    .success());
    assert!(
        run_cli(&dir, &["participant", "add", "--trip", "1", "--name", "妻"],)
            .status
            .success()
    );

    let export_path = dir.join("trip-export-v4.json");
    assert!(run_cli(
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

    let exported: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&export_path).unwrap()).unwrap();
    assert_eq!(exported["schema_version"], 7);
    assert_eq!(exported["participants"].as_array().unwrap().len(), 2);

    assert!(run_cli(&dir, &["db", "reset"]).status.success());
    assert!(
        run_cli(&dir, &["trip", "import", export_path.to_str().unwrap()],)
            .status
            .success()
    );

    let list = run_cli(&dir, &["participant", "list", "--trip", "1", "--json"]);
    let list_json: serde_json::Value = serde_json::from_slice(&list.stdout).unwrap();
    assert_eq!(list_json["counts"]["participant_count"], 2);
    assert_eq!(list_json["counts"]["companion_count"], 1);
}
