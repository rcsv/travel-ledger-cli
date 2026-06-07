use anyhow::{Context, Result};
use chrono::Utc;
use rusqlite::{params, Connection};

use crate::db::now_string;
use crate::models::{
    ExportValidationCheck, ExportValidationCheckId, ExportValidationReport, Trip, TripExport,
    TripExportMetadata, TripImportSummary, TRIP_EXPORT_GENERATOR, TRIP_EXPORT_SCHEMA_VERSION,
};

/// 新しい旅行を追加する
pub(crate) fn add_trip(conn: &Connection, name: &str, start: &str, end: &str) -> Result<i64> {
    let day_count = crate::day::validate_trip_date_range(start, end)?;
    let now = now_string();
    conn.execute(
        "INSERT INTO trips (name, start_date, end_date, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![name, start, end, &now, &now],
    )
    .context("旅行の追加に失敗しました")?;
    let trip_id = conn.last_insert_rowid();
    crate::day::create_days_for_trip(conn, trip_id, day_count)?;
    Ok(trip_id)
}

#[cfg(test)]
pub(crate) fn add_test_trip(conn: &Connection, name: &str) -> Result<i64> {
    add_trip(conn, name, "2026-01-01", "2026-01-03")
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
        crate::day::parse_trip_date(s)?;
        trip.start_date = Some(s.to_string());
    }
    if let Some(e) = end {
        crate::day::parse_trip_date(e)?;
        trip.end_date = Some(e.to_string());
    }

    let start_date = trip
        .start_date
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("開始日は必須です (--start)"))?;
    let end_date = trip
        .end_date
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("終了日は必須です (--end)"))?;
    let day_count = crate::day::validate_trip_date_range(start_date, end_date)?;

    let now = now_string();
    conn.execute(
        "UPDATE trips
         SET name = ?1, start_date = ?2, end_date = ?3, updated_at = ?4
         WHERE id = ?5",
        params![trip.name, trip.start_date, trip.end_date, &now, id],
    )
    .context("旅行の更新に失敗しました")?;
    crate::day::sync_days_to_trip_duration(conn, id, day_count)?;
    Ok(())
}

/// エクスポート用データを組み立てる
pub(crate) fn build_trip_export(conn: &Connection, trip_id: i64) -> Result<TripExport> {
    let trip = get_trip(conn, trip_id)?;
    let itinerary_items = crate::itinerary::list_itinerary_items(conn, trip_id)?;
    let checklist_items = crate::checklist::list_checklist_items(conn, trip_id)?;
    Ok(TripExport {
        schema_version: None,
        generator: None,
        generator_version: None,
        exported_at: None,
        trip,
        itinerary_items,
        checklist_items: Some(checklist_items),
    })
}

fn export_timestamp_rfc3339() -> String {
    Utc::now().to_rfc3339()
}

/// export 用 JSON にメタデータを付与する
fn finalize_trip_export(mut export: TripExport) -> TripExport {
    export.schema_version = Some(TRIP_EXPORT_SCHEMA_VERSION);
    export.generator = Some(TRIP_EXPORT_GENERATOR.to_string());
    export.generator_version = Some(env!("CARGO_PKG_VERSION").to_string());
    export.exported_at = Some(export_timestamp_rfc3339());
    export
}

/// 旅行データを pretty JSON 文字列に変換する
pub(crate) fn export_trip_to_json(conn: &Connection, trip_id: i64) -> Result<String> {
    let export = finalize_trip_export(build_trip_export(conn, trip_id)?);
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
    let start = export
        .trip
        .start_date
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("trip.start_date は必須です"))?;
    let end = export
        .trip
        .end_date
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("trip.end_date は必須です"))?;
    let day_count = crate::day::validate_trip_date_range(start, end)?;
    for (index, item) in export.itinerary_items.iter().enumerate() {
        if item.title.trim().is_empty() {
            anyhow::bail!("itinerary_items[{index}].title は必須です");
        }
        if item.day < 1 || item.day > day_count {
            anyhow::bail!(
                "itinerary_items[{index}].day ({}) は旅行期間 (1..={day_count}) の範囲外です",
                item.day
            );
        }
    }
    for (index, item) in export.checklist_items().iter().enumerate() {
        if item.title.trim().is_empty() {
            anyhow::bail!("checklist_items[{index}].title は必須です");
        }
    }
    Ok(())
}

fn push_check(checks: &mut Vec<ExportValidationCheck>, id: ExportValidationCheckId, passed: bool) {
    checks.push(ExportValidationCheck { id, passed });
}

