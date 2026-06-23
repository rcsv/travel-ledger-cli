# Security Policy

Caglla.Travel CLI is a local-first, personal-use command-line tool. This policy describes how security issues are handled and how to protect private travel data stored on your machine.

## Supported Versions

| Version | Supported |
|---|---|
| Latest release | Yes |
| Older releases | No, unless the issue is critical |

Security fixes are generally applied to the latest release only. If you are on an older version, upgrading is the recommended first step.

## Project Security Model

- **Caglla CLI is a local-first command-line application.** It runs on your machine and operates on files you choose.
- **Data is stored in a local SQLite database file** named `caglla.db` (location depends on your setup).
- **There is no hosted backend, cloud sync, user accounts, remote API, or built-in network service** at this time.
- **Most security risks are therefore local:** handling of the database file, JSON import/export, generated Markdown, file paths, dependency vulnerabilities, and accidental sharing of private travel data.

## Local Database Privacy

SQLite database files are binary, but **they are not encrypted by default**. Text values stored in SQLite may be visible with binary viewers, forensic tools, backup tools, or simple string extraction commands (for example `strings caglla.db`).

**Caglla CLI does not currently encrypt `caglla.db`.** Treat `caglla.db` as sensitive private data.

A `caglla.db` file may contain:

- Travel dates
- Hotel names
- Flight details
- Locations
- Expenses
- Notes
- Participant or family information (in future versions)

**Please do not include a real `caglla.db` in public issues, pull requests, screenshots, or attachments.** When reporting bugs or reproducing issues, use fake or anonymized data.

Also be aware of SQLite sidecar files, which may contain the same private data:

- `caglla.db-wal`
- `caglla.db-shm`

## Sensitive Data

In addition to `caglla.db`, treat the following as private data:

- Exported JSON (trip export files)
- Generated Markdown (travel guide output)
- Screenshots of terminal or file contents
- Logs that may include paths or trip details
- Sample data copied from real trips

**Do not paste real travel data into public GitHub issues.** Use synthetic examples when describing problems.

## Reporting a Vulnerability

**Please do not report security vulnerabilities in a public GitHub issue.**

Preferred channels:

1. **GitHub private vulnerability reporting** — if enabled for this repository, use [Security → Report a vulnerability](https://github.com/rcsv/travel-ledger-cli/security/advisories/new) on GitHub.
2. **Direct contact** — otherwise, contact the repository owner privately.

Do not include detailed exploit steps or real private travel data in public issues.

When reporting, please include:

- Affected version or commit
- Operating system
- Steps to reproduce
- Expected impact
- Whether the issue requires:
  - A crafted import file
  - Local filesystem access
  - User interaction
- A proposed fix, if you have one

## Scope

### In scope

- Unsafe handling of imported JSON files
- Path traversal
- Unexpected file overwrite
- Accidental exposure of local database contents through CLI output or exports
- Command injection risks in CLI behavior
- Unsafe temporary file handling
- Dependency vulnerabilities that affect normal CLI usage
- Generated Markdown that could embed unsafe content when opened in downstream tools

### Out of scope

- Issues that require full compromise of the local machine
- Vulnerabilities in unrelated third-party tools
- Social engineering attacks
- Problems caused by manually editing the SQLite database into an invalid state
- Denial-of-service cases that only affect intentionally huge local files, unless severe or easy to trigger accidentally

## Import / Export Safety

JSON import/export and Markdown export are **trust boundaries**.

- **Imported files should be treated as untrusted input** unless you created them or received them from a source you trust.
- The project aims to avoid:
  - Writing files outside the requested output path
  - Overwriting files unexpectedly
  - Treating unknown schema fields as executable behavior
  - Exposing private trip data unless explicitly requested (for example via export commands)

If you import JSON from an untrusted source, review the file before importing and prefer exporting from a known-good copy of your own data.

## Secrets

Do not commit the following to this repository or share them publicly:

- API keys
- Access tokens
- Private certificates
- Real `caglla.db` files
- SQLite sidecar files (`caglla.db-wal`, `caglla.db-shm`, etc.)
- Exported trip JSON containing private data
- Generated Markdown containing private data

This project may use secret-scanning tools (for example Gitleaks- or TruffleHog-style checks) to reduce the chance of accidental credential commits.

## Dependencies

Caglla CLI is written in Rust and depends on crates from crates.io. When reporting a dependency vulnerability, please include:

- Affected crate name
- Vulnerable version
- Fixed version, if known
- Whether Caglla CLI uses the affected code path in normal usage

## Disclosure

We aim to allow reasonable time for investigation and fixes before public disclosure. This is a personal/open-source project, so response times may vary. Issues that could affect user data or local privacy will be prioritized. There is no formal SLA.

Thank you for helping keep Caglla CLI and its users safe.
