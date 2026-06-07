use std::collections::HashSet;

use anyhow::{Context, Result};
use rusqlite::{params, Connection};

use crate::models::{ChecklistItem, ItineraryCategory};

/// チェックリスト自動生成の結果
pub(crate) struct ChecklistGenerateResult {
    pub added: Vec<String>,
    pub skipped: Vec<String>,
}

/// 新しいチェックリスト項目を追加する
pub(crate) fn add_checklist_item(conn: &Connection, trip_id: i64, title: &str) -> Result<i64> {
    add_checklist_item_with_sort_order(conn, trip_id, title, 0)
}

/// 並び順を指定してチェックリスト項目を追加する
pub(crate) fn add_checklist_item_with_sort_order(
    conn: &Connection,
    trip_id: i64,
    title: &str,
    sort_order: i64,
) -> Result<i64> {
    crate::trip::get_trip(conn, trip_id)?;
    let now = crate::db::now_string();
    conn.execute(
        "INSERT INTO checklist_items
         (trip_id, title, is_done, sort_order, created_at, updated_at)
         VALUES (?1, ?2, 0, ?3, ?4, ?5)",
        params![trip_id, title, sort_order, &now, &now],
    )
    .context("チェックリスト項目の追加に失敗しました")?;
    Ok(conn.last_insert_rowid())
}

/// 日程のカテゴリ定義から追加候補を計算する（DB は更新しない）
pub(crate) fn plan_checklist_generation(
    conn: &Connection,
    trip_id: i64,
) -> Result<ChecklistGenerateResult> {
    crate::trip::get_trip(conn, trip_id)?;
    let itinerary_items = crate::itinerary::list_itinerary_items(conn, trip_id)?;
    let existing_items = list_checklist_items(conn, trip_id)?;

    let mut known_titles: HashSet<String> = existing_items
        .iter()
        .map(|item| item.title.clone())
        .collect();

    let mut added = Vec::new();
    let mut skipped = Vec::new();

    for item in &itinerary_items {
        let Some(category) = item.category else {
            continue;
        };
        let definition = category.definition();
        for &title in definition.default_checklist {
            try_plan_generated_checklist_item(title, &mut known_titles, &mut added, &mut skipped);
        }
    }

    let trip_categories: HashSet<ItineraryCategory> = itinerary_items
        .iter()
        .filter_map(|item| item.category)
        .collect();

    for rule in crate::models::checklist_combination_rules() {
        if checklist_rule_matches(&trip_categories, rule) {
            for &title in rule.checklist {
                try_plan_generated_checklist_item(
                    title,
                    &mut known_titles,
                    &mut added,
                    &mut skipped,
                );
            }
        }
    }

    Ok(ChecklistGenerateResult { added, skipped })
}

/// 日程のカテゴリ定義からチェックリスト項目を自動生成する
pub(crate) fn generate_checklist_from_itinerary(
    conn: &Connection,
    trip_id: i64,
) -> Result<ChecklistGenerateResult> {
    let result = plan_checklist_generation(conn, trip_id)?;
    apply_planned_checklist_items(conn, trip_id, &result.added)?;
    Ok(result)
}

fn apply_planned_checklist_items(conn: &Connection, trip_id: i64, added: &[String]) -> Result<()> {
    if added.is_empty() {
        return Ok(());
    }

    let existing_items = list_checklist_items(conn, trip_id)?;
    let base_sort_order = existing_items
        .iter()
        .map(|item| item.sort_order)
        .max()
        .unwrap_or(-1)
        + 1;

    for (offset, title) in added.iter().enumerate() {
        let sort_order = base_sort_order + offset as i64;
        add_checklist_item_with_sort_order(conn, trip_id, title, sort_order)?;
    }

    Ok(())
}

fn checklist_rule_matches(
    trip_categories: &HashSet<ItineraryCategory>,
    rule: &crate::models::ChecklistRule,
) -> bool {
    rule.required_categories
        .iter()
        .all(|category| trip_categories.contains(category))
}

