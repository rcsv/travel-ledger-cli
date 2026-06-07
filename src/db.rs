use anyhow::{Context, Result};
use chrono::Local;
use rusqlite::Connection;

pub(crate) const DB_FILE: &str = "caglla.db";

/// `query_row` の結果を変換する。行が無い場合は rusqlite の cause を残さずドメインエラーにする。
pub(crate) fn map_query_row<T, F>(result: rusqlite::Result<T>, not_found: F) -> Result<T>
where
    F: FnOnce() -> anyhow::Error,
{
    match result {
        Ok(value) => Ok(value),
        Err(rusqlite::Error::QueryReturnedNoRows) => Err(not_found()),
        Err(err) => Err(err.into()),
    }
}

/// 指定パスの DB に接続し、テーブルがなければ作成する
pub(crate) fn open_db_at(path: &str) -> Result<Connection> {
    let conn = Connection::open(path)
        .with_context(|| format!("データベース '{path}' を開けませんでした"))?;
    conn.execute_batch("PRAGMA foreign_keys = ON;")
        .context("外部キー制約の有効化に失敗しました")?;
    init_db(&conn)?;
    Ok(conn)
}

/// 本番 DB (caglla.db) に接続する
pub(crate) fn open_db() -> Result<Connection> {
    open_db_at(DB_FILE)
}

/// テーブルを作成する（既に存在する場合は何もしない）
pub(crate) fn init_db(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS trips (
            id          INTEGER PRIMARY KEY AUTOINCREMENT,
            name        TEXT NOT NULL,
            start_date  TEXT,
            end_date    TEXT,
            created_at  TEXT NOT NULL,
            updated_at  TEXT NOT NULL
        )",
        [],
    )
    .context("trips テーブルの作成に失敗しました")?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS itinerary_items (
            id          INTEGER PRIMARY KEY AUTOINCREMENT,
            trip_id     INTEGER NOT NULL,
            day         INTEGER NOT NULL,
            title       TEXT NOT NULL,
            note        TEXT,
            start_time  TEXT,
            sort_order  INTEGER NOT NULL DEFAULT 0,
            duration_minutes INTEGER,
            travel_minutes INTEGER,
            location    TEXT,
            category    TEXT,
            created_at  TEXT NOT NULL,
            updated_at  TEXT NOT NULL,
            FOREIGN KEY(trip_id) REFERENCES trips(id) ON DELETE CASCADE
        )",
        [],
    )
    .context("itinerary_items テーブルの作成に失敗しました")?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS checklist_items (
            id          INTEGER PRIMARY KEY AUTOINCREMENT,
            trip_id     INTEGER NOT NULL,
            title       TEXT NOT NULL,
            is_done     INTEGER NOT NULL DEFAULT 0,
            sort_order  INTEGER NOT NULL DEFAULT 0,
            created_at  TEXT NOT NULL,
            updated_at  TEXT NOT NULL,
            FOREIGN KEY(trip_id) REFERENCES trips(id) ON DELETE CASCADE
        )",
        [],
    )
    .context("checklist_items テーブルの作成に失敗しました")?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS days (
            id          INTEGER PRIMARY KEY AUTOINCREMENT,
            trip_id     INTEGER NOT NULL,
            day_number  INTEGER NOT NULL,
            title       TEXT NOT NULL DEFAULT '',
            description TEXT,
            created_at  TEXT NOT NULL,
            updated_at  TEXT NOT NULL,
            FOREIGN KEY(trip_id) REFERENCES trips(id) ON DELETE CASCADE,
            UNIQUE(trip_id, day_number)
        )",
        [],
    )
    .context("days テーブルの作成に失敗しました")?;
    migrate_itinerary_items(conn)?;
    migrate_days(conn)?;
    Ok(())
}

/// 列がなければ ALTER TABLE で追加する（既にある場合は何もしない）
fn add_column_if_not_exists(
    conn: &Connection,
    table: &str,
    column: &str,
    definition: &str,
) -> Result<()> {
    let mut stmt = conn
        .prepare(&format!("PRAGMA table_info({table})"))
        .with_context(|| format!("{table} テーブル情報の取得に失敗しました"))?;

    let exists = stmt
        .query_map([], |row| row.get::<_, String>(1))
        .with_context(|| format!("{table} テーブル情報の読み込みに失敗しました"))?
        .any(|name| name.map(|n| n == column).unwrap_or(false));

    if !exists {
        let sql = format!("ALTER TABLE {table} ADD COLUMN {column} {definition}");
        conn.execute(&sql, [])
            .with_context(|| format!("{table}.{column} 列の追加に失敗しました"))?;
    }
    Ok(())
}

/// 既存 DB 向け: Trip ごとに Day 行を backfill する
pub(crate) fn migrate_days(conn: &Connection) -> Result<()> {
    crate::day::migrate_days(conn)
}

