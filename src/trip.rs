use anyhow::{Context, Result};
use chrono::Utc;
use rusqlite::{params, Connection};

use crate::db::now_string;
use crate::models::{
    effective_export_schema_version, is_supported_export_schema_version, ExportDayV3,
    ExportExpenseV3, ExportItineraryV3, ExportReservation, ExportReservationV3,
    ExportValidationCheck, ExportValidationCheckId, ExportValidationReport, ItineraryNoteKey, Trip,
    TripExport, TripExportMetadata, TripExportV3, TripImportSummary, TRIP_EXPORT_GENERATOR,
    TRIP_EXPORT_SCHEMA_VERSION, TRIP_EXPORT_SCHEMA_VERSION_V1, TRIP_EXPORT_SCHEMA_VERSION_V3,
};

/// 新しい旅行を追加する
pub(crate) fn add_trip(
    conn: &Connection,
    name: &str,
    start: &str,
    end: &str,
    summary: Option<&str>,
) -> Result<i64> {
    let day_count = crate::day::validate_trip_date_range(start, end)?;
    let summary = crate::summary::normalize_trip_summary(summary)?;
    let now = now_string();
    conn.execute(
        "INSERT INTO trips (name, start_date, end_date, summary, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![name, start, end, summary, &now, &now],
    )
    .context("旅行の追加に失敗しました")?;
    let trip_id = conn.last_insert_rowid();
    crate::day::create_days_for_trip(conn, trip_id, day_count)?;
    Ok(trip_id)
}

#[cfg(test)]
pub(crate) fn add_test_trip(conn: &Connection, name: &str) -> Result<i64> {
    add_trip(conn, name, "2026-01-01", "2026-01-03", None)
}

#[cfg(test)]
pub(crate) fn add_test_self_participant(conn: &Connection, trip_id: i64) -> Result<i64> {
    crate::participant::create_participant(conn, trip_id, "自分", None, true)
}

