use anyhow::{bail, Context, Result};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use toml::map::Map;
use toml::Value;

pub(crate) const CONFIG_FILE_NAME: &str = "caglla.toml";
pub(crate) const DEFAULT_DB_FILE: &str = "caglla.db";
pub(crate) const ENV_DB_PATH: &str = "CAGLLA_DB";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DbPathSource {
    Cli,
    Env,
    Config,
    Default,
}

impl DbPathSource {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Cli => "cli",
            Self::Env => "env",
            Self::Config => "config",
            Self::Default => "default",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ResolvedDbPath {
    pub path: PathBuf,
    pub source: DbPathSource,
    pub config_path: Option<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DbPathResolveInputs<'a> {
    pub cwd: &'a Path,
    pub cli_db: Option<&'a Path>,
    pub env_caglla_db: Option<&'a str>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DbUseResult {
    pub config_path: PathBuf,
    pub saved_path: Option<String>,
    pub config_removed: bool,
    pub resolved_db_path: PathBuf,
}

pub(crate) fn resolve_db_path_for_cli(cli_db: Option<&Path>) -> Result<ResolvedDbPath> {
    let cwd = std::env::current_dir().context("作業ディレクトリの取得に失敗しました")?;
    resolve_db_path(DbPathResolveInputs {
        cwd: &cwd,
        cli_db,
        env_caglla_db: std::env::var(ENV_DB_PATH).ok().as_deref(),
    })
}

pub(crate) fn resolve_db_path(inputs: DbPathResolveInputs<'_>) -> Result<ResolvedDbPath> {
    if let Some(cli_path) = inputs.cli_db {
        return Ok(ResolvedDbPath {
            path: resolve_user_path(inputs.cwd, cli_path)?,
            source: DbPathSource::Cli,
            config_path: None,
        });
    }

    if let Some(env_path) = inputs.env_caglla_db {
        if !env_path.trim().is_empty() {
            return Ok(ResolvedDbPath {
                path: resolve_user_path(inputs.cwd, Path::new(env_path))?,
                source: DbPathSource::Env,
                config_path: None,
            });
        }
    }

    let config_path = inputs.cwd.join(CONFIG_FILE_NAME);
    if config_path.is_file() {
        if let Some(configured) = read_optional_database_path_from_config(&config_path)? {
            let config_dir = config_path
                .parent()
                .context("config ファイルの親ディレクトリを解決できませんでした")?;
            return Ok(ResolvedDbPath {
                path: resolve_user_path(config_dir, Path::new(&configured))?,
                source: DbPathSource::Config,
                config_path: Some(config_path),
            });
        }
    }

    Ok(ResolvedDbPath {
        path: inputs.cwd.join(DEFAULT_DB_FILE),
        source: DbPathSource::Default,
        config_path: None,
    })
}

pub(crate) fn run_db_use(path: Option<&Path>, clear: bool) -> Result<DbUseResult> {
    if clear {
        return clear_db_use_config();
    }
    let Some(path) = path else {
        bail!("データベースパスを指定するか、--clear を使用してください");
    };
    set_db_use_config(path)
}

pub(crate) fn print_db_use_result(result: &DbUseResult) -> Result<()> {
    if result.config_removed {
        println!("Database path cleared from config");
        println!("  Config : {} (removed)", result.config_path.display());
        println!("  Default: ./{DEFAULT_DB_FILE}");
        return Ok(());
    }

    if let Some(saved) = &result.saved_path {
        println!("Database path saved to config");
        println!("  Config : {}", result.config_path.display());
        println!("  Path   : {saved}");
        if !result.resolved_db_path.exists() {
            println!(
                "Note: database file does not exist yet. It will be created when a command opens it."
            );
        }
        return Ok(());
    }

    println!("Database path cleared from config");
    println!("  Config : {}", result.config_path.display());
    println!("  Default: ./{DEFAULT_DB_FILE}");
    Ok(())
}

fn set_db_use_config(user_path: &Path) -> Result<DbUseResult> {
    let cwd = std::env::current_dir().context("作業ディレクトリの取得に失敗しました")?;
    let config_path = cwd.join(CONFIG_FILE_NAME);
    let saved_path = format_path_for_config_storage(&cwd, user_path)?;
    let resolved_db_path = resolve_user_path(&cwd, user_path)?;

    let mut root = load_config_root_table(&config_path)?;
    let database = root
        .entry("database".to_string())
        .or_insert_with(|| Value::Table(Map::new()));
    let Value::Table(db_table) = database else {
        bail!(
            "config ファイル '{}' の [database] はテーブルである必要があります",
            config_path.display()
        );
    };
    db_table.insert("path".to_string(), Value::String(saved_path.clone()));

    write_config_root_table(&config_path, &root)?;

    Ok(DbUseResult {
        config_path,
        saved_path: Some(saved_path),
        config_removed: false,
        resolved_db_path,
    })
}

fn clear_db_use_config() -> Result<DbUseResult> {
    let cwd = std::env::current_dir().context("作業ディレクトリの取得に失敗しました")?;
    let config_path = cwd.join(CONFIG_FILE_NAME);

    if !config_path.is_file() {
        return Ok(DbUseResult {
            config_path,
            saved_path: None,
            config_removed: false,
            resolved_db_path: cwd.join(DEFAULT_DB_FILE),
        });
    }

    let mut root = load_config_root_table(&config_path)?;
    let remove_file = clear_database_path_in_root(&mut root);

    if remove_file {
        fs::remove_file(&config_path).with_context(|| {
            format!(
                "config ファイル '{}' の削除に失敗しました",
                config_path.display()
            )
        })?;
        return Ok(DbUseResult {
            config_path,
            saved_path: None,
            config_removed: true,
            resolved_db_path: cwd.join(DEFAULT_DB_FILE),
        });
    }

    write_config_root_table(&config_path, &root)?;

    Ok(DbUseResult {
        config_path,
        saved_path: None,
        config_removed: false,
        resolved_db_path: cwd.join(DEFAULT_DB_FILE),
    })
}

fn load_config_root_table(config_path: &Path) -> Result<Map<String, Value>> {
    if !config_path.is_file() {
        return Ok(Map::new());
    }
    let contents = fs::read_to_string(config_path).with_context(|| {
        format!(
            "config ファイル '{}' の読み込みに失敗しました",
            config_path.display()
        )
    })?;
    let parsed: Value = toml::from_str(&contents).with_context(|| {
        format!(
            "config ファイル '{}' の TOML 解析に失敗しました",
            config_path.display()
        )
    })?;
    match parsed {
        Value::Table(table) => Ok(table),
        _ => bail!(
            "config ファイル '{}' のルートはテーブルである必要があります",
            config_path.display()
        ),
    }
}

fn write_config_root_table(config_path: &Path, root: &Map<String, Value>) -> Result<()> {
    let contents = toml::to_string_pretty(&Value::Table(root.clone()))
        .context("config ファイルのシリアライズに失敗しました")?;
    atomic_write(config_path, contents.as_bytes())
}

fn clear_database_path_in_root(root: &mut Map<String, Value>) -> bool {
    if let Some(Value::Table(db)) = root.get_mut("database") {
        db.remove("path");
        if db.is_empty() {
            root.remove("database");
        }
    }
    root.is_empty()
}

fn atomic_write(path: &Path, contents: &[u8]) -> Result<()> {
    let parent = path
        .parent()
        .context("config ファイルの親ディレクトリを解決できませんでした")?;
    fs::create_dir_all(parent).with_context(|| {
        format!(
            "config ファイルの親ディレクトリ '{}' の作成に失敗しました",
            parent.display()
        )
    })?;
    let temp_path = parent.join(format!(
        ".{}.{}.tmp",
        path.file_name()
            .and_then(|name| name.to_str())
            .unwrap_or(CONFIG_FILE_NAME),
        std::process::id()
    ));
    {
        let mut file = fs::File::create(&temp_path).with_context(|| {
            format!(
                "一時 config ファイル '{}' の作成に失敗しました",
                temp_path.display()
            )
        })?;
        file.write_all(contents).with_context(|| {
            format!(
                "一時 config ファイル '{}' への書き込みに失敗しました",
                temp_path.display()
            )
        })?;
    }
    fs::rename(&temp_path, path).with_context(|| {
        format!(
            "config ファイル '{}' への反映に失敗しました",
            path.display()
        )
    })?;
    Ok(())
}

fn read_optional_database_path_from_config(config_path: &Path) -> Result<Option<String>> {
    let contents = fs::read_to_string(config_path).with_context(|| {
        format!(
            "config ファイル '{}' の読み込みに失敗しました",
            config_path.display()
        )
    })?;
    let parsed: Value = toml::from_str(&contents).with_context(|| {
        format!(
            "config ファイル '{}' の TOML 解析に失敗しました",
            config_path.display()
        )
    })?;
    let Value::Table(root) = parsed else {
        bail!(
            "config ファイル '{}' のルートはテーブルである必要があります",
            config_path.display()
        );
    };
    let Some(Value::Table(database)) = root.get("database") else {
        return Ok(None);
    };
    let Some(Value::String(path)) = database.get("path") else {
        return Ok(None);
    };
    let path = path.trim();
    if path.is_empty() {
        bail!(
            "config ファイル '{}' の [database].path が空です",
            config_path.display()
        );
    }
    Ok(Some(path.to_string()))
}

fn format_path_for_config_storage(config_dir: &Path, user_input: &Path) -> Result<String> {
    if user_input.is_absolute() {
        let normalized = normalize_path(user_input.to_path_buf());
        if let Ok(relative) = normalized.strip_prefix(config_dir) {
            return Ok(relative_path_string(relative));
        }
        return Ok(normalized.to_string_lossy().into_owned());
    }

    let normalized = normalize_path(user_input.to_path_buf());
    let relative = normalized.to_string_lossy();
    if relative.starts_with("..") {
        return Ok(relative.into_owned());
    }
    if relative.starts_with("./") {
        return Ok(relative.into_owned());
    }
    Ok(format!("./{relative}"))
}

fn relative_path_string(relative: &Path) -> String {
    let relative = relative.as_os_str().to_string_lossy();
    if relative.is_empty() {
        "./".to_string()
    } else if relative.starts_with("..") {
        relative.into_owned()
    } else {
        format!("./{relative}")
    }
}

fn resolve_user_path(base: &Path, path: &Path) -> Result<PathBuf> {
    let joined = if path.is_absolute() {
        path.to_path_buf()
    } else {
        base.join(path)
    };
    Ok(normalize_path(joined))
}

fn normalize_path(path: PathBuf) -> PathBuf {
    use std::path::Component;
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                if !normalized.pop() {
                    normalized.push(component);
                }
            }
            other => normalized.push(other),
        }
    }
    normalized
}

