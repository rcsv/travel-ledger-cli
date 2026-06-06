mod advisor;
mod checklist;
mod db;
mod diff;
mod doctor;
mod itinerary;
mod markdown;
mod models;
mod stats;
mod trip;

use anyhow::Result;
use clap::{Parser, Subcommand};

// ---------------------------------------------------------------------------
// CLI 定義（clap derive）
// ---------------------------------------------------------------------------

#[derive(Parser)]
#[command(name = "caglla", about = "Caglla.Travel CLI - 旅行管理ツール")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// 旅行 (Trip) の管理
    Trip {
        #[command(subcommand)]
        action: TripAction,
    },
    /// 日程 (Itinerary) の管理
    Itinerary {
        #[command(subcommand)]
        action: ItineraryAction,
    },
    /// 持ち物・準備リスト (Checklist) の管理
    Checklist {
        #[command(subcommand)]
        action: ChecklistAction,
    },
    /// データベース操作（開発用）
    Db {
        #[command(subcommand)]
        action: DbAction,
    },
}

#[derive(Subcommand)]
enum DbAction {
    /// 【開発用】全データを削除して DB を初期状態に戻す（本番運用では使わない）
    Reset,
}

#[derive(Subcommand)]
enum TripAction {
    /// 新しい旅行を追加
    Add {
        /// 旅行名（必須）
        name: String,
        /// 開始日 (YYYY-MM-DD)
        #[arg(long)]
        start: Option<String>,
        /// 終了日 (YYYY-MM-DD)
        #[arg(long)]
        end: Option<String>,
    },
    /// 旅行一覧を表示
    List,
    /// 旅行の詳細を表示
    Show {
        /// 旅行 ID
        id: i64,
    },
    /// 旅行を更新
    Update {
        /// 旅行 ID
        id: i64,
        /// 新しい旅行名
        #[arg(long)]
        name: Option<String>,
        /// 新しい開始日 (YYYY-MM-DD)
        #[arg(long)]
        start: Option<String>,
        /// 新しい終了日 (YYYY-MM-DD)
        #[arg(long)]
        end: Option<String>,
    },
    /// 旅行を削除
    Delete {
        /// 旅行 ID
        id: i64,
    },
    /// 旅行データを JSON でエクスポート
    Export {
        /// 旅行 ID
        id: i64,
        /// 出力先ファイル（省略時は標準出力）
        #[arg(long)]
        output: Option<String>,
    },
    /// 旅行しおりを Markdown でエクスポート
    ExportMd {
        /// 旅行 ID
        id: i64,
        /// 出力先ファイル（省略時は標準出力）
        #[arg(long)]
        output: Option<String>,
    },
    /// JSON ファイルから旅行データをインポート
    Import {
        /// 読み込む JSON ファイル
        file: String,
    },
    /// 2つの旅行 JSON を比較
    Diff {
        /// 比較元 JSON ファイル
        old_file: String,
        /// 比較先 JSON ファイル
        new_file: String,
    },
    /// カテゴリ定義からチェックリストを自動生成
    #[command(name = "checklist-generate")]
    ChecklistGenerate {
        /// 旅行 ID
        id: i64,
    },
    /// 旅行の統計を表示
    Stats {
        /// 旅行 ID
        trip_id: i64,
    },
    /// 旅行計画を点検する
    Doctor {
        /// 旅行 ID
        trip_id: i64,
    },
    /// 旅行計画の改善提案を表示する
    Advisor {
        /// 旅行 ID
        trip_id: i64,
        /// 改善提案に加えて次に試せる CLI コマンド例を表示する
        #[arg(long)]
        with_commands: bool,
    },
}

