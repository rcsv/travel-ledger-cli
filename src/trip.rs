use anyhow::{Context, Result};
use rusqlite::{params, Connection};

use crate::db::now_string;
use crate::models::{Trip, TripExport};

/// 新しい旅行を追加する
pub(crate) fn add_trip(
    conn: &Connection,
    name: &str,
    start: Option<&str>,
    end: Option<&str>,
) -> Result<i64> {
    let now = now_string();
    conn.execute(
        "INSERT INTO trips (name, start_date, end_date, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![name, start, end, &now, &now],
    )
    .context("旅行の追加に失敗しました")?;
    Ok(conn.last_insert_rowid())
}

/// すべての旅行を取得する
pub(crate) fn list_trips(conn: &Connection) -> Result<Vec<Trip>> {
    let mut stmt = conn
        .prepare(
            "SELECT id, name, start_date, end_date, created_at, updated_at
             FROM trips
             ORDER BY id",
        )
        .context("一覧取得の準備に失敗しました")?;

    let trips = stmt
        .query_map([], row_to_trip)
        .context("一覧取得に失敗しました")?
        .collect::<std::result::Result<Vec<_>, _>>()
        .context("一覧の読み込みに失敗しました")?;

    Ok(trips)
}

/// ID を指定して1件の旅行を取得する
pub(crate) fn get_trip(conn: &Connection, id: i64) -> Result<Trip> {
    crate::db::map_query_row(
        conn.query_row(
            "SELECT id, name, start_date, end_date, created_at, updated_at
         FROM trips
         WHERE id = ?1",
            params![id],
            row_to_trip,
        ),
        || anyhow::anyhow!("Trip not found: {id}"),
    )
}

/// 旅行を更新する（指定されたフィールドのみ上書き）
pub(crate) fn update_trip(
    conn: &Connection,
    id: i64,
    name: Option<&str>,
    start: Option<&str>,
    end: Option<&str>,
) -> Result<()> {
    if name.is_none() && start.is_none() && end.is_none() {
        anyhow::bail!("更新する項目を1つ以上指定してください (--name, --start, --end)");
    }

    // 既存データを読み込み、指定された項目だけ上書きする
    let mut trip = get_trip(conn, id)?;
    if let Some(n) = name {
        trip.name = n.to_string();
    }
    if let Some(s) = start {
        trip.start_date = Some(s.to_string());
    }
    if let Some(e) = end {
        trip.end_date = Some(e.to_string());
    }

    let now = now_string();
    conn.execute(
        "UPDATE trips
         SET name = ?1, start_date = ?2, end_date = ?3, updated_at = ?4
         WHERE id = ?5",
        params![trip.name, trip.start_date, trip.end_date, &now, id],
    )
    .context("旅行の更新に失敗しました")?;
    Ok(())
}

/// エクスポート用データを組み立てる
pub(crate) fn build_trip_export(conn: &Connection, trip_id: i64) -> Result<TripExport> {
    let trip = get_trip(conn, trip_id)?;
    let itinerary_items = crate::itinerary::list_itinerary_items(conn, trip_id)?;
    let checklist_items = crate::checklist::list_checklist_items(conn, trip_id)?;
    Ok(TripExport {
        trip,
        itinerary_items,
        checklist_items: Some(checklist_items),
    })
}

/// 旅行データを pretty JSON 文字列に変換する
pub(crate) fn export_trip_to_json(conn: &Connection, trip_id: i64) -> Result<String> {
    let export = build_trip_export(conn, trip_id)?;
    serde_json::to_string_pretty(&export).context("JSON の生成に失敗しました")
}

