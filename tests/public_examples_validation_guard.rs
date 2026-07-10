//! CI guard: public / non-normative examples must validate with the correct command only.
//!
//! Run via `make check` → `cargo test` (including this integration test binary).
//!
//! Each test uses an isolated temp working directory so parallel `cargo test` does not
//! share a default `./caglla.db` with other integration tests (migration races).

mod common;

use std::path::{Path, PathBuf};

fn manifest_dir() -> PathBuf {
    common::manifest_dir()
}

fn public_example(name: &str) -> PathBuf {
    manifest_dir().join("docs/public/examples").join(name)
}

fn non_normative_example(name: &str) -> PathBuf {
    manifest_dir()
        .join("docs/public/examples-non-normative")
        .join(name)
}

/// schema v8 Trip export — `trip validate-export` only
const SCHEMA_V8_PUBLIC_EXAMPLES: &[&str] = &[
    "schema-v8-minimal-trip.json",
    "schema-v8-okinawa-sesoko-trip.json",
    "schema-v8-with-reservations-expenses-notes.json",
];

/// Trip Proposal Envelope — `proposal validate` only
const ENVELOPE_NON_NORMATIVE_EXAMPLES: &[&str] = &["trip-proposal-envelope.example.json"];

/// Proposal Fragment — `fragment validate` only
const FRAGMENT_NON_NORMATIVE_EXAMPLES: &[&str] = &["proposal-fragment.example.json"];

fn run_cli(cwd: &Path, args: &[&str]) -> std::process::Output {
    common::run_cli_in(cwd, args)
}

fn combined_output(output: &std::process::Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

fn path_arg(path: &Path) -> String {
    path.to_str()
        .unwrap_or_else(|| panic!("non-UTF-8 path: {}", path.display()))
        .to_string()
}

#[test]
fn guard_schema_v8_public_examples_pass_trip_validate_export() {
    let workspace = common::TestWorkspace::new();
    let workdir = workspace.path();
    for name in SCHEMA_V8_PUBLIC_EXAMPLES {
        let path = public_example(name);
        assert!(path.is_file(), "missing public example: {}", path.display());
        let arg = path_arg(&path);
        let output = run_cli(&workdir, &["trip", "validate-export", &arg]);
        assert!(
            output.status.success(),
            "trip validate-export should PASS for {name}\n{}",
            combined_output(&output)
        );
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            stdout.contains("有効な export ファイル"),
            "expected valid export message for {name}"
        );
    }
}

#[test]
fn guard_envelope_non_normative_examples_pass_proposal_validate() {
    let workspace = common::TestWorkspace::new();
    let workdir = workspace.path();
    for name in ENVELOPE_NON_NORMATIVE_EXAMPLES {
        let path = non_normative_example(name);
        assert!(
            path.is_file(),
            "missing non-normative envelope example: {}",
            path.display()
        );
        let arg = path_arg(&path);
        let output = run_cli(&workdir, &["proposal", "validate", &arg]);
        assert!(
            output.status.success(),
            "proposal validate should PASS for {name}\n{}",
            combined_output(&output)
        );
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            stdout.contains("valid"),
            "expected valid proposal envelope for {name}"
        );
    }
}

#[test]
fn guard_fragment_non_normative_examples_pass_fragment_validate() {
    let workspace = common::TestWorkspace::new();
    let workdir = workspace.path();
    for name in FRAGMENT_NON_NORMATIVE_EXAMPLES {
        let path = non_normative_example(name);
        assert!(
            path.is_file(),
            "missing non-normative fragment example: {}",
            path.display()
        );
        let arg = path_arg(&path);
        let output = run_cli(&workdir, &["fragment", "validate", &arg]);
        assert!(
            output.status.success(),
            "fragment validate should PASS for {name}\n{}",
            combined_output(&output)
        );
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            stdout.contains("valid"),
            "expected valid proposal fragment for {name}"
        );
    }
}