#[derive(Subcommand)]
enum ItineraryAction {
    /// 日程を追加
    Add {
        /// 旅行 ID
        trip_id: i64,
        /// 何日目か
        #[arg(long)]
        day: i64,
        /// タイトル（必須）
        title: String,
        /// メモ
        #[arg(long)]
        note: Option<String>,
        /// 開始時刻 (HH:MM)
        #[arg(long)]
        time: Option<String>,
        /// 並び順（小さいほど先）
        #[arg(long)]
        order: Option<i64>,
        /// 所要時間（分）
        #[arg(long)]
        duration: Option<i64>,
        /// 次の予定までの移動時間（分）
        #[arg(long)]
        travel: Option<i64>,
        /// 場所
        #[arg(long)]
        location: Option<String>,
    },
    /// 旅行の日程一覧を表示
    List {
        /// 旅行 ID
        trip_id: i64,
    },
    /// 旅行のタイムラインを表示
    Timeline {
        /// 旅行 ID
        trip_id: i64,
    },
    /// 日程の詳細を表示
    Show {
        /// 日程 ID
        id: i64,
    },
    /// 日程を更新
    Update {
        /// 日程 ID
        id: i64,
        /// 何日目か
        #[arg(long)]
        day: Option<i64>,
        /// タイトル
        #[arg(long)]
        title: Option<String>,
        /// メモ
        #[arg(long)]
        note: Option<String>,
        /// 開始時刻 (HH:MM)
        #[arg(long)]
        time: Option<String>,
        /// 並び順（小さいほど先）
        #[arg(long)]
        order: Option<i64>,
        /// 所要時間（分）
        #[arg(long)]
        duration: Option<i64>,
        /// 次の予定までの移動時間（分）
        #[arg(long)]
        travel: Option<i64>,
        /// 場所
        #[arg(long)]
        location: Option<String>,
        /// カテゴリ（flight, hotel など。`none` で解除）
        #[arg(long)]
        category: Option<String>,
    },
    /// 日程を削除
    Delete {
        /// 日程 ID
        id: i64,
    },
}