/// 旅行データを JSON で出力する（ファイルまたは標準出力）
pub(crate) fn write_trip_export(
    conn: &Connection,
    trip_id: i64,
    output_path: Option<&str>,
) -> Result<()> {
    let json = export_trip_to_json(conn, trip_id)?;
    match output_path {
        Some(path) => {
            std::fs::write(path, &json)
                .with_context(|| format!("ファイル '{path}' への書き込みに失敗しました"))?;
            println!("旅行をエクスポートしました: {path}");
        }
        None => println!("{json}"),
    }
    Ok(())
}
/// インポート用 JSON の必須項目を検証する
pub(crate) fn validate_trip_export(export: &TripExport) -> Result<()> {
    if export.trip.name.trim().is_empty() {
        anyhow::bail!("trip.name は必須です");
    }
    for (index, item) in export.itinerary_items.iter().enumerate() {
        if item.title.trim().is_empty() {
            anyhow::bail!("itinerary_items[{index}].title は必須です");
        }
    }
    for (index, item) in export.checklist_items().iter().enumerate() {
        if item.title.trim().is_empty() {
            anyhow::bail!("checklist_items[{index}].title は必須です");
        }
    }
    Ok(())
}

/// JSON 文字列から旅行をインポートする（ID は新規採番）
pub(crate) fn import_trip_from_json(conn: &Connection, json: &str) -> Result<i64> {
    let export: TripExport = serde_json::from_str(json).context("JSON の形式が不正です")?;
    validate_trip_export(&export)?;

    // JSON 内の id / trip_id は無視し、新しい Trip として登録する
    // created_at / updated_at は add_trip / add_itinerary_item で現在時刻に作り直す
    let new_trip_id = add_trip(
        conn,
        &export.trip.name,
        export.trip.start_date.as_deref(),
        export.trip.end_date.as_deref(),
    )?;

    let checklist_items: Vec<_> = export.checklist_items().to_vec();

    for item in export.itinerary_items {
        crate::itinerary::add_itinerary_item(
            conn,
            new_trip_id,
            item.day,
            &item.title,
            item.note.as_deref(),
            item.start_time.as_deref(),
            Some(item.sort_order),
            item.duration_minutes,
            item.travel_minutes,
            item.location.as_deref(),
            item.category,
        )?;
    }

    for item in checklist_items {
        crate::checklist::import_checklist_item(
            conn,
            new_trip_id,
            &item.title,
            item.is_done,
            item.sort_order,
        )?;
    }

    Ok(new_trip_id)
}

/// JSON ファイルから旅行をインポートする
pub(crate) fn import_trip_from_file(conn: &Connection, path: &str) -> Result<i64> {
    let json = std::fs::read_to_string(path)
        .with_context(|| format!("ファイル '{path}' を読み込めませんでした"))?;
    import_trip_from_json(conn, &json)
}

/// export JSON ファイルを読み込む（DB は使わない）
pub(crate) fn load_trip_export_from_file(path: &str) -> Result<TripExport> {
    let json = std::fs::read_to_string(path)
        .with_context(|| format!("ファイル '{path}' を読み込めませんでした"))?;
    serde_json::from_str(&json).context("JSON の形式が不正です")
}
/// 旅行を削除する
pub(crate) fn delete_trip(conn: &Connection, id: i64) -> Result<()> {
    // 存在確認（見つからなければエラー）
    get_trip(conn, id)?;
    conn.execute("DELETE FROM trips WHERE id = ?1", params![id])
        .context("旅行の削除に失敗しました")?;
    Ok(())
}

/// rusqlite の行データを Trip 構造体に変換する
pub(crate) fn row_to_trip(row: &rusqlite::Row) -> rusqlite::Result<Trip> {
    Ok(Trip {
        id: row.get(0)?,
        name: row.get(1)?,
        start_date: row.get(2)?,
        end_date: row.get(3)?,
        created_at: row.get(4)?,
        updated_at: row.get(5)?,
    })
}
/// 日付を表示用に整形する（未設定なら "-"）
pub(crate) fn fmt_date(date: &Option<String>) -> &str {
    date.as_deref().unwrap_or("-")
}

