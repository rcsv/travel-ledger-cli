mod analysis;
mod checklist;
mod cli;
mod commands;
mod config;
mod day;
mod domain;
mod estimate;
mod expense;
mod io;
mod itinerary;
mod money;
mod note;
mod output;
mod participant;
mod receipt;
mod reservation;
mod services;
mod storage;
mod summary;
mod trip;

use anyhow::{bail, Result};
use clap::{CommandFactory, Parser};
use cli::{
    ChecklistAction, Cli, Command, DayAction, EstimateAction, ExpenseAction, ItineraryAction,
    NoteAction, ParticipantAction, ReceiptAction, ReservationAction, TripAction,
};

fn main() -> Result<()> {
    let cli = Cli::parse();

    if cli.about {
        commands::about::print();
        return Ok(());
    }

    let Some(command) = cli.command else {
        let mut cmd = Cli::command();
        cmd.print_help()?;
        println!();
        bail!("a subcommand is required");
    };

    let resolved = config::resolve_db_path_for_cli(cli.db.as_deref())?;

    if commands::db::run_before_open_db(&command, &resolved)? {
        return Ok(());
    }

    let db_path = resolved.path.to_string_lossy().into_owned();
    let conn = storage::db::open_db_at(&db_path)?;

    match command {
        Command::Db { action } => commands::db::run_after_open(&conn, action, &resolved)?,
        Command::Itinerary { action } => match action {
            ItineraryAction::Add {
                trip_id,
                day,
                title,
                note,
                time,
                order,
                after,
                before,
                duration,
                travel,
                location,
            } => {
                let id = crate::itinerary::add_itinerary_item_extended(
                    &conn,
                    trip_id,
                    day,
                    &title,
                    note.as_deref(),
                    time.as_deref(),
                    order,
                    duration,
                    travel,
                    location.as_deref(),
                    None,
                    after,
                    before,
                )?;
                let item = crate::itinerary::get_itinerary_item(&conn, id)?;
                println!("日程を追加しました (ID: {id})");
                println!("  旅行 ID : {trip_id}");
                println!("  日目    : {day}");
                println!("  時刻    : {}", crate::itinerary::fmt_text(&time));
                println!("  並び順  : {}", item.sort_order);
                println!("  所要時間: {}", crate::itinerary::fmt_minutes(duration));
                println!("  移動時間: {}", crate::itinerary::fmt_minutes(travel));
                println!("  タイトル: {title}");
                println!("  場所    : {}", crate::itinerary::fmt_text(&location));
                println!("  メモ    : {}", crate::itinerary::fmt_text(&note));
            }
            ItineraryAction::List { trip_id, json } => {
                let result = crate::services::itinerary_list::list_itineraries(&conn, trip_id)?;
                if json {
                    crate::output::json::print_json(&result.items)?;
                } else {
                    println!("旅行 ID {trip_id} の日程:");
                    crate::itinerary::print_itinerary_list(&result.items);
                }
            }
            ItineraryAction::Timeline { trip_id } => {
                let result = crate::services::itinerary_timeline::get_timeline(&conn, trip_id)?;
                println!("{} のタイムライン:", result.trip.name);
                println!();
                crate::itinerary::print_itinerary_timeline(&result.items);
            }
            ItineraryAction::Show { id, json } => {
                let result = crate::services::itinerary_show::show_itinerary(&conn, id)?;
                if json {
                    crate::output::json::print_json(&result.item)?;
                } else {
                    crate::itinerary::print_itinerary_detail(&result.item);
                    let reservations =
                        crate::reservation::list_reservations_for_itinerary(&conn, id)?;
                    if !reservations.is_empty() {
                        println!();
                        println!("Reservations ({}):", reservations.len());
                        for res in &reservations {
                            println!(
                                "  [{}] {}  {}  {}",
                                res.id,
                                res.reservation_type,
                                res.provider_name,
                                crate::reservation::fmt_optional_text(&res.confirmation_code)
                            );
                            let period =
                                crate::reservation::format_period(&res.start_at, &res.end_at);
                            if period != "-" {
                                println!("      {period}");
                            }
                        }
                    }
                }
            }
            ItineraryAction::Update {
                id,
                day,
                title,
                note,
                time,
                order,
                duration,
                travel,
                location,
                category,
            } => {
                let note_update = note.as_ref().map(|n| Some(n.as_str()));
                let time_update = time.as_ref().map(|t| Some(t.as_str()));
                let location_update = location.as_ref().map(|l| Some(l.as_str()));
                let category_update = match category.as_deref() {
                    None => None,
                    Some("none") => Some(None),
                    Some(value) => Some(Some(crate::domain::models::parse_itinerary_category(
                        value,
                    )?)),
                };
                crate::itinerary::update_itinerary_item(
                    &conn,
                    id,
                    day,
                    title.as_deref(),
                    note_update,
                    time_update,
                    order,
                    duration,
                    travel,
                    location_update,
                    category_update,
                )?;
                println!("日程を更新しました (ID: {id})");
                let item = crate::itinerary::get_itinerary_item(&conn, id)?;
                crate::itinerary::print_itinerary_detail(&item);
            }
            ItineraryAction::Delete { id } => {
                let item = crate::itinerary::get_itinerary_item(&conn, id)?;
                crate::itinerary::delete_itinerary_item(&conn, id)?;
                println!("日程を削除しました (ID: {id})");
                println!("  タイトル: {}", item.title);
            }
            ItineraryAction::Normalize { trip_id, day } => {
                crate::itinerary::normalize_day_sort_order(&conn, trip_id, day)?;
                println!("Day {day} の sort_order を正規化しました (旅行 ID: {trip_id})");
                let items = crate::itinerary::list_itinerary_items_for_day(&conn, trip_id, day)?;
                crate::itinerary::print_itinerary_list(&items);
            }
            ItineraryAction::Move { id, after, before } => {
                crate::itinerary::move_itinerary_item(&conn, id, after, before)?;
                println!("日程を移動しました (ID: {id})");
                let item = crate::itinerary::get_itinerary_item(&conn, id)?;
                crate::itinerary::print_itinerary_detail(&item);
            }
            ItineraryAction::Replicate {
                items,
                to_days,
                without_notes,
                dry_run,
            } => {
                let item_ids = crate::itinerary::parse_item_id_list(&items)?;
                let target_days = crate::itinerary::parse_target_day_list(&to_days)?;
                let result = crate::itinerary::replicate_itinerary_items(
                    &conn,
                    &item_ids,
                    &target_days,
                    !without_notes,
                    dry_run,
                )?;
                crate::itinerary::print_replicate_result(&result, dry_run);
            }
        },
        Command::Checklist { action } => match action {
            ChecklistAction::Add { trip_id, title } => {
                let id = crate::checklist::add_checklist_item(&conn, trip_id, &title)?;
                println!("チェックリスト項目を追加しました (ID: {id})");
                println!("  旅行 ID : {trip_id}");
                println!("  タイトル: {title}");
            }
            ChecklistAction::List { trip_id, json } => {
                let result = crate::services::checklist_list::list_checklist(&conn, trip_id)?;
                if json {
                    crate::output::json::print_json(&result.items)?;
                } else {
                    println!("旅行 ID {trip_id} のチェックリスト:");
                    crate::checklist::print_checklist_list(&result.items);
                }
            }
            ChecklistAction::Show { id, json } => {
                let result = crate::services::checklist_show::show_checklist(&conn, id)?;
                if json {
                    crate::output::json::print_json(&result.item)?;
                } else {
                    crate::checklist::print_checklist_detail(&result.item);
                }
            }
            ChecklistAction::Update {
                id,
                title,
                sort_order,
            } => {
                crate::checklist::update_checklist_item(&conn, id, title.as_deref(), sort_order)?;
                println!("チェックリスト項目を更新しました (ID: {id})");
                let item = crate::checklist::get_checklist_item(&conn, id)?;
                crate::checklist::print_checklist_detail(&item);
            }
            ChecklistAction::Check { id } => {
                crate::checklist::set_checklist_done(&conn, id, true)?;
                println!("チェックリスト項目を完了にしました (ID: {id})");
                let item = crate::checklist::get_checklist_item(&conn, id)?;
                crate::checklist::print_checklist_detail(&item);
            }
            ChecklistAction::Uncheck { id } => {
                crate::checklist::set_checklist_done(&conn, id, false)?;
                println!("チェックリスト項目を未完了に戻しました (ID: {id})");
                let item = crate::checklist::get_checklist_item(&conn, id)?;
                crate::checklist::print_checklist_detail(&item);
            }
            ChecklistAction::Delete { id } => {
                let item = crate::checklist::get_checklist_item(&conn, id)?;
                crate::checklist::delete_checklist_item(&conn, id)?;
                println!("チェックリスト項目を削除しました (ID: {id})");
                println!("  タイトル: {}", item.title);
            }
        },
        Command::Day { action } => match action {
            DayAction::List { trip_id, json } => {
                let result = crate::services::day_list::list_days(&conn, trip_id)?;
                if json {
                    crate::day::print_day_list_json(&result.trip, trip_id, &result.days)?;
                } else {
                    crate::day::print_day_list_display(&result.trip, &result.days)?;
                }
            }
            DayAction::Show {
                trip_id,
                day_number,
                json,
            } => {
                crate::day::run_day_show(&conn, trip_id, day_number, json)?;
            }
            DayAction::Update {
                trip_id,
                day_number,
                summary,
                clear_summary,
            } => {
                crate::day::run_day_update(
                    &conn,
                    trip_id,
                    day_number,
                    summary.as_deref(),
                    clear_summary,
                )?;
            }
            DayAction::Swap {
                trip_id,
                day_a,
                day_b,
            } => {
                let updated = crate::day::swap_day_plan_payload(&conn, trip_id, day_a, day_b)?;
                println!("Day {day_a} と Day {day_b} の計画内容を入れ替えました");
                println!("  更新件数: {updated}");
            }
        },
        Command::Note { action } => match action {
            NoteAction::Add {
                trip,
                day,
                itinerary,
                title,
                body,
            } => {
                let owner = crate::note::resolve_note_owner_for_add(&conn, trip, day, itinerary)?;
                let id = crate::note::add_note(&conn, owner, title.as_deref(), &body)?;
                println!("Note を追加しました (ID: {id})");
                let note = crate::note::get_note(&conn, id)?;
                crate::note::print_note_detail(&note);
            }
            NoteAction::List {
                trip,
                day,
                itinerary,
                json,
            } => {
                let result = crate::services::note_list::list_notes(&conn, trip, day, itinerary)?;
                if json {
                    crate::output::json::print_json(&crate::note::NoteListJson {
                        owner_type: result.owner_type,
                        owner_id: result.owner_id,
                        notes: result.notes,
                    })?;
                } else {
                    crate::note::print_note_list(result.owner_type, result.owner_id, &result.notes);
                }
            }
            NoteAction::Show { id, json } => {
                let note = crate::note::get_note(&conn, id)?;
                if json {
                    crate::output::json::print_json(&note)?;
                } else {
                    crate::note::print_note_detail(&note);
                }
            }
            NoteAction::Update { id, title, body } => {
                crate::note::update_note(&conn, id, title.as_deref(), body.as_deref())?;
                println!("Note を更新しました (ID: {id})");
                let note = crate::note::get_note(&conn, id)?;
                crate::note::print_note_detail(&note);
            }
            NoteAction::Delete { id } => {
                let note = crate::note::get_note(&conn, id)?;
                crate::note::delete_note(&conn, id)?;
                println!("Note を削除しました (ID: {id})");
                println!("  Title: {}", note.title.as_deref().unwrap_or("-"));
            }
        },
        Command::Expense { action } => match action {
            ExpenseAction::Add {
                itinerary,
                amount,
                currency,
                title,
                note,
                paid_by_name,
                paid_by_participant,
                beneficiary,
                shared_with,
                expense_date,
            } => {
                let shared = crate::expense::parse_expense_shared_options_for_add(
                    &conn,
                    itinerary,
                    paid_by_participant.as_deref(),
                    &beneficiary,
                    shared_with.as_deref(),
                )?;
                let id = crate::expense::add_expense(
                    &conn,
                    itinerary,
                    &amount,
                    &currency,
                    title.as_deref(),
                    note.as_deref(),
                    paid_by_name.as_deref(),
                    expense_date.as_deref(),
                    &shared,
                )?;
                println!("Expense を追加しました (ID: {id})");
                let expense = crate::expense::get_expense(&conn, id)?;
                crate::expense::print_expense_detail(&conn, &expense)?;
            }
            ExpenseAction::List {
                trip,
                itinerary,
                json,
            } => {
                let result = crate::services::expense_list::list_expenses(&conn, trip, itinerary)?;
                if json {
                    let (trip_id, itinerary_id) = match result.target {
                        crate::expense::ExpenseListTarget::Trip(id) => (Some(id), None),
                        crate::expense::ExpenseListTarget::Itinerary(id) => (None, Some(id)),
                    };
                    let json_expenses: Vec<crate::expense::ExpenseJson> = result
                        .expenses
                        .iter()
                        .map(|e| crate::expense::expense_to_json(&conn, e))
                        .collect::<Result<Vec<_>>>()?;
                    crate::output::json::print_json(&crate::expense::ExpenseListJson {
                        trip_id,
                        itinerary_id,
                        expenses: json_expenses,
                    })?;
                } else {
                    crate::expense::print_expense_list(&conn, result.target, &result.expenses)?;
                }
            }
            ExpenseAction::Show { id, json } => {
                let expense = crate::expense::get_expense(&conn, id)?;
                if json {
                    crate::output::json::print_json(&crate::expense::expense_to_json(
                        &conn, &expense,
                    )?)?;
                } else {
                    crate::expense::print_expense_detail(&conn, &expense)?;
                }
            }
            ExpenseAction::Update {
                id,
                title,
                amount,
                currency,
                paid_by_name,
                paid_by_participant,
                beneficiary,
                shared_with,
                clear_paid_by,
                clear_beneficiaries,
                expense_date,
                note,
            } => {
                let expense = crate::expense::get_expense(&conn, id)?;
                let shared = crate::expense::parse_expense_shared_options_for_update(
                    &conn,
                    expense.itinerary_id,
                    paid_by_participant.as_deref(),
                    &beneficiary,
                    shared_with.as_deref(),
                    clear_paid_by,
                    clear_beneficiaries,
                )?;
                crate::expense::update_expense(
                    &conn,
                    id,
                    title.as_deref(),
                    amount.as_deref(),
                    currency.as_deref(),
                    paid_by_name.as_deref(),
                    expense_date.as_deref(),
                    note.as_deref(),
                    &shared,
                )?;
                println!("Expense を更新しました (ID: {id})");
                let expense = crate::expense::get_expense(&conn, id)?;
                crate::expense::print_expense_detail(&conn, &expense)?;
            }
            ExpenseAction::Delete { id } => {
                let expense = crate::expense::get_expense(&conn, id)?;
                crate::expense::delete_expense(&conn, id)?;
                println!("Expense を削除しました (ID: {id})");
                println!(
                    "  Amount: {}",
                    crate::expense::format_amount_display(expense.amount, &expense.currency)
                );
            }
        },
        Command::Estimate { action } => match action {
            EstimateAction::Add {
                itinerary,
                amount,
                currency,
                title,
                note,
                sort_order,
            } => {
                let id = crate::estimate::add_estimate(
                    &conn,
                    itinerary,
                    &amount,
                    &currency,
                    title.as_deref(),
                    note.as_deref(),
                    sort_order,
                )?;
                println!("Estimate を追加しました (ID: {id})");
                let estimate = crate::estimate::get_estimate(&conn, id)?;
                crate::estimate::print_estimate_detail(&estimate)?;
            }
            EstimateAction::List {
                trip,
                itinerary,
                json,
            } => {
                let target = crate::estimate::resolve_estimate_list_target(trip, itinerary)?;
                let estimates = match target {
                    crate::estimate::EstimateListTarget::Trip(trip_id) => {
                        crate::estimate::list_estimates_for_trip(&conn, trip_id)?
                    }
                    crate::estimate::EstimateListTarget::Itinerary(itinerary_id) => {
                        crate::estimate::list_estimates_for_itinerary(&conn, itinerary_id)?
                    }
                };
                if json {
                    let (trip_id, itinerary_id) = match target {
                        crate::estimate::EstimateListTarget::Trip(id) => (Some(id), None),
                        crate::estimate::EstimateListTarget::Itinerary(id) => (None, Some(id)),
                    };
                    let json_estimates: Vec<crate::estimate::EstimateJson> = estimates
                        .iter()
                        .map(crate::estimate::estimate_to_json)
                        .collect();
                    crate::output::json::print_json(&crate::estimate::EstimateListJson {
                        trip_id,
                        itinerary_id,
                        estimates: json_estimates,
                    })?;
                } else {
                    crate::estimate::print_estimate_list(target, &estimates)?;
                }
            }
            EstimateAction::Show { id, json } => {
                let estimate = crate::estimate::get_estimate(&conn, id)?;
                if json {
                    crate::output::json::print_json(&crate::estimate::estimate_to_json(&estimate))?;
                } else {
                    crate::estimate::print_estimate_detail(&estimate)?;
                }
            }
            EstimateAction::Update {
                id,
                title,
                note,
                amount,
                currency,
                sort_order,
                clear_title,
                clear_note,
            } => {
                crate::estimate::update_estimate(
                    &conn,
                    id,
                    &crate::estimate::UpdateEstimateParams {
                        title: title.as_deref(),
                        note: note.as_deref(),
                        amount_input: amount.as_deref(),
                        currency_input: currency.as_deref(),
                        sort_order,
                        clear_title,
                        clear_note,
                    },
                )?;
                println!("Estimate を更新しました (ID: {id})");
                let estimate = crate::estimate::get_estimate(&conn, id)?;
                crate::estimate::print_estimate_detail(&estimate)?;
            }
            EstimateAction::Delete { id } => {
                let estimate = crate::estimate::get_estimate(&conn, id)?;
                crate::estimate::delete_estimate(&conn, id)?;
                println!("Estimate を削除しました (ID: {id})");
                println!(
                    "  Amount: {}",
                    crate::money::format_amount_display(estimate.amount, &estimate.currency)
                );
            }
        },
        Command::Reservation { action } => match action {
            ReservationAction::Add {
                itinerary,
                reservation_type,
                provider,
                confirmation,
                site_url,
                remark,
                start_at,
                end_at,
            } => {
                let id = crate::reservation::add_reservation(
                    &conn,
                    itinerary,
                    &reservation_type,
                    &provider,
                    confirmation.as_deref(),
                    site_url.as_deref(),
                    remark.as_deref(),
                    start_at.as_deref(),
                    end_at.as_deref(),
                )?;
                println!("Reservation を追加しました (ID: {id})");
                let reservation = crate::reservation::get_reservation(&conn, id)?;
                crate::reservation::print_reservation_detail(&conn, &reservation);
            }
            ReservationAction::List {
                trip,
                itinerary,
                json,
            } => {
                let result =
                    crate::services::reservation_list::list_reservations(&conn, trip, itinerary)?;
                if json {
                    let (trip_id, itinerary_id) = match result.target {
                        crate::reservation::ReservationListTarget::Trip(id) => (Some(id), None),
                        crate::reservation::ReservationListTarget::Itinerary(id) => {
                            (None, Some(id))
                        }
                    };
                    crate::output::json::print_json(&crate::reservation::ReservationListJson {
                        trip_id,
                        itinerary_id,
                        reservations: result.reservations,
                    })?;
                } else {
                    crate::reservation::print_reservation_list(
                        result.target,
                        &result.reservations,
                        result.trip_context.as_deref(),
                    );
                }
            }
            ReservationAction::Show { id, json } => {
                let reservation = crate::reservation::get_reservation(&conn, id)?;
                if json {
                    crate::output::json::print_json(&reservation)?;
                } else {
                    crate::reservation::print_reservation_detail(&conn, &reservation);
                }
            }
            ReservationAction::Update {
                id,
                reservation_type,
                provider,
                confirmation,
                site_url,
                remark,
                start_at,
                end_at,
                clear_confirmation,
                clear_site_url,
                clear_remark,
                clear_start_at,
                clear_end_at,
            } => {
                let confirmation_update = if clear_confirmation {
                    Some(None)
                } else {
                    confirmation.as_ref().map(|value| Some(value.as_str()))
                };
                let site_url_update = if clear_site_url {
                    Some(None)
                } else {
                    site_url.as_ref().map(|value| Some(value.as_str()))
                };
                let remark_update = if clear_remark {
                    Some(None)
                } else {
                    remark.as_ref().map(|value| Some(value.as_str()))
                };
                let start_at_update = if clear_start_at {
                    Some(None)
                } else {
                    start_at.as_ref().map(|value| Some(value.as_str()))
                };
                let end_at_update = if clear_end_at {
                    Some(None)
                } else {
                    end_at.as_ref().map(|value| Some(value.as_str()))
                };
                crate::reservation::update_reservation(
                    &conn,
                    id,
                    reservation_type.as_deref(),
                    provider.as_deref(),
                    confirmation_update,
                    site_url_update,
                    remark_update,
                    start_at_update,
                    end_at_update,
                )?;
                println!("Reservation を更新しました (ID: {id})");
                let reservation = crate::reservation::get_reservation(&conn, id)?;
                crate::reservation::print_reservation_detail(&conn, &reservation);
            }
            ReservationAction::Delete { id } => {
                let reservation = crate::reservation::get_reservation(&conn, id)?;
                crate::reservation::delete_reservation(&conn, id)?;
                println!("Reservation を削除しました (ID: {id})");
                println!("  Provider: {}", reservation.provider_name);
            }
        },
        Command::Receipt { action } => match action {
            ReceiptAction::Add {
                trip,
                day,
                amount,
                currency,
                occurred_date,
                memo,
            } => {
                let id = crate::receipt::add_receipt(
                    &conn,
                    crate::receipt::AddReceiptParams {
                        trip_id: trip,
                        day_number: day,
                        amount_input: amount.as_deref(),
                        currency_input: currency.as_deref(),
                        occurred_date: occurred_date.as_deref(),
                        memo: memo.as_deref(),
                    },
                )?;
                println!("Receipt を追加しました (ID: {id})");
                let receipt = crate::receipt::get_receipt(&conn, id)?;
                crate::receipt::print_receipt_detail(&conn, &receipt)?;
            }
            ReceiptAction::List {
                trip,
                trashed,
                all,
                unreviewed,
                status,
                json,
            } => {
                if trashed && all {
                    anyhow::bail!("--trashed と --all は同時に指定できません");
                }
                let status_filter = if unreviewed {
                    Some(crate::receipt::RECEIPT_STATUS_UNREVIEWED)
                } else {
                    status.as_deref()
                };
                let pending_summary = crate::receipt::compute_pending_receipt_summary(&conn, trip)?;
                let receipts = crate::receipt::list_receipts_for_trip(
                    &conn,
                    trip,
                    status_filter,
                    all,
                    trashed,
                )?;
                if json {
                    let json_receipts: Vec<crate::receipt::ReceiptJson> = receipts
                        .iter()
                        .map(|r| crate::receipt::receipt_to_json(&conn, r))
                        .collect::<Result<Vec<_>>>()?;
                    crate::output::json::print_json(&crate::receipt::ReceiptListJson {
                        trip_id: trip,
                        summary: pending_summary,
                        receipts: json_receipts,
                    })?;
                } else {
                    println!("旅行 ID {trip} の Receipt:");
                    crate::receipt::print_pending_receipt_summary(&pending_summary);
                    crate::receipt::print_receipt_list(&conn, &receipts)?;
                }
            }
            ReceiptAction::Show { id, json } => {
                let receipt = crate::receipt::get_receipt(&conn, id)?;
                if json {
                    crate::output::json::print_json(&crate::receipt::receipt_to_json(
                        &conn, &receipt,
                    )?)?;
                } else {
                    crate::receipt::print_receipt_detail(&conn, &receipt)?;
                }
            }
            ReceiptAction::Update {
                id,
                day,
                amount,
                currency,
                occurred_date,
                memo,
                clear_day,
                clear_amount,
                clear_occurred_date,
                clear_memo,
            } => {
                let occurred_date_update = if clear_occurred_date {
                    Some(None)
                } else {
                    occurred_date.as_ref().map(|value| Some(value.as_str()))
                };
                let memo_update = if clear_memo {
                    Some(None)
                } else {
                    memo.as_ref().map(|value| Some(value.as_str()))
                };
                crate::receipt::update_receipt(
                    &conn,
                    id,
                    crate::receipt::UpdateReceiptParams {
                        day_number: day,
                        amount_input: amount.as_deref(),
                        currency_input: currency.as_deref(),
                        occurred_date: occurred_date_update,
                        memo: memo_update,
                        clear_day,
                        clear_amount_currency: clear_amount,
                    },
                )?;
                println!("Receipt を更新しました (ID: {id})");
                let receipt = crate::receipt::get_receipt(&conn, id)?;
                crate::receipt::print_receipt_detail(&conn, &receipt)?;
            }
            ReceiptAction::Assign {
                id,
                itinerary,
                amount,
                currency,
                memo,
            } => {
                let expense_id = crate::receipt::assign_receipt_to_itinerary(
                    &conn,
                    id,
                    itinerary,
                    amount.as_deref(),
                    currency.as_deref(),
                    memo.as_deref(),
                )?;
                println!("Receipt を Expense に昇格しました (Expense ID: {expense_id})");
            }
            ReceiptAction::Trash { id } => {
                crate::receipt::trash_receipt(&conn, id)?;
                println!("Receipt を Trash に移動しました (ID: {id})");
                let receipt = crate::receipt::get_receipt(&conn, id)?;
                crate::receipt::print_receipt_detail(&conn, &receipt)?;
            }
            ReceiptAction::Restore { id } => {
                crate::receipt::restore_receipt(&conn, id)?;
                println!("Receipt を復元しました (ID: {id})");
                let receipt = crate::receipt::get_receipt(&conn, id)?;
                crate::receipt::print_receipt_detail(&conn, &receipt)?;
            }
            ReceiptAction::Ignore { id, memo } => {
                crate::receipt::ignore_receipt(&conn, id, memo.as_deref())?;
                println!("Receipt を Trash に移動しました (deprecated: ignore) (ID: {id})");
                let receipt = crate::receipt::get_receipt(&conn, id)?;
                crate::receipt::print_receipt_detail(&conn, &receipt)?;
            }
            ReceiptAction::Delete { id } => {
                let receipt = crate::receipt::get_receipt(&conn, id)?;
                crate::receipt::delete_receipt(&conn, id)?;
                println!("Receipt を削除しました (ID: {id})");
                if let Some(memo) = &receipt.memo {
                    println!("  Memo: {memo}");
                }
                if let (Some(amount), Some(currency)) = (receipt.amount, &receipt.currency) {
                    println!(
                        "  Amount: {}",
                        crate::money::format_amount_display(amount, currency)
                    );
                }
            }
        },
        Command::Participant { action } => match action {
            ParticipantAction::Add {
                trip,
                name,
                sort_order,
                self_marker,
            } => {
                let id = crate::participant::create_participant(
                    &conn,
                    trip,
                    &name,
                    sort_order,
                    self_marker,
                )?;
                let participant = crate::participant::get_participant(&conn, id)?;
                let self_note = if participant.is_self { " (self)" } else { "" };
                println!(
                    "Participant を追加しました (ID: {id}){self_note}: {}",
                    participant.name
                );
                println!("  Trip ID: {trip}");
            }
            ParticipantAction::List { trip, json } => {
                let participants = crate::participant::list_participants_by_trip(&conn, trip)?;
                let counts = crate::participant::compute_participant_counts_for_trip(&conn, trip)?;
                if json {
                    crate::output::json::print_json(&crate::participant::ParticipantListJson {
                        schema_version: 1,
                        trip_id: trip,
                        participants,
                        counts,
                    })?;
                } else {
                    crate::participant::print_participant_list_human(&participants, &counts);
                }
            }
            ParticipantAction::Show { id, json } => {
                let participant = crate::participant::get_participant(&conn, id)?;
                if json {
                    crate::output::json::print_json(&participant)?;
                } else {
                    crate::participant::print_participant_detail(&participant);
                }
            }
            ParticipantAction::Update {
                id,
                name,
                sort_order,
                self_marker,
                not_self,
            } => {
                if self_marker && not_self {
                    anyhow::bail!("--self と --not-self は同時に指定できません");
                }
                let set_self = if self_marker {
                    Some(true)
                } else if not_self {
                    Some(false)
                } else {
                    None
                };
                crate::participant::update_participant(
                    &conn,
                    id,
                    name.as_deref(),
                    sort_order,
                    set_self,
                )?;
                println!("Participant を更新しました (ID: {id})");
                let participant = crate::participant::get_participant(&conn, id)?;
                crate::participant::print_participant_detail(&participant);
            }
            ParticipantAction::Delete { id } => {
                let participant = crate::participant::get_participant(&conn, id)?;
                crate::participant::delete_participant(&conn, id)?;
                println!("Participant を削除しました (ID: {id})");
                println!("  Name: {}", participant.name);
            }
        },
        Command::Trip { action } => match action {
            TripAction::Add {
                name,
                start,
                end,
                summary,
            } => {
                let id = crate::trip::add_trip(&conn, &name, &start, &end, summary.as_deref())?;
                println!("旅行を追加しました (ID: {id})");
                println!("  名前   : {name}");
                println!("  開始日 : {start}");
                println!("  終了日 : {end}");
                if let Some(text) = summary {
                    println!("  概要   : {text}");
                }
            }
            TripAction::List { json } => {
                let result = crate::services::trip_list::list_trips(&conn)?;
                if json {
                    crate::output::json::print_json(&result.trips)?;
                } else {
                    crate::trip::print_trip_list(&result.trips);
                }
            }
            TripAction::Show { id, json } => {
                let result = crate::services::trip_show::show_trip(&conn, id)?;
                if json {
                    crate::output::json::print_json(&result.trip)?;
                } else {
                    crate::trip::print_trip_detail(&result.trip);
                }
            }
            TripAction::Update {
                id,
                name,
                start,
                end,
                summary,
                clear_summary,
            } => {
                crate::trip::update_trip(
                    &conn,
                    id,
                    name.as_deref(),
                    start.as_deref(),
                    end.as_deref(),
                    summary.as_deref(),
                    clear_summary,
                )?;
                println!("旅行を更新しました (ID: {id})");
                let trip = crate::trip::get_trip(&conn, id)?;
                crate::trip::print_trip_detail(&trip);
            }
            TripAction::Delete { id } => {
                let trip = crate::trip::get_trip(&conn, id)?;
                crate::trip::delete_trip(&conn, id)?;
                println!("旅行を削除しました (ID: {id})");
                println!("  名前: {}", trip.name);
            }
            TripAction::Duplicate { id, name } => {
                let new_id = crate::trip::duplicate_trip(&conn, id, name.as_deref())?;
                println!("Created trip {new_id} from trip {id}");
            }
            TripAction::Export { id, output } => {
                crate::trip::write_trip_export(&conn, id, output.as_deref())?;
            }
            TripAction::ExportMd { id, output } => {
                crate::io::markdown::write_trip_markdown(&conn, id, output.as_deref())?;
            }
            TripAction::Import { file } => {
                crate::trip::run_trip_import(&conn, &file)?;
            }
            TripAction::ValidateExport { file, json } => {
                crate::trip::run_trip_validate_export(&file, json)?;
            }
            TripAction::Diff { old_file, new_file } => {
                crate::io::diff::run_trip_diff(&old_file, &new_file)?;
            }
            TripAction::ChecklistGenerate { id, dry_run } => {
                if dry_run {
                    let result = crate::checklist::plan_checklist_generation(&conn, id)?;
                    crate::checklist::print_checklist_generate_dry_run_result(&result);
                } else {
                    let result = crate::checklist::generate_checklist_from_itinerary(&conn, id)?;
                    crate::checklist::print_checklist_generate_result(&result);
                }
            }
            TripAction::Stats { trip_id, json } => {
                let result = crate::services::trip_stats::get_trip_stats(&conn, trip_id)?;
                if json {
                    crate::output::json::print_json(&result.stats)?;
                } else {
                    crate::analysis::statistics::print_trip_stats_display(&result.stats)?;
                }
            }
            TripAction::Doctor { trip_id, json } => {
                crate::analysis::doctor::run_trip_doctor(&conn, trip_id, json)?;
            }
            TripAction::Advisor {
                trip_id,
                with_commands,
                json,
            } => {
                crate::analysis::advisor::run_trip_advisor(&conn, trip_id, with_commands, json)?;
            }
        },
    }

    Ok(())
}
