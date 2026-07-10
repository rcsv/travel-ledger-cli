//! Shared helpers for integration tests — isolated workspaces and CLI diagnostics.

use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// Owns a unique temporary directory for the lifetime of a test.
pub struct TestWorkspace {
    _dir: tempfile::TempDir,
}

impl TestWorkspace {
    pub fn new() -> Self {
        let dir = tempfile::Builder::new()
            .prefix("travel-ledger-cli-")
            .tempdir()
            .expect("create isolated test workspace");
        Self { _dir: dir }
    }

    pub fn path(&self) -> &Path {
        self._dir.path()
    }
}

pub fn manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

pub fn cli_binary() -> &'static str {
    env!("CARGO_BIN_EXE_travel-ledger-cli")
}

pub fn run_cli(args: &[&str]) -> Output {
    run_cli_in(manifest_dir().as_path(), args)
}

pub fn run_cli_in(cwd: &Path, args: &[&str]) -> Output {
    let binary = cli_binary();
    let mut command = Command::new(binary);
    command.current_dir(cwd).args(args);
    match command.output() {
        Ok(output) => output,
        Err(err) => panic!("{}", format_cli_spawn_failure(binary, cwd, args, &err)),
    }
}

pub fn run_seed_script(workspace: &TestWorkspace, seed_script: &Path) -> Output {
    let root = manifest_dir();
    let binary = cli_binary();
    let mut command = Command::new("bash");
    command
        .current_dir(&root)
        .env("CAGLLA_SAMPLE_WORKDIR", workspace.path())
        .env("CAGLLA_BIN", binary)
        .arg(seed_script);
    match command.output() {
        Ok(output) => output,
        Err(err) => panic!(
            "{}",
            format_seed_spawn_failure(seed_script, workspace.path(), binary, &err)
        ),
    }
}

pub fn assert_seed_success(output: &Output, workspace: &TestWorkspace, seed_script: &Path) {
    assert!(
        output.status.success(),
        "{}",
        format_seed_failure(output, workspace.path(), seed_script)
    );
}

pub fn format_cli_spawn_failure(
    binary: &str,
    cwd: &Path,
    args: &[&str],
    err: &std::io::Error,
) -> String {
    format!(
        "failed to run CLI\n\
         command: {binary} {}\n\
         cwd: {}\n\
         binary: {binary}\n\
         default_db: {}/caglla.db\n\
         error: {err}",
        args.join(" "),
        cwd.display(),
        cwd.display(),
    )
}

fn format_seed_spawn_failure(
    seed_script: &Path,
    workspace: &Path,
    binary: &str,
    err: &std::io::Error,
) -> String {
    format!(
        "failed to run seed.sh\n\
         script: {}\n\
         workspace: {}\n\
         binary: {binary}\n\
         error: {err}",
        seed_script.display(),
        workspace.display(),
    )
}

pub fn format_seed_failure(output: &Output, workspace: &Path, seed_script: &Path) -> String {
    format!(
        "seed.sh failed\n\
         script: {}\n\
         workspace: {}\n\
         binary: {}\n\
         exit status: {:?}\n\
         stdout:\n{}\n\
         stderr:\n{}",
        seed_script.display(),
        workspace.display(),
        cli_binary(),
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    )
}

pub fn format_cli_failure(output: &Output, cwd: &Path, args: &[&str]) -> String {
    format!(
        "CLI command failed\n\
         command: {} {}\n\
         cwd: {}\n\
         binary: {}\n\
         exit status: {:?}\n\
         stdout:\n{}\n\
         stderr:\n{}",
        cli_binary(),
        args.join(" "),
        cwd.display(),
        cli_binary(),
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    )
}

/// Run a shell command with diagnostics on spawn failure.
pub fn run_command(program: impl AsRef<OsStr>, cwd: &Path, args: &[&str]) -> Output {
    let program = program.as_ref();
    let mut command = Command::new(program);
    command.current_dir(cwd).args(args);
    match command.output() {
        Ok(output) => output,
        Err(err) => panic!(
            "failed to run subprocess\n\
             program: {}\n\
             args: {}\n\
             cwd: {}\n\
             error: {err}",
            program.to_string_lossy(),
            args.join(" "),
            cwd.display(),
        ),
    }
}