#[derive(Subcommand)]
enum ChecklistAction {
    /// チェックリスト項目を追加
    Add {
        /// 旅行 ID
        trip_id: i64,
        /// 項目名（必須）
        title: String,
    },
    /// 旅行のチェックリスト一覧を表示
    List {
        /// 旅行 ID
        trip_id: i64,
    },
    /// チェックリスト項目の詳細を表示
    Show {
        /// 項目 ID
        id: i64,
    },
    /// チェックリスト項目を更新
    Update {
        /// 項目 ID
        id: i64,
        /// 新しい項目名
        #[arg(long)]
        title: Option<String>,
        /// 並び順（小さいほど先）
        #[arg(long)]
        sort_order: Option<i64>,
    },
    /// 項目を完了にする
    Check {
        /// 項目 ID
        id: i64,
    },
    /// 項目を未完了に戻す
    Uncheck {
        /// 項目 ID
        id: i64,
    },
    /// チェックリスト項目を削除
    Delete {
        /// 項目 ID
        id: i64,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let conn = crate::db::open_db()?;

    match cli.command {
        Command::Db { action } => match action {
            DbAction::Reset => {
                crate::db::reset_db(&conn)?;
                println!("【開発用】データベースを初期化しました");
                println!("  - checklist_items / itinerary_items / trips の全データを削除");
                println!("  - AUTOINCREMENT の採番をリセット");
            }
        },
        Command::Itinerary { action } => match action {
            ItineraryAction::Add {
                trip_id,
                day,
                title,
                note,
                time,
                order,
                duration,
                travel,
                location,
            } => {
                let id = crate::itinerary::add_itinerary_item(
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
                )?;
                println!("日程を追加しました (ID: {id})");
                println!("  旅行 ID : {trip_id}");
                println!("  日目    : {day}");
                println!("  時刻    : {}", crate::itinerary::fmt_text(&time));
                println!("  並び順  : {}", order.unwrap_or(0));
                println!("  所要時間: {}", crate::itinerary::fmt_minutes(duration));
                println!("  移動時間: {}", crate::itinerary::fmt_minutes(travel));
                println!("  タイトル: {title}");
                println!("  場所    : {}", crate::itinerary::fmt_text(&location));
                println!("  メモ    : {}", crate::itinerary::fmt_text(&note));
            }
            ItineraryAction::List { trip_id } => {
                let items = crate::itinerary::list_itinerary_items(&conn, trip_id)?;
                println!("旅行 ID {trip_id} の日程:");
                crate::itinerary::print_itinerary_list(&items);
            }
            ItineraryAction::Timeline { trip_id } => {
                let items = crate::itinerary::list_itinerary_items(&conn, trip_id)?;
                let trip = crate::trip::get_trip(&conn, trip_id)?;
                println!("{} のタイムライン:", trip.name);
                println!();
                crate::itinerary::print_itinerary_timeline(&items);
            }
            ItineraryAction::Show { id } => {
                let item = crate::itinerary::get_itinerary_item(&conn, id)?;
                crate::itinerary::print_itinerary_detail(&item);
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
                    Some(value) => Some(Some(crate::models::parse_itinerary_category(value)?)),
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
        },
        Command::Checklist { action } => match action {
            ChecklistAction::Add { trip_id, title } => {
                let id = crate::checklist::add_checklist_item(&conn, trip_id, &title)?;
                println!("チェックリスト項目を追加しました (ID: {id})");
                println!("  旅行 ID : {trip_id}");
                println!("  タイトル: {title}");
            }
            ChecklistAction::List { trip_id } => {
                let items = crate::checklist::list_checklist_items(&conn, trip_id)?;
                println!("旅行 ID {trip_id} のチェックリスト:");
                crate::checklist::print_checklist_list(&items);
            }
            ChecklistAction::Show { id } => {
                let item = crate::checklist::get_checklist_item(&conn, id)?;
                crate::checklist::print_checklist_detail(&item);
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
        Command::Trip { action } => match action {
            TripAction::Add { name, start, end } => {
                let id = crate::trip::add_trip(&conn, &name, start.as_deref(), end.as_deref())?;
                println!("旅行を追加しました (ID: {id})");
                println!("  名前   : {name}");
                println!("  開始日 : {}", crate::trip::fmt_date(&start));
                println!("  終了日 : {}", crate::trip::fmt_date(&end));
            }
            TripAction::List => {
                let trips = crate::trip::list_trips(&conn)?;
                crate::trip::print_trip_list(&trips);
            }
            TripAction::Show { id } => {
                let trip = crate::trip::get_trip(&conn, id)?;
                crate::trip::print_trip_detail(&trip);
            }
            TripAction::Update {
                id,
                name,
                start,
                end,
            } => {
                crate::trip::update_trip(
                    &conn,
                    id,
                    name.as_deref(),
                    start.as_deref(),
                    end.as_deref(),
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
            TripAction::Export { id, output } => {
                crate::trip::write_trip_export(&conn, id, output.as_deref())?;
            }
            TripAction::ExportMd { id, output } => {
                crate::markdown::write_trip_markdown(&conn, id, output.as_deref())?;
            }
            TripAction::Import { file } => {
                let new_id = crate::trip::import_trip_from_file(&conn, &file)?;
                let trip = crate::trip::get_trip(&conn, new_id)?;
                let items = crate::itinerary::list_itinerary_items(&conn, new_id)?;
                println!("旅行をインポートしました (ID: {new_id})");
                println!("  名前: {}", trip.name);
                println!("  日程: {} 件", items.len());
            }
            TripAction::Diff { old_file, new_file } => {
                crate::diff::run_trip_diff(&old_file, &new_file)?;
            }
            TripAction::ChecklistGenerate { id } => {
                let result = crate::checklist::generate_checklist_from_itinerary(&conn, id)?;
                crate::checklist::print_checklist_generate_result(&result);
            }
            TripAction::Stats { trip_id } => {
                crate::stats::print_trip_stats(&conn, trip_id)?;
            }
            TripAction::Doctor { trip_id } => {
                crate::doctor::run_trip_doctor(&conn, trip_id)?;
            }
            TripAction::Advisor {
                trip_id,
                with_commands,
            } => {
                crate::advisor::run_trip_advisor(&conn, trip_id, with_commands)?;
            }
        },
    }

    Ok(())
}