fn try_plan_generated_checklist_item(
    title: &str,
    known_titles: &mut HashSet<String>,
    added: &mut Vec<String>,
    skipped: &mut Vec<String>,
) {
    if known_titles.contains(title) {
        if !skipped.iter().any(|existing| existing == title) {
            skipped.push(title.to_string());
        }
        return;
    }

    known_titles.insert(title.to_string());
    added.push(title.to_string());
}

/// チェックリスト自動生成の dry-run 結果を表示する
pub(crate) fn print_checklist_generate_dry_run_result(result: &ChecklistGenerateResult) {
    println!("Would add: {}", result.added.len());
    if !result.added.is_empty() {
        for title in &result.added {
            println!("- {title}");
        }
    }

    println!();
    println!("Would skip: {}", result.skipped.len());
    if !result.skipped.is_empty() {
        for title in &result.skipped {
            println!("- {title}");
        }
    }
}

/// チェックリスト自動生成の結果を表示する
pub(crate) fn print_checklist_generate_result(result: &ChecklistGenerateResult) {
    println!("チェックリストを自動生成しました");
    println!("追加: {} 件", result.added.len());
    println!("スキップ: {} 件", result.skipped.len());

    if !result.added.is_empty() {
        println!();
        println!("追加された項目:");
        for title in &result.added {
            println!("- {title}");
        }
    }

    if !result.skipped.is_empty() {
        println!();
        println!("スキップされた項目:");
        for title in &result.skipped {
            println!("- {title}");
        }
    }
}

