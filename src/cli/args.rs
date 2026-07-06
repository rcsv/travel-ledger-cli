use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "caglla",
    author,
    version,
    about,
    long_about = None,
    next_line_help = true
)]
pub struct Cli {
    /// Print a short English overview and exit
    #[arg(long)]
    pub about: bool,

    /// SQLite database file path (overrides CAGLLA_DB and caglla.toml)
    #[arg(long, value_name = "PATH", global = true)]
    pub db: Option<std::path::PathBuf>,

    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Subcommand)]
pub enum Command {
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
    /// データベース操作
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
    /// 事前見積 (Estimate) の管理
    Estimate {
        #[command(subcommand)]
        action: EstimateAction,
    },
    /// 予約 (Reservation) の管理
    Reservation {
        #[command(subcommand)]
        action: ReservationAction,
    },
    /// Receipt Inbox（支払い証拠メタデータ）の管理
    Receipt {
        #[command(subcommand)]
        action: ReceiptAction,
    },
    /// 参加者 (Participant) の管理
    Participant {
        #[command(subcommand)]
        action: ParticipantAction,
    },
    /// Trip Proposal Envelope（未採用案）の file 検証
    Proposal {
        #[command(subcommand)]
        action: ProposalAction,
    },
    /// Proposal Fragment（既存 Trip への部分提案）の file 検証
    Fragment {
        #[command(subcommand)]
        action: FragmentAction,
    },
}

#[derive(Subcommand)]
pub enum DbAction {
    /// 使用中の DB ファイルパスを表示（ファイルは作成しない）
    Path,
    /// DB ファイルの存在と概要を表示
    Status {
        /// JSON 形式で出力
        #[arg(long)]
        json: bool,
    },
    /// このディレクトリの既定 DB を caglla.toml に記録（DB は開かない）
    Use {
        /// データベースファイルパス（CWD 基準）
        path: Option<std::path::PathBuf>,
        /// 記録を消去し default ./caglla.db に戻す
        #[arg(long, conflicts_with = "path")]
        clear: bool,
    },
    /// 【開発用】全データを削除して DB を初期状態に戻す（本番運用では使わない）
    Reset,
}