/// すべての旅行を取得する
pub(crate) fn list_trips(conn: &Connection) -> Result<Vec<Trip>> {
    let mut stmt = conn
        .prepare(
            "SELECT id, name, start_date, end_date, summary, created_at, updated_at
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
            "SELECT id, name, start_date, end_date, summary, created_at, updated_at
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
    summary: Option<&str>,
    clear_summary: bool,
) -> Result<()> {
    if name.is_none() && start.is_none() && end.is_none() && summary.is_none() && !clear_summary {
        anyhow::bail!(
            "更新する項目を1つ以上指定してください (--name, --start, --end, --summary, --clear-summary)"
        );
    }

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
    if clear_summary {
        trip.summary = None;
    } else if let Some(s) = summary {
        trip.summary = crate::summary::normalize_trip_summary(Some(s))?;
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
    crate::db::with_transaction(conn, "trip update", |tx| {
        tx.execute(
            "UPDATE trips
             SET name = ?1, start_date = ?2, end_date = ?3, summary = ?4, updated_at = ?5
             WHERE id = ?6",
            params![
                trip.name,
                trip.start_date,
                trip.end_date,
                trip.summary,
                &now,
                id
            ],
        )
        .context("旅行の更新に失敗しました")?;
        crate::day::sync_days_to_trip_duration(tx, id, day_count)?;
        Ok(())
    })?;
    Ok(())
}

/// エクスポート用データを組み立てる
pub(crate) fn build_trip_export(conn: &Connection, trip_id: i64) -> Result<TripExport> {
    let trip = get_trip(conn, trip_id)?;
    let itinerary_items = crate::itinerary::list_itinerary_items(conn, trip_id)?;
    let checklist_items = crate::checklist::list_checklist_items(conn, trip_id)?;
    let notes = crate::note::build_export_notes(conn, trip_id)?;
    let day_summaries = crate::day::list_days(conn, trip_id)?
        .into_iter()
        .filter(|day| day.summary.is_some())
        .map(|day| crate::models::ExportDaySummary {
            day_number: day.day_number,
            summary: day.summary.clone(),
        })
        .collect();
    let mut reservations = Vec::new();
    for item in &itinerary_items {
        let itinerary_key = ItineraryNoteKey {
            day_number: item.day,
            sort_order: item.sort_order,
            start_time: item.start_time.clone(),
            title: item.title.clone(),
        };
        for reservation in crate::reservation::list_reservations_for_itinerary(conn, item.id)? {
            reservations.push(ExportReservation {
                itinerary_key: itinerary_key.clone(),
                reservation: ExportReservationV3 {
                    reservation_type: reservation.reservation_type,
                    provider_name: reservation.provider_name,
                    confirmation_code: reservation.confirmation_code,
                    reservation_site_url: reservation.reservation_site_url,
                    remark: reservation.remark,
                    start_at: reservation.start_at,
                    end_at: reservation.end_at,
                },
            });
        }
    }
    Ok(TripExport {
        schema_version: None,
        generator: None,
        generator_version: None,
        exported_at: None,
        trip,
        itinerary_items,
        checklist_items: Some(checklist_items),
        notes: Some(notes),
        day_summaries,
        reservations,
        participants: vec![],
    })
}

/// schema v3 用 export データを組み立てる（Expense は Itinerary 配下にネスト）
pub(crate) fn build_trip_export_v3(conn: &Connection, trip_id: i64) -> Result<TripExportV3> {
    let base = build_trip_export(conn, trip_id)?;

    // Itinerary を day_number でグルーピングし、各 Itinerary に expenses を付与する
    let mut day_map: std::collections::BTreeMap<i64, ExportDayV3> =
        std::collections::BTreeMap::new();
    for day in crate::day::list_days(conn, trip_id)? {
        day_map.insert(
            day.day_number,
            ExportDayV3 {
                day_number: day.day_number,
                summary: day.summary.clone(),
                itineraries: Vec::new(),
            },
        );
    }
    for item in &base.itinerary_items {
        let expenses = crate::expense::list_expenses_for_itinerary(conn, item.id)?
            .into_iter()
            .map(|e| ExportExpenseV3 {
                title: e.title,
                amount: e.amount,
                currency: e.currency,
                paid_by_name: e.paid_by_name,
                expense_date: e.expense_date,
                note: e.note,
                sort_order: e.sort_order,
            })
            .collect::<Vec<_>>();
        let reservations = crate::reservation::list_reservations_for_itinerary(conn, item.id)?
            .into_iter()
            .map(|r| ExportReservationV3 {
                reservation_type: r.reservation_type,
                provider_name: r.provider_name,
                confirmation_code: r.confirmation_code,
                reservation_site_url: r.reservation_site_url,
                remark: r.remark,
                start_at: r.start_at,
                end_at: r.end_at,
            })
            .collect::<Vec<_>>();

        day_map
            .entry(item.day)
            .or_insert_with(|| ExportDayV3 {
                day_number: item.day,
                summary: None,
                itineraries: Vec::new(),
            })
            .itineraries
            .push(ExportItineraryV3 {
                title: item.title.clone(),
                note: item.note.clone(),
                start_time: item.start_time.clone(),
                sort_order: item.sort_order,
                duration_minutes: item.duration_minutes,
                travel_minutes: item.travel_minutes,
                location: item.location.clone(),
                category: item.category,
                expenses,
                reservations,
            });
    }

    let mut days = Vec::new();
    for (day_number, day) in day_map {
        days.push(ExportDayV3 {
            day_number,
            summary: day.summary,
            itineraries: day.itineraries,
        });
    }

    Ok(TripExportV3 {
        schema_version: None,
        generator: None,
        generator_version: None,
        exported_at: None,
        trip: base.trip,
        days,
        checklist_items: base.checklist_items,
        notes: base.notes,
        participants: Some(crate::participant::build_export_participants(
            conn, trip_id,
        )?),
    })
}

fn export_timestamp_rfc3339() -> String {
    Utc::now().to_rfc3339()
}

fn finalize_trip_export_v3(mut export: TripExportV3) -> TripExportV3 {
    export.schema_version = Some(TRIP_EXPORT_SCHEMA_VERSION);
    export.generator = Some(TRIP_EXPORT_GENERATOR.to_string());
    export.generator_version = Some(env!("CARGO_PKG_VERSION").to_string());
    export.exported_at = Some(export_timestamp_rfc3339());
    export
}

/// 旅行データを pretty JSON 文字列に変換する
pub(crate) fn export_trip_to_json(conn: &Connection, trip_id: i64) -> Result<String> {
    let export = finalize_trip_export_v3(build_trip_export_v3(conn, trip_id)?);
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
/// インポート用 JSON の必須項目を検証する（Note を除く）
fn validate_trip_export_core(export: &TripExport) -> Result<()> {
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
    if !is_supported_export_schema_version(export.schema_version) {
        anyhow::bail!(
            "schema_version {} はサポートされていません",
            export.schema_version.unwrap_or_default()
        );
    }
    Ok(())
}

fn validate_trip_export_v3(export: &TripExportV3) -> Result<()> {
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

    crate::summary::normalize_summary_for_import(
        export.trip.summary.as_deref(),
        crate::summary::TRIP_SUMMARY_MAX_LEN,
    )?;

    // days/itineraries
    for (d_index, day) in export.days.iter().enumerate() {
        crate::summary::normalize_summary_for_import(
            day.summary.as_deref(),
            crate::summary::DAY_SUMMARY_MAX_LEN,
        )
        .with_context(|| format!("days[{d_index}].summary is invalid"))?;
        if day.day_number < 1 || day.day_number > day_count {
            anyhow::bail!(
                "days[{d_index}].day_number ({}) は旅行期間 (1..={day_count}) の範囲外です",
                day.day_number
            );
        }
        for (i_index, it) in day.itineraries.iter().enumerate() {
            if it.title.trim().is_empty() {
                anyhow::bail!("days[{d_index}].itineraries[{i_index}].title は必須です");
            }
            // expenses
            for (e_index, exp) in it.expenses.iter().enumerate() {
                // amount/currency 必須
                let _ = exp.amount;
                if exp.currency.trim().is_empty() {
                    anyhow::bail!(
                        "days[{d_index}].itineraries[{i_index}].expenses[{e_index}].currency は必須です"
                    );
                }
                let _ = crate::expense::validate_currency_code(&exp.currency)?;
                crate::expense::validate_expense_date_opt(&exp.expense_date)?;
            }
            for (r_index, res) in it.reservations.iter().enumerate() {
                crate::reservation::validate_export_reservation_v3(res).with_context(|| {
                    format!(
                        "days[{d_index}].itineraries[{i_index}].reservations[{r_index}] is invalid"
                    )
                })?;
            }
        }
    }

    // checklist / notes は v2 と同様
    for (index, item) in export.checklist_items().iter().enumerate() {
        if item.title.trim().is_empty() {
            anyhow::bail!("checklist_items[{index}].title は必須です");
        }
    }

    // notes 検証のため itinerary_items 互換の配列を生成
    let itinerary_items: Vec<crate::models::ItineraryItem> = export
        .days
        .iter()
        .flat_map(|day| {
            day.itineraries
                .iter()
                .map(move |it| crate::models::ItineraryItem {
                    id: 0,
                    trip_id: 0,
                    day: day.day_number,
                    title: it.title.clone(),
                    note: it.note.clone(),
                    start_time: it.start_time.clone(),
                    sort_order: it.sort_order,
                    duration_minutes: it.duration_minutes,
                    travel_minutes: it.travel_minutes,
                    location: it.location.clone(),
                    category: it.category,
                    created_at: String::new(),
                    updated_at: String::new(),
                })
        })
        .collect();

    let day_summaries = export
        .days
        .iter()
        .filter(|day| day.summary.is_some())
        .map(|day| crate::models::ExportDaySummary {
            day_number: day.day_number,
            summary: day.summary.clone(),
        })
        .collect();

    let v2_like = TripExport {
        schema_version: export.schema_version,
        generator: export.generator.clone(),
        generator_version: export.generator_version.clone(),
        exported_at: export.exported_at.clone(),
        trip: export.trip.clone(),
        itinerary_items,
        checklist_items: export.checklist_items.clone(),
        notes: export.notes.clone(),
        day_summaries,
        reservations: flatten_reservations_from_v3(export),
        participants: export.participants().to_vec(),
    };
    if let Some(error) = crate::note::collect_export_note_validation_errors(&v2_like)
        .into_iter()
        .next()
    {
        anyhow::bail!(error);
    }
    if let Some(error) =
        crate::participant::collect_export_participant_validation_errors(export.participants())
            .into_iter()
            .next()
    {
        anyhow::bail!(error);
    }

    Ok(())
}

/// インポート用 JSON の必須項目を検証する
pub(crate) fn validate_trip_export(export: &TripExport) -> Result<()> {
    validate_trip_export_core(export)?;
    if let Some(error) = crate::note::collect_export_note_validation_errors(export)
        .into_iter()
        .next()
    {
        anyhow::bail!(error);
    }
    Ok(())
}

fn is_v3_or_later_export_schema(schema_version: Option<i32>) -> bool {
    matches!(
        effective_export_schema_version(schema_version),
        TRIP_EXPORT_SCHEMA_VERSION_V3 | TRIP_EXPORT_SCHEMA_VERSION
    )
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

    let schema_version = root
        .get("schema_version")
        .and_then(|v| v.as_i64())
        .map(|v| v as i32);
    let effective_schema = effective_export_schema_version(schema_version);
    report.export_schema_version = schema_version;

    // schema check
    let has_schema_version = root.get("schema_version").is_some();
    let schema_check_passed = is_supported_export_schema_version(schema_version);
    push_check(
        &mut report.checks,
        ExportValidationCheckId::SchemaVersion,
        schema_check_passed,
    );
    if !has_schema_version {
        report
            .warnings
            .push("schema_version がありません（旧形式 v1）".to_string());
    } else if effective_schema == TRIP_EXPORT_SCHEMA_VERSION_V1 {
        report
            .warnings
            .push("schema_version 1（v1 形式）".to_string());
    } else if !schema_check_passed {
        report.warnings.push(format!(
            "schema_version {} は未対応です。import は試行可能ですが、正式サポート外の形式です。",
            schema_version.unwrap_or_default()
        ));
    } else if effective_schema != TRIP_EXPORT_SCHEMA_VERSION {
        report.warnings.push(format!(
            "schema_version {} は現行 export ({}) ではありません",
            effective_schema, TRIP_EXPORT_SCHEMA_VERSION
        ));
    }

    // deserialize
    if is_v3_or_later_export_schema(schema_version) {
        let export: TripExportV3 = match serde_json::from_value(root.clone()) {
            Ok(export) => export,
            Err(error) => {
                push_check(&mut report.checks, ExportValidationCheckId::Trip, false);
                push_check(
                    &mut report.checks,
                    ExportValidationCheckId::ItineraryItems,
                    root.get("days").map(|v| v.is_array()).unwrap_or(false),
                );
                push_check(&mut report.checks, ExportValidationCheckId::Expenses, true);
                push_check(
                    &mut report.checks,
                    ExportValidationCheckId::Reservations,
                    true,
                );
                push_check(
                    &mut report.checks,
                    ExportValidationCheckId::ChecklistItems,
                    root.get("checklist_items").is_some(),
                );
                push_check(
                    &mut report.checks,
                    ExportValidationCheckId::Notes,
                    root.get("notes").map(|v| v.is_array()).unwrap_or(true),
                );
                push_check(
                    &mut report.checks,
                    ExportValidationCheckId::Participants,
                    root.get("participants")
                        .map(|v| v.is_array())
                        .unwrap_or(true),
                );
                report
                    .errors
                    .push(format!("export JSON の構造が不正です: {error}"));
                return report;
            }
        };

        let metadata = crate::models::TripExportMetadata {
            generator_present: root.get("generator").is_some(),
            generator: export.generator.clone(),
            generator_version_present: root.get("generator_version").is_some(),
            generator_version: export.generator_version.clone(),
            exported_at_present: root.get("exported_at").is_some(),
            exported_at: export.exported_at.clone(),
        };
        report.export_metadata = Some(metadata.clone());
        report.generator = metadata.json_generator();
        report.generator_version = metadata.json_generator_version();
        report.exported_at = metadata.json_exported_at();

        report.trip_name = Some(export.trip.name.clone());
        report.itinerary_count = export.days.iter().map(|d| d.itineraries.len()).sum();
        report.checklist_count = export.checklist_items().len();
        report.note_count = export.notes().len();
        report.participant_count = export.participants().len();

        // checks
        push_check(
            &mut report.checks,
            ExportValidationCheckId::Trip,
            !export.trip.name.trim().is_empty(),
        );
        let has_days = root.get("days").is_some();
        push_check(
            &mut report.checks,
            ExportValidationCheckId::ItineraryItems,
            has_days && root.get("days").map(|v| v.is_array()).unwrap_or(false),
        );
        push_check(
            &mut report.checks,
            ExportValidationCheckId::ChecklistItems,
            root.get("checklist_items").is_some(),
        );
        push_check(
            &mut report.checks,
            ExportValidationCheckId::Notes,
            root.get("notes").map(|v| v.is_array()).unwrap_or(true),
        );
        let participants_is_array = root
            .get("participants")
            .map(|v| v.is_array())
            .unwrap_or(effective_schema == TRIP_EXPORT_SCHEMA_VERSION_V3);
        push_check(
            &mut report.checks,
            ExportValidationCheckId::Participants,
            participants_is_array,
        );
        if effective_schema == TRIP_EXPORT_SCHEMA_VERSION && root.get("participants").is_none() {
            report
                .warnings
                .push("participants がありません（schema v4 では空配列を推奨）".to_string());
        }
        push_check(&mut report.checks, ExportValidationCheckId::Expenses, true);
        push_check(
            &mut report.checks,
            ExportValidationCheckId::Reservations,
            true,
        );

        // validate
        if let Err(error) = validate_trip_export_v3(&export) {
            report.errors.push(error.to_string());
        }
        for error in
            crate::participant::collect_export_participant_validation_errors(export.participants())
        {
            report.errors.push(error);
        }
        report.valid = report.errors.is_empty();
        return report;
    }

    // v1/v2
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
                ExportValidationCheckId::Notes,
                root.get("notes").map(|v| v.is_array()).unwrap_or(true),
            );
            push_check(
                &mut report.checks,
                ExportValidationCheckId::Expenses,
                root.get("days").is_none(),
            );
            push_check(
                &mut report.checks,
                ExportValidationCheckId::Reservations,
                root.get("days").is_none(),
            );
            report
                .errors
                .push(format!("export JSON の構造が不正です: {error}"));
            return report;
        }
    };

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

    let has_notes = root.get("notes").is_some();
    let notes_is_array = root.get("notes").map(|v| v.is_array()).unwrap_or(true);
    push_check(
        &mut report.checks,
        ExportValidationCheckId::Notes,
        notes_is_array,
    );
    if effective_schema == 2 && !has_notes {
        report
            .warnings
            .push("notes がありません（schema v2 では推奨）".to_string());
    } else if has_notes && !notes_is_array {
        report
            .errors
            .push("notes は配列である必要があります".to_string());
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
    report.note_count = export.notes().len();

    if let Err(error) = validate_trip_export_core(&export) {
        report.errors.push(error.to_string());
    }
    for error in crate::note::collect_export_note_validation_errors(&export) {
        report.errors.push(error);
    }

    // v1/v2 は Expense / Reservation 非対象（v3 で追加）
    push_check(&mut report.checks, ExportValidationCheckId::Expenses, false);
    push_check(
        &mut report.checks,
        ExportValidationCheckId::Reservations,
        false,
    );
    report.valid = report.errors.is_empty();
    report
}

/// export JSON ファイルを検証する（DB は使わない）
pub(crate) fn analyze_trip_export(path: &str) -> Result<ExportValidationReport> {
    let json = std::fs::read_to_string(path)
        .with_context(|| format!("ファイル '{path}' を読み込めませんでした"))?;
    Ok(analyze_trip_export_json(path, &json))
}

const EXPORT_VALIDATION_CHECK_ORDER: [ExportValidationCheckId; 9] = [
    ExportValidationCheckId::JsonFormat,
    ExportValidationCheckId::SchemaVersion,
    ExportValidationCheckId::Trip,
    ExportValidationCheckId::ItineraryItems,
    ExportValidationCheckId::ChecklistItems,
    ExportValidationCheckId::Notes,
    ExportValidationCheckId::Expenses,
    ExportValidationCheckId::Reservations,
    ExportValidationCheckId::Participants,
];

fn export_validation_check_label(id: ExportValidationCheckId) -> &'static str {
    match id {
        ExportValidationCheckId::JsonFormat => "JSON形式",
        ExportValidationCheckId::SchemaVersion => "schema_version",
        ExportValidationCheckId::Trip => "trip",
        ExportValidationCheckId::ItineraryItems => "itinerary_items",
        ExportValidationCheckId::ChecklistItems => "checklist_items",
        ExportValidationCheckId::Notes => "notes",
        ExportValidationCheckId::Expenses => "expenses",
        ExportValidationCheckId::Reservations => "reservations",
        ExportValidationCheckId::Participants => "participants",
    }
}