#[test]
fn guard_schema_v8_trip_rejects_proposal_validate() {
    let workspace = common::TestWorkspace::new();
    let workdir = workspace.path();
    for name in SCHEMA_V8_PUBLIC_EXAMPLES {
        let path = public_example(name);
        let arg = path_arg(&path);
        let output = run_cli(&workdir, &["proposal", "validate", &arg]);
        assert!(
            !output.status.success(),
            "proposal validate must FAIL for schema v8 Trip {name}"
        );
        let combined = combined_output(&output);
        assert!(
            combined.contains("schema_version") || combined.contains("trip"),
            "expected Trip export rejection for {name}: {combined}"
        );
    }
}

#[test]
fn guard_schema_v8_trip_rejects_fragment_validate() {
    let workspace = common::TestWorkspace::new();
    let workdir = workspace.path();
    for name in SCHEMA_V8_PUBLIC_EXAMPLES {
        let path = public_example(name);
        let arg = path_arg(&path);
        let output = run_cli(&workdir, &["fragment", "validate", &arg]);
        assert!(
            !output.status.success(),
            "fragment validate must FAIL for schema v8 Trip {name}"
        );
        let combined = combined_output(&output);
        assert!(
            combined.contains("schema_version") || combined.contains("trip validate-export"),
            "expected Trip export rejection for {name}: {combined}"
        );
    }
}

#[test]
fn guard_envelope_rejects_fragment_validate() {
    let workspace = common::TestWorkspace::new();
    let workdir = workspace.path();
    for name in ENVELOPE_NON_NORMATIVE_EXAMPLES {
        let path = non_normative_example(name);
        let arg = path_arg(&path);
        let output = run_cli(&workdir, &["fragment", "validate", &arg]);
        assert!(
            !output.status.success(),
            "fragment validate must FAIL for Trip Proposal Envelope {name}"
        );
        let combined = combined_output(&output);
        assert!(
            combined.contains("Envelope") || combined.contains("proposal validate"),
            "expected Envelope rejection for {name}: {combined}"
        );
    }
}

#[test]
fn guard_fragment_rejects_proposal_validate() {
    let workspace = common::TestWorkspace::new();
    let workdir = workspace.path();
    for name in FRAGMENT_NON_NORMATIVE_EXAMPLES {
        let path = non_normative_example(name);
        let arg = path_arg(&path);
        let output = run_cli(&workdir, &["proposal", "validate", &arg]);
        assert!(
            !output.status.success(),
            "proposal validate must FAIL for Proposal Fragment {name}"
        );
        let combined = combined_output(&output);
        assert!(
            combined.contains("proposal"),
            "expected Fragment / missing proposal rejection for {name}: {combined}"
        );
    }
}

#[test]
fn guard_envelope_rejects_trip_validate_export() {
    let workspace = common::TestWorkspace::new();
    let workdir = workspace.path();
    for name in ENVELOPE_NON_NORMATIVE_EXAMPLES {
        let path = non_normative_example(name);
        let arg = path_arg(&path);
        let output = run_cli(&workdir, &["trip", "validate-export", &arg]);
        assert!(
            !output.status.success(),
            "trip validate-export must FAIL for Trip Proposal Envelope {name}"
        );
        let combined = combined_output(&output);
        assert!(
            combined.contains("無効な export") || combined.contains("構造が不正"),
            "expected invalid export for envelope {name}: {combined}"
        );
    }
}

#[test]
fn guard_fragment_rejects_trip_validate_export() {
    let workspace = common::TestWorkspace::new();
    let workdir = workspace.path();
    for name in FRAGMENT_NON_NORMATIVE_EXAMPLES {
        let path = non_normative_example(name);
        let arg = path_arg(&path);
        let output = run_cli(&workdir, &["trip", "validate-export", &arg]);
        assert!(
            !output.status.success(),
            "trip validate-export must FAIL for Proposal Fragment {name}"
        );
        let combined = combined_output(&output);
        assert!(
            combined.contains("無効な export") || combined.contains("構造が不正"),
            "expected invalid export for fragment {name}: {combined}"
        );
    }
}