/// import 用にチェックリスト項目を追加する（ID / 日時は新規採番）
pub(crate) fn import_checklist_item(
    conn: &Connection,
    trip_id: i64,
    title: &str,
    is_done: bool,
    sort_order: i64,
) -> Result<i64> {
    let now = crate::db::now_string();
    conn.execute(
        "INSERT INTO checklist_items
         (trip_id, title, is_done, sort_order, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![trip_id, title, i64::from(is_done), sort_order, &now, &now],
    )
    .context("チェックリスト項目のインポートに失敗しました")?;
    Ok(conn.last_insert_rowid())
}

/// 旅行に紐づくチェックリスト一覧を取得する
pub(crate) fn list_checklist_items(conn: &Connection, trip_id: i64) -> Result<Vec<ChecklistItem>> {
    crate::trip::get_trip(conn, trip_id)?;
    let mut stmt = conn
        .prepare(
            "SELECT id, trip_id, title, is_done, sort_order, created_at, updated_at
             FROM checklist_items
             WHERE trip_id = ?1
             ORDER BY is_done ASC, sort_order ASC, id ASC",
        )
        .context("チェックリスト一覧取得の準備に失敗しました")?;

    let items = stmt
        .query_map(params![trip_id], row_to_checklist_item)
        .context("チェックリスト一覧取得に失敗しました")?
        .collect::<std::result::Result<Vec<_>, _>>()
        .context("チェックリスト一覧の読み込みに失敗しました")?;

    Ok(items)
}

/// ID を指定して1件のチェックリスト項目を取得する
pub(crate) fn get_checklist_item(conn: &Connection, id: i64) -> Result<ChecklistItem> {
    crate::db::map_query_row(
        conn.query_row(
            "SELECT id, trip_id, title, is_done, sort_order, created_at, updated_at
         FROM checklist_items
         WHERE id = ?1",
            params![id],
            row_to_checklist_item,
        ),
        || anyhow::anyhow!("Checklist item not found: {id}"),
    )
}

/// チェックリスト項目を更新する（指定されたフィールドのみ上書き）
pub(crate) fn update_checklist_item(
    conn: &Connection,
    id: i64,
    title: Option<&str>,
    sort_order: Option<i64>,
) -> Result<()> {
    if title.is_none() && sort_order.is_none() {
        anyhow::bail!("更新する項目を1つ以上指定してください (--title, --sort-order)");
    }

    let mut item = get_checklist_item(conn, id)?;
    if let Some(t) = title {
        item.title = t.to_string();
    }
    if let Some(order) = sort_order {
        item.sort_order = order;
    }

    let now = crate::db::now_string();
    conn.execute(
        "UPDATE checklist_items
         SET title = ?1, sort_order = ?2, updated_at = ?3
         WHERE id = ?4",
        params![item.title, item.sort_order, &now, id],
    )
    .context("チェックリスト項目の更新に失敗しました")?;
    Ok(())
}

/// チェックリスト項目の完了状態を変更する
pub(crate) fn set_checklist_done(conn: &Connection, id: i64, is_done: bool) -> Result<()> {
    get_checklist_item(conn, id)?;
    let now = crate::db::now_string();
    let done_value = i64::from(is_done);
    conn.execute(
        "UPDATE checklist_items SET is_done = ?1, updated_at = ?2 WHERE id = ?3",
        params![done_value, &now, id],
    )
    .context("チェックリスト項目の状態変更に失敗しました")?;
    Ok(())
}

/// チェックリスト項目を削除する
pub(crate) fn delete_checklist_item(conn: &Connection, id: i64) -> Result<()> {
    get_checklist_item(conn, id)?;
    conn.execute("DELETE FROM checklist_items WHERE id = ?1", params![id])
        .context("チェックリスト項目の削除に失敗しました")?;
    Ok(())
}

/// rusqlite の行データを ChecklistItem 構造体に変換する
fn row_to_checklist_item(row: &rusqlite::Row) -> rusqlite::Result<ChecklistItem> {
    let is_done: i64 = row.get(3)?;
    Ok(ChecklistItem {
        id: row.get(0)?,
        trip_id: row.get(1)?,
        title: row.get(2)?,
        is_done: is_done != 0,
        sort_order: row.get(4)?,
        created_at: row.get(5)?,
        updated_at: row.get(6)?,
    })
}
/// チェック状態を表示用に整形する
fn fmt_checklist_mark(is_done: bool) -> char {
    if is_done {
        'x'
    } else {
        ' '
    }
}

/// チェックリスト一覧を表示する
pub(crate) fn print_checklist_list(items: &[ChecklistItem]) {
    if items.is_empty() {
        println!("チェックリストはまだ登録されていません。");
        return;
    }

    for item in items {
        let mark = fmt_checklist_mark(item.is_done);
        println!("[{mark}] {}. {}", item.id, item.title);
    }
}

/// チェックリスト項目の詳細を表示する
pub(crate) fn print_checklist_detail(item: &ChecklistItem) {
    let mark = fmt_checklist_mark(item.is_done);
    println!("ID        : {}", item.id);
    println!("旅行 ID   : {}", item.trip_id);
    println!("状態      : [{mark}]");
    println!("並び順    : {}", item.sort_order);
    println!("タイトル  : {}", item.title);
    println!("作成日時  : {}", item.created_at);
    println!("更新日時  : {}", item.updated_at);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::open_db_at;
    use crate::trip::add_test_trip;
    use rusqlite::Connection;

    fn test_db() -> Connection {
        open_db_at(":memory:").expect("インメモリ DB の作成に失敗")
    }

    #[test]
    fn test_add_checklist_item() {
        let conn = test_db();
        let trip_id = add_test_trip(&conn, "沖縄旅行").unwrap();
        let id = add_checklist_item(&conn, trip_id, "パスポート").unwrap();

        assert_eq!(id, 1);
        let item = get_checklist_item(&conn, id).unwrap();
        assert_eq!(item.trip_id, trip_id);
        assert_eq!(item.title, "パスポート");
        assert!(!item.is_done);
        assert_eq!(item.sort_order, 0);
    }

    #[test]
    fn test_check_and_uncheck_checklist_item() {
        let conn = test_db();
        let trip_id = add_test_trip(&conn, "沖縄旅行").unwrap();
        let id = add_checklist_item(&conn, trip_id, "パスポート").unwrap();

        set_checklist_done(&conn, id, true).unwrap();
        let checked = get_checklist_item(&conn, id).unwrap();
        assert!(checked.is_done);

        set_checklist_done(&conn, id, false).unwrap();
        let unchecked = get_checklist_item(&conn, id).unwrap();
        assert!(!unchecked.is_done);
    }

    #[test]
    fn test_delete_checklist_item() {
        let conn = test_db();
        let trip_id = add_test_trip(&conn, "沖縄旅行").unwrap();
        let id = add_checklist_item(&conn, trip_id, "パスポート").unwrap();

        delete_checklist_item(&conn, id).unwrap();
        assert!(get_checklist_item(&conn, id).is_err());
        assert!(list_checklist_items(&conn, trip_id).unwrap().is_empty());
    }

    #[test]
    fn test_list_checklist_items_by_trip() {
        let conn = test_db();
        let trip_id = add_test_trip(&conn, "沖縄旅行").unwrap();
        let other_trip_id = add_test_trip(&conn, "京都旅行").unwrap();

        add_checklist_item(&conn, trip_id, "パスポート").unwrap();
        add_checklist_item(&conn, trip_id, "充電器").unwrap();
        add_checklist_item(&conn, other_trip_id, "雨具").unwrap();

        let items = list_checklist_items(&conn, trip_id).unwrap();
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].title, "パスポート");
        assert_eq!(items[1].title, "充電器");
    }

    #[test]
    fn test_list_checklist_items_sorted() {
        let conn = test_db();
        let trip_id = add_test_trip(&conn, "沖縄旅行").unwrap();

        let passport_id = add_checklist_item(&conn, trip_id, "パスポート").unwrap();
        let etc_id = add_checklist_item(&conn, trip_id, "ETCカード").unwrap();
        let charger_id = add_checklist_item(&conn, trip_id, "充電器").unwrap();

        update_checklist_item(&conn, passport_id, None, Some(1)).unwrap();
        update_checklist_item(&conn, etc_id, None, Some(3)).unwrap();
        update_checklist_item(&conn, charger_id, None, Some(2)).unwrap();
        set_checklist_done(&conn, charger_id, true).unwrap();

        let items = list_checklist_items(&conn, trip_id).unwrap();
        assert_eq!(items.len(), 3);
        assert_eq!(items[0].title, "パスポート");
        assert_eq!(items[1].title, "ETCカード");
        assert_eq!(items[2].title, "充電器");
        assert!(!items[0].is_done);
        assert!(!items[1].is_done);
        assert!(items[2].is_done);
    }

    #[test]
    fn test_update_checklist_item() {
        let conn = test_db();
        let trip_id = add_test_trip(&conn, "沖縄旅行").unwrap();
        let id = add_checklist_item(&conn, trip_id, "パスポート").unwrap();

        update_checklist_item(&conn, id, Some("旅券"), Some(5)).unwrap();
        let item = get_checklist_item(&conn, id).unwrap();
        assert_eq!(item.title, "旅券");
        assert_eq!(item.sort_order, 5);
    }

    #[test]
    fn test_generate_checklist_from_categorized_itinerary() {
        let conn = test_db();
        let trip_id = add_test_trip(&conn, "沖縄旅行").unwrap();
        crate::itinerary::add_itinerary_item(
            &conn,
            trip_id,
            1,
            "ホテル",
            None,
            None,
            None,
            None,
            None,
            None,
            Some(crate::models::ItineraryCategory::Hotel),
        )
        .unwrap();

        let result = generate_checklist_from_itinerary(&conn, trip_id).unwrap();
        assert_eq!(result.added.len(), 3);
        assert!(result.skipped.is_empty());
        assert!(result.added.contains(&"宿泊予約確認".to_string()));
        assert!(result.added.contains(&"チェックイン時間確認".to_string()));
        assert!(result.added.contains(&"住所確認".to_string()));

        let items = list_checklist_items(&conn, trip_id).unwrap();
        assert_eq!(items.len(), 3);
    }

    #[test]
    fn test_generate_checklist_from_multiple_categories() {
        let conn = test_db();
        let trip_id = add_test_trip(&conn, "沖縄旅行").unwrap();
        crate::itinerary::add_itinerary_item(
            &conn,
            trip_id,
            1,
            "ホテル",
            None,
            None,
            None,
            None,
            None,
            None,
            Some(crate::models::ItineraryCategory::Hotel),
        )
        .unwrap();
        crate::itinerary::add_itinerary_item(
            &conn,
            trip_id,
            2,
            "ビーチ",
            None,
            None,
            None,
            None,
            None,
            None,
            Some(crate::models::ItineraryCategory::Beach),
        )
        .unwrap();

        let result = generate_checklist_from_itinerary(&conn, trip_id).unwrap();
        assert_eq!(result.added.len(), 7);
        assert!(result.added.contains(&"水着".to_string()));
        assert!(result.added.contains(&"宿泊予約確認".to_string()));
        assert!(result.added.contains(&"サンダル".to_string()));
    }

    #[test]
    fn test_generate_checklist_skips_duplicate_title() {
        let conn = test_db();
        let trip_id = add_test_trip(&conn, "沖縄旅行").unwrap();
        add_checklist_item(&conn, trip_id, "タオル").unwrap();
        crate::itinerary::add_itinerary_item(
            &conn,
            trip_id,
            1,
            "ビーチ",
            None,
            None,
            None,
            None,
            None,
            None,
            Some(crate::models::ItineraryCategory::Beach),
        )
        .unwrap();

        let result = generate_checklist_from_itinerary(&conn, trip_id).unwrap();
        assert_eq!(result.added.len(), 3);
        assert!(result.added.contains(&"水着".to_string()));
        assert!(result.added.contains(&"日焼け止め".to_string()));
        assert!(result.added.contains(&"サンダル".to_string()));
        assert!(result.skipped.contains(&"タオル".to_string()));
    }

    #[test]
    fn test_generate_checklist_sort_order_after_max() {
        let conn = test_db();
        let trip_id = add_test_trip(&conn, "沖縄旅行").unwrap();
        let existing_id = add_checklist_item(&conn, trip_id, "パスポート").unwrap();
        update_checklist_item(&conn, existing_id, None, Some(3)).unwrap();
        crate::itinerary::add_itinerary_item(
            &conn,
            trip_id,
            1,
            "ホテル",
            None,
            None,
            None,
            None,
            None,
            None,
            Some(crate::models::ItineraryCategory::Hotel),
        )
        .unwrap();

        generate_checklist_from_itinerary(&conn, trip_id).unwrap();

        let items = list_checklist_items(&conn, trip_id).unwrap();
        let hotel_items: Vec<_> = items.iter().filter(|i| i.title != "パスポート").collect();
        assert_eq!(hotel_items.len(), 3);
        assert_eq!(hotel_items[0].sort_order, 4);
        assert_eq!(hotel_items[1].sort_order, 5);
        assert_eq!(hotel_items[2].sort_order, 6);
    }

    #[test]
    fn test_generate_checklist_ignores_items_without_category() {
        let conn = test_db();
        let trip_id = add_test_trip(&conn, "沖縄旅行").unwrap();
        crate::itinerary::add_itinerary_item(
            &conn,
            trip_id,
            1,
            "首里城",
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .unwrap();
        crate::itinerary::add_itinerary_item(
            &conn,
            trip_id,
            1,
            "ビーチ",
            None,
            None,
            None,
            None,
            None,
            None,
            Some(crate::models::ItineraryCategory::Beach),
        )
        .unwrap();

        let result = generate_checklist_from_itinerary(&conn, trip_id).unwrap();
        assert_eq!(result.added.len(), 4);
        assert!(result.added.contains(&"水着".to_string()));
        assert!(result.added.contains(&"サンダル".to_string()));
    }

    #[test]
    fn test_generate_checklist_succeeds_with_zero_additions() {
        let conn = test_db();
        let trip_id = add_test_trip(&conn, "沖縄旅行").unwrap();
        crate::itinerary::add_itinerary_item(
            &conn,
            trip_id,
            1,
            "首里城",
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .unwrap();

        let result = generate_checklist_from_itinerary(&conn, trip_id).unwrap();
        assert!(result.added.is_empty());
        assert!(result.skipped.is_empty());
        assert!(list_checklist_items(&conn, trip_id).unwrap().is_empty());
    }

    #[test]
    fn test_generate_checklist_flight_hotel_combination() {
        let conn = test_db();
        let trip_id = add_test_trip(&conn, "フライト宿泊旅行").unwrap();
        crate::itinerary::add_itinerary_item(
            &conn,
            trip_id,
            1,
            "フライト",
            None,
            None,
            None,
            None,
            None,
            None,
            Some(crate::models::ItineraryCategory::Flight),
        )
        .unwrap();
        crate::itinerary::add_itinerary_item(
            &conn,
            trip_id,
            1,
            "ホテル",
            None,
            None,
            None,
            None,
            None,
            None,
            Some(crate::models::ItineraryCategory::Hotel),
        )
        .unwrap();

        let result = generate_checklist_from_itinerary(&conn, trip_id).unwrap();
        assert!(result.added.contains(&"航空券確認".to_string()));
        assert!(result.added.contains(&"身分証明書".to_string()));
        assert!(result.added.contains(&"充電器".to_string()));
        assert!(result.skipped.contains(&"宿泊予約確認".to_string()));
    }

    #[test]
    fn test_generate_checklist_beach_activity_combination() {
        let conn = test_db();
        let trip_id = add_test_trip(&conn, "ビーチアクティビティ旅行").unwrap();
        crate::itinerary::add_itinerary_item(
            &conn,
            trip_id,
            1,
            "ビーチ",
            None,
            None,
            None,
            None,
            None,
            None,
            Some(crate::models::ItineraryCategory::Beach),
        )
        .unwrap();
        crate::itinerary::add_itinerary_item(
            &conn,
            trip_id,
            1,
            "シュノーケル",
            None,
            None,
            None,
            None,
            None,
            None,
            Some(crate::models::ItineraryCategory::Activity),
        )
        .unwrap();

        let result = generate_checklist_from_itinerary(&conn, trip_id).unwrap();
        assert!(result.added.contains(&"着替え".to_string()));
        assert!(result.added.contains(&"防水バッグ".to_string()));
        assert!(result.added.contains(&"酔い止め".to_string()));
        assert!(result.added.contains(&"サンダル".to_string()));
    }

    #[test]
    fn test_generate_checklist_combination_skips_duplicate_with_existing() {
        let conn = test_db();
        let trip_id = add_test_trip(&conn, "フライト宿泊旅行").unwrap();
        add_checklist_item(&conn, trip_id, "充電器").unwrap();
        crate::itinerary::add_itinerary_item(
            &conn,
            trip_id,
            1,
            "フライト",
            None,
            None,
            None,
            None,
            None,
            None,
            Some(crate::models::ItineraryCategory::Flight),
        )
        .unwrap();
        crate::itinerary::add_itinerary_item(
            &conn,
            trip_id,
            1,
            "ホテル",
            None,
            None,
            None,
            None,
            None,
            None,
            Some(crate::models::ItineraryCategory::Hotel),
        )
        .unwrap();

        let result = generate_checklist_from_itinerary(&conn, trip_id).unwrap();
        assert!(!result.added.contains(&"充電器".to_string()));
        assert!(result.skipped.contains(&"充電器".to_string()));
    }

    #[test]
    fn test_generate_checklist_combination_applied_once_for_duplicate_categories() {
        let conn = test_db();
        let trip_id = add_test_trip(&conn, "ショッピング旅行").unwrap();
        for i in 1..=3 {
            crate::itinerary::add_itinerary_item(
                &conn,
                trip_id,
                1,
                &format!("買い物{i}"),
                None,
                None,
                None,
                None,
                None,
                None,
                Some(crate::models::ItineraryCategory::Shopping),
            )
            .unwrap();
        }

        let result = generate_checklist_from_itinerary(&conn, trip_id).unwrap();
        let eco_bag_count = result
            .added
            .iter()
            .filter(|title| *title == "エコバッグ")
            .count();
        assert_eq!(eco_bag_count, 1);
        assert!(result.added.contains(&"現金（小銭）".to_string()));
    }

    #[test]
    fn test_checklist_list_json_empty() {
        let conn = test_db();
        let trip_id = add_test_trip(&conn, "沖縄旅行").unwrap();

        let items = list_checklist_items(&conn, trip_id).unwrap();
        let json = serde_json::to_string_pretty(&items).unwrap();

        assert_eq!(json, "[]");
    }

    #[test]
    fn test_checklist_list_json() {
        let conn = test_db();
        let trip_id = add_test_trip(&conn, "沖縄旅行").unwrap();
        add_checklist_item(&conn, trip_id, "パスポート").unwrap();
        add_checklist_item(&conn, trip_id, "充電器").unwrap();
        set_checklist_done(&conn, 2, true).unwrap();

        let items = list_checklist_items(&conn, trip_id).unwrap();
        let json = serde_json::to_string_pretty(&items).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        assert!(parsed.is_array());
        assert_eq!(parsed.as_array().unwrap().len(), 2);
        assert_eq!(parsed[0]["title"], "パスポート");
        assert_eq!(parsed[0]["is_done"], false);
        assert_eq!(parsed[1]["title"], "充電器");
        assert_eq!(parsed[1]["is_done"], true);
    }

    #[test]
    fn test_checklist_show_json() {
        let conn = test_db();
        let trip_id = add_test_trip(&conn, "沖縄旅行").unwrap();
        let id = add_checklist_item(&conn, trip_id, "パスポート").unwrap();
        update_checklist_item(&conn, id, None, Some(5)).unwrap();

        let item = get_checklist_item(&conn, id).unwrap();
        let json = serde_json::to_string_pretty(&item).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed["id"], id);
        assert_eq!(parsed["trip_id"], trip_id);
        assert_eq!(parsed["title"], "パスポート");
        assert_eq!(parsed["is_done"], false);
        assert_eq!(parsed["sort_order"], 5);
    }

    #[test]
    fn test_get_checklist_item_not_found() {
        let conn = test_db();
        let err = get_checklist_item(&conn, 9999)
            .err()
            .expect("expected error");
        assert_eq!(err.to_string(), "Checklist item not found: 9999");
        assert!(!format!("{err:#}").contains("Query returned no rows"));
    }

    #[test]
    fn test_dry_run_does_not_modify_db() {
        let conn = test_db();
        let trip_id = add_test_trip(&conn, "Dry Run Trip").unwrap();
        crate::itinerary::add_itinerary_item(
            &conn,
            trip_id,
            1,
            "ホテル",
            None,
            None,
            None,
            None,
            None,
            None,
            Some(crate::models::ItineraryCategory::Hotel),
        )
        .unwrap();

        let before = list_checklist_items(&conn, trip_id).unwrap();
        let result = plan_checklist_generation(&conn, trip_id).unwrap();
        let after = list_checklist_items(&conn, trip_id).unwrap();

        assert!(!result.added.is_empty());
        assert_eq!(before.len(), after.len());
        for (before_item, after_item) in before.iter().zip(after.iter()) {
            assert_eq!(before_item.title, after_item.title);
            assert_eq!(before_item.is_done, after_item.is_done);
            assert_eq!(before_item.sort_order, after_item.sort_order);
        }
    }

    #[test]
    fn test_dry_run_shows_added_candidates() {
        let conn = test_db();
        let trip_id = add_test_trip(&conn, "Dry Run Added").unwrap();
        crate::itinerary::add_itinerary_item(
            &conn,
            trip_id,
            1,
            "ビーチ",
            None,
            None,
            None,
            None,
            None,
            None,
            Some(crate::models::ItineraryCategory::Beach),
        )
        .unwrap();

        let result = plan_checklist_generation(&conn, trip_id).unwrap();
        assert!(result.added.contains(&"水着".to_string()));
        assert!(result.added.contains(&"タオル".to_string()));
    }

    #[test]
    fn test_dry_run_shows_skipped_candidates() {
        let conn = test_db();
        let trip_id = add_test_trip(&conn, "Dry Run Skipped").unwrap();
        add_checklist_item(&conn, trip_id, "宿泊予約確認").unwrap();
        crate::itinerary::add_itinerary_item(
            &conn,
            trip_id,
            1,
            "ホテル",
            None,
            None,
            None,
            None,
            None,
            None,
            Some(crate::models::ItineraryCategory::Hotel),
        )
        .unwrap();

        let result = plan_checklist_generation(&conn, trip_id).unwrap();
        assert!(result.skipped.contains(&"宿泊予約確認".to_string()));
        assert!(!result.added.contains(&"宿泊予約確認".to_string()));
    }
}