#[derive(Subcommand)]
pub enum TripAction {
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
pub enum ItineraryAction {
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
        #[arg(long, conflicts_with_all = ["after", "before"])]
        order: Option<i64>,
        /// 指定 Itinerary の直後に追加
        #[arg(long, conflicts_with = "before")]
        after: Option<i64>,
        /// 指定 Itinerary の直前に追加
        #[arg(long, conflicts_with = "after")]
        before: Option<i64>,
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
    /// Day 内の sort_order を正規化（1000, 2000, 3000...）
    Normalize {
        /// 旅行 ID
        trip_id: i64,
        /// 何日目か
        #[arg(long)]
        day: i64,
    },
    /// 日程を別の位置へ移動
    Move {
        /// 日程 ID
        id: i64,
        /// 指定 Itinerary の直後へ移動
        #[arg(long, conflicts_with = "before")]
        after: Option<i64>,
        /// 指定 Itinerary の直前へ移動
        #[arg(long, conflicts_with = "after")]
        before: Option<i64>,
    },
    /// 既存 Itinerary を指定 Day 群へ複製
    Replicate {
        /// 複製元 Itinerary ID（カンマ区切り）
        #[arg(long)]
        items: String,
        /// コピー先 Day（例: 3, 3-5, 2,4-6）
        #[arg(long)]
        to_days: String,
        /// Itinerary-level notes をコピーしない
        #[arg(long)]
        without_notes: bool,
        /// DB を更新せず、作成予定のみ表示
        #[arg(long)]
        dry_run: bool,
    },
}

#[derive(Subcommand)]
pub enum ChecklistAction {
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
pub enum DayAction {
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
pub enum NoteAction {
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
pub enum ReservationAction {
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
pub enum ExpenseAction {
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
        /// 構造化 payer（Participant ID または name）
        #[arg(long)]
        paid_by_participant: Option<String>,
        /// shared beneficiary（繰り返し可）
        #[arg(long)]
        beneficiary: Vec<String>,
        /// Trip 全 Participant を beneficiary に展開
        #[arg(long)]
        shared_with: Option<String>,
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
        /// 構造化 payer（Participant ID または name）
        #[arg(long)]
        paid_by_participant: Option<String>,
        /// shared beneficiary（繰り返し可、指定時は全置換）
        #[arg(long)]
        beneficiary: Vec<String>,
        /// Trip 全 Participant を beneficiary に展開（全置換）
        #[arg(long)]
        shared_with: Option<String>,
        /// payer ID と paid_by_name をクリア
        #[arg(long)]
        clear_paid_by: bool,
        /// beneficiary を全削除（personal に戻す）
        #[arg(long)]
        clear_beneficiaries: bool,
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
pub enum EstimateAction {
    /// Estimate を追加
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
        /// 並び順
        #[arg(long)]
        sort_order: Option<i64>,
    },
    /// Estimate 一覧を表示
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
    /// Estimate 詳細を表示
    Show {
        /// Estimate ID
        id: i64,
        /// JSON 形式で出力する
        #[arg(long)]
        json: bool,
    },
    /// Estimate を更新
    Update {
        /// Estimate ID
        id: i64,
        /// タイトル
        #[arg(long)]
        title: Option<String>,
        /// メモ
        #[arg(long)]
        note: Option<String>,
        /// 金額
        #[arg(long)]
        amount: Option<String>,
        /// 通貨コード
        #[arg(long)]
        currency: Option<String>,
        /// 並び順
        #[arg(long)]
        sort_order: Option<i64>,
        /// title をクリア
        #[arg(long)]
        clear_title: bool,
        /// note をクリア
        #[arg(long)]
        clear_note: bool,
    },
    /// Estimate を削除
    Delete {
        /// Estimate ID
        id: i64,
    },
}

#[derive(Subcommand)]
pub enum ReceiptAction {
    /// Receipt を追加（metadata-only）
    Add {
        /// Trip ID（必須）
        #[arg(long)]
        trip: i64,
        /// 日目（任意）
        #[arg(long)]
        day: Option<i64>,
        /// 金額（任意。`--currency` とセット）
        #[arg(long)]
        amount: Option<String>,
        /// 通貨（任意。`--amount` とセット）
        #[arg(long)]
        currency: Option<String>,
        /// 支払い日（YYYY-MM-DD）
        #[arg(long = "occurred-date")]
        occurred_date: Option<String>,
        /// メモ（任意）
        #[arg(long)]
        memo: Option<String>,
    },
    /// Receipt 一覧を表示
    List {
        /// Trip ID（必須）
        #[arg(long)]
        trip: i64,
        /// Trash のみ表示する（`trashed_at IS NOT NULL`）
        #[arg(long)]
        trashed: bool,
        /// Trash を含めて表示する（default は active のみ）
        #[arg(long)]
        all: bool,
        /// 未確認（`unreviewed`）のみ
        #[arg(long)]
        unreviewed: bool,
        /// status で絞り込み（`unreviewed` / `ignored`）
        #[arg(long)]
        status: Option<String>,
        /// JSON 形式で出力する
        #[arg(long)]
        json: bool,
    },
    /// Receipt を Itinerary に割り当てて Expense 化する（Receipt は削除される）
    Assign {
        /// Receipt ID
        id: i64,
        /// Itinerary ID（必須）
        #[arg(long)]
        itinerary: i64,
        /// assign 時に金額を補完する（`--currency` とセット）
        #[arg(long)]
        amount: Option<String>,
        /// assign 時に通貨を補完する（`--amount` とセット）
        #[arg(long)]
        currency: Option<String>,
        /// assign 時にメモを補完する（Expense title の候補になる）
        #[arg(long)]
        memo: Option<String>,
    },
    /// Receipt を Trash に移動する（物理削除しない）
    Trash {
        /// Receipt ID
        id: i64,
    },
    /// Trash から Receipt を復元する（Inbox に戻す）
    Restore {
        /// Receipt ID
        id: i64,
    },
    /// Receipt 詳細を表示
    Show {
        /// Receipt ID
        id: i64,
        /// JSON 形式で出力する
        #[arg(long)]
        json: bool,
    },
    /// Receipt を更新
    Update {
        /// Receipt ID
        id: i64,
        /// 日目
        #[arg(long)]
        day: Option<i64>,
        /// 金額（`--currency` とセット）
        #[arg(long)]
        amount: Option<String>,
        /// 通貨（`--amount` とセット）
        #[arg(long)]
        currency: Option<String>,
        /// 支払い日（空文字でクリア）
        #[arg(long = "occurred-date")]
        occurred_date: Option<String>,
        /// メモ（空文字でクリア）
        #[arg(long)]
        memo: Option<String>,
        /// day 紐づけをクリア
        #[arg(long)]
        clear_day: bool,
        /// amount / currency をクリア
        #[arg(long)]
        clear_amount: bool,
        /// occurred_date をクリア
        #[arg(long)]
        clear_occurred_date: bool,
        /// memo をクリア
        #[arg(long)]
        clear_memo: bool,
    },
    /// Receipt を対象外（`ignored`）にする
    Ignore {
        /// Receipt ID
        id: i64,
        /// 追記または更新するメモ
        #[arg(long)]
        memo: Option<String>,
    },
    /// Receipt を削除
    Delete {
        /// Receipt ID
        id: i64,
    },
}

#[derive(Subcommand)]
pub enum ParticipantAction {
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

#[derive(Subcommand)]
pub enum ProposalAction {
    /// Trip Proposal Envelope JSON ファイルを検証する（schema v8 Trip とは別責務）
    Validate {
        /// 検証する JSON ファイル
        file: String,
        /// JSON 形式で出力
        #[arg(long)]
        json: bool,
    },
    /// Trip Proposal Envelope の概要を表示する
    Show {
        /// 表示する JSON ファイル
        file: String,
    },
    /// Trip Proposal Envelope の構造と validation 詳細を表示する
    Inspect {
        /// 詳細表示する JSON ファイル
        file: String,
    },
    /// Trip Proposal Envelope から schema v8 Trip JSON 候補を生成する（file-only）
    Materialize {
        /// 対象の Trip Proposal Envelope JSON ファイル
        file: String,
        /// Dry-run — DB に書き込まず schema v8 Trip JSON 候補のみ生成（--confirm と併用不可）
        #[arg(long, conflicts_with = "confirm")]
        dry_run: bool,
        /// 明示的採用 — gate 通過後に新規 Trip として DB に保存（--dry-run と併用不可）
        #[arg(long, conflicts_with = "dry_run")]
        confirm: bool,
        /// 生成した schema v8 Trip JSON 候補の出力先。後続の trip validate-export 等では --output を推奨（省略時は human モードで stdout に混在）
        #[arg(long)]
        output: Option<String>,
        /// 旅行開始日（YYYY-MM-DD）— Envelope 内で未確定のときに指定
        #[arg(long)]
        start: Option<String>,
        /// 旅行終了日（YYYY-MM-DD）— Envelope 内で未確定のときに指定
        #[arg(long)]
        end: Option<String>,
        /// materialize gate report を JSON 出力（Trip JSON 候補そのものではない。候補は --output へ）
        #[arg(long)]
        json: bool,
    },
}

#[derive(Subcommand)]
pub enum FragmentAction {
    /// Proposal Fragment JSON ファイルを検証する（schema v8 Trip / Envelope とは別責務）
    Validate {
        /// 検証する JSON ファイル
        file: String,
        /// JSON 形式で出力
        #[arg(long)]
        json: bool,
    },
    /// Proposal Fragment を既存 Trip に適用する apply preview / simulation（file + DB read のみ）
    Apply {
        /// 対象の Proposal Fragment JSON ファイル
        file: String,
        /// Dry-run — apply preview / simulation（read-only DB access、Trip domain data 更新なし）
        #[arg(long)]
        dry_run: bool,
        /// 適用先 Trip ID
        #[arg(long)]
        trip: i64,
        /// apply preview（schema v8 Trip JSON）の出力先。後続の trip diff 等では --output を推奨
        #[arg(long)]
        output: Option<String>,
        /// apply gate report を JSON 出力（preview Trip JSON そのものではない）
        #[arg(long)]
        json: bool,
    },
}