#[cfg(test)]
pub(crate) fn set_db_use_config_at(cwd: &Path, user_path: &Path) -> Result<DbUseResult> {
    let config_path = cwd.join(CONFIG_FILE_NAME);
    let saved_path = format_path_for_config_storage(cwd, user_path)?;
    let resolved_db_path = resolve_user_path(cwd, user_path)?;

    let mut root = load_config_root_table(&config_path)?;
    let database = root
        .entry("database".to_string())
        .or_insert_with(|| Value::Table(Map::new()));
    let Value::Table(db_table) = database else {
        bail!("[database] must be a table");
    };
    db_table.insert("path".to_string(), Value::String(saved_path.clone()));

    write_config_root_table(&config_path, &root)?;

    Ok(DbUseResult {
        config_path,
        saved_path: Some(saved_path),
        config_removed: false,
        resolved_db_path,
    })
}

#[cfg(test)]
pub(crate) fn clear_db_use_config_at(cwd: &Path) -> Result<DbUseResult> {
    let config_path = cwd.join(CONFIG_FILE_NAME);
    if !config_path.is_file() {
        return Ok(DbUseResult {
            config_path,
            saved_path: None,
            config_removed: false,
            resolved_db_path: cwd.join(DEFAULT_DB_FILE),
        });
    }

    let mut root = load_config_root_table(&config_path)?;
    let remove_file = clear_database_path_in_root(&mut root);

    if remove_file {
        fs::remove_file(&config_path)?;
        return Ok(DbUseResult {
            config_path,
            saved_path: None,
            config_removed: true,
            resolved_db_path: cwd.join(DEFAULT_DB_FILE),
        });
    }

    write_config_root_table(&config_path, &root)?;

    Ok(DbUseResult {
        config_path,
        saved_path: None,
        config_removed: false,
        resolved_db_path: cwd.join(DEFAULT_DB_FILE),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::sync::atomic::{AtomicU64, Ordering};

    static TEST_DIR_COUNTER: AtomicU64 = AtomicU64::new(0);

    struct TestWorkdir(std::path::PathBuf);

    impl TestWorkdir {
        fn new() -> Self {
            let n = TEST_DIR_COUNTER.fetch_add(1, Ordering::Relaxed);
            let dir = std::env::temp_dir().join(format!("travel-ledger-cli-config-test-{n}"));
            let _ = fs::remove_dir_all(&dir);
            fs::create_dir_all(&dir).unwrap();
            Self(dir)
        }

        fn path(&self) -> &Path {
            &self.0
        }
    }

    impl Drop for TestWorkdir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn write_config(dir: &Path, contents: &str) {
        fs::write(dir.join(CONFIG_FILE_NAME), contents).unwrap();
    }

    #[test]
    fn resolve_cli_path_has_highest_priority() {
        let dir = TestWorkdir::new();
        write_config(dir.path(), "[database]\npath = \"./from-config.db\"\n");
        let resolved = resolve_db_path(DbPathResolveInputs {
            cwd: dir.path(),
            cli_db: Some(Path::new("./from-cli.db")),
            env_caglla_db: Some("./from-env.db"),
        })
        .unwrap();
        assert_eq!(resolved.source, DbPathSource::Cli);
        assert_eq!(resolved.path, dir.path().join("from-cli.db"));
        assert!(resolved.config_path.is_none());
    }

    #[test]
    fn resolve_env_path_beats_config_and_default() {
        let dir = TestWorkdir::new();
        write_config(dir.path(), "[database]\npath = \"./from-config.db\"\n");
        let resolved = resolve_db_path(DbPathResolveInputs {
            cwd: dir.path(),
            cli_db: None,
            env_caglla_db: Some("./from-env.db"),
        })
        .unwrap();
        assert_eq!(resolved.source, DbPathSource::Env);
        assert_eq!(resolved.path, dir.path().join("from-env.db"));
    }

    #[test]
    fn resolve_config_path_beats_default() {
        let dir = TestWorkdir::new();
        write_config(dir.path(), "[database]\npath = \"./from-config.db\"\n");
        let resolved = resolve_db_path(DbPathResolveInputs {
            cwd: dir.path(),
            cli_db: None,
            env_caglla_db: None,
        })
        .unwrap();
        assert_eq!(resolved.source, DbPathSource::Config);
        assert_eq!(resolved.path, dir.path().join("from-config.db"));
        assert_eq!(
            resolved.config_path,
            Some(dir.path().join(CONFIG_FILE_NAME))
        );
    }

    #[test]
    fn resolve_default_uses_cwd_caglla_db() {
        let dir = TestWorkdir::new();
        let resolved = resolve_db_path(DbPathResolveInputs {
            cwd: dir.path(),
            cli_db: None,
            env_caglla_db: None,
        })
        .unwrap();
        assert_eq!(resolved.source, DbPathSource::Default);
        assert_eq!(resolved.path, dir.path().join(DEFAULT_DB_FILE));
    }

    #[test]
    fn resolve_config_relative_path_uses_config_dir() {
        let dir = TestWorkdir::new();
        fs::write(
            dir.path().join(CONFIG_FILE_NAME),
            "[database]\npath = \"data/app.db\"\n",
        )
        .unwrap();
        let resolved = resolve_db_path(DbPathResolveInputs {
            cwd: dir.path(),
            cli_db: None,
            env_caglla_db: None,
        })
        .unwrap();
        assert_eq!(resolved.path, dir.path().join("data/app.db"));
    }

    #[test]
    fn resolve_invalid_toml_is_error() {
        let dir = TestWorkdir::new();
        write_config(dir.path(), "not = [valid");
        let err = resolve_db_path(DbPathResolveInputs {
            cwd: dir.path(),
            cli_db: None,
            env_caglla_db: None,
        })
        .unwrap_err();
        assert!(err.to_string().contains("TOML"));
    }

    #[test]
    fn resolve_empty_database_path_is_error() {
        let dir = TestWorkdir::new();
        write_config(dir.path(), "[database]\npath = \"\"\n");
        let err = resolve_db_path(DbPathResolveInputs {
            cwd: dir.path(),
            cli_db: None,
            env_caglla_db: None,
        })
        .unwrap_err();
        assert!(err.to_string().contains("path が空"));
    }

    #[test]
    fn resolve_config_without_database_path_uses_default() {
        let dir = TestWorkdir::new();
        write_config(dir.path(), "[other]\nkey = \"value\"\n");
        let resolved = resolve_db_path(DbPathResolveInputs {
            cwd: dir.path(),
            cli_db: None,
            env_caglla_db: None,
        })
        .unwrap();
        assert_eq!(resolved.source, DbPathSource::Default);
        assert_eq!(resolved.path, dir.path().join(DEFAULT_DB_FILE));
    }

    #[test]
    fn format_path_for_storage_normalizes_relative_inputs() {
        let dir = TestWorkdir::new();
        assert_eq!(
            format_path_for_config_storage(dir.path(), Path::new("data/app.db")).unwrap(),
            "./data/app.db"
        );
        assert_eq!(
            format_path_for_config_storage(dir.path(), Path::new("./data/app.db")).unwrap(),
            "./data/app.db"
        );
        assert_eq!(
            format_path_for_config_storage(dir.path(), Path::new("../shared/other.db")).unwrap(),
            "../shared/other.db"
        );
    }

    #[test]
    fn format_path_for_storage_relativizes_absolute_under_config_dir() {
        let dir = TestWorkdir::new();
        let abs = dir.path().join("data/app.db");
        assert_eq!(
            format_path_for_config_storage(dir.path(), &abs).unwrap(),
            "./data/app.db"
        );
    }

    #[test]
    fn format_path_for_storage_keeps_absolute_outside_config_dir() {
        let dir = TestWorkdir::new();
        let outside = std::env::temp_dir().join("outside-caglla.db");
        assert_eq!(
            format_path_for_config_storage(dir.path(), &outside).unwrap(),
            normalize_path(outside).to_string_lossy()
        );
    }

    #[test]
    fn set_db_use_config_creates_new_file() {
        let dir = TestWorkdir::new();
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(dir.path()).unwrap();
        let result = set_db_use_config(Path::new("./data/app.db")).unwrap();
        std::env::set_current_dir(original_dir).unwrap();

        let contents = fs::read_to_string(dir.path().join(CONFIG_FILE_NAME)).unwrap();
        let parsed: Value = toml::from_str(&contents).unwrap();
        assert_eq!(
            parsed["database"]["path"].as_str().unwrap(),
            "./data/app.db"
        );
        assert_eq!(result.saved_path.as_deref(), Some("./data/app.db"));
    }

    #[test]
    fn set_db_use_config_overwrites_existing_path() {
        let dir = TestWorkdir::new();
        write_config(
            dir.path(),
            "[database]\npath = \"./old.db\"\nother = \"keep\"\n",
        );
        set_db_use_config_at(dir.path(), Path::new("data/new.db")).unwrap();
        let parsed: Value =
            toml::from_str(&fs::read_to_string(dir.path().join(CONFIG_FILE_NAME)).unwrap())
                .unwrap();
        assert_eq!(
            parsed["database"]["path"].as_str().unwrap(),
            "./data/new.db"
        );
        assert_eq!(parsed["database"]["other"].as_str().unwrap(), "keep");
    }

    #[test]
    fn set_db_use_config_preserves_unknown_top_level_table() {
        let dir = TestWorkdir::new();
        write_config(
            dir.path(),
            "[other]\nkey = \"value\"\n\n[database]\npath = \"./old.db\"\n",
        );
        set_db_use_config_at(dir.path(), Path::new("./new.db")).unwrap();
        let parsed: Value =
            toml::from_str(&fs::read_to_string(dir.path().join(CONFIG_FILE_NAME)).unwrap())
                .unwrap();
        assert_eq!(parsed["other"]["key"].as_str().unwrap(), "value");
    }

    #[test]
    fn set_db_use_config_allows_missing_db_file() {
        let dir = TestWorkdir::new();
        let result = set_db_use_config_at(dir.path(), Path::new("./missing.db")).unwrap();
        assert!(!result.resolved_db_path.exists());
        assert!(dir.path().join(CONFIG_FILE_NAME).is_file());
    }

    #[test]
    fn set_db_use_config_invalid_toml_does_not_corrupt_file() {
        let dir = TestWorkdir::new();
        let invalid = "not = [valid";
        write_config(dir.path(), invalid);
        let err = set_db_use_config_at(dir.path(), Path::new("./app.db")).unwrap_err();
        assert!(err.to_string().contains("TOML"));
        let contents = fs::read_to_string(dir.path().join(CONFIG_FILE_NAME)).unwrap();
        assert_eq!(contents, invalid);
    }

    #[test]
    fn clear_db_use_config_removes_path_and_file_when_empty() {
        let dir = TestWorkdir::new();
        write_config(dir.path(), "[database]\npath = \"./app.db\"\n");
        let result = clear_db_use_config_at(dir.path()).unwrap();
        assert!(result.config_removed);
        assert!(!dir.path().join(CONFIG_FILE_NAME).exists());
    }

    #[test]
    fn clear_db_use_config_keeps_other_sections() {
        let dir = TestWorkdir::new();
        write_config(
            dir.path(),
            "[other]\nkey = \"value\"\n\n[database]\npath = \"./app.db\"\n",
        );
        let result = clear_db_use_config_at(dir.path()).unwrap();
        assert!(!result.config_removed);
        let parsed: Value =
            toml::from_str(&fs::read_to_string(dir.path().join(CONFIG_FILE_NAME)).unwrap())
                .unwrap();
        assert!(parsed.get("database").is_none());
        assert_eq!(parsed["other"]["key"].as_str().unwrap(), "value");
    }

    #[test]
    fn clear_db_use_config_keeps_other_database_keys() {
        let dir = TestWorkdir::new();
        write_config(
            dir.path(),
            "[database]\npath = \"./app.db\"\nother = \"keep\"\n",
        );
        clear_db_use_config_at(dir.path()).unwrap();
        let parsed: Value =
            toml::from_str(&fs::read_to_string(dir.path().join(CONFIG_FILE_NAME)).unwrap())
                .unwrap();
        assert!(parsed["database"].get("path").is_none());
        assert_eq!(parsed["database"]["other"].as_str().unwrap(), "keep");
    }
}
