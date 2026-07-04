use anyhow::{Context, Result};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

use crate::domain::models::{ExportReservationV3, Reservation};

pub(crate) const RESERVATION_TYPES: &[&str] = &[
    "hotel",
    "flight",
    "restaurant",
    "rental_car",
    "activity",
    "parking",
    "ticket",
    "other",
];

const RESERVATION_SELECT_SQL: &str = "
    SELECT id, itinerary_id, reservation_type, provider_name, confirmation_code,
           reservation_site_url, remark, start_at, end_at, created_at, updated_at
    FROM reservations";

fn normalize_optional_text(value: Option<&str>) -> Option<String> {
    value.and_then(|s| {
        let trimmed = s.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    })
}

pub(crate) fn validate_reservation_type(value: &str) -> Result<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        anyhow::bail!("reservation_type は必須です");
    }
    if !RESERVATION_TYPES.contains(&trimmed) {
        anyhow::bail!(
            "不正な reservation_type です: {trimmed}. 有効な値: {}",
            RESERVATION_TYPES.join(", ")
        );
    }
    Ok(trimmed.to_string())
}

pub(crate) fn validate_provider_name(value: &str) -> Result<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        anyhow::bail!("provider_name は必須です");
    }
    Ok(trimmed.to_string())
}

pub(crate) fn validate_export_reservation_v3(res: &ExportReservationV3) -> Result<()> {
    validate_reservation_type(&res.reservation_type)?;
    validate_provider_name(&res.provider_name)?;
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ReservationWithContext {
    pub reservation: Reservation,
    pub day_number: i64,
    pub itinerary_title: String,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct ReservationListJson {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trip_id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub itinerary_id: Option<i64>,
    pub reservations: Vec<Reservation>,
}

pub(crate) fn resolve_reservation_list_target(
    trip: Option<i64>,
    itinerary: Option<i64>,
) -> Result<ReservationListTarget> {
    match (trip, itinerary) {
        (Some(trip_id), None) => Ok(ReservationListTarget::Trip(trip_id)),
        (None, Some(itinerary_id)) => Ok(ReservationListTarget::Itinerary(itinerary_id)),
        (Some(_), Some(_)) => {
            anyhow::bail!("--trip と --itinerary は同時に指定できません");
        }
        (None, None) => {
            anyhow::bail!("--trip または --itinerary のいずれかを指定してください");
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ReservationListTarget {
    Trip(i64),
    Itinerary(i64),
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn add_reservation(
    conn: &Connection,
    itinerary_id: i64,
    reservation_type: &str,
    provider_name: &str,
    confirmation_code: Option<&str>,
    reservation_site_url: Option<&str>,
    remark: Option<&str>,
    start_at: Option<&str>,
    end_at: Option<&str>,
) -> Result<i64> {
    crate::itinerary::get_itinerary_item(conn, itinerary_id)?;
    let reservation_type = validate_reservation_type(reservation_type)?;
    let provider_name = validate_provider_name(provider_name)?;

    let now = crate::storage::db::now_string();
    conn.execute(
        "INSERT INTO reservations
         (itinerary_id, reservation_type, provider_name, confirmation_code,
          reservation_site_url, remark, start_at, end_at, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        params![
            itinerary_id,
            reservation_type,
            provider_name,
            normalize_optional_text(confirmation_code),
            normalize_optional_text(reservation_site_url),
            normalize_optional_text(remark),
            normalize_optional_text(start_at),
            normalize_optional_text(end_at),
            &now,
            &now,
        ],
    )
    .context("Reservation の追加に失敗しました")?;
    Ok(conn.last_insert_rowid())
}

pub(crate) fn import_reservation_v3(
    conn: &Connection,
    itinerary_id: i64,
    export: &ExportReservationV3,
) -> Result<i64> {
    validate_export_reservation_v3(export)?;
    crate::itinerary::get_itinerary_item(conn, itinerary_id)?;

    let now = crate::storage::db::now_string();
    conn.execute(
        "INSERT INTO reservations
         (itinerary_id, reservation_type, provider_name, confirmation_code,
          reservation_site_url, remark, start_at, end_at, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        params![
            itinerary_id,
            export.reservation_type,
            export.provider_name,
            export.confirmation_code,
            export.reservation_site_url,
            export.remark,
            export.start_at,
            export.end_at,
            &now,
            &now,
        ],
    )
    .context("Reservation の import に失敗しました")?;
    Ok(conn.last_insert_rowid())
}

pub(crate) fn list_reservations_for_itinerary(
    conn: &Connection,
    itinerary_id: i64,
) -> Result<Vec<Reservation>> {
    crate::itinerary::get_itinerary_item(conn, itinerary_id)?;
    list_reservations_where(conn, "itinerary_id = ?1", params![itinerary_id])
}

pub(crate) fn list_reservations_for_trip(
    conn: &Connection,
    trip_id: i64,
) -> Result<Vec<ReservationWithContext>> {
    crate::trip::get_trip(conn, trip_id)?;
    let sql = "
        SELECT r.id, r.itinerary_id, r.reservation_type, r.provider_name, r.confirmation_code,
               r.reservation_site_url, r.remark, r.start_at, r.end_at, r.created_at, r.updated_at,
               i.day, i.title
        FROM reservations r
        JOIN itinerary_items i ON r.itinerary_id = i.id
        WHERE i.trip_id = ?1
        ORDER BY i.day ASC, i.sort_order ASC, i.id ASC, r.id ASC";
    let mut stmt = conn
        .prepare(sql)
        .context("Reservation 一覧取得の準備に失敗しました")?;
    let rows = stmt
        .query_map(params![trip_id], |row| {
            Ok(ReservationWithContext {
                reservation: Reservation {
                    id: row.get(0)?,
                    itinerary_id: row.get(1)?,
                    reservation_type: row.get(2)?,
                    provider_name: row.get(3)?,
                    confirmation_code: row.get(4)?,
                    reservation_site_url: row.get(5)?,
                    remark: row.get(6)?,
                    start_at: row.get(7)?,
                    end_at: row.get(8)?,
                    created_at: row.get(9)?,
                    updated_at: row.get(10)?,
                },
                day_number: row.get(11)?,
                itinerary_title: row.get(12)?,
            })
        })
        .context("Reservation 一覧取得に失敗しました")?
        .collect::<std::result::Result<Vec<_>, _>>()
        .context("Reservation 一覧の読み込みに失敗しました")?;
    Ok(rows)
}

fn list_reservations_where<P: rusqlite::Params>(
    conn: &Connection,
    where_clause: &str,
    params: P,
) -> Result<Vec<Reservation>> {
    let sql = format!(
        "{RESERVATION_SELECT_SQL}
         WHERE {where_clause}
         ORDER BY id ASC"
    );
    let mut stmt = conn
        .prepare(&sql)
        .context("Reservation 一覧取得の準備に失敗しました")?;
    let reservations = stmt
        .query_map(params, row_to_reservation)
        .context("Reservation 一覧取得に失敗しました")?
        .collect::<std::result::Result<Vec<_>, _>>()
        .context("Reservation 一覧の読み込みに失敗しました")?;
    Ok(reservations)
}

pub(crate) fn get_reservation(conn: &Connection, id: i64) -> Result<Reservation> {
    crate::storage::db::map_query_row(
        conn.query_row(
            &format!("{RESERVATION_SELECT_SQL} WHERE id = ?1"),
            params![id],
            row_to_reservation,
        ),
        || anyhow::anyhow!("Reservation not found: {id}"),
    )
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn update_reservation(
    conn: &Connection,
    id: i64,
    reservation_type: Option<&str>,
    provider_name: Option<&str>,
    confirmation_code: Option<Option<&str>>,
    reservation_site_url: Option<Option<&str>>,
    remark: Option<Option<&str>>,
    start_at: Option<Option<&str>>,
    end_at: Option<Option<&str>>,
) -> Result<()> {
    if reservation_type.is_none()
        && provider_name.is_none()
        && confirmation_code.is_none()
        && reservation_site_url.is_none()
        && remark.is_none()
        && start_at.is_none()
        && end_at.is_none()
    {
        anyhow::bail!(
            "更新する項目を1つ以上指定してください (--type, --provider, --confirmation, --site-url, --remark, --start-at, --end-at)"
        );
    }

    let mut reservation = get_reservation(conn, id)?;
    if let Some(value) = reservation_type {
        reservation.reservation_type = validate_reservation_type(value)?;
    }
    if let Some(value) = provider_name {
        reservation.provider_name = validate_provider_name(value)?;
    }
    if let Some(value) = confirmation_code {
        reservation.confirmation_code = normalize_optional_text(value);
    }
    if let Some(value) = reservation_site_url {
        reservation.reservation_site_url = normalize_optional_text(value);
    }
    if let Some(value) = remark {
        reservation.remark = normalize_optional_text(value);
    }
    if let Some(value) = start_at {
        reservation.start_at = normalize_optional_text(value);
    }
    if let Some(value) = end_at {
        reservation.end_at = normalize_optional_text(value);
    }

    let now = crate::storage::db::now_string();
    conn.execute(
        "UPDATE reservations
         SET reservation_type = ?1, provider_name = ?2, confirmation_code = ?3,
             reservation_site_url = ?4, remark = ?5, start_at = ?6, end_at = ?7,
             updated_at = ?8
         WHERE id = ?9",
        params![
            reservation.reservation_type,
            reservation.provider_name,
            reservation.confirmation_code,
            reservation.reservation_site_url,
            reservation.remark,
            reservation.start_at,
            reservation.end_at,
            &now,
            id,
        ],
    )
    .context("Reservation の更新に失敗しました")?;
    Ok(())
}

pub(crate) fn delete_reservation(conn: &Connection, id: i64) -> Result<()> {
    get_reservation(conn, id)?;
    conn.execute("DELETE FROM reservations WHERE id = ?1", params![id])
        .context("Reservation の削除に失敗しました")?;
    Ok(())
}

pub(crate) fn delete_reservations_for_itinerary(
    conn: &Connection,
    itinerary_id: i64,
) -> Result<()> {
    conn.execute(
        "DELETE FROM reservations WHERE itinerary_id = ?1",
        params![itinerary_id],
    )
    .context("Itinerary 配下 Reservation の削除に失敗しました")?;
    Ok(())
}

pub(crate) fn delete_reservations_for_trip(conn: &Connection, trip_id: i64) -> Result<()> {
    conn.execute(
        "DELETE FROM reservations
         WHERE itinerary_id IN (SELECT id FROM itinerary_items WHERE trip_id = ?1)",
        params![trip_id],
    )
    .context("Trip 配下 Reservation の削除に失敗しました")?;
    Ok(())
}

fn row_to_reservation(row: &rusqlite::Row) -> rusqlite::Result<Reservation> {
    Ok(Reservation {
        id: row.get(0)?,
        itinerary_id: row.get(1)?,
        reservation_type: row.get(2)?,
        provider_name: row.get(3)?,
        confirmation_code: row.get(4)?,
        reservation_site_url: row.get(5)?,
        remark: row.get(6)?,
        start_at: row.get(7)?,
        end_at: row.get(8)?,
        created_at: row.get(9)?,
        updated_at: row.get(10)?,
    })
}

pub(crate) fn fmt_optional_text(value: &Option<String>) -> &str {
    value.as_deref().unwrap_or("-")
}

pub(crate) fn format_period(start_at: &Option<String>, end_at: &Option<String>) -> String {
    match (start_at.as_deref(), end_at.as_deref()) {
        (Some(start), Some(end)) => format!("{start} — {end}"),
        (Some(start), None) => start.to_string(),
        (None, Some(end)) => end.to_string(),
        (None, None) => "-".to_string(),
    }
}

pub(crate) fn print_reservation_list(
    target: ReservationListTarget,
    reservations: &[Reservation],
    trip_context: Option<&[ReservationWithContext]>,
) {
    let label = match target {
        ReservationListTarget::Trip(id) => format!("Trip {id}"),
        ReservationListTarget::Itinerary(id) => format!("Itinerary {id}"),
    };
    println!("{label} の Reservation ({} 件):", reservations.len());
    if reservations.is_empty() {
        println!("  （なし）");
        return;
    }

    if let Some(context_rows) = trip_context {
        for row in context_rows {
            let res = &row.reservation;
            println!(
                "Day {} / Itinerary {} {}",
                row.day_number, res.itinerary_id, row.itinerary_title
            );
            println!(
                "  [{}] {}  {}  {}",
                res.id,
                res.reservation_type,
                res.provider_name,
                fmt_optional_text(&res.confirmation_code)
            );
            let period = format_period(&res.start_at, &res.end_at);
            if period != "-" {
                println!("      {period}");
            }
        }
        return;
    }

    println!(
        "{:<4} {:<6} {:<12} {:<28} {:<14}",
        "ID", "Itin.", "Type", "Provider", "Confirmation"
    );
    for res in reservations {
        println!(
            "{:<4} {:<6} {:<12} {:<28} {:<14}",
            res.id,
            res.itinerary_id,
            res.reservation_type,
            res.provider_name,
            fmt_optional_text(&res.confirmation_code),
        );
    }
}

/// Loads Day / Itinerary display context for human reservation detail (read show + write add/update).
pub(crate) fn load_reservation_display_context(
    conn: &Connection,
    itinerary_id: i64,
) -> (Option<i64>, Option<String>) {
    let itinerary = crate::itinerary::get_itinerary_item(conn, itinerary_id)
        .ok()
        .map(|item| (item.day, item.title));
    itinerary
        .map(|(day, title)| (Some(day), Some(title)))
        .unwrap_or((None, None))
}

pub(crate) fn print_reservation_detail_with_context(
    reservation: &Reservation,
    day_number: Option<i64>,
    itinerary_title: Option<&str>,
) {
    println!("Reservation ID : {}", reservation.id);
    println!("Itinerary ID   : {}", reservation.itinerary_id);
    if let (Some(day), Some(title)) = (day_number, itinerary_title) {
        println!("Itinerary      : Day {day} / {title}");
    }
    println!("Type           : {}", reservation.reservation_type);
    println!("Provider       : {}", reservation.provider_name);
    println!(
        "Confirmation   : {}",
        fmt_optional_text(&reservation.confirmation_code)
    );
    println!(
        "Site URL       : {}",
        fmt_optional_text(&reservation.reservation_site_url)
    );
    println!(
        "Remark         : {}",
        fmt_optional_text(&reservation.remark)
    );
    println!(
        "Period         : {}",
        format_period(&reservation.start_at, &reservation.end_at)
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::itinerary::add_itinerary_item;
    use crate::storage::db::reset_db;
    use crate::trip::add_test_trip;
    use rusqlite::Connection;

    fn test_db() -> Connection {
        crate::storage::db::open_db_at(":memory:").expect("インメモリ DB")
    }

    fn setup_itinerary(conn: &Connection) -> i64 {
        let trip_id = add_test_trip(conn, "Reservation Trip").unwrap();
        add_itinerary_item(
            conn,
            trip_id,
            1,
            "Check-in",
            None,
            Some("16:40"),
            None,
            None,
            None,
            Some("Hilton Sesoko"),
            None,
        )
        .unwrap()
    }

    #[test]
    fn test_validate_reservation_type_rejects_unknown() {
        assert!(validate_reservation_type("hotel").is_ok());
        assert!(validate_reservation_type("").is_err());
        assert!(validate_reservation_type("cruise").is_err());
    }

    #[test]
    fn test_validate_provider_name_required() {
        assert!(validate_provider_name("Hilton").is_ok());
        assert!(validate_provider_name("  ").is_err());
    }

    #[test]
    fn test_add_list_show_update_delete_reservation() {
        let conn = test_db();
        let itinerary_id = setup_itinerary(&conn);

        let id = add_reservation(
            &conn,
            itinerary_id,
            "hotel",
            "Hilton Sesoko Resort",
            Some("ABC123"),
            Some("https://example.com/booking"),
            Some("Twin room"),
            Some("2026-04-26T16:40"),
            Some("2026-04-29T10:00"),
        )
        .unwrap();

        let listed = list_reservations_for_itinerary(&conn, itinerary_id).unwrap();
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].provider_name, "Hilton Sesoko Resort");

        update_reservation(
            &conn,
            id,
            None,
            None,
            Some(Some("XYZ789")),
            None,
            Some(Some("Updated remark")),
            None,
            None,
        )
        .unwrap();
        let updated = get_reservation(&conn, id).unwrap();
        assert_eq!(updated.confirmation_code.as_deref(), Some("XYZ789"));
        assert_eq!(updated.remark.as_deref(), Some("Updated remark"));

        delete_reservation(&conn, id).unwrap();
        assert!(get_reservation(&conn, id).is_err());
    }

    #[test]
    fn test_list_reservations_for_trip() {
        let conn = test_db();
        let itinerary_id = setup_itinerary(&conn);
        add_reservation(
            &conn,
            itinerary_id,
            "hotel",
            "Hilton Sesoko Resort",
            Some("ABC123"),
            None,
            None,
            None,
            None,
        )
        .unwrap();

        let rows = list_reservations_for_trip(&conn, 1).unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].day_number, 1);
        assert_eq!(rows[0].itinerary_title, "Check-in");
    }

    #[test]
    fn test_delete_reservations_for_itinerary_cascade() {
        let conn = test_db();
        let itinerary_id = setup_itinerary(&conn);
        add_reservation(
            &conn,
            itinerary_id,
            "hotel",
            "Hilton",
            None,
            None,
            None,
            None,
            None,
        )
        .unwrap();

        delete_reservations_for_itinerary(&conn, itinerary_id).unwrap();
        assert!(list_reservations_for_itinerary(&conn, itinerary_id)
            .unwrap()
            .is_empty());
    }

    #[test]
    fn test_itinerary_delete_cascades_reservations() {
        let conn = test_db();
        let itinerary_id = setup_itinerary(&conn);
        add_reservation(
            &conn,
            itinerary_id,
            "hotel",
            "Hilton",
            None,
            None,
            None,
            None,
            None,
        )
        .unwrap();

        crate::itinerary::delete_itinerary_item(&conn, itinerary_id).unwrap();
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM reservations", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn test_reset_db_clears_reservations() {
        let conn = test_db();
        let itinerary_id = setup_itinerary(&conn);
        add_reservation(
            &conn,
            itinerary_id,
            "hotel",
            "Hilton",
            None,
            None,
            None,
            None,
            None,
        )
        .unwrap();

        reset_db(&conn).unwrap();
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM reservations", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 0);
    }
}
