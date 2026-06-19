mod advisor;
mod checklist;
mod day;
mod db;
mod diff;
mod doctor;
mod expense;
mod itinerary;
mod markdown;
mod models;
mod note;
mod participant;
mod reservation;
mod stats;
mod summary;
mod trip;

use anyhow::Result;
use clap::{Parser, Subcommand};

// ---------------------------------------------------------------------------
// CLI 定義（clap derive）
// ---------------------------------------------------------------------------

#[derive(Parser)]
#[command(
    name = "caglla",
    author,
    version,
    about,
    long_about = None,
    next_line_help = true
)]
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
    /// 日 (Day) の閲覧・入れ替え
    Day {
        #[command(subcommand)]
        action: DayAction,
    },
    /// メモ (Note) の管理
    Note {
        #[command(subcommand)]
        action: NoteAction,
    },
    /// 支出 (Expense) の管理
    Expense {
        #[command(subcommand)]
        action: ExpenseAction,
    },
    /// 予約 (Reservation) の管理
    Reservation {
        #[command(subcommand)]
        action: ReservationAction,
    },
    /// 参加者 (Participant) の管理
    Participant {
        #[command(subcommand)]
        action: ParticipantAction,
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
        #[arg(long, required = true)]
        start: String,
        /// 終了日 (YYYY-MM-DD)
        #[arg(long, required = true)]
        end: String,
        /// 旅行の概要（任意）
        #[arg(long)]
        summary: Option<String>,
    },
    /// 旅行一覧を表示
    List {
        /// JSON 形式で出力する
        #[arg(long)]
        json: bool,
    },
    /// 旅行の詳細を表示
    Show {
        /// 旅行 ID
        id: i64,
        /// JSON 形式で出力する
        #[arg(long)]
        json: bool,
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
        /// 旅行の概要
        #[arg(long)]
        summary: Option<String>,
        /// 旅行の概要をクリアする
        #[arg(long)]
        clear_summary: bool,
    },
    /// 旅行を削除
    Delete {
        /// 旅行 ID
        id: i64,
    },
    /// 旅行を複製（Trip / Itinerary / Checklist）
    Duplicate {
        /// 複製元の旅行 ID
        id: i64,
        /// 複製後の旅行名（省略時は「元の名前 (Copy)」）
        #[arg(long)]
        name: Option<String>,
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
    /// export JSON ファイルの健全性を検証する
    #[command(name = "validate-export")]
    ValidateExport {
        /// 検証する JSON ファイル
        file: String,
        /// JSON 形式で出力
        #[arg(long)]
        json: bool,
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
        /// DB を更新せず、追加・スキップ候補のみ表示する
        #[arg(long)]
        dry_run: bool,
    },
    /// 旅行の統計を表示
    Stats {
        /// 旅行 ID
        trip_id: i64,
        /// JSON 形式で出力する
        #[arg(long)]
        json: bool,
    },
    /// 旅行計画を点検する
    Doctor {
        /// 旅行 ID
        trip_id: i64,
        /// JSON 形式で出力
        #[arg(long)]
        json: bool,
    },
    /// 旅行計画の改善提案を表示する
    Advisor {
        /// 旅行 ID
        trip_id: i64,
        /// 改善提案に加えて次に試せる CLI コマンド例を表示する
        #[arg(long)]
        with_commands: bool,
        /// JSON 形式で出力
        #[arg(long)]
        json: bool,
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
        /// JSON 形式で出力する
        #[arg(long)]
        json: bool,
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
        /// JSON 形式で出力する
        #[arg(long)]
        json: bool,
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
        /// JSON 形式で出力
        #[arg(long)]
        json: bool,
    },
    /// チェックリスト項目の詳細を表示
    Show {
        /// 項目 ID
        id: i64,
        /// JSON 形式で出力
        #[arg(long)]
        json: bool,
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

#[derive(Subcommand)]
enum DayAction {
    /// 旅行の Day 一覧を表示
    List {
        /// 旅行 ID
        trip_id: i64,
        /// JSON 形式で出力する
        #[arg(long)]
        json: bool,
    },
    /// Day 詳細と配下の Itinerary を表示
    Show {
        /// 旅行 ID
        trip_id: i64,
        /// 何日目か
        day_number: i64,
        /// JSON 形式で出力する
        #[arg(long)]
        json: bool,
    },
    /// Day の概要を更新
    Update {
        /// 旅行 ID
        trip_id: i64,
        /// 何日目か
        day_number: i64,
        /// その日の概要
        #[arg(long)]
        summary: Option<String>,
        /// 概要をクリアする
        #[arg(long)]
        clear_summary: bool,
    },
    /// 2 つの Day 配下の Itinerary を入れ替える
    Swap {
        /// 旅行 ID
        trip_id: i64,
        /// 入れ替え元の日目
        day_a: i64,
        /// 入れ替え先の日目
        day_b: i64,
    },
}

#[derive(Subcommand)]
enum NoteAction {
    /// Note を追加
    Add {
        /// Trip ID（Trip Note / Day Note）
        #[arg(long)]
        trip: Option<i64>,
        /// 日目（Day Note のとき --trip とセット）
        #[arg(long)]
        day: Option<i64>,
        /// Itinerary ID（Itinerary Note）
        #[arg(long)]
        itinerary: Option<i64>,
        /// タイトル（任意）
        #[arg(long)]
        title: Option<String>,
        /// 本文（必須）
        #[arg(long)]
        body: String,
    },
    /// owner 配下の Note 一覧を表示
    List {
        /// Trip ID
        #[arg(long)]
        trip: Option<i64>,
        /// 日目
        #[arg(long)]
        day: Option<i64>,
        /// Itinerary ID
        #[arg(long)]
        itinerary: Option<i64>,
        /// JSON 形式で出力する
        #[arg(long)]
        json: bool,
    },
    /// Note 詳細を表示
    Show {
        /// Note ID
        id: i64,
        /// JSON 形式で出力する
        #[arg(long)]
        json: bool,
    },
    /// Note を更新
    Update {
        /// Note ID
        id: i64,
        /// 新しいタイトル
        #[arg(long)]
        title: Option<String>,
        /// 新しい本文
        #[arg(long)]
        body: Option<String>,
    },
    /// Note を削除
    Delete {
        /// Note ID
        id: i64,
    },
}

#[derive(Subcommand)]
enum ReservationAction {
    /// Reservation を追加
    Add {
        /// Itinerary ID（必須）
        #[arg(long)]
        itinerary: i64,
        /// 予約種別（hotel, flight, rental_car, …）
        #[arg(long)]
        reservation_type: String,
        /// 事業者名（必須）
        #[arg(long)]
        provider: String,
        /// 予約番号・確認コード
        #[arg(long)]
        confirmation: Option<String>,
        /// 予約確認ページ URL
        #[arg(long)]
        site_url: Option<String>,
        /// 短文補足
        #[arg(long)]
        remark: Option<String>,
        /// 利用開始（ISO 8601 または日時文字列）
        #[arg(long)]
        start_at: Option<String>,
        /// 利用終了
        #[arg(long)]
        end_at: Option<String>,
    },
    /// Reservation 一覧を表示
    List {
        /// Trip ID（Trip 配下を集約表示）
        #[arg(long)]
        trip: Option<i64>,
        /// Itinerary ID
        #[arg(long)]
        itinerary: Option<i64>,
        /// JSON 形式で出力する
        #[arg(long)]
        json: bool,
    },
    /// Reservation 詳細を表示
    Show {
        /// Reservation ID
        id: i64,
        /// JSON 形式で出力する
        #[arg(long)]
        json: bool,
    },
    /// Reservation を更新
    Update {
        /// Reservation ID
        id: i64,
        /// 予約種別
        #[arg(long)]
        reservation_type: Option<String>,
        /// 事業者名
        #[arg(long)]
        provider: Option<String>,
        /// 予約番号・確認コード（空文字でクリア）
        #[arg(long)]
        confirmation: Option<String>,
        /// 予約確認ページ URL（空文字でクリア）
        #[arg(long)]
        site_url: Option<String>,
        /// 短文補足（空文字でクリア）
        #[arg(long)]
        remark: Option<String>,
        /// 利用開始（空文字でクリア）
        #[arg(long)]
        start_at: Option<String>,
        /// 利用終了（空文字でクリア）
        #[arg(long)]
        end_at: Option<String>,
        /// confirmation をクリアする
        #[arg(long)]
        clear_confirmation: bool,
        /// site_url をクリアする
        #[arg(long)]
        clear_site_url: bool,
        /// remark をクリアする
        #[arg(long)]
        clear_remark: bool,
        /// start_at をクリアする
        #[arg(long)]
        clear_start_at: bool,
        /// end_at をクリアする
        #[arg(long)]
        clear_end_at: bool,
    },
    /// Reservation を削除
    Delete {
        /// Reservation ID
        id: i64,
    },
}

#[derive(Subcommand)]
enum ExpenseAction {
    /// Expense を追加
    Add {
        /// Itinerary ID（必須）
        #[arg(long)]
        itinerary: i64,
        /// 金額（必須。JPY は整数、USD は小数可）
        #[arg(long)]
        amount: String,
        /// 通貨コード（必須。例: JPY, USD）
        #[arg(long)]
        currency: String,
        /// タイトル（任意）
        #[arg(long)]
        title: Option<String>,
        /// メモ（任意）
        #[arg(long)]
        note: Option<String>,
        /// 支払者名（任意）
        #[arg(long)]
        paid_by_name: Option<String>,
        /// 支出日 YYYY-MM-DD（任意）
        #[arg(long)]
        expense_date: Option<String>,
    },
    /// Expense 一覧を表示
    List {
        /// Trip ID（Trip 配下を集約表示）
        #[arg(long)]
        trip: Option<i64>,
        /// Itinerary ID
        #[arg(long)]
        itinerary: Option<i64>,
        /// JSON 形式で出力する
        #[arg(long)]
        json: bool,
    },
    /// Expense 詳細を表示
    Show {
        /// Expense ID
        id: i64,
        /// JSON 形式で出力する
        #[arg(long)]
        json: bool,
    },
    /// Expense を更新
    Update {
        /// Expense ID
        id: i64,
        /// タイトル
        #[arg(long)]
        title: Option<String>,
        /// 金額
        #[arg(long)]
        amount: Option<String>,
        /// 通貨コード
        #[arg(long)]
        currency: Option<String>,
        /// 支払者名
        #[arg(long)]
        paid_by_name: Option<String>,
        /// 支出日 YYYY-MM-DD
        #[arg(long)]
        expense_date: Option<String>,
        /// メモ
        #[arg(long)]
        note: Option<String>,
    },
    /// Expense を削除
    Delete {
        /// Expense ID
        id: i64,
    },
}

#[derive(Subcommand)]
enum ParticipantAction {
    /// Participant を追加
    Add {
        /// Trip ID（必須）
        #[arg(long)]
        trip: i64,
        /// 表示名（必須）
        #[arg(long)]
        name: String,
        /// 並び順
        #[arg(long)]
        sort_order: Option<i64>,
        /// この Trip における自分としてマーク
        #[arg(long = "self")]
        self_marker: bool,
    },
    /// Trip 内 Participant 一覧を表示
    List {
        /// Trip ID（必須）
        #[arg(long)]
        trip: i64,
        /// JSON 形式で出力する
        #[arg(long)]
        json: bool,
    },
    /// Participant 詳細を表示
    Show {
        /// Participant ID
        id: i64,
        /// JSON 形式で出力する
        #[arg(long)]
        json: bool,
    },
    /// Participant を更新
    Update {
        /// Participant ID
        id: i64,
        /// 表示名
        #[arg(long)]
        name: Option<String>,
        /// 並び順
        #[arg(long)]
        sort_order: Option<i64>,
        /// この Trip における自分としてマーク
        #[arg(long = "self")]
        self_marker: bool,
        /// self マーカーを外す
        #[arg(long = "not-self")]
        not_self: bool,
    },
    /// Participant を削除
    Delete {
        /// Participant ID
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
            ItineraryAction::List { trip_id, json } => {
                let items = crate::itinerary::list_itinerary_items(&conn, trip_id)?;
                if json {
                    crate::trip::print_json(&items)?;
                } else {
                    println!("旅行 ID {trip_id} の日程:");
                    crate::itinerary::print_itinerary_list(&items);
                }
            }
            ItineraryAction::Timeline { trip_id } => {
                let items = crate::itinerary::list_itinerary_items(&conn, trip_id)?;
                let trip = crate::trip::get_trip(&conn, trip_id)?;
                println!("{} のタイムライン:", trip.name);
                println!();
                crate::itinerary::print_itinerary_timeline(&items);
            }
            ItineraryAction::Show { id, json } => {
                let item = crate::itinerary::get_itinerary_item(&conn, id)?;
                if json {
                    crate::trip::print_json(&item)?;
                } else {
                    crate::itinerary::print_itinerary_detail(&item);
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
            ChecklistAction::List { trip_id, json } => {
                let items = crate::checklist::list_checklist_items(&conn, trip_id)?;
                if json {
                    crate::trip::print_json(&items)?;
                } else {
                    println!("旅行 ID {trip_id} のチェックリスト:");
                    crate::checklist::print_checklist_list(&items);
                }
            }
            ChecklistAction::Show { id, json } => {
                let item = crate::checklist::get_checklist_item(&conn, id)?;
                if json {
                    crate::trip::print_json(&item)?;
                } else {
                    crate::checklist::print_checklist_detail(&item);
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
                crate::day::run_day_list(&conn, trip_id, json)?;
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
                let updated = crate::day::swap_day_itineraries(&conn, trip_id, day_a, day_b)?;
                println!("Day {day_a} と Day {day_b} の日程を入れ替えました");
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
                let owner = crate::note::resolve_note_owner_for_list(&conn, trip, day, itinerary)?;
                let notes =
                    crate::note::list_notes_for_owner(&conn, owner.owner_type(), owner.owner_id())?;
                if json {
                    crate::trip::print_json(&crate::note::NoteListJson {
                        owner_type: owner.owner_type(),
                        owner_id: owner.owner_id(),
                        notes,
                    })?;
                } else {
                    crate::note::print_note_list(owner.owner_type(), owner.owner_id(), &notes);
                }
            }
            NoteAction::Show { id, json } => {
                let note = crate::note::get_note(&conn, id)?;
                if json {
                    crate::trip::print_json(&note)?;
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
                expense_date,
            } => {
                let id = crate::expense::add_expense(
                    &conn,
                    itinerary,
                    &amount,
                    &currency,
                    title.as_deref(),
                    note.as_deref(),
                    paid_by_name.as_deref(),
                    expense_date.as_deref(),
                )?;
                println!("Expense を追加しました (ID: {id})");
                let expense = crate::expense::get_expense(&conn, id)?;
                crate::expense::print_expense_detail(&expense);
            }
            ExpenseAction::List {
                trip,
                itinerary,
                json,
            } => {
                let target = crate::expense::resolve_expense_list_target(trip, itinerary)?;
                let expenses = match target {
                    crate::expense::ExpenseListTarget::Trip(trip_id) => {
                        crate::expense::list_expenses_for_trip(&conn, trip_id)?
                    }
                    crate::expense::ExpenseListTarget::Itinerary(itinerary_id) => {
                        crate::expense::list_expenses_for_itinerary(&conn, itinerary_id)?
                    }
                };
                if json {
                    let (trip_id, itinerary_id) = match target {
                        crate::expense::ExpenseListTarget::Trip(id) => (Some(id), None),
                        crate::expense::ExpenseListTarget::Itinerary(id) => (None, Some(id)),
                    };
                    crate::trip::print_json(&crate::expense::ExpenseListJson {
                        trip_id,
                        itinerary_id,
                        expenses,
                    })?;
                } else {
                    crate::expense::print_expense_list(target, &expenses);
                }
            }
            ExpenseAction::Show { id, json } => {
                let expense = crate::expense::get_expense(&conn, id)?;
                if json {
                    crate::trip::print_json(&expense)?;
                } else {
                    crate::expense::print_expense_detail(&expense);
                }
            }
            ExpenseAction::Update {
                id,
                title,
                amount,
                currency,
                paid_by_name,
                expense_date,
                note,
            } => {
                crate::expense::update_expense(
                    &conn,
                    id,
                    title.as_deref(),
                    amount.as_deref(),
                    currency.as_deref(),
                    paid_by_name.as_deref(),
                    expense_date.as_deref(),
                    note.as_deref(),
                )?;
                println!("Expense を更新しました (ID: {id})");
                let expense = crate::expense::get_expense(&conn, id)?;
                crate::expense::print_expense_detail(&expense);
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
                let target = crate::reservation::resolve_reservation_list_target(trip, itinerary)?;
                match target {
                    crate::reservation::ReservationListTarget::Trip(trip_id) => {
                        let context_rows =
                            crate::reservation::list_reservations_for_trip(&conn, trip_id)?;
                        let reservations = context_rows
                            .iter()
                            .map(|row| row.reservation.clone())
                            .collect::<Vec<_>>();
                        if json {
                            crate::trip::print_json(&crate::reservation::ReservationListJson {
                                trip_id: Some(trip_id),
                                itinerary_id: None,
                                reservations,
                            })?;
                        } else {
                            crate::reservation::print_reservation_list(
                                target,
                                &reservations,
                                Some(&context_rows),
                            );
                        }
                    }
                    crate::reservation::ReservationListTarget::Itinerary(itinerary_id) => {
                        let reservations = crate::reservation::list_reservations_for_itinerary(
                            &conn,
                            itinerary_id,
                        )?;
                        if json {
                            crate::trip::print_json(&crate::reservation::ReservationListJson {
                                trip_id: None,
                                itinerary_id: Some(itinerary_id),
                                reservations: reservations.clone(),
                            })?;
                        } else {
                            crate::reservation::print_reservation_list(target, &reservations, None);
                        }
                    }
                }
            }
            ReservationAction::Show { id, json } => {
                let reservation = crate::reservation::get_reservation(&conn, id)?;
                if json {
                    crate::trip::print_json(&reservation)?;
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
                    crate::trip::print_json(&crate::participant::ParticipantListJson {
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
                    crate::trip::print_json(&participant)?;
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
                let trips = crate::trip::list_trips(&conn)?;
                if json {
                    crate::trip::print_json(&trips)?;
                } else {
                    crate::trip::print_trip_list(&trips);
                }
            }
            TripAction::Show { id, json } => {
                let trip = crate::trip::get_trip(&conn, id)?;
                if json {
                    crate::trip::print_json(&trip)?;
                } else {
                    crate::trip::print_trip_detail(&trip);
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
                crate::markdown::write_trip_markdown(&conn, id, output.as_deref())?;
            }
            TripAction::Import { file } => {
                crate::trip::run_trip_import(&conn, &file)?;
            }
            TripAction::ValidateExport { file, json } => {
                crate::trip::run_trip_validate_export(&file, json)?;
            }
            TripAction::Diff { old_file, new_file } => {
                crate::diff::run_trip_diff(&old_file, &new_file)?;
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
                if json {
                    let stats = crate::stats::compute_trip_stats(&conn, trip_id)?;
                    crate::trip::print_json(&stats)?;
                } else {
                    crate::stats::print_trip_stats(&conn, trip_id)?;
                }
            }
            TripAction::Doctor { trip_id, json } => {
                crate::doctor::run_trip_doctor(&conn, trip_id, json)?;
            }
            TripAction::Advisor {
                trip_id,
                with_commands,
                json,
            } => {
                crate::advisor::run_trip_advisor(&conn, trip_id, with_commands, json)?;
            }
        },
    }

    Ok(())
}
