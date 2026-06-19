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

/// 変更を 1 トランザクションにまとめて commit する。`f` が Err のときは rollback。
pub(crate) fn with_transaction(
    conn: &Connection,
    label: &str,
    f: impl FnOnce(&Connection) -> Result<()>,
) -> Result<()> {
    let tx = conn
        .unchecked_transaction()
        .with_context(|| format!("{label}: トランザクション開始に失敗しました"))?;
    f(&tx).with_context(|| format!("{label}: 処理に失敗しました"))?;
    tx.commit()
        .with_context(|| format!("{label}: トランザクション確定に失敗しました"))?;
    Ok(())
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
            summary     TEXT,
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
            summary     TEXT,
            created_at  TEXT NOT NULL,
            updated_at  TEXT NOT NULL,
            FOREIGN KEY(trip_id) REFERENCES trips(id) ON DELETE CASCADE,
            UNIQUE(trip_id, day_number)
        )",
        [],
    )
    .context("days テーブルの作成に失敗しました")?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS notes (
            id          INTEGER PRIMARY KEY AUTOINCREMENT,
            owner_type  TEXT NOT NULL,
            owner_id    INTEGER NOT NULL,
            title       TEXT,
            body        TEXT NOT NULL,
            sort_order  INTEGER NOT NULL DEFAULT 0,
            created_at  TEXT NOT NULL,
            updated_at  TEXT NOT NULL,
            CHECK (owner_type IN ('trip', 'day', 'itinerary'))
        )",
        [],
    )
    .context("notes テーブルの作成に失敗しました")?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS expenses (
            id              INTEGER PRIMARY KEY AUTOINCREMENT,
            itinerary_id    INTEGER NOT NULL,
            title           TEXT,
            amount          INTEGER NOT NULL,
            currency        TEXT NOT NULL,
            paid_by_name    TEXT,
            expense_date    TEXT,
            note            TEXT,
            sort_order      INTEGER NOT NULL DEFAULT 0,
            created_at      TEXT NOT NULL,
            updated_at      TEXT NOT NULL
        )",
        [],
    )
    .context("expenses テーブルの作成に失敗しました")?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS reservations (
            id                  INTEGER PRIMARY KEY AUTOINCREMENT,
            itinerary_id        INTEGER NOT NULL,
            reservation_type    TEXT NOT NULL,
            provider_name       TEXT NOT NULL,
            confirmation_code   TEXT,
            reservation_site_url TEXT,
            remark              TEXT,
            start_at            TEXT,
            end_at              TEXT,
            created_at          TEXT NOT NULL,
            updated_at          TEXT NOT NULL
        )",
        [],
    )
    .context("reservations テーブルの作成に失敗しました")?;
    migrate_itinerary_items(conn)?;
    migrate_days(conn)?;
    migrate_itinerary_day_id(conn)?;
    migrate_summaries(conn)?;
    migrate_indexes(conn)?;
    crate::participant::migrate_participants(conn)?;
    Ok(())
}

fn create_index_if_not_exists(conn: &Connection, name: &str, sql: &str) -> Result<()> {
    conn.execute(sql, [])
        .with_context(|| format!("インデックス '{name}' の作成に失敗しました"))?;
    Ok(())
}

