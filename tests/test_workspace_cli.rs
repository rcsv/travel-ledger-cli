mod common;

#[test]
fn temp_workspaces_are_unique_and_persist_while_owned() {
    let first = common::TestWorkspace::new();
    let second = common::TestWorkspace::new();
    assert_ne!(first.path(), second.path());
    assert!(first.path().is_dir());
    assert!(second.path().is_dir());
}

#[test]
fn workspace_cleanup_does_not_remove_other_workspace() {
    let survivor = common::TestWorkspace::new();
    let survivor_path = survivor.path().to_path_buf();
    {
        let ephemeral = common::TestWorkspace::new();
        assert!(ephemeral.path().is_dir());
        assert!(survivor.path().is_dir());
        assert_ne!(ephemeral.path(), survivor.path());
    }
    assert!(
        survivor_path.is_dir(),
        "survivor workspace should remain until its owner drops"
    );
    let db_path = survivor_path.join("caglla.db");
    std::fs::write(&db_path, b"probe").expect("write probe db");
    assert!(db_path.is_file());
    assert!(db_path.starts_with(survivor.path()));
}

#[test]
fn workspace_db_and_config_stay_under_workspace_root() {
    let workspace = common::TestWorkspace::new();
    let db_path = workspace.path().join("caglla.db");
    let config_path = workspace.path().join("caglla.toml");
    assert!(db_path.starts_with(workspace.path()));
    assert!(config_path.starts_with(workspace.path()));
    assert_eq!(
        common::cli_binary(),
        std::path::Path::new(common::cli_binary())
            .to_str()
            .expect("binary path utf-8")
    );
    assert!(
        std::path::Path::new(common::cli_binary()).is_file(),
        "prebuilt CLI binary must exist for integration tests"
    );
}