/// 既存 DB 向け: itinerary_items に不足している列を追加する
pub(crate) fn migrate_itinerary_items(conn: &Connection) -> Result<()> {
    add_column_if_not_exists(conn, "itinerary_items", "start_time", "TEXT")?;
    add_column_if_not_exists(
        conn,
        "itinerary_items",
        "sort_order",
        "INTEGER NOT NULL DEFAULT 0",
    )?;
    add_column_if_not_exists(conn, "itinerary_items", "duration_minutes", "INTEGER")?;
    add_column_if_not_exists(conn, "itinerary_items", "travel_minutes", "INTEGER")?;
    add_column_if_not_exists(conn, "itinerary_items", "location", "TEXT")?;
    add_column_if_not_exists(conn, "itinerary_items", "category", "TEXT")?;
    Ok(())
}

/// 【開発用】全テーブルのデータを削除し、AUTOINCREMENT をリセットする
///
/// - checklist_items / itinerary_items → trips の順で削除する（外部キー参照を考慮）
/// - テーブル定義は残す
/// - 本番運用では使わないこと
pub(crate) fn reset_db(conn: &Connection) -> Result<()> {
    conn.execute("DELETE FROM checklist_items", [])
        .context("checklist_items の全削除に失敗しました")?;
    conn.execute("DELETE FROM itinerary_items", [])
        .context("itinerary_items の全削除に失敗しました")?;
    conn.execute("DELETE FROM days", [])
        .context("days の全削除に失敗しました")?;
    conn.execute("DELETE FROM trips", [])
        .context("trips の全削除に失敗しました")?;
    conn.execute(
        "DELETE FROM sqlite_sequence WHERE name IN ('checklist_items', 'itinerary_items', 'days', 'trips')",
        [],
    )
    .context("AUTOINCREMENT のリセットに失敗しました")?;
    Ok(())
}

/// 現在時刻を文字列で返す（created_at / updated_at 用）
pub(crate) fn now_string() -> String {
    Local::now().format("%Y-%m-%d %H:%M:%S").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{init_db, migrate_itinerary_items, open_db_at, reset_db};
    use crate::itinerary::add_itinerary_item;
    use crate::trip::{add_test_trip, list_trips};
    use rusqlite::Connection;

    fn test_db() -> Connection {
        open_db_at(":memory:").expect("インメモリ DB の作成に失敗")
    }

    #[test]
    fn test_init_db_creates_checklist_items_table() {
        let conn = Connection::open(":memory:").unwrap();
        init_db(&conn).unwrap();

        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master
                 WHERE type = 'table' AND name = 'checklist_items'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_init_db_creates_itinerary_items_table() {
        let conn = Connection::open(":memory:").unwrap();
        init_db(&conn).unwrap();

        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master
                 WHERE type = 'table' AND name = 'itinerary_items'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_init_db_creates_trips_table() {
        let conn = Connection::open(":memory:").unwrap();
        init_db(&conn).unwrap();

        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master
                 WHERE type = 'table' AND name = 'trips'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_migrate_itinerary_items_adds_columns() {
        // 旧スキーマの DB に対して migrate が列を追加できること
        let conn = Connection::open(":memory:").unwrap();
        conn.execute(
            "CREATE TABLE itinerary_items (
                id          INTEGER PRIMARY KEY AUTOINCREMENT,
                trip_id     INTEGER NOT NULL,
                day         INTEGER NOT NULL,
                title       TEXT NOT NULL,
                note        TEXT,
                created_at  TEXT NOT NULL,
                updated_at  TEXT NOT NULL
            )",
            [],
        )
        .unwrap();

        migrate_itinerary_items(&conn).unwrap();
        migrate_itinerary_items(&conn).unwrap(); // 2回実行してもエラーにならない

        let columns: Vec<String> = conn
            .prepare("PRAGMA table_info(itinerary_items)")
            .unwrap()
            .query_map([], |row| row.get(1))
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        assert!(columns.contains(&"start_time".to_string()));
        assert!(columns.contains(&"sort_order".to_string()));
        assert!(columns.contains(&"duration_minutes".to_string()));
        assert!(columns.contains(&"travel_minutes".to_string()));
        assert!(columns.contains(&"location".to_string()));
        assert!(columns.contains(&"category".to_string()));
    }

    #[test]
    fn test_reset_db() {
        let conn = test_db();
        let trip_id = add_test_trip(&conn, "沖縄旅行").unwrap();
        add_itinerary_item(
            &conn,
            trip_id,
            1,
            "首里城",
            None,
            Some("09:00"),
            None,
            None,
            None,
            None,
            None,
        )
        .unwrap();

        reset_db(&conn).unwrap();

        assert!(list_trips(&conn).unwrap().is_empty());

        // AUTOINCREMENT がリセットされ、次の ID は 1 から再開する
        let new_trip_id = add_test_trip(&conn, "新規旅行").unwrap();
        assert_eq!(new_trip_id, 1);

        let new_item_id = add_itinerary_item(
            &conn,
            new_trip_id,
            1,
            "テスト",
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .unwrap();
        assert_eq!(new_item_id, 1);
    }
}