/// 推奨インデックスを作成する（既にある場合は何もしない）
pub(crate) fn migrate_indexes(conn: &Connection) -> Result<()> {
    create_index_if_not_exists(
        conn,
        "idx_itinerary_items_day_id",
        "CREATE INDEX IF NOT EXISTS idx_itinerary_items_day_id ON itinerary_items(day_id)",
    )?;
    create_index_if_not_exists(
        conn,
        "idx_itinerary_items_trip_id",
        "CREATE INDEX IF NOT EXISTS idx_itinerary_items_trip_id ON itinerary_items(trip_id)",
    )?;
    create_index_if_not_exists(
        conn,
        "idx_days_trip_day_number",
        "CREATE INDEX IF NOT EXISTS idx_days_trip_day_number ON days(trip_id, day_number)",
    )?;
    create_index_if_not_exists(
        conn,
        "idx_notes_owner",
        "CREATE INDEX IF NOT EXISTS idx_notes_owner ON notes(owner_type, owner_id)",
    )?;
    create_index_if_not_exists(
        conn,
        "idx_expenses_itinerary",
        "CREATE INDEX IF NOT EXISTS idx_expenses_itinerary ON expenses(itinerary_id)",
    )?;
    create_index_if_not_exists(
        conn,
        "idx_reservations_itinerary",
        "CREATE INDEX IF NOT EXISTS idx_reservations_itinerary ON reservations(itinerary_id)",
    )?;
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

fn column_exists(conn: &Connection, table: &str, column: &str) -> Result<bool> {
    let mut stmt = conn
        .prepare(&format!("PRAGMA table_info({table})"))
        .with_context(|| format!("{table} テーブル情報の取得に失敗しました"))?;
    let mut names = stmt
        .query_map([], |row| row.get::<_, String>(1))
        .with_context(|| format!("{table} テーブル情報の読み込みに失敗しました"))?;
    Ok(names.any(|name| name.map(|n| n == column).unwrap_or(false)))
}

/// trips.summary 追加、days.description → summary リネーム（既存 DB 向け）
pub(crate) fn migrate_summaries(conn: &Connection) -> Result<()> {
    add_column_if_not_exists(conn, "trips", "summary", "TEXT")?;

    let has_summary = column_exists(conn, "days", "summary")?;
    let has_description = column_exists(conn, "days", "description")?;
    if !has_summary && has_description {
        conn.execute("ALTER TABLE days RENAME COLUMN description TO summary", [])
            .context("days.description → summary のリネームに失敗しました")?;
    } else if !has_summary {
        add_column_if_not_exists(conn, "days", "summary", "TEXT")?;
    }
    Ok(())
}

/// 既存 DB 向け: itinerary_items.day_id を backfill する
pub(crate) fn migrate_itinerary_day_id(conn: &Connection) -> Result<()> {
    add_column_if_not_exists(
        conn,
        "itinerary_items",
        "day_id",
        "INTEGER REFERENCES days(id)",
    )?;
    conn.execute(
        "UPDATE itinerary_items
         SET day_id = (
           SELECT d.id FROM days d
           WHERE d.trip_id = itinerary_items.trip_id
             AND d.day_number = itinerary_items.day
         )
         WHERE day_id IS NULL",
        [],
    )
    .context("itinerary_items.day_id の backfill に失敗しました")?;

    let unresolved: i64 = conn.query_row(
        "SELECT COUNT(*) FROM itinerary_items WHERE day_id IS NULL",
        [],
        |row| row.get(0),
    )?;
    if unresolved > 0 {
        anyhow::bail!("itinerary_items.day_id の backfill が未完了です（{unresolved} 件）");
    }
    Ok(())
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
    conn.execute("DELETE FROM notes", [])
        .context("notes の全削除に失敗しました")?;
    conn.execute("DELETE FROM reservations", [])
        .context("reservations の全削除に失敗しました")?;
    conn.execute("DELETE FROM expenses", [])
        .context("expenses の全削除に失敗しました")?;
    conn.execute("DELETE FROM participants", [])
        .context("participants の全削除に失敗しました")?;
    conn.execute("DELETE FROM checklist_items", [])
        .context("checklist_items の全削除に失敗しました")?;
    conn.execute("DELETE FROM itinerary_items", [])
        .context("itinerary_items の全削除に失敗しました")?;
    conn.execute("DELETE FROM days", [])
        .context("days の全削除に失敗しました")?;
    conn.execute("DELETE FROM trips", [])
        .context("trips の全削除に失敗しました")?;
    conn.execute(
        "DELETE FROM sqlite_sequence WHERE name IN ('reservations', 'expenses', 'notes', 'participants', 'checklist_items', 'itinerary_items', 'days', 'trips')",
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
    use rusqlite::{params, Connection};
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
    fn test_init_db_creates_reservations_table() {
        let conn = Connection::open(":memory:").unwrap();
        init_db(&conn).unwrap();

        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master
                 WHERE type = 'table' AND name = 'reservations'",
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
    fn test_migrate_itinerary_day_id_backfills_from_day_number() {
        let conn = Connection::open(":memory:").unwrap();
        init_db(&conn).unwrap();
        let trip_id = add_test_trip(&conn, "Migrate Day Id Trip").unwrap();
        add_itinerary_item(
            &conn, trip_id, 2, "Activity", None, None, None, None, None, None, None,
        )
        .unwrap();

        let day_id: i64 = conn
            .query_row(
                "SELECT day_id FROM itinerary_items WHERE trip_id = ?1",
                params![trip_id],
                |row| row.get(0),
            )
            .unwrap();
        let expected_day_id: i64 = conn
            .query_row(
                "SELECT id FROM days WHERE trip_id = ?1 AND day_number = 2",
                params![trip_id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(day_id, expected_day_id);
    }

    #[test]
    fn test_migrate_itinerary_day_id_from_legacy_schema() {
        let conn = Connection::open(":memory:").unwrap();
        conn.execute_batch("PRAGMA foreign_keys = ON").unwrap();
        let now = crate::db::now_string();
        conn.execute(
            "CREATE TABLE trips (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                start_date TEXT,
                end_date TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )",
            [],
        )
        .unwrap();
        conn.execute(
            "CREATE TABLE days (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                trip_id INTEGER NOT NULL,
                day_number INTEGER NOT NULL,
                title TEXT NOT NULL DEFAULT '',
                description TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                UNIQUE(trip_id, day_number)
            )",
            [],
        )
        .unwrap();
        conn.execute(
            "CREATE TABLE itinerary_items (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                trip_id INTEGER NOT NULL,
                day INTEGER NOT NULL,
                title TEXT NOT NULL,
                note TEXT,
                sort_order INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO trips (name, start_date, end_date, created_at, updated_at)
             VALUES ('Legacy Trip', '2026-01-01', '2026-01-03', ?1, ?1)",
            params![&now],
        )
        .unwrap();
        let trip_id = conn.last_insert_rowid();
        for day_number in 1..=3 {
            conn.execute(
                "INSERT INTO days (trip_id, day_number, title, created_at, updated_at)
                 VALUES (?1, ?2, '', ?3, ?3)",
                params![trip_id, day_number, &now],
            )
            .unwrap();
        }
        conn.execute(
            "INSERT INTO itinerary_items (trip_id, day, title, sort_order, created_at, updated_at)
             VALUES (?1, 2, 'Legacy Item', 0, ?2, ?2)",
            params![trip_id, &now],
        )
        .unwrap();

        migrate_itinerary_day_id(&conn).unwrap();

        let day_id: i64 = conn
            .query_row(
                "SELECT day_id FROM itinerary_items WHERE trip_id = ?1",
                params![trip_id],
                |row| row.get(0),
            )
            .unwrap();
        let expected_day_id: i64 = conn
            .query_row(
                "SELECT id FROM days WHERE trip_id = ?1 AND day_number = 2",
                params![trip_id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(day_id, expected_day_id);
    }

    #[test]
    fn test_migrate_indexes_creates_recommended_indexes() {
        let conn = test_db();
        for name in [
            "idx_itinerary_items_day_id",
            "idx_itinerary_items_trip_id",
            "idx_days_trip_day_number",
        ] {
            let count: i64 = conn
                .query_row(
                    "SELECT COUNT(*) FROM sqlite_master
                     WHERE type = 'index' AND name = ?1",
                    params![name],
                    |row| row.get(0),
                )
                .unwrap();
            assert_eq!(count, 1, "missing index {name}");
        }
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

    #[test]
    fn test_with_transaction_rolls_back_on_error() {
        let conn = test_db();
        let trip_id = add_test_trip(&conn, "Rollback Trip").unwrap();
        let before = crate::trip::get_trip(&conn, trip_id).unwrap();

        let err = with_transaction(&conn, "test rollback", |tx| {
            tx.execute(
                "UPDATE trips SET name = ?1 WHERE id = ?2",
                params!["Changed Name", trip_id],
            )?;
            anyhow::bail!("simulated failure");
        })
        .expect_err("expected transaction to fail");

        assert!(err.to_string().contains("処理に失敗しました"));
        assert!(format!("{err:#}").contains("simulated failure"));
        let after = crate::trip::get_trip(&conn, trip_id).unwrap();
        assert_eq!(after.name, before.name);
    }
}