/// export JSON 文字列を検証する（`valid` = import 可能か）
pub(crate) fn analyze_trip_export_json(file: &str, json: &str) -> ExportValidationReport {
    let mut report = ExportValidationReport::new(file);

    let root: serde_json::Value = match serde_json::from_str(json) {
        Ok(value) => value,
        Err(error) => {
            push_check(
                &mut report.checks,
                ExportValidationCheckId::JsonFormat,
                false,
            );
            report
                .errors
                .push(format!("JSON の形式が不正です: {error}"));
            return report;
        }
    };
    push_check(
        &mut report.checks,
        ExportValidationCheckId::JsonFormat,
        true,
    );

    let export: TripExport = match serde_json::from_value(root.clone()) {
        Ok(export) => export,
        Err(error) => {
            push_check(&mut report.checks, ExportValidationCheckId::Trip, false);
            push_check(
                &mut report.checks,
                ExportValidationCheckId::ItineraryItems,
                root.get("itinerary_items").is_some(),
            );
            push_check(
                &mut report.checks,
                ExportValidationCheckId::ChecklistItems,
                root.get("checklist_items").is_some(),
            );
            push_check(
                &mut report.checks,
                ExportValidationCheckId::SchemaVersion,
                root.get("schema_version").is_some()
                    && root.get("schema_version").and_then(|v| v.as_i64())
                        == Some(i64::from(TRIP_EXPORT_SCHEMA_VERSION)),
            );
            report
                .errors
                .push(format!("export JSON の構造が不正です: {error}"));
            return report;
        }
    };

    let has_schema_version = root.get("schema_version").is_some();
    let export_schema_version = export.schema_version;
    report.export_schema_version = export_schema_version;
    let schema_check_passed =
        has_schema_version && export_schema_version == Some(TRIP_EXPORT_SCHEMA_VERSION);
    push_check(
        &mut report.checks,
        ExportValidationCheckId::SchemaVersion,
        schema_check_passed,
    );
    if !has_schema_version {
        report
            .warnings
            .push("schema_version がありません（旧形式）".to_string());
    } else if export_schema_version != Some(TRIP_EXPORT_SCHEMA_VERSION) {
        report.warnings.push(format!(
            "schema_version {} は未対応です。import は試行可能ですが、正式サポート外の形式です。",
            export_schema_version.unwrap_or_default()
        ));
    }

    let trip_passed = !export.trip.name.trim().is_empty();
    push_check(
        &mut report.checks,
        ExportValidationCheckId::Trip,
        trip_passed,
    );

    let has_itinerary_items = root.get("itinerary_items").is_some();
    push_check(
        &mut report.checks,
        ExportValidationCheckId::ItineraryItems,
        has_itinerary_items,
    );

    let has_checklist_items = root.get("checklist_items").is_some();
    push_check(
        &mut report.checks,
        ExportValidationCheckId::ChecklistItems,
        has_checklist_items,
    );
    if !has_checklist_items {
        report
            .warnings
            .push("checklist_items がありません（旧形式）".to_string());
    }

    let metadata = TripExportMetadata::from_parsed(&root, &export);
    report.export_metadata = Some(metadata.clone());
    report.generator = metadata.json_generator();
    report.generator_version = metadata.json_generator_version();
    report.exported_at = metadata.json_exported_at();

    if metadata.exported_at_present {
        if let Some(exported_at) = metadata.exported_at.as_deref() {
            if chrono::DateTime::parse_from_rfc3339(exported_at).is_err() {
                report.warnings.push(format!(
                    "exported_at の形式が RFC3339 ではありません: {exported_at}"
                ));
            }
        }
    }

    report.trip_name = Some(export.trip.name.clone());
    report.itinerary_count = export.itinerary_items.len();
    report.checklist_count = export.checklist_items().len();

    if let Err(error) = validate_trip_export(&export) {
        report.errors.push(error.to_string());
    }

    report.valid = report.errors.is_empty();
    report
}

/// export JSON ファイルを検証する（DB は使わない）
pub(crate) fn analyze_trip_export(path: &str) -> Result<ExportValidationReport> {
    let json = std::fs::read_to_string(path)
        .with_context(|| format!("ファイル '{path}' を読み込めませんでした"))?;
    Ok(analyze_trip_export_json(path, &json))
}

const EXPORT_VALIDATION_CHECK_ORDER: [ExportValidationCheckId; 5] = [
    ExportValidationCheckId::JsonFormat,
    ExportValidationCheckId::SchemaVersion,
    ExportValidationCheckId::Trip,
    ExportValidationCheckId::ItineraryItems,
    ExportValidationCheckId::ChecklistItems,
];

fn export_validation_check_label(id: ExportValidationCheckId) -> &'static str {
    match id {
        ExportValidationCheckId::JsonFormat => "JSON形式",
        ExportValidationCheckId::SchemaVersion => "schema_version",
        ExportValidationCheckId::Trip => "trip",
        ExportValidationCheckId::ItineraryItems => "itinerary_items",
        ExportValidationCheckId::ChecklistItems => "checklist_items",
    }
}

fn export_validation_error_line(error: &str) -> String {
    if error.starts_with("JSON の形式が不正です") {
        "JSON形式が不正です".to_string()
    } else if error.starts_with("export JSON の構造が不正です") {
        "export JSON の構造が不正です".to_string()
    } else {
        error.to_string()
    }
}

/// export 検証結果を人間向けに表示する
pub(crate) fn print_export_validation_report(report: &ExportValidationReport) {
    println!("Export file: {}", report.file);
    println!();
    println!("Checks:");
    for id in EXPORT_VALIDATION_CHECK_ORDER {
        if let Some(check) = report.checks.iter().find(|check| check.id == id) {
            let mark = if check.passed { "✓" } else { "✗" };
            println!("  {mark} {}", export_validation_check_label(id));
        }
    }

    if report.trip_name.is_some() {
        println!();
        println!("Summary:");
        println!(
            "  Trip         : {}",
            report.trip_name.as_deref().unwrap_or("-")
        );
        println!("  Itineraries  : {} 件", report.itinerary_count);
        println!("  Checklists   : {} 件", report.checklist_count);
    }

    if let Some(metadata) = &report.export_metadata {
        println!();
        println!("Metadata:");
        println!("  Generator : {}", metadata.display_generator());
        println!("  Version   : {}", metadata.display_generator_version());
        println!("  Exported  : {}", metadata.display_exported_at());
    }

    println!();
    println!("Warnings:");
    if report.warnings.is_empty() {
        println!("  なし");
    } else {
        for warning in &report.warnings {
            println!("  - {warning}");
        }
    }

    if !report.errors.is_empty() {
        println!();
        println!("Errors:");
        for error in &report.errors {
            println!("  - {}", export_validation_error_line(error));
        }
    }

    println!();
    println!("Result:");
    if report.valid {
        println!("  有効な export ファイル");
    } else {
        println!("  無効な export ファイル");
    }
}

/// export JSON ファイルを検証して結果を表示する
pub(crate) fn run_trip_validate_export(path: &str, json: bool) -> Result<()> {
    let report = analyze_trip_export(path)?;
    if json {
        print_json(&report)?;
    } else {
        print_export_validation_report(&report);
    }
    if !report.valid {
        anyhow::bail!("無効な export ファイルです");
    }
    Ok(())
}

/// JSON 文字列から旅行をインポートする（ID は新規採番）
#[cfg(test)]
pub(crate) fn import_trip_from_json(conn: &Connection, json: &str) -> Result<i64> {
    let export: TripExport = serde_json::from_str(json).context("JSON の形式が不正です")?;
    import_trip_from_export(conn, &export)
}

