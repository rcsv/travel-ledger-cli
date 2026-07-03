use anyhow::Result;
use rusqlite::Connection;

use crate::domain::models::ChecklistItem;

/// Read-only `checklist show` use case result (CLI / future GUI).
pub struct ChecklistShowServiceResult {
    pub item: ChecklistItem,
}

/// Loads a checklist item without terminal I/O.
pub fn show_checklist(conn: &Connection, id: i64) -> Result<ChecklistShowServiceResult> {
    let item = crate::checklist::get_checklist_item(conn, id)?;
    Ok(ChecklistShowServiceResult { item })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::db::open_db_at;
    use rusqlite::Connection;

    fn test_db() -> Connection {
        open_db_at(":memory:").expect("インメモリ DB の作成に失敗")
    }

    #[test]
    fn service_returns_existing_item() {
        let conn = test_db();
        let trip_id =
            crate::trip::add_trip(&conn, "Show Trip", "2026-06-01", "2026-06-02", None).unwrap();
        let id = crate::checklist::add_checklist_item(&conn, trip_id, "パスポート").unwrap();

        let result = show_checklist(&conn, id).unwrap();
        assert_eq!(result.item.id, id);
        assert_eq!(result.item.trip_id, trip_id);
        assert_eq!(result.item.title, "パスポート");
    }

    #[test]
    fn service_preserves_checked_and_unchecked_state() {
        let conn = test_db();
        let trip_id = crate::trip::add_test_trip(&conn, "State Trip").unwrap();
        let unchecked_id = crate::checklist::add_checklist_item(&conn, trip_id, "未完了").unwrap();
        let checked_id = crate::checklist::add_checklist_item(&conn, trip_id, "完了").unwrap();
        crate::checklist::set_checklist_done(&conn, checked_id, true).unwrap();

        let unchecked = show_checklist(&conn, unchecked_id).unwrap();
        assert!(!unchecked.item.is_done);

        let checked = show_checklist(&conn, checked_id).unwrap();
        assert!(checked.item.is_done);
    }

    #[test]
    fn service_preserves_not_found_error_message() {
        let conn = test_db();
        let err = show_checklist(&conn, 9999).err().expect("expected error");
        assert_eq!(err.to_string(), "Checklist item not found: 9999");
    }
}