pub(crate) fn flatten_reservations_from_v3(export: &TripExportV3) -> Vec<ExportReservation> {
    export
        .days
        .iter()
        .flat_map(|day| {
            day.itineraries.iter().map(|it| {
                (
                    ItineraryNoteKey {
                        day_number: day.day_number,
                        sort_order: it.sort_order,
                        start_time: it.start_time.clone(),
                        title: it.title.clone(),
                    },
                    &it.reservations,
                )
            })
        })
        .flat_map(|(itinerary_key, reservations)| {
            reservations
                .iter()
                .cloned()
                .map(move |reservation| ExportReservation {
                    itinerary_key: itinerary_key.clone(),
                    reservation,
                })
        })
        .collect()
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
        println!("  Notes        : {} 件", report.note_count);
        println!("  Participants : {} 件", report.participant_count);
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
    Ok(import_trip_from_json_with_summary(conn, json)?.trip_id)
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
        export.trip.summary.as_deref(),
    )?;

    for day_summary in &export.day_summaries {
        if let Some(text) = crate::summary::normalize_day_summary(day_summary.summary.as_deref())? {
            crate::day::set_day_summary(conn, new_trip_id, day_summary.day_number, Some(text))?;
        }
    }

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

    let day_count = crate::day::validate_trip_date_range(
        export.trip.start_date.as_deref().expect("validated above"),
        export.trip.end_date.as_deref().expect("validated above"),
    )?;
    if !export.notes().is_empty() {
        crate::note::import_export_notes(conn, new_trip_id, export.notes(), day_count)?;
    }

    Ok(new_trip_id)
}