/// TripExport から旅行をインポートする（ID は新規採番）
pub(crate) fn import_trip_from_export(conn: &Connection, export: &TripExport) -> Result<i64> {
    validate_trip_export(export)?;

    // JSON 内の id / trip_id は無視し、新しい Trip として登録する
    // created_at / updated_at は add_trip / add_itinerary_item で現在時刻に作り直す
    let new_trip_id = add_trip(
        conn,
        &export.trip.name,
        export.trip.start_date.as_deref().expect("validated above"),
        export.trip.end_date.as_deref().expect("validated above"),
    )?;

    let checklist_items: Vec<_> = export.checklist_items().to_vec();

    for item in &export.itinerary_items {
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

fn trip_import_summary_from_export(
    new_trip_id: i64,
    export: &TripExport,
    schema_version_present: bool,
    export_metadata: TripExportMetadata,
) -> TripImportSummary {
    TripImportSummary {
        trip_id: new_trip_id,
        trip_name: export.trip.name.clone(),
        itinerary_count: export.itinerary_items.len(),
        checklist_count: export.checklist_items().len(),
        schema_version_present,
        export_schema_version: export.schema_version,
        export_metadata,
    }
}

/// import 結果の Schema 行を返す
pub(crate) fn import_schema_display_line(summary: &TripImportSummary) -> String {
    if summary.schema_version_present {
        format!(
            "  version {}",
            summary.export_schema_version.unwrap_or_default()
        )
    } else {
        "  未指定（旧形式）".to_string()
    }
}

/// import 完了サマリーを表示する
pub(crate) fn print_trip_import_summary(summary: &TripImportSummary) {
    println!("旅行をインポートしました");
    println!();
    println!("Trip:");
    println!("  {} (ID: {})", summary.trip_name, summary.trip_id);
    println!();
    println!("Created:");
    println!("  日程           : {} 件", summary.itinerary_count);
    println!("  チェックリスト : {} 件", summary.checklist_count);
    println!();
    println!("Schema:");
    println!("{}", import_schema_display_line(summary));
    println!();
    println!("Export:");
    println!(
        "  generator : {}",
        summary.export_metadata.display_generator()
    );
    println!(
        "  version   : {}",
        summary.export_metadata.display_generator_version()
    );
}

fn parse_trip_export_for_import(json: &str) -> Result<(TripExport, bool, TripExportMetadata)> {
    let root: serde_json::Value = serde_json::from_str(json).context("JSON の形式が不正です")?;
    let schema_version_present = root.get("schema_version").is_some();
    let export: TripExport =
        serde_json::from_value(root.clone()).context("JSON の形式が不正です")?;
    let export_metadata = TripExportMetadata::from_parsed(&root, &export);
    Ok((export, schema_version_present, export_metadata))
}

/// JSON 文字列から旅行をインポートし、サマリーを返す
pub(crate) fn import_trip_from_json_with_summary(
    conn: &Connection,
    json: &str,
) -> Result<TripImportSummary> {
    let (export, schema_version_present, export_metadata) = parse_trip_export_for_import(json)?;
    let new_trip_id = import_trip_from_export(conn, &export)?;
    Ok(trip_import_summary_from_export(
        new_trip_id,
        &export,
        schema_version_present,
        export_metadata,
    ))
}

/// JSON ファイルから旅行をインポートし、サマリーを表示する
pub(crate) fn run_trip_import(conn: &Connection, path: &str) -> Result<()> {
    let json = std::fs::read_to_string(path)
        .with_context(|| format!("ファイル '{path}' を読み込めませんでした"))?;
    let summary = import_trip_from_json_with_summary(conn, &json)?;
    print_trip_import_summary(&summary);
    Ok(())
}

/// 旅行を複製する（Trip / Itinerary / Checklist を新しい ID でコピー）
pub(crate) fn duplicate_trip(conn: &Connection, trip_id: i64, name: Option<&str>) -> Result<i64> {
    let source = get_trip(conn, trip_id)?;
    let mut export = build_trip_export(conn, trip_id)?;
    export.trip.name = match name {
        Some(value) => value.to_string(),
        None => format!("{} (Copy)", source.name),
    };
    import_trip_from_export(conn, &export)
}

/// JSON ファイルから旅行をインポートする
#[cfg(test)]
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
        let id = add_trip(&conn, "沖縄旅行", "2025-06-01", "2025-06-05").unwrap();

        assert_eq!(id, 1);
        let trip = get_trip(&conn, id).unwrap();
        assert_eq!(trip.name, "沖縄旅行");
        assert_eq!(trip.start_date.as_deref(), Some("2025-06-01"));
        assert_eq!(trip.end_date.as_deref(), Some("2025-06-05"));
    }

    #[test]
    fn test_add_trip_creates_days() {
        let conn = test_db();
        let trip_id = add_trip(&conn, "Day Trip", "2026-12-01", "2026-12-03").unwrap();
        let days = crate::day::list_days(&conn, trip_id).unwrap();
        assert_eq!(days.len(), 3);
        assert_eq!(days[0].day_number, 1);
        assert_eq!(days[2].day_number, 3);
    }

    #[test]
    fn test_add_trip_rejects_invalid_date_range() {
        let conn = test_db();
        assert!(add_trip(&conn, "Bad Trip", "2026-12-04", "2026-12-01").is_err());
    }

    #[test]
    fn test_update_trip_syncs_days_on_end_extension() {
        let conn = test_db();
        let id = add_trip(&conn, "Extend Trip", "2026-12-01", "2026-12-02").unwrap();
        assert_eq!(crate::day::list_days(&conn, id).unwrap().len(), 2);
        update_trip(&conn, id, None, None, Some("2026-12-04")).unwrap();
        assert_eq!(crate::day::list_days(&conn, id).unwrap().len(), 4);
    }

    #[test]
    fn test_delete_trip() {
        let conn = test_db();
        let id = add_test_trip(&conn, "沖縄旅行").unwrap();

        delete_trip(&conn, id).unwrap();

        assert!(list_trips(&conn).unwrap().is_empty());
        assert!(get_trip(&conn, id).is_err());
    }

    #[test]
    fn test_export_import_roundtrip() {
        let conn = test_db();
        let trip_id = add_trip(&conn, "沖縄旅行", "2026-04-26", "2026-04-29").unwrap();
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
        add_test_trip(&conn, "別の旅行").unwrap();

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
        let id = add_trip(&conn, "北海道旅行", "2025-08-01", "2025-08-10").unwrap();

        let trip = get_trip(&conn, id).unwrap();
        assert_eq!(trip.id, id);
        assert_eq!(trip.name, "北海道旅行");
        assert_eq!(trip.start_date.as_deref(), Some("2025-08-01"));
        assert_eq!(trip.end_date.as_deref(), Some("2025-08-10"));
    }

    #[test]
    fn test_import_from_export_json_three_items() {
        let conn = test_db();
        let trip_id = add_test_trip(&conn, "沖縄旅行").unwrap();
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
        let json = r#"{"trip":{"id":1,"name":"","start_date":"2026-01-01","end_date":"2026-01-03","created_at":"x","updated_at":"x"},"itinerary_items":[]}"#;
        assert!(import_trip_from_json(&conn, json).is_err());
    }

    #[test]
    fn test_import_trip_remaps_ids() {
        let conn = test_db();
        let old_trip_id = add_test_trip(&conn, "沖縄旅行").unwrap();
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
        add_test_trip(&conn, "別の旅行").unwrap();
        add_test_trip(&conn, "もう一つ").unwrap();

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
        add_trip(&conn, "沖縄旅行", "2025-06-01", "2025-06-05").unwrap();
        add_trip(&conn, "京都旅行", "2025-07-01", "2025-07-03").unwrap();

        let trips = list_trips(&conn).unwrap();
        assert_eq!(trips.len(), 2);
        assert_eq!(trips[0].name, "沖縄旅行");
        assert_eq!(trips[1].name, "京都旅行");
    }

    #[test]
    fn test_trip_export_contains_trip_and_items() {
        let conn = test_db();
        let trip_id = add_trip(&conn, "沖縄旅行", "2026-04-26", "2026-04-29").unwrap();
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
        let trip_id = add_test_trip(&conn, "沖縄旅行").unwrap();

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
        let trip_id = add_test_trip(&conn, "沖縄旅行").unwrap();

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
        add_trip(&conn, "沖縄旅行", "2025-06-01", "2025-06-05").unwrap();
        add_test_trip(&conn, "京都旅行").unwrap();

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
        let id = add_trip(&conn, "北海道旅行", "2025-08-01", "2025-08-10").unwrap();

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
        let id = add_test_trip(&conn, "沖縄旅行").unwrap();

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

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct ComparableTripExport {
        trip_name: String,
        trip_start_date: Option<String>,
        trip_end_date: Option<String>,
        itinerary_items: Vec<ComparableItineraryItem>,
        checklist_items: Vec<ComparableChecklistItem>,
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct ComparableItineraryItem {
        day: i64,
        title: String,
        note: Option<String>,
        start_time: Option<String>,
        sort_order: i64,
        duration_minutes: Option<i64>,
        travel_minutes: Option<i64>,
        location: Option<String>,
        category: Option<crate::models::ItineraryCategory>,
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct ComparableChecklistItem {
        title: String,
        is_done: bool,
        sort_order: i64,
    }

    fn comparable_trip_export(export: &TripExport) -> ComparableTripExport {
        ComparableTripExport {
            trip_name: export.trip.name.clone(),
            trip_start_date: export.trip.start_date.clone(),
            trip_end_date: export.trip.end_date.clone(),
            itinerary_items: export
                .itinerary_items
                .iter()
                .map(|item| ComparableItineraryItem {
                    day: item.day,
                    title: item.title.clone(),
                    note: item.note.clone(),
                    start_time: item.start_time.clone(),
                    sort_order: item.sort_order,
                    duration_minutes: item.duration_minutes,
                    travel_minutes: item.travel_minutes,
                    location: item.location.clone(),
                    category: item.category,
                })
                .collect(),
            checklist_items: export
                .checklist_items()
                .iter()
                .map(|item| ComparableChecklistItem {
                    title: item.title.clone(),
                    is_done: item.is_done,
                    sort_order: item.sort_order,
                })
                .collect(),
        }
    }

    #[test]
    fn test_export_includes_checklist_items() {
        let conn = test_db();
        let trip_id = add_test_trip(&conn, "沖縄旅行").unwrap();
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
            "2026-06-01",
            "2026-06-03",
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
    fn test_export_includes_schema_version() {
        let conn = test_db();
        let trip_id = add_test_trip(&conn, "Metadata Trip").unwrap();

        let json = export_trip_to_json(&conn, trip_id).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert!(parsed.get("schema_version").is_some());
    }

    #[test]
    fn test_export_includes_exported_at() {
        let conn = test_db();
        let trip_id = add_test_trip(&conn, "Metadata Trip").unwrap();

        let json = export_trip_to_json(&conn, trip_id).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert!(parsed.get("exported_at").is_some());
    }

    #[test]
    fn test_export_schema_version_is_one() {
        let conn = test_db();
        let trip_id = add_test_trip(&conn, "Metadata Trip").unwrap();

        let json = export_trip_to_json(&conn, trip_id).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["schema_version"], 1);
    }

    #[test]
    fn test_exported_at_parses_as_rfc3339() {
        let conn = test_db();
        let trip_id = add_test_trip(&conn, "Metadata Trip").unwrap();

        let json = export_trip_to_json(&conn, trip_id).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        let exported_at = parsed["exported_at"].as_str().expect("exported_at string");
        chrono::DateTime::parse_from_rfc3339(exported_at).expect("valid RFC3339 timestamp");
    }

    #[test]
    fn test_build_trip_export_leaves_generator_metadata_unset() {
        let conn = test_db();
        let trip_id = add_test_trip(&conn, "Metadata Trip").unwrap();

        let export = build_trip_export(&conn, trip_id).unwrap();
        assert!(export.generator.is_none());
        assert!(export.generator_version.is_none());
        assert!(export.schema_version.is_none());
        assert!(export.exported_at.is_none());
    }

    #[test]
    fn test_export_includes_generator_metadata() {
        let conn = test_db();
        let trip_id = add_test_trip(&conn, "Metadata Trip").unwrap();

        let json = export_trip_to_json(&conn, trip_id).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["generator"], TRIP_EXPORT_GENERATOR);
        assert_eq!(parsed["generator_version"], env!("CARGO_PKG_VERSION"));
        assert_eq!(parsed["schema_version"], 1);
    }

    #[test]
    fn test_comparable_trip_export_ignores_generator_metadata() {
        let conn = test_db();
        let trip_id = add_test_trip(&conn, "Roundtrip Trip").unwrap();
        add_itinerary_item(
            &conn,
            trip_id,
            1,
            "Shuri Castle",
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .unwrap();

        let before = build_trip_export(&conn, trip_id).unwrap();
        let json = export_trip_to_json(&conn, trip_id).unwrap();
        let exported: TripExport = serde_json::from_str(&json).unwrap();

        assert_eq!(exported.generator.as_deref(), Some(TRIP_EXPORT_GENERATOR));
        assert_eq!(
            comparable_trip_export(&before),
            comparable_trip_export(&exported)
        );
    }

    #[test]
    fn test_import_legacy_json_without_generator_metadata() {
        let conn = test_db();
        let json = r#"{
            "schema_version": 1,
            "exported_at": "2026-06-07T00:00:00Z",
            "trip": {
                "id": 1,
                "name": "Legacy Metadata Trip",
                "start_date": "2026-01-01",
                "end_date": "2026-01-03",
                "created_at": "2026-01-01 00:00:00",
                "updated_at": "2026-01-01 00:00:00"
            },
            "itinerary_items": [],
            "checklist_items": []
        }"#;

        let new_id = import_trip_from_json(&conn, json).unwrap();
        let imported = get_trip(&conn, new_id).unwrap();
        assert_eq!(imported.name, "Legacy Metadata Trip");
    }

    #[test]
    fn test_import_json_with_unknown_generator_is_allowed() {
        let conn = test_db();
        let json = r#"{
            "schema_version": 1,
            "generator": "unknown",
            "generator_version": "0.9.0",
            "trip": {
                "id": 1,
                "name": "Unknown Generator Trip",
                "start_date": "2026-01-01",
                "end_date": "2026-01-03",
                "created_at": "2026-01-01 00:00:00",
                "updated_at": "2026-01-01 00:00:00"
            },
            "itinerary_items": [],
            "checklist_items": []
        }"#;

        let new_id = import_trip_from_json(&conn, json).unwrap();
        assert_eq!(
            get_trip(&conn, new_id).unwrap().name,
            "Unknown Generator Trip"
        );
    }

    #[test]
    fn test_import_new_format_with_metadata() {
        let conn = test_db();
        let json = r#"{
            "schema_version": 1,
            "exported_at": "2026-06-07T00:00:00Z",
            "trip": {
                "id": 1,
                "name": "Metadata Import Trip",
                "start_date": "2026-06-01",
                "end_date": "2026-06-03",
                "created_at": "2026-01-01 00:00:00",
                "updated_at": "2026-01-01 00:00:00"
            },
            "itinerary_items": [],
            "checklist_items": []
        }"#;

        let new_id = import_trip_from_json(&conn, json).unwrap();
        let imported = get_trip(&conn, new_id).unwrap();
        assert_eq!(imported.name, "Metadata Import Trip");
    }

    #[test]
    fn test_import_legacy_json_without_metadata() {
        let conn = test_db();
        let json = r#"{
            "trip": {
                "id": 1,
                "name": "Legacy With Checklist",
                "start_date": "2026-01-01",
                "end_date": "2026-01-03",
                "created_at": "2026-01-01 00:00:00",
                "updated_at": "2026-01-01 00:00:00"
            },
            "itinerary_items": [],
            "checklist_items": [
                {
                    "id": 1,
                    "trip_id": 1,
                    "title": "Passport",
                    "is_done": false,
                    "sort_order": 0,
                    "created_at": "2026-01-01 00:00:00",
                    "updated_at": "2026-01-01 00:00:00"
                }
            ]
        }"#;

        let new_id = import_trip_from_json(&conn, json).unwrap();
        let imported = build_trip_export(&conn, new_id).unwrap();
        assert_eq!(imported.trip.name, "Legacy With Checklist");
        assert_eq!(imported.checklist_items().len(), 1);
        assert_eq!(imported.checklist_items()[0].title, "Passport");
    }

    #[test]
    fn test_get_trip_not_found() {
        let conn = test_db();
        let err = get_trip(&conn, 9999).err().expect("expected error");
        assert_eq!(err.to_string(), "Trip not found: 9999");
        assert!(!format!("{err:#}").contains("Query returned no rows"));
    }

    #[test]
    fn test_duplicate_trip_copies_trip_itinerary_and_checklist() {
        let conn = test_db();
        let trip_id = add_trip(&conn, "Okinawa Trip", "2026-06-01", "2026-06-03").unwrap();
        add_itinerary_item(
            &conn,
            trip_id,
            1,
            "Shuri Castle",
            Some("World heritage"),
            Some("09:00"),
            Some(1),
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
        let new_id = duplicate_trip(&conn, trip_id, None).unwrap();

        assert_ne!(new_id, trip_id);
        let duplicated = build_trip_export(&conn, new_id).unwrap();
        assert_eq!(duplicated.trip.name, "Okinawa Trip (Copy)");
        assert_eq!(duplicated.trip.start_date, before.trip.start_date);
        assert_eq!(duplicated.trip.end_date, before.trip.end_date);
        assert_eq!(
            itinerary_sem(&before.itinerary_items),
            itinerary_sem(&duplicated.itinerary_items)
        );
        assert_eq!(
            checklist_sem(before.checklist_items()),
            checklist_sem(duplicated.checklist_items())
        );
    }

    #[test]
    fn test_duplicate_trip_with_custom_name() {
        let conn = test_db();
        let trip_id = add_test_trip(&conn, "Original").unwrap();
        add_itinerary_item(
            &conn, trip_id, 1, "Lunch", None, None, None, None, None, None, None,
        )
        .unwrap();

        let new_id = duplicate_trip(&conn, trip_id, Some("Okinawa Copy")).unwrap();
        let duplicated = get_trip(&conn, new_id).unwrap();
        assert_eq!(duplicated.name, "Okinawa Copy");
    }

    #[test]
    fn test_duplicate_trip_not_found() {
        let conn = test_db();
        let err = duplicate_trip(&conn, 9999, None)
            .err()
            .expect("expected error");
        assert_eq!(err.to_string(), "Trip not found: 9999");
    }

    #[test]
    fn test_export_import_reexport_structural_roundtrip_with_checklist() {
        let conn = test_db();
        let trip_id = add_trip(&conn, "Roundtrip Trip", "2026-07-01", "2026-07-05").unwrap();
        add_itinerary_item(
            &conn,
            trip_id,
            1,
            "Museum",
            Some("Ticket required"),
            Some("10:00"),
            Some(0),
            Some(120),
            Some(15),
            Some("Downtown"),
            Some(crate::models::ItineraryCategory::Museum),
        )
        .unwrap();
        add_checklist_item(&conn, trip_id, "Passport").unwrap();
        let ticket_id = add_checklist_item(&conn, trip_id, "Museum ticket").unwrap();
        set_checklist_done(&conn, ticket_id, true).unwrap();

        let before = build_trip_export(&conn, trip_id).unwrap();
        let before_compare = comparable_trip_export(&before);
        let json = export_trip_to_json(&conn, trip_id).unwrap();

        reset_db(&conn).unwrap();

        let imported_id = import_trip_from_json(&conn, &json).unwrap();
        let after = build_trip_export(&conn, imported_id).unwrap();
        assert_eq!(before_compare, comparable_trip_export(&after));

        let re_json = export_trip_to_json(&conn, imported_id).unwrap();
        let parsed_before: TripExport = serde_json::from_str(&json).unwrap();
        let parsed_after: TripExport = serde_json::from_str(&re_json).unwrap();
        assert_eq!(
            comparable_trip_export(&parsed_before),
            comparable_trip_export(&parsed_after)
        );
    }

    fn check_passed(report: &ExportValidationReport, id: ExportValidationCheckId) -> bool {
        report
            .checks
            .iter()
            .find(|check| check.id == id)
            .expect("check should exist")
            .passed
    }

    #[test]
    fn test_analyze_trip_export_current_format_is_valid() {
        let conn = test_db();
        let trip_id = add_test_trip(&conn, "沖縄家族旅行").unwrap();
        add_itinerary_item(
            &conn,
            trip_id,
            1,
            "首里城",
            None,
            None,
            None,
            Some(90),
            None,
            None,
            None,
        )
        .unwrap();
        add_checklist_item(&conn, trip_id, "Passport").unwrap();

        let json = export_trip_to_json(&conn, trip_id).unwrap();
        let report = analyze_trip_export_json("backup.json", &json);

        assert!(report.valid);
        assert_eq!(
            report.schema_version,
            crate::models::EXPORT_VALIDATION_REPORT_SCHEMA_VERSION
        );
        assert_eq!(
            report.export_schema_version,
            Some(TRIP_EXPORT_SCHEMA_VERSION)
        );
        assert_eq!(report.trip_name.as_deref(), Some("沖縄家族旅行"));
        assert_eq!(report.itinerary_count, 1);
        assert_eq!(report.checklist_count, 1);
        assert!(report.warnings.is_empty());
        assert!(report.errors.is_empty());
        assert!(check_passed(&report, ExportValidationCheckId::JsonFormat));
        assert!(check_passed(
            &report,
            ExportValidationCheckId::SchemaVersion
        ));
        assert!(check_passed(&report, ExportValidationCheckId::Trip));
        assert!(check_passed(
            &report,
            ExportValidationCheckId::ItineraryItems
        ));
        assert!(check_passed(
            &report,
            ExportValidationCheckId::ChecklistItems
        ));
        assert_eq!(report.generator.as_deref(), Some(TRIP_EXPORT_GENERATOR));
        assert!(report.generator_version.is_some());
        assert!(report.exported_at.is_some());
        assert!(report.export_metadata.is_some());
    }

    #[test]
    fn test_analyze_trip_export_legacy_format_is_valid_with_warnings() {
        let json = r#"{
            "trip": {
                "id": 1,
                "name": "Legacy Trip",
                "start_date": "2026-01-01",
                "end_date": "2026-01-03",
                "created_at": "2026-01-01 00:00:00",
                "updated_at": "2026-01-01 00:00:00"
            },
            "itinerary_items": []
        }"#;

        let report = analyze_trip_export_json("legacy.json", json);

        assert!(report.valid);
        assert!(report.errors.is_empty());
        assert_eq!(report.warnings.len(), 2);
        assert!(report.warnings.iter().any(|w| w.contains("schema_version")));
        assert!(report
            .warnings
            .iter()
            .any(|w| w.contains("checklist_items")));
        assert!(!check_passed(
            &report,
            ExportValidationCheckId::SchemaVersion
        ));
        assert!(!check_passed(
            &report,
            ExportValidationCheckId::ChecklistItems
        ));
        assert!(check_passed(&report, ExportValidationCheckId::Trip));
        assert_eq!(report.generator, None);
        assert_eq!(report.generator_version, None);
        assert_eq!(report.exported_at, None);
        let metadata = report.export_metadata.as_ref().unwrap();
        assert_eq!(metadata.display_generator(), "不明");
        assert_eq!(metadata.display_generator_version(), "不明");
        assert_eq!(metadata.display_exported_at(), "不明");
    }

    #[test]
    fn test_analyze_trip_export_empty_checklist_items_key_passes_check() {
        let json = r#"{
            "schema_version": 1,
            "exported_at": "2026-06-07T00:00:00Z",
            "trip": {
                "id": 1,
                "name": "Empty Checklist Trip",
                "start_date": "2026-01-01",
                "end_date": "2026-01-03",
                "created_at": "2026-01-01 00:00:00",
                "updated_at": "2026-01-01 00:00:00"
            },
            "itinerary_items": [],
            "checklist_items": []
        }"#;

        let report = analyze_trip_export_json("empty-checklist.json", json);

        assert!(report.valid);
        assert!(check_passed(
            &report,
            ExportValidationCheckId::ChecklistItems
        ));
        assert_eq!(report.checklist_count, 0);
    }

    #[test]
    fn test_analyze_trip_export_unsupported_schema_version_is_valid_with_warning() {
        let json = r#"{
            "schema_version": 99,
            "trip": {
                "id": 1,
                "name": "Future Schema Trip",
                "start_date": "2026-01-01",
                "end_date": "2026-01-03",
                "created_at": "2026-01-01 00:00:00",
                "updated_at": "2026-01-01 00:00:00"
            },
            "itinerary_items": [],
            "checklist_items": []
        }"#;

        let report = analyze_trip_export_json("future.json", json);

        assert!(report.valid);
        assert!(!check_passed(
            &report,
            ExportValidationCheckId::SchemaVersion
        ));
        assert!(report.warnings.iter().any(|w| w.contains("正式サポート外")));
    }

    #[test]
    fn test_analyze_trip_export_empty_trip_name_is_invalid() {
        let json = r#"{
            "schema_version": 1,
            "trip": {
                "id": 1,
                "name": "   ",
                "start_date": "2026-01-01",
                "end_date": "2026-01-03",
                "created_at": "2026-01-01 00:00:00",
                "updated_at": "2026-01-01 00:00:00"
            },
            "itinerary_items": [],
            "checklist_items": []
        }"#;

        let report = analyze_trip_export_json("bad-trip.json", json);

        assert!(!report.valid);
        assert!(!check_passed(&report, ExportValidationCheckId::Trip));
        assert!(report.errors.iter().any(|e| e.contains("trip.name")));
    }

    #[test]
    fn test_analyze_trip_export_empty_itinerary_title_is_invalid() {
        let json = r#"{
            "schema_version": 1,
            "trip": {
                "id": 1,
                "name": "Bad Itinerary Trip",
                "start_date": "2026-01-01",
                "end_date": "2026-01-03",
                "created_at": "2026-01-01 00:00:00",
                "updated_at": "2026-01-01 00:00:00"
            },
            "itinerary_items": [
                {
                    "id": 1,
                    "trip_id": 1,
                    "day": 1,
                    "title": " ",
                    "note": null,
                    "start_time": null,
                    "sort_order": 0,
                    "duration_minutes": null,
                    "travel_minutes": null,
                    "location": null,
                    "created_at": "2026-01-01 00:00:00",
                    "updated_at": "2026-01-01 00:00:00"
                }
            ],
            "checklist_items": []
        }"#;

        let report = analyze_trip_export_json("bad-itinerary.json", json);

        assert!(!report.valid);
        assert!(report
            .errors
            .iter()
            .any(|e| e.contains("itinerary_items[0].title")));
    }

    #[test]
    fn test_analyze_trip_export_invalid_json_is_invalid() {
        let report = analyze_trip_export_json("broken.json", "not json");

        assert!(!report.valid);
        assert!(!check_passed(&report, ExportValidationCheckId::JsonFormat));
        assert_eq!(report.checks.len(), 1);
        assert!(report.errors.iter().any(|e| e.contains("JSON")));
    }

    #[test]
    fn test_analyze_trip_export_missing_itinerary_items_is_invalid() {
        let json = r#"{
            "schema_version": 1,
            "trip": {
                "id": 1,
                "name": "No Itinerary Key",
                "start_date": "2026-01-01",
                "end_date": "2026-01-03",
                "created_at": "2026-01-01 00:00:00",
                "updated_at": "2026-01-01 00:00:00"
            },
            "checklist_items": []
        }"#;

        let report = analyze_trip_export_json("no-itinerary-key.json", json);

        assert!(!report.valid);
        assert!(!check_passed(
            &report,
            ExportValidationCheckId::ItineraryItems
        ));
    }

    #[test]
    fn test_analyze_trip_export_file_not_found() {
        assert!(analyze_trip_export("nonexistent-export.json").is_err());
    }

    #[test]
    fn test_analyze_trip_export_legacy_matches_import_success() {
        let conn = test_db();
        let json = r#"{
            "trip": {
                "id": 1,
                "name": "Legacy Import Trip",
                "start_date": "2026-01-01",
                "end_date": "2026-01-03",
                "created_at": "2026-01-01 00:00:00",
                "updated_at": "2026-01-01 00:00:00"
            },
            "itinerary_items": []
        }"#;

        let report = analyze_trip_export_json("legacy.json", json);
        assert!(report.valid);
        assert!(import_trip_from_json(&conn, json).is_ok());
    }

    #[test]
    fn test_analyze_trip_export_json_report_serializes_schema_fields() {
        let json = r#"{
            "schema_version": 1,
            "trip": {
                "id": 1,
                "name": "Serialize Trip",
                "start_date": "2026-01-01",
                "end_date": "2026-01-03",
                "created_at": "2026-01-01 00:00:00",
                "updated_at": "2026-01-01 00:00:00"
            },
            "itinerary_items": [],
            "checklist_items": []
        }"#;

        let report = analyze_trip_export_json("serialize.json", json);
        let value = serde_json::to_value(&report).unwrap();
        assert_eq!(value["schema_version"], 1);
        assert_eq!(value["export_schema_version"], 1);
        assert_eq!(value["generator"], serde_json::Value::Null);
        assert_eq!(value["generator_version"], serde_json::Value::Null);
        assert_eq!(value["exported_at"], serde_json::Value::Null);
        assert!(value["checks"].is_array());
        assert_eq!(value["errors"], serde_json::json!([]));
    }

    #[test]
    fn test_analyze_trip_export_json_report_includes_generator_metadata() {
        let json = r#"{
            "schema_version": 1,
            "generator": "caglla-cli",
            "generator_version": "1.0.8",
            "exported_at": "2026-06-07T12:34:56Z",
            "trip": {
                "id": 1,
                "name": "Metadata Trip",
                "start_date": "2026-01-01",
                "end_date": "2026-01-03",
                "created_at": "2026-01-01 00:00:00",
                "updated_at": "2026-01-01 00:00:00"
            },
            "itinerary_items": [],
            "checklist_items": []
        }"#;

        let report = analyze_trip_export_json("metadata.json", json);
        assert_eq!(report.generator.as_deref(), Some("caglla-cli"));
        assert_eq!(report.generator_version.as_deref(), Some("1.0.8"));
        assert_eq!(report.exported_at.as_deref(), Some("2026-06-07T12:34:56Z"));

        let metadata = report.export_metadata.as_ref().unwrap();
        assert_eq!(metadata.display_generator(), "caglla-cli");
        assert_eq!(metadata.display_generator_version(), "1.0.8");
        assert_eq!(metadata.display_exported_at(), "2026-06-07T12:34:56Z");

        let value = serde_json::to_value(&report).unwrap();
        assert_eq!(value["generator"], "caglla-cli");
        assert_eq!(value["generator_version"], "1.0.8");
        assert_eq!(value["exported_at"], "2026-06-07T12:34:56Z");
    }

    #[test]
    fn test_analyze_trip_export_unknown_generator_has_no_metadata_warning() {
        let json = r#"{
            "schema_version": 1,
            "generator": "unknown",
            "generator_version": "0.9.0",
            "trip": {
                "id": 1,
                "name": "Unknown Generator Trip",
                "start_date": "2026-01-01",
                "end_date": "2026-01-03",
                "created_at": "2026-01-01 00:00:00",
                "updated_at": "2026-01-01 00:00:00"
            },
            "itinerary_items": [],
            "checklist_items": []
        }"#;

        let report = analyze_trip_export_json("unknown-generator.json", json);
        assert!(report.valid);
        assert_eq!(report.generator.as_deref(), Some("unknown"));
        assert_eq!(report.generator_version.as_deref(), Some("0.9.0"));
        assert!(report
            .warnings
            .iter()
            .all(|warning| !warning.contains("generator")));
    }

    #[test]
    fn test_analyze_trip_export_invalid_exported_at_is_warning_only() {
        let json = r#"{
            "schema_version": 1,
            "exported_at": "not-rfc3339",
            "trip": {
                "id": 1,
                "name": "Bad Timestamp Trip",
                "start_date": "2026-01-01",
                "end_date": "2026-01-03",
                "created_at": "2026-01-01 00:00:00",
                "updated_at": "2026-01-01 00:00:00"
            },
            "itinerary_items": [],
            "checklist_items": []
        }"#;

        let report = analyze_trip_export_json("bad-exported-at.json", json);
        assert!(report.valid);
        assert_eq!(report.exported_at.as_deref(), Some("not-rfc3339"));
        assert!(report
            .warnings
            .iter()
            .any(|warning| warning.contains("exported_at") && warning.contains("RFC3339")));
    }

    #[test]
    fn test_import_summary_new_format_schema_version() {
        let conn = test_db();
        let json = r#"{
            "schema_version": 1,
            "generator": "caglla-cli",
            "generator_version": "1.0.8",
            "exported_at": "2026-06-07T00:00:00Z",
            "trip": {
                "id": 1,
                "name": "沖縄家族旅行",
                "start_date": "2026-01-01",
                "end_date": "2026-01-03",
                "created_at": "2026-01-01 00:00:00",
                "updated_at": "2026-01-01 00:00:00"
            },
            "itinerary_items": [
                {
                    "id": 1,
                    "trip_id": 1,
                    "day": 1,
                    "title": "首里城",
                    "note": null,
                    "start_time": null,
                    "sort_order": 0,
                    "duration_minutes": null,
                    "travel_minutes": null,
                    "location": null,
                    "created_at": "2026-01-01 00:00:00",
                    "updated_at": "2026-01-01 00:00:00"
                }
            ],
            "checklist_items": [
                {
                    "id": 1,
                    "trip_id": 1,
                    "title": "Passport",
                    "is_done": false,
                    "sort_order": 0,
                    "created_at": "2026-01-01 00:00:00",
                    "updated_at": "2026-01-01 00:00:00"
                }
            ]
        }"#;

        let summary = import_trip_from_json_with_summary(&conn, json).unwrap();
        assert_eq!(summary.trip_name, "沖縄家族旅行");
        assert_eq!(summary.trip_id, 1);
        assert_eq!(summary.itinerary_count, 1);
        assert_eq!(summary.checklist_count, 1);
        assert!(summary.schema_version_present);
        assert_eq!(summary.export_schema_version, Some(1));
        assert_eq!(import_schema_display_line(&summary), "  version 1");
        assert_eq!(summary.export_metadata.display_generator(), "caglla-cli");
        assert_eq!(summary.export_metadata.display_generator_version(), "1.0.8");
    }

    #[test]
    fn test_import_summary_legacy_export_metadata_is_unknown() {
        let conn = test_db();
        let json = r#"{
            "trip": {
                "id": 1,
                "name": "Legacy Trip",
                "start_date": "2026-01-01",
                "end_date": "2026-01-03",
                "created_at": "2026-01-01 00:00:00",
                "updated_at": "2026-01-01 00:00:00"
            },
            "itinerary_items": []
        }"#;

        let summary = import_trip_from_json_with_summary(&conn, json).unwrap();
        assert_eq!(summary.export_metadata.display_generator(), "不明");
        assert_eq!(summary.export_metadata.display_generator_version(), "不明");
    }

    #[test]
    fn test_import_summary_from_export_includes_generator_metadata() {
        let conn = test_db();
        let trip_id = add_test_trip(&conn, "Export Metadata Trip").unwrap();
        add_checklist_item(&conn, trip_id, "Passport").unwrap();
        let json = export_trip_to_json(&conn, trip_id).unwrap();

        let summary = import_trip_from_json_with_summary(&conn, &json).unwrap();
        assert_eq!(
            summary.export_metadata.display_generator(),
            TRIP_EXPORT_GENERATOR
        );
        assert!(summary.export_metadata.generator_version_present);
        assert!(summary.export_metadata.generator_version.is_some());
    }

    #[test]
    fn test_import_summary_legacy_schema_display() {
        let conn = test_db();
        let json = r#"{
            "trip": {
                "id": 1,
                "name": "Legacy Trip",
                "start_date": "2026-01-01",
                "end_date": "2026-01-03",
                "created_at": "2026-01-01 00:00:00",
                "updated_at": "2026-01-01 00:00:00"
            },
            "itinerary_items": []
        }"#;

        let summary = import_trip_from_json_with_summary(&conn, json).unwrap();
        assert!(!summary.schema_version_present);
        assert_eq!(summary.export_schema_version, None);
        assert_eq!(import_schema_display_line(&summary), "  未指定（旧形式）");
    }

    #[test]
    fn test_import_trip_from_json_with_summary_matches_import_only() {
        let conn = test_db();
        let json =
            export_trip_to_json(&conn, add_test_trip(&conn, "Compare Trip").unwrap()).unwrap();

        reset_db(&conn).unwrap();
        let id_only = import_trip_from_json(&conn, &json).unwrap();
        reset_db(&conn).unwrap();
        let summary = import_trip_from_json_with_summary(&conn, &json).unwrap();
        assert_eq!(summary.trip_id, id_only);
    }
}
