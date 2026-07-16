use anyhow::{Context, Result};

use crate::domain::models::{ItineraryCategory, ItineraryItem};
use rusqlite::{params, Connection};

pub(crate) const ITINERARY_ITEM_SELECT_SQL: &str = "
    SELECT i.id, i.trip_id, d.day_number, i.title, i.note, i.start_time, i.sort_order,
           i.duration_minutes, i.travel_minutes, i.location, i.category, i.created_at, i.updated_at
    FROM itinerary_items i
    INNER JOIN days d ON i.day_id = d.id";

/// Itinerary 一覧の Sequence-first 並び（Day → sort_order → id）
pub(crate) const ITINERARY_LIST_ORDER_BY: &str = "ORDER BY d.day_number, i.sort_order, i.id";

/// Day 内 sort_order の標準間隔（sparse ordering）
pub(crate) const SORT_ORDER_STEP: i64 = 1000;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ValidatedItineraryContent {
    pub title: String,
    pub note: Option<String>,
    pub start_time: Option<String>,
    pub location: Option<String>,
}

fn normalize_optional_text(value: Option<&str>) -> Option<String> {
    value.and_then(|raw| {
        let trimmed = raw.trim();
        (!trimmed.is_empty()).then(|| trimmed.to_string())
    })
}

pub(crate) fn validate_itinerary_content_fields(
    title: &str,
    note: Option<&str>,
    start_time: Option<&str>,
    location: Option<&str>,
) -> Result<ValidatedItineraryContent> {
    let title = title.trim();
    if title.is_empty() {
        anyhow::bail!("Itinerary title must not be empty");
    }

    let start_time = normalize_optional_text(start_time);
    if let Some(value) = start_time.as_deref() {
        parse_time_hhmm(value)?;
    }

    Ok(ValidatedItineraryContent {
        title: title.to_string(),
        note: normalize_optional_text(note),
        start_time,
        location: normalize_optional_text(location),
    })
}

pub(crate) fn resolve_itinerary_create_target(
    conn: &Connection,
    trip_id: i64,
    day_number: i64,
) -> Result<i64> {
    crate::trip::get_trip(conn, trip_id)?;
    crate::day::find_day_id_by_trip_and_day_number(conn, trip_id, day_number)
}

/// 新しい日程を追加する
#[allow(clippy::too_many_arguments)]
pub(crate) fn add_itinerary_item(
    conn: &Connection,
    trip_id: i64,
    day: i64,
    title: &str,
    note: Option<&str>,
    start_time: Option<&str>,
    sort_order: Option<i64>,
    duration_minutes: Option<i64>,
    travel_minutes: Option<i64>,
    location: Option<&str>,
    category: Option<ItineraryCategory>,
) -> Result<i64> {
    add_itinerary_item_extended(
        conn,
        trip_id,
        day,
        title,
        note,
        start_time,
        sort_order,
        duration_minutes,
        travel_minutes,
        location,
        category,
        None,
        None,
    )
}