/// 旅行一覧を表形式で表示する
pub(crate) fn print_trip_list(trips: &[Trip]) {
    if trips.is_empty() {
        println!("旅行はまだ登録されていません。");
        return;
    }

    println!(
        "{:<6} {:<20} {:<12} {:<12}",
        "ID", "名前", "開始日", "終了日"
    );
    println!("{}", "-".repeat(52));
    for trip in trips {
        println!(
            "{:<6} {:<20} {:<12} {:<12}",
            trip.id,
            trip.name,
            fmt_date(&trip.start_date),
            fmt_date(&trip.end_date),
        );
    }
    println!();
    println!("合計: {} 件", trips.len());
}

/// 値を pretty JSON で標準出力する
pub(crate) fn print_json<T: serde::Serialize>(value: &T) -> Result<()> {
    let json = serde_json::to_string_pretty(value).context("JSON の生成に失敗しました")?;
    println!("{json}");
    Ok(())
}

/// 旅行の詳細を表示する
pub(crate) fn print_trip_detail(trip: &Trip) {
    println!("ID        : {}", trip.id);
    println!("名前      : {}", trip.name);
    println!("開始日    : {}", fmt_date(&trip.start_date));
    println!("終了日    : {}", fmt_date(&trip.end_date));
    println!("作成日時  : {}", trip.created_at);
    println!("更新日時  : {}", trip.updated_at);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::open_db_at;
    use crate::itinerary::add_itinerary_item;
    use rusqlite::Connection;

    fn test_db() -> Connection {
        open_db_at(":memory:").expect("インメモリ DB の作成に失敗")
    }

    #[test]
    fn test_add_trip() {
        let conn = test_db();
        let id = add_trip(&conn, "沖縄旅行", Some("2025-06-01"), Some("2025-06-05")).unwrap();

        assert_eq!(id, 1);
        let trip = get_trip(&conn, id).unwrap();
        assert_eq!(trip.name, "沖縄旅行");
        assert_eq!(trip.start_date.as_deref(), Some("2025-06-01"));
        assert_eq!(trip.end_date.as_deref(), Some("2025-06-05"));
    }

    #[test]
    fn test_delete_trip() {
        let conn = test_db();
        let id = add_trip(&conn, "沖縄旅行", None, None).unwrap();

        delete_trip(&conn, id).unwrap();

        assert!(list_trips(&conn).unwrap().is_empty());
        assert!(get_trip(&conn, id).is_err());
    }

    #[test]
    fn test_export_import_roundtrip() {
        let conn = test_db();
        let trip_id = add_trip(&conn, "沖縄旅行", Some("2026-04-26"), Some("2026-04-29")).unwrap();
        add_itinerary_item(
            &conn,
            trip_id,
            1,
            "首里城",
            None,
            Some("09:00"),
            None,
            Some(90),
            Some(20),
            Some("沖縄県那覇市首里金城町1-2"),
            None,
        )
        .unwrap();
        add_itinerary_item(
            &conn,
            trip_id,
            1,
            "国際通り",
            None,
            Some("10:50"),
            None,
            Some(60),
            Some(15),
            None,
            None,
        )
        .unwrap();

        let json = export_trip_to_json(&conn, trip_id).unwrap();
        add_trip(&conn, "別の旅行", None, None).unwrap();

        let new_id = import_trip_from_json(&conn, &json).unwrap();
        assert_eq!(new_id, 3);

        let imported = build_trip_export(&conn, new_id).unwrap();
        assert_eq!(imported.trip.name, "沖縄旅行");
        assert_eq!(imported.trip.start_date.as_deref(), Some("2026-04-26"));
        assert_eq!(imported.itinerary_items.len(), 2);
        assert_eq!(imported.itinerary_items[0].title, "首里城");
        assert_eq!(imported.itinerary_items[1].title, "国際通り");
        for item in &imported.itinerary_items {
            assert_eq!(item.trip_id, new_id);
        }
        assert_ne!(new_id, trip_id);
    }

    #[test]
    fn test_get_trip() {
        let conn = test_db();
        let id = add_trip(&conn, "北海道旅行", Some("2025-08-01"), Some("2025-08-10")).unwrap();

        let trip = get_trip(&conn, id).unwrap();
        assert_eq!(trip.id, id);
        assert_eq!(trip.name, "北海道旅行");
        assert_eq!(trip.start_date.as_deref(), Some("2025-08-01"));
        assert_eq!(trip.end_date.as_deref(), Some("2025-08-10"));
    }

    #[test]
    fn test_import_from_export_json_three_items() {
        let conn = test_db();
        let trip_id = add_trip(&conn, "沖縄旅行", None, None).unwrap();
        for (time, title) in [
            (Some("09:00"), "首里城"),
            (Some("10:50"), "国際通り"),
            (Some("13:00"), "ホテルチェックイン"),
        ] {
            add_itinerary_item(
                &conn, trip_id, 1, title, None, time, None, None, None, None, None,
            )
            .unwrap();
        }

        let json = export_trip_to_json(&conn, trip_id).unwrap();
        let new_id = import_trip_from_json(&conn, &json).unwrap();

        let imported = build_trip_export(&conn, new_id).unwrap();
        assert_eq!(imported.trip.name, "沖縄旅行");
        assert_eq!(imported.itinerary_items.len(), 3);
        assert!(imported.itinerary_items.iter().all(|i| i.trip_id == new_id));
    }

    #[test]
    fn test_import_trip_file_not_found() {
        let conn = test_db();
        assert!(import_trip_from_file(&conn, "nonexistent-trip.json").is_err());
    }

    #[test]
    fn test_import_trip_invalid_json() {
        let conn = test_db();
        assert!(import_trip_from_json(&conn, "not json").is_err());
    }

    #[test]
    fn test_import_trip_missing_required_field() {
        let conn = test_db();
        let json = r#"{"trip":{"id":1,"name":"","start_date":null,"end_date":null,"created_at":"x","updated_at":"x"},"itinerary_items":[]}"#;
        assert!(import_trip_from_json(&conn, json).is_err());
    }

    #[test]
    fn test_import_trip_remaps_ids() {
        let conn = test_db();
        let old_trip_id = add_trip(&conn, "沖縄旅行", None, None).unwrap();
        add_itinerary_item(
            &conn,
            old_trip_id,
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

        let json = export_trip_to_json(&conn, old_trip_id).unwrap();
        add_trip(&conn, "別の旅行", None, None).unwrap();
        add_trip(&conn, "もう一つ", None, None).unwrap();

        let new_trip_id = import_trip_from_json(&conn, &json).unwrap();
        assert_eq!(new_trip_id, 4);

        let items = crate::itinerary::list_itinerary_items(&conn, new_trip_id).unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].trip_id, new_trip_id);
        assert_ne!(items[0].trip_id, old_trip_id);
        assert_eq!(items[0].title, "首里城");
    }

    #[test]
    fn test_list_trips() {
        let conn = test_db();
        add_trip(&conn, "沖縄旅行", Some("2025-06-01"), Some("2025-06-05")).unwrap();
        add_trip(&conn, "京都旅行", Some("2025-07-01"), Some("2025-07-03")).unwrap();

        let trips = list_trips(&conn).unwrap();
        assert_eq!(trips.len(), 2);
        assert_eq!(trips[0].name, "沖縄旅行");
        assert_eq!(trips[1].name, "京都旅行");
    }

    #[test]
    fn test_trip_export_contains_trip_and_items() {
        let conn = test_db();
        let trip_id = add_trip(&conn, "沖縄旅行", Some("2026-04-26"), Some("2026-04-29")).unwrap();
        add_itinerary_item(
            &conn,
            trip_id,
            1,
            "首里城",
            None,
            Some("09:00"),
            None,
            Some(90),
            Some(20),
            Some("沖縄県那覇市首里金城町1-2"),
            None,
        )
        .unwrap();

        let export = build_trip_export(&conn, trip_id).unwrap();
        assert_eq!(export.trip.id, trip_id);
        assert_eq!(export.trip.name, "沖縄旅行");
        assert_eq!(export.itinerary_items.len(), 1);
        assert_eq!(export.itinerary_items[0].title, "首里城");
    }

    #[test]
    fn test_trip_export_items_sorted_by_day_and_time() {
        let conn = test_db();
        let trip_id = add_trip(&conn, "沖縄旅行", None, None).unwrap();

        add_itinerary_item(
            &conn,
            trip_id,
            1,
            "国際通り",
            None,
            Some("10:50"),
            None,
            None,
            None,
            None,
            None,
        )
        .unwrap();
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

        let export = build_trip_export(&conn, trip_id).unwrap();
        assert_eq!(export.itinerary_items[0].title, "首里城");
        assert_eq!(export.itinerary_items[1].title, "国際通り");
    }

    #[test]
    fn test_trip_export_to_json_string() {
        let conn = test_db();
        let trip_id = add_trip(&conn, "沖縄旅行", None, None).unwrap();

        let json = export_trip_to_json(&conn, trip_id).unwrap();
        assert!(json.contains("\"trip\""));
        assert!(json.contains("\"itinerary_items\""));
        assert!(json.contains("\"name\": \"沖縄旅行\""));

        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["trip"]["name"], "沖縄旅行");
        assert!(parsed["itinerary_items"].is_array());
    }

    #[test]
    fn test_print_json_empty_trip_list() {
        let json = serde_json::to_string_pretty(&Vec::<Trip>::new()).unwrap();
        assert_eq!(json, "[]");
    }

    #[test]
    fn test_print_json_trip_list() {
        let conn = test_db();
        add_trip(&conn, "沖縄旅行", Some("2025-06-01"), Some("2025-06-05")).unwrap();
        add_trip(&conn, "京都旅行", None, None).unwrap();

        let trips = list_trips(&conn).unwrap();
        let json = serde_json::to_string_pretty(&trips).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        assert!(parsed.is_array());
        assert_eq!(parsed.as_array().unwrap().len(), 2);
        assert_eq!(parsed[0]["name"], "沖縄旅行");
        assert_eq!(parsed[1]["name"], "京都旅行");
    }

    #[test]
    fn test_print_json_trip_show() {
        let conn = test_db();
        let id = add_trip(&conn, "北海道旅行", Some("2025-08-01"), Some("2025-08-10")).unwrap();

        let trip = get_trip(&conn, id).unwrap();
        let json = serde_json::to_string_pretty(&trip).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed["id"], id);
        assert_eq!(parsed["name"], "北海道旅行");
        assert_eq!(parsed["start_date"], "2025-08-01");
        assert_eq!(parsed["end_date"], "2025-08-10");
    }

    #[test]
    fn test_update_trip() {
        let conn = test_db();
        let id = add_trip(&conn, "沖縄旅行", None, None).unwrap();

        update_trip(
            &conn,
            id,
            Some("沖縄・瀬底旅行"),
            Some("2025-06-01"),
            Some("2025-06-07"),
        )
        .unwrap();

        let trip = get_trip(&conn, id).unwrap();
        assert_eq!(trip.name, "沖縄・瀬底旅行");
        assert_eq!(trip.start_date.as_deref(), Some("2025-06-01"));
        assert_eq!(trip.end_date.as_deref(), Some("2025-06-07"));
    }

    use crate::checklist::{add_checklist_item, set_checklist_done};
    use crate::db::reset_db;
    use crate::models::{ChecklistItem, ItineraryItem};

    fn checklist_sem(items: &[ChecklistItem]) -> Vec<(String, bool, i64)> {
        items
            .iter()
            .map(|item| (item.title.clone(), item.is_done, item.sort_order))
            .collect()
    }

    fn itinerary_sem(items: &[ItineraryItem]) -> Vec<(i64, String, Option<String>)> {
        items
            .iter()
            .map(|item| (item.day, item.title.clone(), item.start_time.clone()))
            .collect()
    }

    #[test]
    fn test_export_includes_checklist_items() {
        let conn = test_db();
        let trip_id = add_trip(&conn, "沖縄旅行", None, None).unwrap();
        add_checklist_item(&conn, trip_id, "パスポート").unwrap();
        let charger_id = add_checklist_item(&conn, trip_id, "充電器").unwrap();
        set_checklist_done(&conn, charger_id, true).unwrap();

        let export = build_trip_export(&conn, trip_id).unwrap();
        assert_eq!(export.checklist_items().len(), 2);
        assert_eq!(
            checklist_sem(export.checklist_items()),
            vec![
                ("パスポート".to_string(), false, 0),
                ("充電器".to_string(), true, 0),
            ]
        );

        let json = export_trip_to_json(&conn, trip_id).unwrap();
        assert!(json.contains("\"checklist_items\""));
    }

    #[test]
    fn test_import_legacy_json_without_checklist() {
        let conn = test_db();
        let json = r#"{
            "trip": {
                "id": 1,
                "name": "Legacy Trip",
                "start_date": "2026-06-01",
                "end_date": "2026-06-03",
                "created_at": "2026-01-01 00:00:00",
                "updated_at": "2026-01-01 00:00:00"
            },
            "itinerary_items": []
        }"#;

        let new_id = import_trip_from_json(&conn, json).unwrap();
        let imported = build_trip_export(&conn, new_id).unwrap();
        assert_eq!(imported.trip.name, "Legacy Trip");
        assert!(imported.checklist_items().is_empty());
    }

    #[test]
    fn test_export_import_full_roundtrip_with_checklist() {
        let conn = test_db();
        let trip_id = add_trip(
            &conn,
            "Import Export Verify Trip",
            Some("2026-06-01"),
            Some("2026-06-03"),
        )
        .unwrap();
        add_itinerary_item(
            &conn,
            trip_id,
            1,
            "Shuri Castle",
            None,
            Some("09:00"),
            None,
            Some(90),
            Some(20),
            Some("Naha"),
            None,
        )
        .unwrap();
        add_checklist_item(&conn, trip_id, "Passport").unwrap();
        let charger_id = add_checklist_item(&conn, trip_id, "Charger").unwrap();
        set_checklist_done(&conn, charger_id, true).unwrap();

        let before = build_trip_export(&conn, trip_id).unwrap();
        let json = export_trip_to_json(&conn, trip_id).unwrap();

        reset_db(&conn).unwrap();

        let new_id = import_trip_from_json(&conn, &json).unwrap();
        assert_eq!(new_id, 1);

        let after = build_trip_export(&conn, new_id).unwrap();
        assert_eq!(before.trip.name, after.trip.name);
        assert_eq!(before.trip.start_date, after.trip.start_date);
        assert_eq!(before.trip.end_date, after.trip.end_date);
        assert_eq!(
            itinerary_sem(&before.itinerary_items),
            itinerary_sem(&after.itinerary_items)
        );
        assert_eq!(
            checklist_sem(before.checklist_items()),
            checklist_sem(after.checklist_items())
        );

        let re_json = export_trip_to_json(&conn, new_id).unwrap();
        let parsed_before: TripExport = serde_json::from_str(&json).unwrap();
        let parsed_after: TripExport = serde_json::from_str(&re_json).unwrap();
        assert_eq!(
            checklist_sem(parsed_before.checklist_items()),
            checklist_sem(parsed_after.checklist_items())
        );
    }

    #[test]
    fn test_get_trip_not_found() {
        let conn = test_db();
        let err = get_trip(&conn, 9999).err().expect("expected error");
        assert_eq!(err.to_string(), "Trip not found: 9999");
        assert!(!format!("{err:#}").contains("Query returned no rows"));
    }
}