pub(crate) fn import_trip_from_export_v3(conn: &Connection, export: &TripExportV3) -> Result<i64> {
    validate_trip_export_v3(export)?;

    let new_trip_id = add_trip(
        conn,
        &export.trip.name,
        export.trip.start_date.as_deref().expect("validated above"),
        export.trip.end_date.as_deref().expect("validated above"),
        export.trip.summary.as_deref(),
    )?;

    if !export.participants().is_empty() {
        crate::participant::import_export_participants(conn, new_trip_id, export.participants())?;
    }

    for day in &export.days {
        if let Some(text) = crate::summary::normalize_day_summary(day.summary.as_deref())? {
            crate::day::set_day_summary(conn, new_trip_id, day.day_number, Some(text))?;
        }
    }

    // Itinerary → Expense の順で復元する（id は新規採番）
    for day in &export.days {
        for it in &day.itineraries {
            let itinerary_id = crate::itinerary::add_itinerary_item(
                conn,
                new_trip_id,
                day.day_number,
                &it.title,
                it.note.as_deref(),
                it.start_time.as_deref(),
                Some(it.sort_order),
                it.duration_minutes,
                it.travel_minutes,
                it.location.as_deref(),
                it.category,
            )?;
            for exp in &it.expenses {
                crate::expense::import_expense_v3(conn, itinerary_id, exp)?;
            }
            for res in &it.reservations {
                crate::reservation::import_reservation_v3(conn, itinerary_id, res)?;
            }
        }
    }

    for item in export.checklist_items() {
        crate::checklist::import_checklist_item(
            conn,
            new_trip_id,
            &item.title,
            item.is_done,
            item.sort_order,
        )?;
    }

    let day_count = crate::day::validate_trip_date_range(
        export.trip.start_date.as_deref().expect("validated above"),
        export.trip.end_date.as_deref().expect("validated above"),
    )?;
    if !export.notes().is_empty() {
        crate::note::import_export_notes(conn, new_trip_id, export.notes(), day_count)?;
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
        note_count: export.notes().len(),
        participant_count: export.participants().len(),
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
    println!("  Note           : {} 件", summary.note_count);
    println!("  Participant    : {} 件", summary.participant_count);
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

enum ParsedTripExportForImport {
    V1V2(TripExport),
    V3(TripExportV3),
}

fn parse_trip_export_for_import(
    json: &str,
) -> Result<(
    ParsedTripExportForImport,
    bool,
    TripExportMetadata,
    Option<i32>,
)> {
    let root: serde_json::Value = serde_json::from_str(json).context("JSON の形式が不正です")?;
    let schema_version_present = root.get("schema_version").is_some();
    let schema_version = root
        .get("schema_version")
        .and_then(|v| v.as_i64())
        .map(|v| v as i32);
    let effective_schema = effective_export_schema_version(schema_version);

    match effective_schema {
        TRIP_EXPORT_SCHEMA_VERSION_V3 | TRIP_EXPORT_SCHEMA_VERSION => {
            let export: TripExportV3 =
                serde_json::from_value(root.clone()).context("JSON の形式が不正です")?;
            // metadata は v2 構造体依存のため、必要最低限だけ拾う
            let export_metadata = TripExportMetadata {
                generator_present: root.get("generator").is_some(),
                generator: export.generator.clone(),
                generator_version_present: root.get("generator_version").is_some(),
                generator_version: export.generator_version.clone(),
                exported_at_present: root.get("exported_at").is_some(),
                exported_at: export.exported_at.clone(),
            };
            Ok((
                ParsedTripExportForImport::V3(export),
                schema_version_present,
                export_metadata,
                schema_version,
            ))
        }
        _ => {
            let export: TripExport =
                serde_json::from_value(root.clone()).context("JSON の形式が不正です")?;
            let export_metadata = TripExportMetadata::from_parsed(&root, &export);
            Ok((
                ParsedTripExportForImport::V1V2(export),
                schema_version_present,
                export_metadata,
                schema_version,
            ))
        }
    }
}

/// JSON 文字列から旅行をインポートし、サマリーを返す
pub(crate) fn import_trip_from_json_with_summary(
    conn: &Connection,
    json: &str,
) -> Result<TripImportSummary> {
    let (parsed, schema_version_present, export_metadata, export_schema_version) =
        parse_trip_export_for_import(json)?;
    match parsed {
        ParsedTripExportForImport::V1V2(export) => {
            let new_trip_id = import_trip_from_export(conn, &export)?;
            Ok(trip_import_summary_from_export(
                new_trip_id,
                &export,
                schema_version_present,
                export_metadata,
            ))
        }
        ParsedTripExportForImport::V3(export) => {
            let new_trip_id = import_trip_from_export_v3(conn, &export)?;
            Ok(TripImportSummary {
                trip_id: new_trip_id,
                trip_name: export.trip.name.clone(),
                itinerary_count: export
                    .days
                    .iter()
                    .map(|d| d.itineraries.len())
                    .sum::<usize>(),
                checklist_count: export.checklist_items().len(),
                note_count: export.notes().len(),
                participant_count: export.participants().len(),
                schema_version_present,
                export_schema_version,
                export_metadata,
            })
        }
    }
}

/// JSON ファイルから旅行をインポートし、サマリーを表示する
pub(crate) fn run_trip_import(conn: &Connection, path: &str) -> Result<()> {
    let json = std::fs::read_to_string(path)
        .with_context(|| format!("ファイル '{path}' を読み込めませんでした"))?;
    let summary = import_trip_from_json_with_summary(conn, &json)?;
    print_trip_import_summary(&summary);
    Ok(())
}

/// 旅行を複製する（Trip / Itinerary / Checklist / Note / Expense を新しい ID でコピー）
pub(crate) fn duplicate_trip(conn: &Connection, trip_id: i64, name: Option<&str>) -> Result<i64> {
    let source = get_trip(conn, trip_id)?;
    let mut export = build_trip_export_v3(conn, trip_id)?;
    export.trip.name = match name {
        Some(value) => value.to_string(),
        None => format!("{} (Copy)", source.name),
    };
    import_trip_from_export_v3(conn, &export)
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
    let root: serde_json::Value = serde_json::from_str(&json).context("JSON の形式が不正です")?;
    let schema_version = root
        .get("schema_version")
        .and_then(|v| v.as_i64())
        .map(|v| v as i32);
    if is_v3_or_later_export_schema(schema_version) {
        let export: TripExportV3 = serde_json::from_value(root).context("JSON の形式が不正です")?;
        // v3 から v2 相当へ flatten（trip diff 等の既存処理のため）
        let itinerary_items = export
            .days
            .iter()
            .flat_map(|day| {
                day.itineraries
                    .iter()
                    .map(move |it| crate::models::ItineraryItem {
                        id: 0,
                        trip_id: 0,
                        day: day.day_number,
                        title: it.title.clone(),
                        note: it.note.clone(),
                        start_time: it.start_time.clone(),
                        sort_order: it.sort_order,
                        duration_minutes: it.duration_minutes,
                        travel_minutes: it.travel_minutes,
                        location: it.location.clone(),
                        category: it.category,
                        created_at: String::new(),
                        updated_at: String::new(),
                    })
            })
            .collect::<Vec<_>>();
        let day_summaries = export
            .days
            .iter()
            .filter(|day| day.summary.is_some())
            .map(|day| crate::models::ExportDaySummary {
                day_number: day.day_number,
                summary: day.summary.clone(),
            })
            .collect::<Vec<_>>();
        let reservations = flatten_reservations_from_v3(&export);
        let participants = export.participants().to_vec();
        Ok(TripExport {
            schema_version: export.schema_version,
            generator: export.generator,
            generator_version: export.generator_version,
            exported_at: export.exported_at,
            trip: export.trip,
            itinerary_items,
            checklist_items: export.checklist_items,
            notes: export.notes,
            day_summaries,
            reservations,
            participants,
        })
    } else {
        serde_json::from_str(&json).context("JSON の形式が不正です")
    }
}

/// 旅行を削除する
pub(crate) fn delete_trip(conn: &Connection, id: i64) -> Result<()> {
    get_trip(conn, id)?;
    crate::db::with_transaction(conn, "trip delete", |tx| {
        crate::participant::delete_participants_for_trip(tx, id)?;
        crate::note::delete_notes_for_trip(tx, id)?;
        crate::reservation::delete_reservations_for_trip(tx, id)?;
        crate::expense::delete_expenses_for_trip(tx, id)?;
        tx.execute("DELETE FROM trips WHERE id = ?1", params![id])
            .context("旅行の削除に失敗しました")?;
        Ok(())
    })
}

/// rusqlite の行データを Trip 構造体に変換する
pub(crate) fn row_to_trip(row: &rusqlite::Row) -> rusqlite::Result<Trip> {
    Ok(Trip {
        id: row.get(0)?,
        name: row.get(1)?,
        start_date: row.get(2)?,
        end_date: row.get(3)?,
        summary: row.get(4)?,
        created_at: row.get(5)?,
        updated_at: row.get(6)?,
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
    if let Some(summary) = &trip.summary {
        println!("概要      :");
        for line in summary.lines() {
            println!("            {line}");
        }
    }
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
        let id = add_trip(&conn, "沖縄旅行", "2025-06-01", "2025-06-05", None).unwrap();

        assert_eq!(id, 1);
        let trip = get_trip(&conn, id).unwrap();
        assert_eq!(trip.name, "沖縄旅行");
        assert_eq!(trip.start_date.as_deref(), Some("2025-06-01"));
        assert_eq!(trip.end_date.as_deref(), Some("2025-06-05"));
    }

    #[test]
    fn test_add_trip_creates_days() {
        let conn = test_db();
        let trip_id = add_trip(&conn, "Day Trip", "2026-12-01", "2026-12-03", None).unwrap();
        let days = crate::day::list_days(&conn, trip_id).unwrap();
        assert_eq!(days.len(), 3);
        assert_eq!(days[0].day_number, 1);
        assert_eq!(days[2].day_number, 3);
    }

    #[test]
    fn test_add_trip_rejects_invalid_date_range() {
        let conn = test_db();
        assert!(add_trip(&conn, "Bad Trip", "2026-12-04", "2026-12-01", None).is_err());
    }

    #[test]
    fn test_update_trip_syncs_days_on_end_extension() {
        let conn = test_db();
        let id = add_trip(&conn, "Extend Trip", "2026-12-01", "2026-12-02", None).unwrap();
        assert_eq!(crate::day::list_days(&conn, id).unwrap().len(), 2);
        update_trip(&conn, id, None, None, Some("2026-12-04"), None, false).unwrap();
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
        let trip_id = add_trip(&conn, "沖縄旅行", "2026-04-26", "2026-04-29", None).unwrap();
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
            let day_id: i64 = conn
                .query_row(
                    "SELECT day_id FROM itinerary_items WHERE id = ?1",
                    rusqlite::params![item.id],
                    |row| row.get(0),
                )
                .unwrap();
            let expected_day_id =
                crate::day::find_day_id_by_trip_and_day_number(&conn, new_id, item.day).unwrap();
            assert_eq!(day_id, expected_day_id);
        }
        assert_ne!(new_id, trip_id);
    }

    #[test]
    fn test_get_trip() {
        let conn = test_db();
        let id = add_trip(&conn, "北海道旅行", "2025-08-01", "2025-08-10", None).unwrap();

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
        add_trip(&conn, "沖縄旅行", "2025-06-01", "2025-06-05", None).unwrap();
        add_trip(&conn, "京都旅行", "2025-07-01", "2025-07-03", None).unwrap();

        let trips = list_trips(&conn).unwrap();
        assert_eq!(trips.len(), 2);
        assert_eq!(trips[0].name, "沖縄旅行");
        assert_eq!(trips[1].name, "京都旅行");
    }

    #[test]
    fn test_trip_export_contains_trip_and_items() {
        let conn = test_db();
        let trip_id = add_trip(&conn, "沖縄旅行", "2026-04-26", "2026-04-29", None).unwrap();
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
    fn test_trip_export_items_sorted_by_day_and_sort_order() {
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
        assert_eq!(export.itinerary_items[0].title, "国際通り");
        assert_eq!(export.itinerary_items[1].title, "首里城");
    }

    #[test]
    fn test_itinerary_ordering_consistent_across_list_export_md_and_v3() {
        let conn = test_db();
        let trip_id = add_test_trip(&conn, "Ordering Trip").unwrap();

        add_itinerary_item(
            &conn,
            trip_id,
            1,
            "Early time late order",
            None,
            Some("08:00"),
            Some(30),
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
            "Middle no time",
            None,
            None,
            Some(10),
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
            "Late time early order",
            None,
            Some("18:00"),
            Some(5),
            None,
            None,
            None,
            None,
        )
        .unwrap();

        let expected = vec![
            "Late time early order".to_string(),
            "Middle no time".to_string(),
            "Early time late order".to_string(),
        ];

        let list_titles: Vec<_> = crate::itinerary::list_itinerary_items(&conn, trip_id)
            .unwrap()
            .into_iter()
            .map(|i| i.title)
            .collect();
        assert_eq!(list_titles, expected);

        let v3_titles: Vec<_> = build_trip_export_v3(&conn, trip_id)
            .unwrap()
            .days
            .into_iter()
            .flat_map(|d| d.itineraries.into_iter().map(|i| i.title))
            .collect();
        assert_eq!(v3_titles, expected);

        let md = crate::markdown::generate_trip_markdown(&conn, trip_id).unwrap();
        let pos_late = md.find("### 18:00 Late time early order").unwrap();
        let pos_middle = md.find("### Middle no time").unwrap();
        let pos_early = md.find("### 08:00 Early time late order").unwrap();
        assert!(pos_late < pos_middle);
        assert!(pos_middle < pos_early);
    }

    #[test]
    fn test_import_preserves_sort_order_sequence() {
        let conn = test_db();
        let trip_id = add_test_trip(&conn, "Import Order Trip").unwrap();

        add_itinerary_item(
            &conn,
            trip_id,
            1,
            "Third",
            None,
            Some("12:00"),
            Some(30),
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
            "First",
            None,
            None,
            Some(10),
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
            "Second",
            None,
            Some("09:00"),
            Some(20),
            None,
            None,
            None,
            None,
        )
        .unwrap();

        let json = export_trip_to_json(&conn, trip_id).unwrap();
        let new_id = import_trip_from_json(&conn, &json).unwrap();

        let imported_titles: Vec<_> = crate::itinerary::list_itinerary_items(&conn, new_id)
            .unwrap()
            .into_iter()
            .map(|i| i.title)
            .collect();
        assert_eq!(
            imported_titles,
            vec!["First", "Second", "Third"],
            "import 後も sort_order 順が維持されること"
        );
    }

    #[test]
    fn test_trip_export_to_json_string() {
        let conn = test_db();
        let trip_id = add_test_trip(&conn, "沖縄旅行").unwrap();

        let json = export_trip_to_json(&conn, trip_id).unwrap();
        assert!(json.contains("\"trip\""));
        assert!(json.contains("\"days\""));
        assert!(json.contains("\"name\": \"沖縄旅行\""));

        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["trip"]["name"], "沖縄旅行");
        assert!(parsed["days"].is_array());
    }

    #[test]
    fn test_print_json_empty_trip_list() {
        let json = serde_json::to_string_pretty(&Vec::<Trip>::new()).unwrap();
        assert_eq!(json, "[]");
    }

    #[test]
    fn test_print_json_trip_list() {
        let conn = test_db();
        add_trip(&conn, "沖縄旅行", "2025-06-01", "2025-06-05", None).unwrap();
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
        let id = add_trip(&conn, "北海道旅行", "2025-08-01", "2025-08-10", None).unwrap();

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
            None,
            false,
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
        notes: Vec<crate::models::ExportNote>,
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
            notes: export.notes().to_vec(),
        }
    }

    #[test]
    fn test_export_import_full_roundtrip_with_notes() {
        use crate::note::{add_note, ResolvedNoteOwner};

        let conn = test_db();
        let trip_id = add_trip(
            &conn,
            "Note Roundtrip Trip",
            "2026-06-01",
            "2026-06-03",
            None,
        )
        .unwrap();
        let itinerary_id = add_itinerary_item(
            &conn,
            trip_id,
            2,
            "美ら海水族館",
            None,
            Some("09:00"),
            Some(3),
            None,
            None,
            None,
            None,
        )
        .unwrap();
        let day2_id = crate::day::find_day_id_by_trip_and_day_number(&conn, trip_id, 2).unwrap();

        add_note(
            &conn,
            ResolvedNoteOwner::Trip(trip_id),
            Some("全体メモ"),
            "trip note body",
        )
        .unwrap();
        add_note(
            &conn,
            ResolvedNoteOwner::Day(day2_id),
            Some("2日目メモ"),
            "day note body",
        )
        .unwrap();
        add_note(
            &conn,
            ResolvedNoteOwner::Itinerary(itinerary_id),
            Some("水族館メモ"),
            "itinerary note body",
        )
        .unwrap();

        let before = build_trip_export(&conn, trip_id).unwrap();
        assert_eq!(before.notes().len(), 3);

        let json = export_trip_to_json(&conn, trip_id).unwrap();
        reset_db(&conn).unwrap();

        let new_id = import_trip_from_json(&conn, &json).unwrap();
        let after = build_trip_export(&conn, new_id).unwrap();

        assert_eq!(
            comparable_trip_export(&before),
            comparable_trip_export(&after)
        );
    }

    #[test]
    fn test_import_v1_export_without_notes_still_works() {
        let conn = test_db();
        let json = r#"{
            "schema_version": 1,
            "trip": {
                "id": 1,
                "name": "V1 Trip",
                "start_date": "2026-06-01",
                "end_date": "2026-06-03",
                "created_at": "2026-01-01 00:00:00",
                "updated_at": "2026-01-01 00:00:00"
            },
            "itinerary_items": [],
            "checklist_items": []
        }"#;

        let new_id = import_trip_from_json(&conn, json).unwrap();
        let imported = build_trip_export(&conn, new_id).unwrap();
        assert_eq!(imported.trip.name, "V1 Trip");
        assert!(imported.notes().is_empty());
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
            None,
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
        let parsed_before: TripExportV3 = serde_json::from_str(&json).unwrap();
        let parsed_after: TripExportV3 = serde_json::from_str(&re_json).unwrap();
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
    fn test_export_schema_version_is_four() {
        let conn = test_db();
        let trip_id = add_test_trip(&conn, "Metadata Trip").unwrap();

        let json = export_trip_to_json(&conn, trip_id).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["schema_version"], 4);
        assert!(parsed.get("notes").is_some());
        assert!(parsed.get("days").is_some());
        assert!(parsed.get("participants").is_some());
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
        assert_eq!(parsed["schema_version"], 4);
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
        let exported_v3: TripExportV3 = serde_json::from_str(&json).unwrap();

        assert_eq!(
            exported_v3.generator.as_deref(),
            Some(TRIP_EXPORT_GENERATOR)
        );

        // v3 を v2 相当へ flatten して比較（generator metadata を無視できること）
        let tmp = std::env::temp_dir().join("caglla-cli-test-export-v3.json");
        std::fs::write(&tmp, &json).unwrap();
        let exported = load_trip_export_from_file(tmp.to_str().unwrap()).unwrap();
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
        let trip_id = add_trip(&conn, "Okinawa Trip", "2026-06-01", "2026-06-03", None).unwrap();
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
    fn test_duplicate_trip_copies_expenses() {
        let conn = test_db();
        let trip_id = add_trip(&conn, "Expense Trip", "2026-06-01", "2026-06-03", None).unwrap();
        let itinerary_id = add_itinerary_item(
            &conn,
            trip_id,
            1,
            "Aquarium",
            None,
            Some("09:00"),
            Some(0),
            Some(120),
            None,
            Some("Motobu"),
            None,
        )
        .unwrap();
        crate::expense::add_expense(
            &conn,
            itinerary_id,
            "2500",
            "JPY",
            Some("入館料"),
            None,
            None,
            None,
        )
        .unwrap();
        crate::expense::add_expense(
            &conn,
            itinerary_id,
            "500",
            "JPY",
            Some("駐車場"),
            None,
            None,
            None,
        )
        .unwrap();

        let before = build_trip_export_v3(&conn, trip_id).unwrap();
        let new_id = duplicate_trip(&conn, trip_id, None).unwrap();
        let duplicated = build_trip_export_v3(&conn, new_id).unwrap();

        assert_eq!(duplicated.trip.name, "Expense Trip (Copy)");
        assert_eq!(before.days, duplicated.days);
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
        let err = duplicate_trip(&conn, 9999, None).expect_err("expected error");
        assert_eq!(err.to_string(), "Trip not found: 9999");
    }

    #[test]
    fn test_export_import_reexport_structural_roundtrip_with_checklist() {
        let conn = test_db();
        let trip_id = add_trip(&conn, "Roundtrip Trip", "2026-07-01", "2026-07-05", None).unwrap();
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
        let parsed_before: TripExportV3 = serde_json::from_str(&json).unwrap();
        let parsed_after: TripExportV3 = serde_json::from_str(&re_json).unwrap();
        assert_eq!(
            checklist_sem(parsed_before.checklist_items()),
            checklist_sem(parsed_after.checklist_items())
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
            ExportValidationCheckId::ChecklistItems
        ));
        assert!(check_passed(
            &report,
            ExportValidationCheckId::SchemaVersion
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

        assert!(!report.valid);
        assert!(!check_passed(
            &report,
            ExportValidationCheckId::SchemaVersion
        ));
        assert!(report
            .errors
            .iter()
            .any(|e| e.contains("サポートされていません")));
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