/// `--after` / `--before` を指定して日程を追加する
#[allow(clippy::too_many_arguments)]
pub(crate) fn add_itinerary_item_extended(
    conn: &Connection,
    trip_id: i64,
    day: i64,
    title: &str,
    note: Option<&str>,
    start_time: Option<&str>,
    sort_order: Option<i64>,
    duration_minutes: Option<i64>,
    travel_minutes: Option<i64>,
    location: Option<&str>,
    category: Option<ItineraryCategory>,
    after: Option<i64>,
    before: Option<i64>,
) -> Result<i64> {
    validate_itinerary_position_options(sort_order, after, before)?;
    let validated = validate_itinerary_content_fields(title, note, start_time, location)?;
    let day_id = resolve_itinerary_create_target(conn, trip_id, day)?;
    let resolved_sort_order =
        resolve_sort_order_for_add(conn, trip_id, day, sort_order, after, before)?;
    insert_validated_itinerary_item(
        conn,
        trip_id,
        day_id,
        day,
        &validated,
        resolved_sort_order,
        duration_minutes,
        travel_minutes,
        category,
    )
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn insert_validated_itinerary_item(
    conn: &Connection,
    trip_id: i64,
    day_id: i64,
    day_number: i64,
    validated: &ValidatedItineraryContent,
    sort_order: i64,
    duration_minutes: Option<i64>,
    travel_minutes: Option<i64>,
    category: Option<ItineraryCategory>,
) -> Result<i64> {
    let now = crate::storage::db::now_string();
    let category = category.map(|c| c.as_str().to_string());
    conn.execute(
        "INSERT INTO itinerary_items
         (trip_id, day_id, day, title, note, start_time, sort_order, duration_minutes, travel_minutes,
          location, category, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
        params![
            trip_id,
            day_id,
            day_number,
            &validated.title,
            validated.note.as_deref(),
            validated.start_time.as_deref(),
            sort_order,
            duration_minutes,
            travel_minutes,
            validated.location.as_deref(),
            category,
            &now,
            &now
        ],
    )
    .context("日程の追加に失敗しました")?;
    Ok(conn.last_insert_rowid())
}

fn validate_itinerary_position_options(
    sort_order: Option<i64>,
    after: Option<i64>,
    before: Option<i64>,
) -> Result<()> {
    if after.is_some() && before.is_some() {
        anyhow::bail!("--after と --before は同時に指定できません");
    }
    if sort_order.is_some() && (after.is_some() || before.is_some()) {
        anyhow::bail!("--order と --after / --before は同時に指定できません");
    }
    Ok(())
}

fn validate_reference_itinerary(
    reference: &ItineraryItem,
    trip_id: i64,
    day: i64,
    label: &str,
) -> Result<()> {
    if reference.trip_id != trip_id {
        anyhow::bail!(
            "{label} の Itinerary (ID: {}) は旅行 ID {trip_id} に属していません",
            reference.id
        );
    }
    if reference.day != day {
        anyhow::bail!(
            "{label} の Itinerary (ID: {}) は Day {day} に属していません（Day {}）",
            reference.id,
            reference.day
        );
    }
    Ok(())
}

pub(crate) fn max_sort_order_in_day(
    conn: &Connection,
    trip_id: i64,
    day_number: i64,
) -> Result<i64> {
    let max: Option<i64> = conn
        .query_row(
            "SELECT MAX(i.sort_order)
             FROM itinerary_items i
             INNER JOIN days d ON i.day_id = d.id
             WHERE i.trip_id = ?1 AND d.day_number = ?2",
            params![trip_id, day_number],
            |row| row.get(0),
        )
        .context("Day 内の最大 sort_order 取得に失敗しました")?;
    Ok(max.unwrap_or(0))
}

fn sort_order_midpoint(prev: i64, next: i64) -> Option<i64> {
    if next - prev > 1 {
        Some(prev + (next - prev) / 2)
    } else {
        None
    }
}

fn sort_order_before_first(next: i64) -> Option<i64> {
    sort_order_midpoint(0, next)
}

/// Day 内の Itinerary を表示順のまま 1000, 2000, 3000... に振り直す
pub(crate) fn normalize_day_sort_order(
    conn: &Connection,
    trip_id: i64,
    day_number: i64,
) -> Result<()> {
    crate::trip::get_trip(conn, trip_id)?;
    let _day = crate::day::find_day_by_trip_and_day_number(conn, trip_id, day_number)?;
    crate::storage::db::with_transaction(conn, "itinerary normalize sort order", |tx| {
        let items = list_itinerary_items_for_day(tx, trip_id, day_number)?;
        let now = crate::storage::db::now_string();
        for (idx, item) in items.iter().enumerate() {
            let new_order = SORT_ORDER_STEP * (idx as i64 + 1);
            if item.sort_order != new_order {
                tx.execute(
                    "UPDATE itinerary_items SET sort_order = ?1, updated_at = ?2 WHERE id = ?3",
                    params![new_order, &now, item.id],
                )
                .context("sort_order の正規化に失敗しました")?;
            }
        }
        Ok(())
    })
}

fn resolve_sort_order_for_add(
    conn: &Connection,
    trip_id: i64,
    day: i64,
    sort_order: Option<i64>,
    after: Option<i64>,
    before: Option<i64>,
) -> Result<i64> {
    if let Some(order) = sort_order {
        return Ok(order);
    }
    if let Some(ref_id) = after {
        return resolve_sort_order_after(conn, trip_id, day, ref_id);
    }
    if let Some(ref_id) = before {
        return resolve_sort_order_before(conn, trip_id, day, ref_id);
    }
    Ok(max_sort_order_in_day(conn, trip_id, day)? + SORT_ORDER_STEP)
}

fn resolve_sort_order_after(conn: &Connection, trip_id: i64, day: i64, ref_id: i64) -> Result<i64> {
    resolve_sort_order_after_excluding(conn, trip_id, day, ref_id, None)
}

fn resolve_sort_order_after_excluding(
    conn: &Connection,
    trip_id: i64,
    day: i64,
    ref_id: i64,
    exclude_id: Option<i64>,
) -> Result<i64> {
    let reference = get_itinerary_item(conn, ref_id)?;
    validate_reference_itinerary(&reference, trip_id, day, "--after")?;
    try_resolve_sort_order_after(conn, trip_id, day, ref_id, exclude_id).or_else(|_| {
        normalize_day_sort_order(conn, trip_id, day)?;
        try_resolve_sort_order_after(conn, trip_id, day, ref_id, exclude_id)
    })
}

fn try_resolve_sort_order_after(
    conn: &Connection,
    trip_id: i64,
    day: i64,
    ref_id: i64,
    exclude_id: Option<i64>,
) -> Result<i64> {
    let items = filter_day_items(conn, trip_id, day, exclude_id)?;
    let Some(idx) = items.iter().position(|item| item.id == ref_id) else {
        anyhow::bail!("Itinerary not found: {ref_id}");
    };
    let reference = &items[idx];
    if let Some(next) = items.get(idx + 1) {
        sort_order_midpoint(reference.sort_order, next.sort_order)
            .ok_or_else(|| anyhow::anyhow!("sort_order の隙間が不足しています"))
    } else {
        Ok(reference.sort_order + SORT_ORDER_STEP)
    }
}

fn resolve_sort_order_before(
    conn: &Connection,
    trip_id: i64,
    day: i64,
    ref_id: i64,
) -> Result<i64> {
    resolve_sort_order_before_excluding(conn, trip_id, day, ref_id, None)
}

fn resolve_sort_order_before_excluding(
    conn: &Connection,
    trip_id: i64,
    day: i64,
    ref_id: i64,
    exclude_id: Option<i64>,
) -> Result<i64> {
    let reference = get_itinerary_item(conn, ref_id)?;
    validate_reference_itinerary(&reference, trip_id, day, "--before")?;
    normalize_if_before_first_with_low_sort_order(conn, trip_id, day, ref_id, exclude_id)?;
    try_resolve_sort_order_before(conn, trip_id, day, ref_id, exclude_id).or_else(|_| {
        normalize_day_sort_order(conn, trip_id, day)?;
        try_resolve_sort_order_before(conn, trip_id, day, ref_id, exclude_id)
    })
}

/// 先頭 item への `--before` で `sort_order <= 1` のとき、中間値が取れないため先に正規化する
fn normalize_if_before_first_with_low_sort_order(
    conn: &Connection,
    trip_id: i64,
    day: i64,
    ref_id: i64,
    exclude_id: Option<i64>,
) -> Result<()> {
    let items = filter_day_items(conn, trip_id, day, exclude_id)?;
    if items
        .first()
        .is_some_and(|first| first.id == ref_id && first.sort_order <= 1)
    {
        normalize_day_sort_order(conn, trip_id, day)?;
    }
    Ok(())
}

fn try_resolve_sort_order_before(
    conn: &Connection,
    trip_id: i64,
    day: i64,
    ref_id: i64,
    exclude_id: Option<i64>,
) -> Result<i64> {
    let items = filter_day_items(conn, trip_id, day, exclude_id)?;
    let Some(idx) = items.iter().position(|item| item.id == ref_id) else {
        anyhow::bail!("Itinerary not found: {ref_id}");
    };
    let reference = &items[idx];
    if let Some(prev) = idx.checked_sub(1).and_then(|i| items.get(i)) {
        sort_order_midpoint(prev.sort_order, reference.sort_order)
            .ok_or_else(|| anyhow::anyhow!("sort_order の隙間が不足しています"))
    } else {
        sort_order_before_first(reference.sort_order)
            .ok_or_else(|| anyhow::anyhow!("sort_order の隙間が不足しています"))
    }
}

fn filter_day_items(
    conn: &Connection,
    trip_id: i64,
    day: i64,
    exclude_id: Option<i64>,
) -> Result<Vec<ItineraryItem>> {
    let items = list_itinerary_items_for_day(conn, trip_id, day)?;
    Ok(match exclude_id {
        Some(id) => items.into_iter().filter(|item| item.id != id).collect(),
        None => items,
    })
}

/// 既存 Itinerary を別の位置へ移動する
pub(crate) fn move_itinerary_item(
    conn: &Connection,
    id: i64,
    after: Option<i64>,
    before: Option<i64>,
) -> Result<()> {
    if after.is_some() && before.is_some() {
        anyhow::bail!("--after と --before は同時に指定できません");
    }
    if after.is_none() && before.is_none() {
        anyhow::bail!("--after または --before のいずれかを指定してください");
    }
    if after == Some(id) || before == Some(id) {
        anyhow::bail!("自分自身を基準位置に指定できません");
    }

    let item = get_itinerary_item(conn, id)?;
    let target_day = if let Some(ref_id) = after {
        let reference = get_itinerary_item(conn, ref_id)?;
        if reference.trip_id != item.trip_id {
            anyhow::bail!(
                "--after の Itinerary (ID: {ref_id}) は旅行 ID {} に属していません",
                item.trip_id
            );
        }
        reference.day
    } else {
        let ref_id = before.expect("validated above");
        let reference = get_itinerary_item(conn, ref_id)?;
        if reference.trip_id != item.trip_id {
            anyhow::bail!(
                "--before の Itinerary (ID: {ref_id}) は旅行 ID {} に属していません",
                item.trip_id
            );
        }
        reference.day
    };

    let new_sort_order = if let Some(ref_id) = after {
        resolve_sort_order_after_excluding(conn, item.trip_id, target_day, ref_id, Some(id))?
    } else {
        let ref_id = before.expect("validated above");
        resolve_sort_order_before_excluding(conn, item.trip_id, target_day, ref_id, Some(id))?
    };

    update_itinerary_item(
        conn,
        id,
        Some(target_day),
        None,
        None,
        None,
        Some(new_sort_order),
        None,
        None,
        None,
        None,
    )
}

/// `--items` のカンマ区切り ID リストをパースする
pub(crate) fn parse_item_id_list(spec: &str) -> Result<Vec<i64>> {
    let spec = spec.trim();
    if spec.is_empty() {
        anyhow::bail!("--items は1件以上指定してください");
    }
    let mut ids = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for segment in spec.split(',') {
        let segment = segment.trim();
        if segment.is_empty() {
            continue;
        }
        let id: i64 = segment
            .parse()
            .with_context(|| format!("不正な Itinerary ID です: {segment}"))?;
        if id < 1 {
            anyhow::bail!("Itinerary ID は 1 以上である必要があります: {id}");
        }
        if !seen.insert(id) {
            anyhow::bail!("--items に重複する ID が含まれています: {id}");
        }
        ids.push(id);
    }
    if ids.is_empty() {
        anyhow::bail!("--items は1件以上指定してください");
    }
    Ok(ids)
}

/// `--to-days` の Day 指定（`3`, `3,4,5`, `3-5`, `2,4-6`）をパースする
pub(crate) fn parse_target_day_list(spec: &str) -> Result<Vec<i64>> {
    let spec = spec.trim();
    if spec.is_empty() {
        anyhow::bail!("--to-days は1件以上指定してください");
    }
    let mut days = Vec::new();
    for segment in spec.split(',') {
        let segment = segment.trim();
        if segment.is_empty() {
            continue;
        }
        if let Some((start_str, end_str)) = segment.split_once('-') {
            let start: i64 = start_str
                .trim()
                .parse()
                .with_context(|| format!("不正な Day 範囲です: {segment}"))?;
            let end: i64 = end_str
                .trim()
                .parse()
                .with_context(|| format!("不正な Day 範囲です: {segment}"))?;
            if start < 1 || end < 1 {
                anyhow::bail!("Day 番号は 1 以上である必要があります: {segment}");
            }
            if start > end {
                anyhow::bail!("不正な Day 範囲です: {segment}");
            }
            for day in start..=end {
                if !days.contains(&day) {
                    days.push(day);
                }
            }
        } else {
            let day: i64 = segment
                .parse()
                .with_context(|| format!("不正な Day 番号です: {segment}"))?;
            if day < 1 {
                anyhow::bail!("Day 番号は 1 以上である必要があります: {day}");
            }
            if !days.contains(&day) {
                days.push(day);
            }
        }
    }
    if days.is_empty() {
        anyhow::bail!("--to-days は1件以上指定してください");
    }
    days.sort_unstable();
    Ok(days)
}

/// 1 コピー先 Day ごとの replicate 結果
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ReplicateDayResult {
    pub day: i64,
    pub created_ids: Vec<i64>,
}

/// replicate 全体の結果
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ReplicateResult {
    pub source_day: i64,
    pub target_days: Vec<i64>,
    pub by_day: Vec<ReplicateDayResult>,
}

impl ReplicateResult {
    pub(crate) fn total_created(&self) -> usize {
        self.by_day.iter().map(|day| day.created_ids.len()).sum()
    }
}

fn validate_replicate_inputs(
    conn: &Connection,
    item_ids: &[i64],
    target_days: &[i64],
) -> Result<(i64, i64, Vec<ItineraryItem>)> {
    if item_ids.is_empty() {
        anyhow::bail!("--items は1件以上指定してください");
    }
    if target_days.is_empty() {
        anyhow::bail!("--to-days は1件以上指定してください");
    }

    let mut source_items = Vec::with_capacity(item_ids.len());
    for &id in item_ids {
        source_items.push(get_itinerary_item(conn, id)?);
    }

    let trip_id = source_items[0].trip_id;
    if !source_items.iter().all(|item| item.trip_id == trip_id) {
        anyhow::bail!("指定された Itinerary は同一 Trip に属している必要があります");
    }

    let source_day = source_items[0].day;
    if !source_items.iter().all(|item| item.day == source_day) {
        anyhow::bail!("指定された Itinerary は同一 Day に属している必要があります");
    }

    if target_days.contains(&source_day) {
        anyhow::bail!("コピー先 Day に source Day ({source_day}) を含めることはできません");
    }

    for &day in target_days {
        crate::day::find_day_by_trip_and_day_number(conn, trip_id, day)?;
    }

    source_items.sort_by(|a, b| {
        a.sort_order
            .cmp(&b.sort_order)
            .then_with(|| a.id.cmp(&b.id))
    });

    Ok((trip_id, source_day, source_items))
}

fn insert_itinerary_item_copy(
    conn: &Connection,
    source: &ItineraryItem,
    target_day: i64,
) -> Result<i64> {
    let day_id = crate::day::find_day_id_by_trip_and_day_number(conn, source.trip_id, target_day)?;
    let now = crate::storage::db::now_string();
    let category = source.category.map(|c| c.as_str().to_string());
    conn.execute(
        "INSERT INTO itinerary_items
         (trip_id, day_id, day, title, note, start_time, sort_order, duration_minutes, travel_minutes,
          location, category, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
        params![
            source.trip_id,
            day_id,
            target_day,
            source.title,
            source.note,
            source.start_time,
            source.sort_order,
            source.duration_minutes,
            source.travel_minutes,
            source.location,
            category,
            &now,
            &now
        ],
    )
    .context("Itinerary の複製に失敗しました")?;
    Ok(conn.last_insert_rowid())
}

fn copy_itinerary_level_notes(
    conn: &Connection,
    source_itinerary_id: i64,
    new_itinerary_id: i64,
) -> Result<()> {
    use crate::domain::models::NoteOwnerType;

    let notes =
        crate::note::list_notes_for_owner(conn, NoteOwnerType::Itinerary, source_itinerary_id)?;
    let now = crate::storage::db::now_string();
    for note in notes {
        conn.execute(
            "INSERT INTO notes
             (owner_type, owner_id, title, body, sort_order, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                NoteOwnerType::Itinerary.as_str(),
                new_itinerary_id,
                note.title,
                note.body,
                note.sort_order,
                &now,
                &now
            ],
        )
        .context("Itinerary-level Note の複製に失敗しました")?;
    }
    Ok(())
}

fn replicate_itinerary_items_inner(
    conn: &Connection,
    source_items: &[ItineraryItem],
    target_days: &[i64],
    copy_notes: bool,
    dry_run: bool,
    source_day: i64,
) -> Result<ReplicateResult> {
    let mut by_day = Vec::with_capacity(target_days.len());

    for &target_day in target_days {
        let mut created_ids = Vec::with_capacity(source_items.len());
        if dry_run {
            created_ids = vec![0; source_items.len()];
        } else {
            for source in source_items {
                let new_id = insert_itinerary_item_copy(conn, source, target_day)?;
                if copy_notes {
                    copy_itinerary_level_notes(conn, source.id, new_id)?;
                }
                crate::estimate::copy_estimates_for_itinerary(conn, source.id, new_id)?;
                created_ids.push(new_id);
            }
        }
        by_day.push(ReplicateDayResult {
            day: target_day,
            created_ids,
        });
    }

    Ok(ReplicateResult {
        source_day,
        target_days: target_days.to_vec(),
        by_day,
    })
}

/// 既存 Itinerary を指定 Day 群へ独立した Itinerary として複製する
pub(crate) fn replicate_itinerary_items(
    conn: &Connection,
    item_ids: &[i64],
    target_days: &[i64],
    copy_notes: bool,
    dry_run: bool,
) -> Result<ReplicateResult> {
    let (_trip_id, source_day, source_items) =
        validate_replicate_inputs(conn, item_ids, target_days)?;

    if dry_run {
        return replicate_itinerary_items_inner(
            conn,
            &source_items,
            target_days,
            copy_notes,
            true,
            source_day,
        );
    }

    let mut result = None;
    crate::storage::db::with_transaction(conn, "itinerary replicate", |tx| {
        result = Some(replicate_itinerary_items_inner(
            tx,
            &source_items,
            target_days,
            copy_notes,
            false,
            source_day,
        )?);
        Ok(())
    })?;
    result.ok_or_else(|| anyhow::anyhow!("replicate 結果の取得に失敗しました"))
}

fn format_day_number_list(days: &[i64]) -> String {
    days.iter()
        .map(|day| day.to_string())
        .collect::<Vec<_>>()
        .join(", ")
}

fn format_id_list(ids: &[i64]) -> String {
    ids.iter()
        .map(|id| id.to_string())
        .collect::<Vec<_>>()
        .join(", ")
}

/// replicate 結果を表示する
pub(crate) fn print_replicate_result(result: &ReplicateResult, dry_run: bool) {
    if dry_run {
        println!("Dry run — no changes written.");
        println!();
    } else {
        println!("Itineraries replicated.");
    }
    println!("Source Day: {}", result.source_day);
    println!(
        "Target Days: {}",
        format_day_number_list(&result.target_days)
    );
    println!();
    if dry_run {
        println!("Would create:");
    } else {
        println!("Created:");
    }
    for day_result in &result.by_day {
        println!(
            "  Day {}: {} items",
            day_result.day,
            day_result.created_ids.len()
        );
    }
    println!();
    println!("Total: {} items", result.total_created());
    if !dry_run && result.total_created() > 0 {
        println!();
        for day_result in &result.by_day {
            println!(
                "Day {}: {}",
                day_result.day,
                format_id_list(&day_result.created_ids)
            );
        }
    }
}

/// 旅行に紐づく日程一覧を取得する
pub(crate) fn list_itinerary_items(conn: &Connection, trip_id: i64) -> Result<Vec<ItineraryItem>> {
    crate::trip::get_trip(conn, trip_id)?;
    let mut stmt = conn
        .prepare(&format!(
            "{ITINERARY_ITEM_SELECT_SQL}
             WHERE i.trip_id = ?1
             {ITINERARY_LIST_ORDER_BY}"
        ))
        .context("日程一覧取得の準備に失敗しました")?;

    let items = stmt
        .query_map(params![trip_id], row_to_itinerary_item)
        .context("日程一覧取得に失敗しました")?
        .collect::<std::result::Result<Vec<_>, _>>()
        .context("日程一覧の読み込みに失敗しました")?;

    Ok(items)
}

/// 指定 Day に属する日程一覧を取得する（timeline と同じ並び順）
pub(crate) fn list_itinerary_items_for_day(
    conn: &Connection,
    trip_id: i64,
    day_number: i64,
) -> Result<Vec<ItineraryItem>> {
    crate::trip::get_trip(conn, trip_id)?;
    let _day = crate::day::find_day_by_trip_and_day_number(conn, trip_id, day_number)?;
    let mut stmt = conn
        .prepare(&format!(
            "{ITINERARY_ITEM_SELECT_SQL}
             WHERE i.trip_id = ?1 AND d.day_number = ?2
             {ITINERARY_LIST_ORDER_BY}"
        ))
        .context("日程一覧取得の準備に失敗しました")?;

    let items = stmt
        .query_map(params![trip_id, day_number], row_to_itinerary_item)
        .context("日程一覧取得に失敗しました")?
        .collect::<std::result::Result<Vec<_>, _>>()
        .context("日程一覧の読み込みに失敗しました")?;

    Ok(items)
}

/// ID を指定して1件の日程を取得する
pub(crate) fn get_itinerary_item(conn: &Connection, id: i64) -> Result<ItineraryItem> {
    crate::storage::db::map_query_row(
        conn.query_row(
            &format!("{ITINERARY_ITEM_SELECT_SQL} WHERE i.id = ?1"),
            params![id],
            row_to_itinerary_item,
        ),
        || anyhow::anyhow!("Itinerary not found: {id}"),
    )
}

pub(crate) fn resolve_itinerary_update_target(
    conn: &Connection,
    itinerary_id: i64,
    trip_id: i64,
    day_number: i64,
) -> Result<()> {
    let matches_target: bool = conn
        .query_row(
            "SELECT EXISTS(
                SELECT 1
                FROM itinerary_items
                WHERE id = ?1 AND trip_id = ?2 AND day = ?3
            )",
            params![itinerary_id, trip_id, day_number],
            |row| row.get(0),
        )
        .context("Itinerary update target lookup failed")?;
    if !matches_target {
        anyhow::bail!(
            "Itinerary target not found: itinerary {itinerary_id}, trip {trip_id}, day {day_number}"
        );
    }
    Ok(())
}

pub(crate) fn update_validated_itinerary_content(
    conn: &Connection,
    itinerary_id: i64,
    trip_id: i64,
    day_number: i64,
    validated: &ValidatedItineraryContent,
) -> Result<usize> {
    let now = crate::storage::db::now_string();
    conn.execute(
        "UPDATE itinerary_items
         SET title = ?1, note = ?2, start_time = ?3, location = ?4, updated_at = ?5
         WHERE id = ?6 AND trip_id = ?7 AND day = ?8",
        params![
            &validated.title,
            validated.note.as_deref(),
            validated.start_time.as_deref(),
            validated.location.as_deref(),
            &now,
            itinerary_id,
            trip_id,
            day_number
        ],
    )
    .context("Itinerary content update failed")
}

/// 日程を更新する（指定されたフィールドのみ上書き）
#[allow(clippy::too_many_arguments)]
pub(crate) fn update_itinerary_item(
    conn: &Connection,
    id: i64,
    day: Option<i64>,
    title: Option<&str>,
    note: Option<Option<&str>>,
    start_time: Option<Option<&str>>,
    sort_order: Option<i64>,
    duration_minutes: Option<i64>,
    travel_minutes: Option<i64>,
    location: Option<Option<&str>>,
    category: Option<Option<ItineraryCategory>>,
) -> Result<()> {
    if day.is_none()
        && title.is_none()
        && note.is_none()
        && start_time.is_none()
        && sort_order.is_none()
        && duration_minutes.is_none()
        && travel_minutes.is_none()
        && location.is_none()
        && category.is_none()
    {
        anyhow::bail!(
            "更新する項目を1つ以上指定してください \
             (--day, --title, --note, --time, --order, --duration, --travel, --location, --category)"
        );
    }

    let mut item = get_itinerary_item(conn, id)?;
    let mut day_id = crate::day::find_day_id_by_trip_and_day_number(conn, item.trip_id, item.day)?;
    if let Some(d) = day {
        day_id = crate::day::find_day_id_by_trip_and_day_number(conn, item.trip_id, d)?;
        item.day = d;
    }
    if let Some(t) = title {
        item.title = t.to_string();
    }
    if let Some(n) = note {
        item.note = n.map(str::to_string);
    }
    if let Some(t) = start_time {
        if let Some(time_str) = t {
            parse_time_hhmm(time_str)?;
        }
        item.start_time = t.map(str::to_string);
    }
    if let Some(o) = sort_order {
        item.sort_order = o;
    }
    if let Some(d) = duration_minutes {
        item.duration_minutes = Some(d);
    }
    if let Some(t) = travel_minutes {
        item.travel_minutes = Some(t);
    }
    if let Some(l) = location {
        item.location = l.map(str::to_string);
    }
    if let Some(c) = category {
        item.category = c;
    }

    let now = crate::storage::db::now_string();
    let category_db = item.category.map(|c| c.as_str().to_string());
    conn.execute(
        "UPDATE itinerary_items
         SET day_id = ?1, day = ?2, title = ?3, note = ?4, start_time = ?5, sort_order = ?6,
             duration_minutes = ?7, travel_minutes = ?8, location = ?9, category = ?10,
             updated_at = ?11
         WHERE id = ?12",
        params![
            day_id,
            item.day,
            item.title,
            item.note,
            item.start_time,
            item.sort_order,
            item.duration_minutes,
            item.travel_minutes,
            item.location,
            category_db,
            &now,
            id
        ],
    )
    .context("日程の更新に失敗しました")?;
    Ok(())
}

/// 日程を削除する（子 entity の cascade なし — fragment apply delete_itinerary --confirm 用）
pub(crate) fn delete_itinerary_item_row_only(conn: &Connection, id: i64) -> Result<()> {
    get_itinerary_item(conn, id)?;
    crate::storage::db::with_transaction(conn, "itinerary row-only delete", |tx| {
        let deleted = tx
            .execute("DELETE FROM itinerary_items WHERE id = ?1", params![id])
            .context("日程の削除に失敗しました")?;
        if deleted != 1 {
            anyhow::bail!("itinerary DELETE が 1 行ではありません（{deleted}）— DB 更新しません");
        }
        Ok(())
    })
}

/// 日程を削除する
pub(crate) fn delete_itinerary_item(conn: &Connection, id: i64) -> Result<()> {
    get_itinerary_item(conn, id)?;
    crate::storage::db::with_transaction(conn, "itinerary delete", |tx| {
        crate::note::delete_notes_for_itinerary(tx, id)?;
        crate::estimate::delete_estimates_for_itinerary(tx, id)?;
        crate::expense::delete_expenses_for_itinerary(tx, id)?;
        crate::reservation::delete_reservations_for_itinerary(tx, id)?;
        tx.execute("DELETE FROM itinerary_items WHERE id = ?1", params![id])
            .context("日程の削除に失敗しました")?;
        Ok(())
    })
}

/// rusqlite の行データを ItineraryItem 構造体に変換する
pub(crate) fn row_to_itinerary_item(row: &rusqlite::Row) -> rusqlite::Result<ItineraryItem> {
    let category_raw: Option<String> = row.get(10)?;
    let category = match &category_raw {
        None => None,
        Some(value) => Some(
            crate::domain::models::parse_itinerary_category(value).map_err(|_| {
                rusqlite::Error::InvalidColumnType(10, value.clone(), rusqlite::types::Type::Text)
            })?,
        ),
    };
    Ok(ItineraryItem {
        id: row.get(0)?,
        trip_id: row.get(1)?,
        day: row.get(2)?,
        title: row.get(3)?,
        note: row.get(4)?,
        start_time: row.get(5)?,
        sort_order: row.get(6)?,
        duration_minutes: row.get(7)?,
        travel_minutes: row.get(8)?,
        location: row.get(9)?,
        category,
        created_at: row.get(11)?,
        updated_at: row.get(12)?,
    })
}
/// テキストを表示用に整形する（未設定なら "-"）
pub(crate) fn fmt_text(text: &Option<String>) -> &str {
    text.as_deref().unwrap_or("-")
}

/// 分数を表示用に整形する（未設定なら "-"）
pub(crate) fn fmt_minutes(minutes: Option<i64>) -> String {
    match minutes {
        Some(m) => format!("{m}分"),
        None => "-".to_string(),
    }
}

/// HH:MM 形式を検証し、(時, 分) を返す
pub(crate) fn parse_time_hhmm(time: &str) -> Result<(i32, i32)> {
    let parts: Vec<&str> = time.split(':').collect();
    if parts.len() != 2 || parts[0].len() != 2 || parts[1].len() != 2 {
        anyhow::bail!("時刻は HH:MM 形式で指定してください: {time}");
    }
    let hour: i32 = parts[0]
        .parse()
        .with_context(|| format!("不正な時刻です: {time}"))?;
    let minute: i32 = parts[1]
        .parse()
        .with_context(|| format!("不正な時刻です: {time}"))?;
    if !(0..=23).contains(&hour) || !(0..=59).contains(&minute) {
        anyhow::bail!("不正な時刻です: {time}");
    }
    Ok((hour, minute))
}

/// HH:MM に分数を加算した時刻を返す（日をまたぐ計算はしない）
pub(crate) fn add_minutes_to_time(time: &str, minutes: i64) -> Result<String> {
    let (hour, minute) = parse_time_hhmm(time)?;
    let total = hour * 60 + minute + minutes as i32;
    if total < 0 {
        anyhow::bail!("時刻の計算結果が不正です");
    }
    let new_hour = total / 60;
    let new_minute = total % 60;
    if new_hour >= 24 {
        anyhow::bail!("終了予定時刻が24時を超えました（日跨ぎには未対応です）");
    }
    Ok(format!("{new_hour:02}:{new_minute:02}"))
}
/// 日程一覧を表形式で表示する
pub(crate) fn print_itinerary_list(items: &[ItineraryItem]) {
    if items.is_empty() {
        println!("日程はまだ登録されていません。");
        return;
    }

    println!(
        "{:<6} {:<6} {:<8} {:<8} {:<14} {:<20} {:<8} {:<8} {:<12}",
        "ID", "日目", "順序", "時刻", "タイトル", "場所", "所要", "移動", "メモ"
    );
    println!("{}", "-".repeat(98));
    for item in items {
        println!(
            "{:<6} {:<6} {:<8} {:<8} {:<14} {:<20} {:<8} {:<8} {:<12}",
            item.id,
            item.day,
            item.sort_order,
            fmt_text(&item.start_time),
            item.title,
            fmt_text(&item.location),
            fmt_minutes(item.duration_minutes),
            fmt_minutes(item.travel_minutes),
            fmt_text(&item.note),
        );
    }
    println!();
    println!("合計: {} 件", items.len());
}

/// 日程の詳細を表示する
pub(crate) fn print_itinerary_detail(item: &ItineraryItem) {
    println!("ID        : {}", item.id);
    println!("旅行 ID   : {}", item.trip_id);
    println!("日目      : {}", item.day);
    println!("時刻      : {}", fmt_text(&item.start_time));
    println!("並び順    : {}", item.sort_order);
    println!("所要時間  : {}", fmt_minutes(item.duration_minutes));
    println!("移動時間  : {}", fmt_minutes(item.travel_minutes));
    println!("タイトル  : {}", item.title);
    if let Some(category) = item.category {
        println!("Category  : {}", category.as_str());
    }
    println!("場所      : {}", fmt_text(&item.location));
    println!("メモ      : {}", fmt_text(&item.note));
    println!("作成日時  : {}", item.created_at);
    println!("更新日時  : {}", item.updated_at);
}

/// 旅行のタイムラインを表示する
pub(crate) fn print_itinerary_timeline(items: &[ItineraryItem]) {
    if items.is_empty() {
        println!("日程はまだ登録されていません。");
        return;
    }

    let mut current_day: Option<i64> = None;
    for (index, item) in items.iter().enumerate() {
        if current_day != Some(item.day) {
            if current_day.is_some() {
                println!();
            }
            println!("Day {}", item.day);
            println!();
            current_day = Some(item.day);
        }

        match &item.start_time {
            Some(time) => {
                println!("{time} {}", item.title);
                if let Some(loc) = &item.location {
                    println!("  場所: {loc}");
                }
                if let Some(duration) = item.duration_minutes {
                    println!("  所要時間: {duration}分");
                    if let Ok(end_time) = add_minutes_to_time(time, duration) {
                        println!("  終了予定: {end_time}");
                    }
                }
            }
            None => {
                println!("時刻: 未定");
                println!("{}", item.title);
                if let Some(loc) = &item.location {
                    println!("  場所: {loc}");
                }
                if let Some(duration) = item.duration_minutes {
                    println!("  所要時間: {duration}分");
                }
            }
        }

        // 次の予定への移動時間を表示（同じ日の次の予定がある場合）
        if let Some(travel) = item.travel_minutes {
            let has_next_same_day = items
                .get(index + 1)
                .is_some_and(|next| next.day == item.day);
            if has_next_same_day {
                println!();
                println!("  ↓ 移動 {travel}分");
                println!();
            }
        } else if items
            .get(index + 1)
            .is_some_and(|next| next.day == item.day)
        {
            println!();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::models::ItineraryCategory;
    use crate::storage::db::open_db_at;
    use crate::trip::add_test_trip;
    use rusqlite::Connection;

    fn test_db() -> Connection {
        open_db_at(":memory:").expect("インメモリ DB の作成に失敗")
    }

    fn itinerary_category_line(item: &crate::domain::models::ItineraryItem) -> Option<String> {
        item.category.map(|c| format!("Category: {}", c.as_str()))
    }

    #[test]
    fn test_add_itinerary_item() {
        let conn = test_db();
        let trip_id = add_test_trip(&conn, "沖縄旅行").unwrap();

        let id = add_itinerary_item(
            &conn,
            trip_id,
            1,
            "首里城",
            Some("午前"),
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .unwrap();
        assert_eq!(id, 1);

        let item = get_itinerary_item(&conn, id).unwrap();
        assert_eq!(item.trip_id, trip_id);
        assert_eq!(item.day, 1);
        assert_eq!(item.title, "首里城");
        assert_eq!(item.note.as_deref(), Some("午前"));
        assert_eq!(item.sort_order, SORT_ORDER_STEP);

        let day_id: i64 = conn
            .query_row(
                "SELECT day_id FROM itinerary_items WHERE id = ?1",
                rusqlite::params![id],
                |row| row.get(0),
            )
            .unwrap();
        let expected_day_id =
            crate::day::find_day_id_by_trip_and_day_number(&conn, trip_id, 1).unwrap();
        assert_eq!(day_id, expected_day_id);
    }

    #[test]
    fn test_add_itinerary_item_rejects_day_outside_trip_range() {
        let conn = test_db();
        let trip_id = add_test_trip(&conn, "Short Trip").unwrap();
        assert!(add_itinerary_item(
            &conn,
            trip_id,
            99,
            "Out of range",
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .is_err());
    }

    #[test]
    fn test_add_itinerary_item_with_duration_and_travel() {
        let conn = test_db();
        let trip_id = add_test_trip(&conn, "沖縄旅行").unwrap();

        let id = add_itinerary_item(
            &conn,
            trip_id,
            1,
            "首里城",
            None,
            Some("09:00"),
            None,
            Some(90),
            Some(20),
            None,
            None,
        )
        .unwrap();

        let item = get_itinerary_item(&conn, id).unwrap();
        assert_eq!(item.duration_minutes, Some(90));
        assert_eq!(item.travel_minutes, Some(20));
    }

    #[test]
    fn test_add_itinerary_item_with_location() {
        let conn = test_db();
        let trip_id = add_test_trip(&conn, "沖縄旅行").unwrap();

        let id = add_itinerary_item(
            &conn,
            trip_id,
            1,
            "首里城",
            None,
            Some("09:00"),
            None,
            None,
            None,
            Some("沖縄県那覇市首里金城町1-2"),
            None,
        )
        .unwrap();

        let item = get_itinerary_item(&conn, id).unwrap();
        assert_eq!(item.location.as_deref(), Some("沖縄県那覇市首里金城町1-2"));
    }

    #[test]
    fn test_add_itinerary_item_with_start_time() {
        let conn = test_db();
        let trip_id = add_test_trip(&conn, "沖縄旅行").unwrap();

        let id = add_itinerary_item(
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

        let item = get_itinerary_item(&conn, id).unwrap();
        assert_eq!(item.start_time.as_deref(), Some("09:00"));
    }

    #[test]
    fn test_add_itinerary_item_without_start_time() {
        let conn = test_db();
        let trip_id = add_test_trip(&conn, "沖縄旅行").unwrap();

        let id = add_itinerary_item(
            &conn,
            trip_id,
            1,
            "ホテルチェックイン",
            None,
            None,
            Some(99),
            None,
            None,
            None,
            None,
        )
        .unwrap();

        let item = get_itinerary_item(&conn, id).unwrap();
        assert!(item.start_time.is_none());
        assert_eq!(item.sort_order, 99);
    }

    #[test]
    fn test_add_minutes_to_time() {
        assert_eq!(add_minutes_to_time("09:00", 90).unwrap(), "10:30");
        assert_eq!(add_minutes_to_time("12:30", 30).unwrap(), "13:00");
        assert!(parse_time_hhmm("25:00").is_err());
        assert!(parse_time_hhmm("9:00").is_err());
        assert!(add_minutes_to_time("23:00", 120).is_err());
    }

    #[test]
    fn test_clear_itinerary_category() {
        let conn = test_db();
        let trip_id = add_test_trip(&conn, "沖縄旅行").unwrap();
        let id = add_itinerary_item(
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
            Some(ItineraryCategory::Hotel),
        )
        .unwrap();

        update_itinerary_item(
            &conn,
            id,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            Some(None),
        )
        .unwrap();

        let item = get_itinerary_item(&conn, id).unwrap();
        assert!(item.category.is_none());
    }

    #[test]
    fn test_delete_itinerary_item() {
        let conn = test_db();
        let trip_id = add_test_trip(&conn, "沖縄旅行").unwrap();
        let id = add_itinerary_item(
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

        delete_itinerary_item(&conn, id).unwrap();

        assert!(list_itinerary_items(&conn, trip_id).unwrap().is_empty());
        assert!(get_itinerary_item(&conn, id).is_err());
    }

    #[test]
    fn test_get_itinerary_item() {
        let conn = test_db();
        let trip_id = add_test_trip(&conn, "沖縄旅行").unwrap();
        let id = add_itinerary_item(
            &conn,
            trip_id,
            1,
            "首里城",
            Some("午前"),
            Some("09:00"),
            None,
            None,
            None,
            None,
            None,
        )
        .unwrap();

        let item = get_itinerary_item(&conn, id).unwrap();
        assert_eq!(item.id, id);
        assert_eq!(item.day, 1);
        assert_eq!(item.title, "首里城");
    }

    #[test]
    fn test_itinerary_show_displays_category() {
        let conn = test_db();
        let trip_id = add_test_trip(&conn, "沖縄旅行").unwrap();
        let id = add_itinerary_item(
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
            Some(ItineraryCategory::Hotel),
        )
        .unwrap();

        let item = get_itinerary_item(&conn, id).unwrap();
        assert_eq!(
            itinerary_category_line(&item).as_deref(),
            Some("Category: hotel")
        );
    }

    #[test]
    fn test_itinerary_show_omits_category_when_unset() {
        let conn = test_db();
        let trip_id = add_test_trip(&conn, "沖縄旅行").unwrap();
        let id = add_itinerary_item(
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

        let item = get_itinerary_item(&conn, id).unwrap();
        assert!(itinerary_category_line(&item).is_none());
    }

    #[test]
    fn test_list_itinerary_items() {
        let conn = test_db();
        let trip_id = add_test_trip(&conn, "沖縄旅行").unwrap();
        add_itinerary_item(
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
        add_itinerary_item(
            &conn,
            trip_id,
            2,
            "美ら海水族館",
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .unwrap();

        let items = list_itinerary_items(&conn, trip_id).unwrap();
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].title, "首里城");
        assert_eq!(items[1].title, "美ら海水族館");
    }

    #[test]
    fn test_list_itinerary_items_sorted_by_day_and_sort_order() {
        let conn = test_db();
        let trip_id = add_test_trip(&conn, "沖縄旅行").unwrap();

        // 登録順・sort_order 混在でも、一覧は day → sort_order → id 順になること
        add_itinerary_item(
            &conn,
            trip_id,
            1,
            "昼食",
            None,
            Some("12:30"),
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
        add_itinerary_item(
            &conn,
            trip_id,
            1,
            "ホテル",
            None,
            None,
            Some(99),
            None,
            None,
            None,
            None,
        )
        .unwrap();
        add_itinerary_item(
            &conn,
            trip_id,
            2,
            "2日目",
            None,
            Some("10:00"),
            None,
            None,
            None,
            None,
            None,
        )
        .unwrap();

        let items = list_itinerary_items(&conn, trip_id).unwrap();
        assert_eq!(items.len(), 4);
        assert_eq!(items[0].title, "ホテル");
        assert_eq!(items[1].title, "昼食");
        assert_eq!(items[2].title, "首里城");
        assert_eq!(items[3].title, "2日目");
    }

    #[test]
    fn test_list_itinerary_sort_order_without_start_time_in_middle() {
        let conn = test_db();
        let trip_id = add_test_trip(&conn, "Ordering Trip").unwrap();

        add_itinerary_item(
            &conn,
            trip_id,
            1,
            "First",
            None,
            Some("08:00"),
            Some(1),
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
            "Last",
            None,
            Some("18:00"),
            Some(20),
            None,
            None,
            None,
            None,
        )
        .unwrap();

        let items = list_itinerary_items(&conn, trip_id).unwrap();
        assert_eq!(
            items.iter().map(|i| i.title.as_str()).collect::<Vec<_>>(),
            vec!["First", "Middle no time", "Last"]
        );
    }

    #[test]
    fn test_timeline_items_sorted_by_day_and_sort_order() {
        let conn = test_db();
        let trip_id = add_test_trip(&conn, "沖縄旅行").unwrap();

        add_itinerary_item(
            &conn,
            trip_id,
            1,
            "国際通り",
            None,
            Some("10:50"),
            Some(2),
            Some(60),
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
            Some(1),
            Some(90),
            Some(20),
            None,
            None,
        )
        .unwrap();
        add_itinerary_item(
            &conn,
            trip_id,
            2,
            "2日目",
            None,
            Some("10:00"),
            None,
            None,
            None,
            None,
            None,
        )
        .unwrap();

        let items = list_itinerary_items(&conn, trip_id).unwrap();
        assert_eq!(items.len(), 3);
        assert_eq!(items[0].title, "首里城");
        assert_eq!(items[1].title, "国際通り");
        assert_eq!(items[2].title, "2日目");
        assert_eq!(items[0].day, 1);
        assert_eq!(items[0].start_time.as_deref(), Some("09:00"));
    }

    #[test]
    fn test_update_itinerary_item() {
        let conn = test_db();
        let trip_id = add_test_trip(&conn, "沖縄旅行").unwrap();
        let id = add_itinerary_item(
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

        update_itinerary_item(
            &conn,
            id,
            Some(2),
            Some("美ら海水族館"),
            Some(Some("終日")),
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .unwrap();

        let item = get_itinerary_item(&conn, id).unwrap();
        assert_eq!(item.day, 2);
        assert_eq!(item.title, "美ら海水族館");
        assert_eq!(item.note.as_deref(), Some("終日"));

        let day_id: i64 = conn
            .query_row(
                "SELECT day_id FROM itinerary_items WHERE id = ?1",
                rusqlite::params![id],
                |row| row.get(0),
            )
            .unwrap();
        let expected_day_id =
            crate::day::find_day_id_by_trip_and_day_number(&conn, trip_id, 2).unwrap();
        assert_eq!(day_id, expected_day_id);
    }

    #[test]
    fn test_update_itinerary_item_category() {
        let conn = test_db();
        let trip_id = add_test_trip(&conn, "沖縄旅行").unwrap();
        let id = add_itinerary_item(
            &conn,
            trip_id,
            1,
            "Hilton Hawaiian Village",
            None,
            None,
            None,
            None,
            None,
            Some("Waikiki"),
            None,
        )
        .unwrap();

        update_itinerary_item(
            &conn,
            id,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            Some(Some(ItineraryCategory::Hotel)),
        )
        .unwrap();

        let item = get_itinerary_item(&conn, id).unwrap();
        assert_eq!(item.category, Some(ItineraryCategory::Hotel));
    }

    #[test]
    fn test_update_itinerary_item_duration_and_travel() {
        let conn = test_db();
        let trip_id = add_test_trip(&conn, "沖縄旅行").unwrap();
        let id = add_itinerary_item(
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

        update_itinerary_item(
            &conn,
            id,
            None,
            None,
            None,
            None,
            None,
            Some(90),
            Some(20),
            None,
            None,
        )
        .unwrap();

        let item = get_itinerary_item(&conn, id).unwrap();
        assert_eq!(item.duration_minutes, Some(90));
        assert_eq!(item.travel_minutes, Some(20));
    }

    #[test]
    fn test_update_itinerary_item_location() {
        let conn = test_db();
        let trip_id = add_test_trip(&conn, "沖縄旅行").unwrap();
        let id = add_itinerary_item(
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

        update_itinerary_item(
            &conn,
            id,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            Some(Some("沖縄県那覇市首里金城町1-2")),
            None,
        )
        .unwrap();

        let item = get_itinerary_item(&conn, id).unwrap();
        assert_eq!(item.location.as_deref(), Some("沖縄県那覇市首里金城町1-2"));
    }

    #[test]
    fn test_update_itinerary_item_start_time_and_sort_order() {
        let conn = test_db();
        let trip_id = add_test_trip(&conn, "沖縄旅行").unwrap();
        let id = add_itinerary_item(
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

        update_itinerary_item(
            &conn,
            id,
            None,
            None,
            None,
            Some(Some("09:30")),
            Some(5),
            None,
            None,
            None,
            None,
        )
        .unwrap();

        let item = get_itinerary_item(&conn, id).unwrap();
        assert_eq!(item.start_time.as_deref(), Some("09:30"));
        assert_eq!(item.sort_order, 5);
    }

    #[test]
    fn test_itinerary_list_json_empty() {
        let conn = test_db();
        let trip_id = add_test_trip(&conn, "沖縄旅行").unwrap();

        let items = list_itinerary_items(&conn, trip_id).unwrap();
        let json = serde_json::to_string_pretty(&items).unwrap();

        assert_eq!(json, "[]");
    }

    #[test]
    fn test_itinerary_list_json() {
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
            Some(90),
            Some(20),
            Some("那覇市"),
            Some(ItineraryCategory::Activity),
        )
        .unwrap();
        add_itinerary_item(
            &conn,
            trip_id,
            2,
            "美ら海",
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .unwrap();

        let items = list_itinerary_items(&conn, trip_id).unwrap();
        let json = serde_json::to_string_pretty(&items).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        assert!(parsed.is_array());
        assert_eq!(parsed.as_array().unwrap().len(), 2);
        assert_eq!(parsed[0]["title"], "首里城");
        assert_eq!(parsed[0]["category"], "activity");
        assert_eq!(parsed[1]["title"], "美ら海");
        assert!(parsed[1].get("category").is_none());
    }

    #[test]
    fn test_itinerary_show_json() {
        let conn = test_db();
        let trip_id = add_test_trip(&conn, "沖縄旅行").unwrap();
        let id = add_itinerary_item(
            &conn,
            trip_id,
            1,
            "首里城",
            Some("見学"),
            Some("09:00"),
            Some(1),
            Some(90),
            Some(20),
            Some("那覇市"),
            Some(ItineraryCategory::Museum),
        )
        .unwrap();

        let item = get_itinerary_item(&conn, id).unwrap();
        let json = serde_json::to_string_pretty(&item).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed["id"], id);
        assert_eq!(parsed["trip_id"], trip_id);
        assert_eq!(parsed["day"], 1);
        assert_eq!(parsed["title"], "首里城");
        assert_eq!(parsed["note"], "見学");
        assert_eq!(parsed["start_time"], "09:00");
        assert_eq!(parsed["sort_order"], 1);
        assert_eq!(parsed["duration_minutes"], 90);
        assert_eq!(parsed["travel_minutes"], 20);
        assert_eq!(parsed["location"], "那覇市");
        assert_eq!(parsed["category"], "museum");
    }

    #[test]
    fn test_get_itinerary_item_not_found() {
        let conn = test_db();
        let err = get_itinerary_item(&conn, 9999)
            .err()
            .expect("expected error");
        assert_eq!(err.to_string(), "Itinerary not found: 9999");
        assert!(!format!("{err:#}").contains("Query returned no rows"));
    }

    #[test]
    fn test_add_itinerary_item_appends_to_day_end() {
        let conn = test_db();
        let trip_id = add_test_trip(&conn, "Sort Trip").unwrap();

        let id1 = add_itinerary_item(
            &conn, trip_id, 1, "First", None, None, None, None, None, None, None,
        )
        .unwrap();
        let id2 = add_itinerary_item(
            &conn, trip_id, 1, "Second", None, None, None, None, None, None, None,
        )
        .unwrap();

        assert_eq!(get_itinerary_item(&conn, id1).unwrap().sort_order, 1000);
        assert_eq!(get_itinerary_item(&conn, id2).unwrap().sort_order, 2000);
    }

    #[test]
    fn test_add_itinerary_item_after_uses_midpoint() {
        let conn = test_db();
        let trip_id = add_test_trip(&conn, "Sort Trip").unwrap();

        let first = add_itinerary_item(
            &conn,
            trip_id,
            1,
            "空港",
            None,
            None,
            Some(1000),
            None,
            None,
            None,
            None,
        )
        .unwrap();
        let second = add_itinerary_item(
            &conn,
            trip_id,
            1,
            "搭乗",
            None,
            None,
            Some(2000),
            None,
            None,
            None,
            None,
        )
        .unwrap();
        let wifi = add_itinerary_item_extended(
            &conn,
            trip_id,
            1,
            "Wi-Fi",
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            Some(first),
            None,
        )
        .unwrap();

        let item = get_itinerary_item(&conn, wifi).unwrap();
        assert_eq!(item.sort_order, 1500);

        let items = list_itinerary_items_for_day(&conn, trip_id, 1).unwrap();
        let titles: Vec<_> = items.iter().map(|i| i.title.as_str()).collect();
        assert_eq!(titles, vec!["空港", "Wi-Fi", "搭乗"]);
        let _ = second;
    }

    #[test]
    fn test_add_itinerary_item_before_uses_midpoint() {
        let conn = test_db();
        let trip_id = add_test_trip(&conn, "Sort Trip").unwrap();

        add_itinerary_item(
            &conn,
            trip_id,
            1,
            "空港",
            None,
            None,
            Some(1000),
            None,
            None,
            None,
            None,
        )
        .unwrap();
        let boarding = add_itinerary_item(
            &conn,
            trip_id,
            1,
            "搭乗",
            None,
            None,
            Some(2000),
            None,
            None,
            None,
            None,
        )
        .unwrap();
        let wifi = add_itinerary_item_extended(
            &conn,
            trip_id,
            1,
            "Wi-Fi",
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            Some(boarding),
        )
        .unwrap();

        assert_eq!(get_itinerary_item(&conn, wifi).unwrap().sort_order, 1500);
    }

    #[test]
    fn test_add_itinerary_item_after_last_appends_with_step() {
        let conn = test_db();
        let trip_id = add_test_trip(&conn, "Sort Trip").unwrap();

        let last = add_itinerary_item(
            &conn,
            trip_id,
            1,
            "搭乗",
            None,
            None,
            Some(2000),
            None,
            None,
            None,
            None,
        )
        .unwrap();
        let extra = add_itinerary_item_extended(
            &conn,
            trip_id,
            1,
            "出国審査",
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            Some(last),
            None,
        )
        .unwrap();

        assert_eq!(get_itinerary_item(&conn, extra).unwrap().sort_order, 3000);
    }

    #[test]
    fn test_add_itinerary_item_normalizes_when_gap_too_small() {
        let conn = test_db();
        let trip_id = add_test_trip(&conn, "Sort Trip").unwrap();

        add_itinerary_item(
            &conn,
            trip_id,
            1,
            "A",
            None,
            None,
            Some(1000),
            None,
            None,
            None,
            None,
        )
        .unwrap();
        let b = add_itinerary_item(
            &conn,
            trip_id,
            1,
            "B",
            None,
            None,
            Some(1001),
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
            "C",
            None,
            None,
            Some(1002),
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
            "D",
            None,
            None,
            Some(1003),
            None,
            None,
            None,
            None,
        )
        .unwrap();

        let a = 1_i64;
        let inserted = add_itinerary_item_extended(
            &conn,
            trip_id,
            1,
            "Wi-Fi",
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            Some(a),
            None,
        )
        .unwrap();

        let items = list_itinerary_items_for_day(&conn, trip_id, 1).unwrap();
        assert_eq!(
            items.iter().map(|i| i.title.as_str()).collect::<Vec<_>>(),
            vec!["A", "Wi-Fi", "B", "C", "D"]
        );
        assert_eq!(
            get_itinerary_item(&conn, inserted).unwrap().sort_order,
            1500
        );
        assert_eq!(get_itinerary_item(&conn, b).unwrap().sort_order, 2000);
    }

    #[test]
    fn test_normalize_day_sort_order_preserves_display_order() {
        let conn = test_db();
        let trip_id = add_test_trip(&conn, "Sort Trip").unwrap();

        add_itinerary_item(
            &conn,
            trip_id,
            1,
            "A",
            None,
            None,
            Some(0),
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
            "B",
            None,
            None,
            Some(0),
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
            "C",
            None,
            None,
            Some(0),
            None,
            None,
            None,
            None,
        )
        .unwrap();

        normalize_day_sort_order(&conn, trip_id, 1).unwrap();

        let items = list_itinerary_items_for_day(&conn, trip_id, 1).unwrap();
        assert_eq!(
            items.iter().map(|i| i.title.as_str()).collect::<Vec<_>>(),
            vec!["A", "B", "C"]
        );
        assert_eq!(
            items.iter().map(|i| i.sort_order).collect::<Vec<_>>(),
            vec![1000, 2000, 3000]
        );
    }

    #[test]
    fn test_add_itinerary_rejects_after_from_other_day() {
        let conn = test_db();
        let trip_id = add_test_trip(&conn, "Sort Trip").unwrap();

        let day2_item = add_itinerary_item(
            &conn, trip_id, 2, "Day2", None, None, None, None, None, None, None,
        )
        .unwrap();

        let err = add_itinerary_item_extended(
            &conn,
            trip_id,
            1,
            "Wi-Fi",
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            Some(day2_item),
            None,
        )
        .unwrap_err();
        assert!(err.to_string().contains("Day 1"));
    }

    #[test]
    fn test_add_itinerary_rejects_order_with_after() {
        let conn = test_db();
        let trip_id = add_test_trip(&conn, "Sort Trip").unwrap();
        let anchor = add_itinerary_item(
            &conn, trip_id, 1, "Anchor", None, None, None, None, None, None, None,
        )
        .unwrap();

        let err = add_itinerary_item_extended(
            &conn,
            trip_id,
            1,
            "Wi-Fi",
            None,
            None,
            Some(500),
            None,
            None,
            None,
            None,
            Some(anchor),
            None,
        )
        .unwrap_err();
        assert!(err.to_string().contains("--order"));
    }

    #[test]
    fn test_move_itinerary_item_after() {
        let conn = test_db();
        let trip_id = add_test_trip(&conn, "Sort Trip").unwrap();

        let a = add_itinerary_item(
            &conn,
            trip_id,
            1,
            "A",
            None,
            None,
            Some(1000),
            None,
            None,
            None,
            None,
        )
        .unwrap();
        let b = add_itinerary_item(
            &conn,
            trip_id,
            1,
            "B",
            None,
            None,
            Some(2000),
            None,
            None,
            None,
            None,
        )
        .unwrap();
        let c = add_itinerary_item(
            &conn,
            trip_id,
            1,
            "C",
            None,
            None,
            Some(3000),
            None,
            None,
            None,
            None,
        )
        .unwrap();

        move_itinerary_item(&conn, c, Some(a), None).unwrap();

        let items = list_itinerary_items_for_day(&conn, trip_id, 1).unwrap();
        let titles: Vec<_> = items.iter().map(|i| i.title.as_str()).collect();
        assert_eq!(titles, vec!["A", "C", "B"]);
        let _ = b;
    }

    #[test]
    fn test_add_before_first_with_legacy_sort_order_zero() {
        let conn = test_db();
        let trip_id = add_test_trip(&conn, "Legacy Sort Trip").unwrap();

        add_itinerary_item(
            &conn,
            trip_id,
            1,
            "First",
            None,
            None,
            Some(0),
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
            None,
            Some(0),
            None,
            None,
            None,
            None,
        )
        .unwrap();
        let first_id = 1_i64;

        let prep = add_itinerary_item_extended(
            &conn,
            trip_id,
            1,
            "Prep",
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            Some(first_id),
        )
        .unwrap();

        let items = list_itinerary_items_for_day(&conn, trip_id, 1).unwrap();
        assert_eq!(
            items.iter().map(|i| i.title.as_str()).collect::<Vec<_>>(),
            vec!["Prep", "First", "Second"]
        );
        assert!(get_itinerary_item(&conn, prep).unwrap().sort_order < 1000);
        assert_eq!(
            get_itinerary_item(&conn, first_id).unwrap().sort_order,
            1000
        );
    }

    #[test]
    fn test_move_itinerary_item_rejects_self_reference() {
        let conn = test_db();
        let trip_id = add_test_trip(&conn, "Sort Trip").unwrap();
        let id = add_itinerary_item(
            &conn, trip_id, 1, "Only", None, None, None, None, None, None, None,
        )
        .unwrap();

        let after_err = move_itinerary_item(&conn, id, Some(id), None).unwrap_err();
        assert!(after_err.to_string().contains("自分自身"));

        let before_err = move_itinerary_item(&conn, id, None, Some(id)).unwrap_err();
        assert!(before_err.to_string().contains("自分自身"));
    }

    fn add_five_day_trip(conn: &Connection) -> i64 {
        crate::trip::add_trip(conn, "Replicate Trip", "2026-01-01", "2026-01-05", None).unwrap()
    }

    fn add_hotel_pattern(conn: &Connection, trip_id: i64, day: i64) -> Vec<i64> {
        vec![
            add_itinerary_item(
                &conn,
                trip_id,
                day,
                "ホテルで朝食",
                Some("7:00頃"),
                Some("07:00"),
                Some(1000),
                Some(45),
                Some(10),
                Some("ホテル"),
                Some(ItineraryCategory::Restaurant),
            )
            .unwrap(),
            add_itinerary_item(
                &conn,
                trip_id,
                day,
                "ホテルを出発",
                None,
                Some("08:30"),
                Some(2000),
                None,
                Some(20),
                None,
                Some(ItineraryCategory::Transport),
            )
            .unwrap(),
            add_itinerary_item(
                &conn,
                trip_id,
                day,
                "ホテルに戻る",
                None,
                Some("18:00"),
                Some(7000),
                None,
                None,
                None,
                Some(ItineraryCategory::Hotel),
            )
            .unwrap(),
            add_itinerary_item(
                &conn,
                trip_id,
                day,
                "ラウンジで夕食",
                None,
                Some("19:00"),
                Some(8000),
                Some(90),
                None,
                Some("ラウンジ"),
                Some(ItineraryCategory::Restaurant),
            )
            .unwrap(),
        ]
    }

    #[test]
    fn test_parse_target_day_list_supports_mixed_spec() {
        assert_eq!(parse_target_day_list("3-5").unwrap(), vec![3, 4, 5]);
        assert_eq!(parse_target_day_list("3,4,5").unwrap(), vec![3, 4, 5]);
        assert_eq!(parse_target_day_list("2,4-6").unwrap(), vec![2, 4, 5, 6]);
    }

    #[test]
    fn test_replicate_single_item_to_multiple_days() {
        let conn = test_db();
        let trip_id = add_five_day_trip(&conn);
        let source_id = add_itinerary_item(
            &conn,
            trip_id,
            2,
            "朝食",
            Some("短いメモ"),
            Some("07:00"),
            Some(1500),
            Some(30),
            Some(5),
            Some("Lobby"),
            Some(ItineraryCategory::Restaurant),
        )
        .unwrap();

        let result =
            replicate_itinerary_items(&conn, &[source_id], &[3, 4, 5], true, false).unwrap();
        assert_eq!(result.source_day, 2);
        assert_eq!(result.total_created(), 3);

        for day_result in &result.by_day {
            let copied = get_itinerary_item(&conn, day_result.created_ids[0]).unwrap();
            assert_eq!(copied.day, day_result.day);
            assert_eq!(copied.title, "朝食");
            assert_eq!(copied.note.as_deref(), Some("短いメモ"));
            assert_eq!(copied.start_time.as_deref(), Some("07:00"));
            assert_eq!(copied.sort_order, 1500);
            assert_eq!(copied.duration_minutes, Some(30));
            assert_eq!(copied.travel_minutes, Some(5));
            assert_eq!(copied.location.as_deref(), Some("Lobby"));
            assert_eq!(copied.category, Some(ItineraryCategory::Restaurant));
        }
    }

    #[test]
    fn test_replicate_multiple_items_preserves_sort_order() {
        let conn = test_db();
        let trip_id = add_five_day_trip(&conn);
        let source_ids = add_hotel_pattern(&conn, trip_id, 2);

        let result =
            replicate_itinerary_items(&conn, &source_ids, &[3, 4, 5], true, false).unwrap();
        assert_eq!(result.total_created(), 12);

        for day in [3, 4, 5] {
            let items = list_itinerary_items_for_day(&conn, trip_id, day).unwrap();
            assert_eq!(items.len(), 4);
            assert_eq!(
                items.iter().map(|i| i.sort_order).collect::<Vec<_>>(),
                vec![1000, 2000, 7000, 8000]
            );
            assert_eq!(
                items.iter().map(|i| i.title.as_str()).collect::<Vec<_>>(),
                vec![
                    "ホテルで朝食",
                    "ホテルを出発",
                    "ホテルに戻る",
                    "ラウンジで夕食"
                ]
            );
        }
    }

    #[test]
    fn test_replicate_copies_itinerary_level_notes_by_default() {
        use crate::domain::models::NoteOwnerType;
        use crate::note::{add_note, list_notes_for_owner, ResolvedNoteOwner};

        let conn = test_db();
        let trip_id = add_five_day_trip(&conn);
        let source_id = add_itinerary_item(
            &conn,
            trip_id,
            2,
            "朝食",
            None,
            None,
            Some(1000),
            None,
            None,
            None,
            None,
        )
        .unwrap();
        add_note(
            &conn,
            ResolvedNoteOwner::Itinerary(source_id),
            Some("補足"),
            "アレルギー確認",
        )
        .unwrap();

        let result = replicate_itinerary_items(&conn, &[source_id], &[3], true, false).unwrap();
        let copied_id = result.by_day[0].created_ids[0];
        let notes = list_notes_for_owner(&conn, NoteOwnerType::Itinerary, copied_id).unwrap();
        assert_eq!(notes.len(), 1);
        assert_eq!(notes[0].title.as_deref(), Some("補足"));
        assert_eq!(notes[0].body, "アレルギー確認");
    }

    #[test]
    fn test_replicate_without_notes_skips_itinerary_level_notes() {
        use crate::domain::models::NoteOwnerType;
        use crate::note::{add_note, list_notes_for_owner, ResolvedNoteOwner};

        let conn = test_db();
        let trip_id = add_five_day_trip(&conn);
        let source_id = add_itinerary_item(
            &conn,
            trip_id,
            2,
            "朝食",
            Some("本体メモ"),
            None,
            Some(1000),
            None,
            None,
            None,
            None,
        )
        .unwrap();
        add_note(
            &conn,
            ResolvedNoteOwner::Itinerary(source_id),
            None,
            "Itinerary note",
        )
        .unwrap();

        let result = replicate_itinerary_items(&conn, &[source_id], &[3], false, false).unwrap();
        let copied_id = result.by_day[0].created_ids[0];
        let copied = get_itinerary_item(&conn, copied_id).unwrap();
        assert_eq!(copied.note.as_deref(), Some("本体メモ"));
        let notes = list_notes_for_owner(&conn, NoteOwnerType::Itinerary, copied_id).unwrap();
        assert!(notes.is_empty());
    }

    #[test]
    fn test_replicate_copies_estimates() {
        use crate::estimate::{add_estimate, list_estimates_for_itinerary};

        let conn = test_db();
        let trip_id = add_five_day_trip(&conn);
        let source_id = add_itinerary_item(
            &conn,
            trip_id,
            2,
            "朝食",
            None,
            None,
            Some(1000),
            None,
            None,
            None,
            None,
        )
        .unwrap();
        add_estimate(
            &conn,
            source_id,
            "1400",
            "JPY",
            Some("朝食代"),
            Some("2名分"),
            Some(10),
        )
        .unwrap();
        add_estimate(
            &conn,
            source_id,
            "500",
            "JPY",
            Some("ドリンク"),
            None,
            Some(20),
        )
        .unwrap();

        let result = replicate_itinerary_items(&conn, &[source_id], &[3, 4], true, false).unwrap();
        assert_eq!(result.by_day.len(), 2);

        let source_estimates = list_estimates_for_itinerary(&conn, source_id).unwrap();
        assert_eq!(source_estimates.len(), 2);

        for day_result in &result.by_day {
            let copied_id = day_result.created_ids[0];
            let copied_estimates = list_estimates_for_itinerary(&conn, copied_id).unwrap();
            assert_eq!(copied_estimates.len(), 2);
            assert_eq!(
                copied_estimates[0].title.as_deref(),
                source_estimates[0].title.as_deref()
            );
            assert_eq!(copied_estimates[0].amount, source_estimates[0].amount);
            assert_eq!(copied_estimates[0].currency, source_estimates[0].currency);
            assert_eq!(
                copied_estimates[0].note.as_deref(),
                source_estimates[0].note.as_deref()
            );
            assert_eq!(
                copied_estimates[0].sort_order,
                source_estimates[0].sort_order
            );
            assert_ne!(copied_estimates[0].id, source_estimates[0].id);
            assert_ne!(copied_estimates[1].id, source_estimates[1].id);
            assert_eq!(copied_estimates[0].itinerary_id, copied_id);
        }
    }

    #[test]
    fn test_replicated_estimates_are_independent() {
        use crate::estimate::{
            add_estimate, get_estimate, list_estimates_for_itinerary, update_estimate,
            UpdateEstimateParams,
        };

        let conn = test_db();
        let trip_id = add_five_day_trip(&conn);
        let source_id = add_itinerary_item(
            &conn,
            trip_id,
            2,
            "朝食",
            None,
            None,
            Some(1000),
            None,
            None,
            None,
            None,
        )
        .unwrap();
        let source_estimate_id =
            add_estimate(&conn, source_id, "1400", "JPY", Some("朝食代"), None, None).unwrap();

        let result = replicate_itinerary_items(&conn, &[source_id], &[3], true, false).unwrap();
        let copied_id = result.by_day[0].created_ids[0];
        let copied_estimate_id = list_estimates_for_itinerary(&conn, copied_id).unwrap()[0].id;

        update_estimate(
            &conn,
            copied_estimate_id,
            &UpdateEstimateParams {
                amount_input: Some("1600"),
                ..Default::default()
            },
        )
        .unwrap();

        let source = get_estimate(&conn, source_estimate_id).unwrap();
        let copied = get_estimate(&conn, copied_estimate_id).unwrap();
        assert_eq!(source.amount, 1400);
        assert_eq!(copied.amount, 1600);

        update_estimate(
            &conn,
            source_estimate_id,
            &UpdateEstimateParams {
                title: Some("改定後"),
                ..Default::default()
            },
        )
        .unwrap();

        let source = get_estimate(&conn, source_estimate_id).unwrap();
        let copied = get_estimate(&conn, copied_estimate_id).unwrap();
        assert_eq!(source.title.as_deref(), Some("改定後"));
        assert_eq!(copied.title.as_deref(), Some("朝食代"));
    }

    #[test]
    fn test_replicate_does_not_copy_expense_or_reservation() {
        use crate::expense::add_expense;
        use crate::reservation::add_reservation;

        let conn = test_db();
        let trip_id = add_five_day_trip(&conn);
        let source_id = add_itinerary_item(
            &conn,
            trip_id,
            2,
            "朝食",
            None,
            None,
            Some(1000),
            None,
            None,
            None,
            None,
        )
        .unwrap();
        add_expense(
            &conn,
            source_id,
            "1500",
            "JPY",
            None,
            None,
            None,
            None,
            &Default::default(),
        )
        .unwrap();
        add_reservation(
            &conn,
            source_id,
            "hotel",
            "ホテル",
            Some("ABC123"),
            None,
            None,
            None,
            None,
        )
        .unwrap();

        let result = replicate_itinerary_items(&conn, &[source_id], &[3], true, false).unwrap();
        let copied_id = result.by_day[0].created_ids[0];

        let source_expenses =
            crate::expense::list_expenses_for_itinerary(&conn, source_id).unwrap();
        let copied_expenses =
            crate::expense::list_expenses_for_itinerary(&conn, copied_id).unwrap();
        assert_eq!(source_expenses.len(), 1);
        assert!(copied_expenses.is_empty());

        let source_reservations =
            crate::reservation::list_reservations_for_itinerary(&conn, source_id).unwrap();
        let copied_reservations =
            crate::reservation::list_reservations_for_itinerary(&conn, copied_id).unwrap();
        assert_eq!(source_reservations.len(), 1);
        assert!(copied_reservations.is_empty());
    }

    #[test]
    fn test_replicate_rejects_mixed_trips() {
        let conn = test_db();
        let trip_a = add_five_day_trip(&conn);
        let trip_b = add_five_day_trip(&conn);
        let a = add_itinerary_item(
            &conn, trip_a, 2, "A", None, None, None, None, None, None, None,
        )
        .unwrap();
        let b = add_itinerary_item(
            &conn, trip_b, 2, "B", None, None, None, None, None, None, None,
        )
        .unwrap();

        let err = replicate_itinerary_items(&conn, &[a, b], &[3], true, false).unwrap_err();
        assert!(err.to_string().contains("同一 Trip"));
    }

    #[test]
    fn test_replicate_rejects_mixed_source_days() {
        let conn = test_db();
        let trip_id = add_five_day_trip(&conn);
        let day2 = add_itinerary_item(
            &conn, trip_id, 2, "A", None, None, None, None, None, None, None,
        )
        .unwrap();
        let day3 = add_itinerary_item(
            &conn, trip_id, 3, "B", None, None, None, None, None, None, None,
        )
        .unwrap();

        let err = replicate_itinerary_items(&conn, &[day2, day3], &[4], true, false).unwrap_err();
        assert!(err.to_string().contains("同一 Day"));
    }

    #[test]
    fn test_replicate_rejects_source_day_in_target_days() {
        let conn = test_db();
        let trip_id = add_five_day_trip(&conn);
        let source_id = add_itinerary_item(
            &conn, trip_id, 2, "A", None, None, None, None, None, None, None,
        )
        .unwrap();

        let err = replicate_itinerary_items(&conn, &[source_id], &[2, 3], true, false).unwrap_err();
        assert!(err.to_string().contains("source Day"));
    }

    #[test]
    fn test_replicate_rejects_missing_target_day() {
        let conn = test_db();
        let trip_id = add_five_day_trip(&conn);
        let source_id = add_itinerary_item(
            &conn, trip_id, 2, "A", None, None, None, None, None, None, None,
        )
        .unwrap();

        let err = replicate_itinerary_items(&conn, &[source_id], &[6], true, false).unwrap_err();
        assert!(err.to_string().contains("Day not found"));
    }

    #[test]
    fn test_replicated_items_are_independent() {
        let conn = test_db();
        let trip_id = add_five_day_trip(&conn);
        let source_id = add_itinerary_item(
            &conn,
            trip_id,
            2,
            "朝食",
            None,
            Some("07:00"),
            Some(1000),
            None,
            None,
            None,
            None,
        )
        .unwrap();

        let result = replicate_itinerary_items(&conn, &[source_id], &[3], true, false).unwrap();
        let copied_id = result.by_day[0].created_ids[0];

        update_itinerary_item(
            &conn,
            copied_id,
            None,
            Some("早めの朝食"),
            None,
            Some(Some("06:30")),
            None,
            None,
            None,
            None,
            None,
        )
        .unwrap();

        let source = get_itinerary_item(&conn, source_id).unwrap();
        let copied = get_itinerary_item(&conn, copied_id).unwrap();
        assert_eq!(source.title, "朝食");
        assert_eq!(source.start_time.as_deref(), Some("07:00"));
        assert_eq!(copied.title, "早めの朝食");
        assert_eq!(copied.start_time.as_deref(), Some("06:30"));
    }

    #[test]
    fn test_replicate_dry_run_does_not_write() {
        use crate::estimate::{add_estimate, list_estimates_for_itinerary};

        let conn = test_db();
        let trip_id = add_five_day_trip(&conn);
        let source_ids = add_hotel_pattern(&conn, trip_id, 2);
        add_estimate(
            &conn,
            source_ids[0],
            "1400",
            "JPY",
            Some("朝食代"),
            None,
            None,
        )
        .unwrap();

        let estimate_count_before: i64 = conn
            .query_row("SELECT COUNT(*) FROM estimates", [], |row| row.get(0))
            .unwrap();

        let result = replicate_itinerary_items(&conn, &source_ids, &[3, 4], true, true).unwrap();
        assert_eq!(result.total_created(), 8);
        assert!(list_itinerary_items_for_day(&conn, trip_id, 3)
            .unwrap()
            .is_empty());
        assert!(list_itinerary_items_for_day(&conn, trip_id, 4)
            .unwrap()
            .is_empty());

        let estimate_count_after: i64 = conn
            .query_row("SELECT COUNT(*) FROM estimates", [], |row| row.get(0))
            .unwrap();
        assert_eq!(estimate_count_before, estimate_count_after);
        assert_eq!(
            list_estimates_for_itinerary(&conn, source_ids[0])
                .unwrap()
                .len(),
            1
        );
    }
}
