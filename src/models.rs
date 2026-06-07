use anyhow::Result;
use serde::{Deserialize, Serialize};

/// days テーブルの1行分のデータ
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Day {
    pub id: i64,
    pub trip_id: i64,
    pub day_number: i64,
    pub title: String,
    pub description: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// trips テーブルの1行分のデータ
#[derive(Clone, Serialize, Deserialize)]
pub struct Trip {
    pub id: i64,
    pub name: String,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// trip doctor / advisor が検出する問題種別
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DoctorIssueCode {
    EmptyItinerary,
    OverloadedDay,
    NoRestaurant,
    HighTravelTime,
    MissingDuration,
}

/// trip doctor / advisor が検出した問題の対象（内部モデル）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DoctorIssueTarget {
    Trip,
    Day(i64),
    Itinerary(i64),
}

/// JSON 出力用の issue 対象種別
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IssueTargetType {
    Trip,
    Day,
    Itinerary,
}

/// JSON 出力用の issue 対象（`target.id` の意味は `type` 依存）
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct IssueTarget {
    #[serde(rename = "type")]
    pub target_type: IssueTargetType,
    pub id: i64,
}

impl IssueTarget {
    pub fn from_doctor_target(target: DoctorIssueTarget, trip_id: i64) -> Self {
        match target {
            DoctorIssueTarget::Trip => Self {
                target_type: IssueTargetType::Trip,
                id: trip_id,
            },
            DoctorIssueTarget::Day(day) => Self {
                target_type: IssueTargetType::Day,
                id: day,
            },
            DoctorIssueTarget::Itinerary(id) => Self {
                target_type: IssueTargetType::Itinerary,
                id,
            },
        }
    }
}

/// JSON 出力用の issue 付加情報（`code` ごとに使用フィールドが決まる）
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct IssueDetails {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub day: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub itinerary_id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub itinerary_count: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub travel_minutes: Option<i64>,
}

impl IssueDetails {
    pub fn is_empty(&self) -> bool {
        self.day.is_none()
            && self.itinerary_id.is_none()
            && self.itinerary_count.is_none()
            && self.travel_minutes.is_none()
    }
}

/// trip doctor JSON 出力用の重要度
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DoctorIssueSeverity {
    Info,
    Warning,
}

/// trip doctor JSON 出力用の1件の問題
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DoctorIssueJson {
    pub code: DoctorIssueCode,
    pub severity: DoctorIssueSeverity,
    pub message: String,
    pub target: IssueTarget,
    #[serde(skip_serializing_if = "IssueDetails::is_empty")]
    pub details: IssueDetails,
}

/// trip doctor `--json` の envelope
pub const DOCTOR_REPORT_SCHEMA_VERSION: i32 = 1;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DoctorReportJson {
    pub schema_version: i32,
    pub trip_id: i64,
    pub issues: Vec<DoctorIssueJson>,
}

impl DoctorReportJson {
    pub fn new(trip_id: i64, issues: Vec<DoctorIssueJson>) -> Self {
        Self {
            schema_version: DOCTOR_REPORT_SCHEMA_VERSION,
            trip_id,
            issues,
        }
    }
}

/// trip advisor JSON 出力用の1件（診断 + 改善提案）
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AdvisorIssueJson {
    #[serde(flatten)]
    pub issue: DoctorIssueJson,
    pub advice: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub commands: Vec<String>,
}

/// trip advisor `--json` の envelope
pub const ADVISOR_REPORT_SCHEMA_VERSION: i32 = 1;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AdvisorReportJson {
    pub schema_version: i32,
    pub trip_id: i64,
    pub with_commands: bool,
    pub issues: Vec<AdvisorIssueJson>,
}

impl AdvisorReportJson {
    pub fn new(trip_id: i64, with_commands: bool, issues: Vec<AdvisorIssueJson>) -> Self {
        Self {
            schema_version: ADVISOR_REPORT_SCHEMA_VERSION,
            trip_id,
            with_commands,
            issues,
        }
    }
}

/// trip doctor / advisor が扱う1件の問題
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DoctorIssue {
    pub code: DoctorIssueCode,
    pub target: DoctorIssueTarget,
    pub day: Option<i64>,
    pub itinerary_count: Option<usize>,
    pub travel_minutes: Option<i64>,
}

impl DoctorIssue {
    /// 対象 day（`DoctorIssueTarget::Day` または `day` フィールド）
    pub fn target_day(&self) -> Option<i64> {
        match self.target {
            DoctorIssueTarget::Day(day) => Some(day),
            _ => self.day,
        }
    }

    /// 対象 itinerary ID（`DoctorIssueTarget::Itinerary`）
    pub fn target_itinerary_id(&self) -> Option<i64> {
        match self.target {
            DoctorIssueTarget::Itinerary(id) => Some(id),
            _ => None,
        }
    }

    /// 警告・Info 表示用の1行メッセージ（advisor および issue 単位の表示）
    pub fn warning_message(&self) -> String {
        match self.code {
            DoctorIssueCode::EmptyItinerary => "No itinerary found.".to_string(),
            DoctorIssueCode::OverloadedDay => format!(
                "Day {} has many itineraries ({})",
                self.target_day().unwrap_or(0),
                self.itinerary_count.unwrap_or(0)
            ),
            DoctorIssueCode::NoRestaurant => {
                format!("Day {} has no restaurant", self.target_day().unwrap_or(0))
            }
            DoctorIssueCode::HighTravelTime => format!(
                "Day {} has high travel time ({})",
                self.target_day().unwrap_or(0),
                crate::stats::format_minutes_duration(self.travel_minutes.unwrap_or(0))
            ),
            DoctorIssueCode::MissingDuration => match self.target {
                DoctorIssueTarget::Itinerary(id) => {
                    format!("Itinerary {id} has no duration estimate")
                }
                _ => "1 itinerary has no duration estimate".to_string(),
            },
        }
    }

    fn issue_severity(&self) -> DoctorIssueSeverity {
        match self.code {
            DoctorIssueCode::EmptyItinerary => DoctorIssueSeverity::Info,
            _ => DoctorIssueSeverity::Warning,
        }
    }

    /// JSON 出力用の `details` を組み立てる
    pub fn to_issue_details(&self) -> IssueDetails {
        match self.code {
            DoctorIssueCode::EmptyItinerary => IssueDetails::default(),
            DoctorIssueCode::OverloadedDay => IssueDetails {
                day: self.target_day(),
                itinerary_count: self.itinerary_count,
                ..IssueDetails::default()
            },
            DoctorIssueCode::NoRestaurant => IssueDetails {
                day: self.target_day(),
                ..IssueDetails::default()
            },
            DoctorIssueCode::HighTravelTime => IssueDetails {
                day: self.target_day(),
                travel_minutes: self.travel_minutes,
                ..IssueDetails::default()
            },
            DoctorIssueCode::MissingDuration => IssueDetails {
                itinerary_id: self.target_itinerary_id(),
                ..IssueDetails::default()
            },
        }
    }

    /// JSON 出力用の表現に変換する
    pub fn to_issue_json(&self, trip_id: i64) -> DoctorIssueJson {
        DoctorIssueJson {
            code: self.code,
            severity: self.issue_severity(),
            message: self.warning_message(),
            target: IssueTarget::from_doctor_target(self.target, trip_id),
            details: self.to_issue_details(),
        }
    }
}

/// 日程カテゴリ（定義済みのみ受け付ける）
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ItineraryCategory {
    Flight,
    Hotel,
    Restaurant,
    Activity,
    Transport,
    Shopping,
    Beach,
    Museum,
}

/// カテゴリの表示名と標準チェックリスト候補
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CategoryDefinition {
    pub display_name: &'static str,
    pub default_checklist: &'static [&'static str],
}

impl ItineraryCategory {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Flight => "flight",
            Self::Hotel => "hotel",
            Self::Restaurant => "restaurant",
            Self::Activity => "activity",
            Self::Transport => "transport",
            Self::Shopping => "shopping",
            Self::Beach => "beach",
            Self::Museum => "museum",
        }
    }

    /// カテゴリ定義（表示名・標準チェックリスト候補）を返す
    pub fn definition(self) -> CategoryDefinition {
        match self {
            Self::Flight => CategoryDefinition {
                display_name: "フライト",
                default_checklist: &["航空券確認", "身分証明書確認", "空港到着時刻確認"],
            },
            Self::Hotel => CategoryDefinition {
                display_name: "ホテル",
                default_checklist: &["宿泊予約確認", "チェックイン時間確認", "住所確認"],
            },
            Self::Restaurant => CategoryDefinition {
                display_name: "食事",
                default_checklist: &["予約確認", "営業時間確認"],
            },
            Self::Activity => CategoryDefinition {
                display_name: "アクティビティ",
                default_checklist: &["予約確認", "所要時間確認", "服装確認"],
            },
            Self::Transport => CategoryDefinition {
                display_name: "移動",
                default_checklist: &["移動手段確認", "所要時間確認"],
            },
            Self::Shopping => CategoryDefinition {
                display_name: "買い物",
                default_checklist: &["営業時間確認", "支払い方法確認"],
            },
            Self::Beach => CategoryDefinition {
                display_name: "ビーチ",
                default_checklist: &["水着", "タオル", "日焼け止め"],
            },
            Self::Museum => CategoryDefinition {
                display_name: "博物館・展示",
                default_checklist: &["営業時間確認", "チケット確認"],
            },
        }
    }

    /// 定義済みの全カテゴリを返す
    pub fn all() -> [Self; 8] {
        [
            Self::Flight,
            Self::Hotel,
            Self::Restaurant,
            Self::Activity,
            Self::Transport,
            Self::Shopping,
            Self::Beach,
            Self::Museum,
        ]
    }
}

/// カテゴリ組み合わせに応じたチェックリスト追加ルール
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChecklistRule {
    pub required_categories: &'static [ItineraryCategory],
    pub checklist: &'static [&'static str],
}

/// 旅行全体のカテゴリ構成に応じて適用するチェックリスト組み合わせルール
pub fn checklist_combination_rules() -> &'static [ChecklistRule] {
    use ItineraryCategory::*;

    const RULES: &[ChecklistRule] = &[
        ChecklistRule {
            required_categories: &[Flight, Hotel],
            checklist: &["宿泊予約確認", "身分証明書", "充電器"],
        },
        ChecklistRule {
            required_categories: &[Flight, Transport],
            checklist: &["ETCカード", "運転免許証", "レンタカー予約確認"],
        },
        ChecklistRule {
            required_categories: &[Beach],
            checklist: &["水着", "タオル", "日焼け止め", "サンダル"],
        },
        ChecklistRule {
            required_categories: &[Beach, Activity],
            checklist: &["着替え", "防水バッグ", "酔い止め"],
        },
        ChecklistRule {
            required_categories: &[Shopping],
            checklist: &["エコバッグ", "現金（小銭）"],
        },
        ChecklistRule {
            required_categories: &[Museum, Activity],
            checklist: &["事前予約確認", "入場チケット"],
        },
    ];

    RULES
}

const ITINERARY_CATEGORY_VALUES: &[&str] = &[
    "flight",
    "hotel",
    "restaurant",
    "activity",
    "transport",
    "shopping",
    "beach",
    "museum",
];

/// CLI 文字列からカテゴリを変換する（`none` は解除用のためここでは受け付けない）
pub(crate) fn parse_itinerary_category(s: &str) -> Result<ItineraryCategory> {
    match s {
        "flight" => Ok(ItineraryCategory::Flight),
        "hotel" => Ok(ItineraryCategory::Hotel),
        "restaurant" => Ok(ItineraryCategory::Restaurant),
        "activity" => Ok(ItineraryCategory::Activity),
        "transport" => Ok(ItineraryCategory::Transport),
        "shopping" => Ok(ItineraryCategory::Shopping),
        "beach" => Ok(ItineraryCategory::Beach),
        "museum" => Ok(ItineraryCategory::Museum),
        _ => anyhow::bail!(
            "不正なカテゴリです: {s}. 有効な値: {}",
            ITINERARY_CATEGORY_VALUES.join(", ")
        ),
    }
}

/// itinerary_items テーブルの1行分のデータ
#[derive(Clone, Serialize, Deserialize)]
pub struct ItineraryItem {
    pub id: i64,
    pub trip_id: i64,
    pub day: i64,
    pub title: String,
    pub note: Option<String>,
    pub start_time: Option<String>,
    pub sort_order: i64,
    pub duration_minutes: Option<i64>,
    pub travel_minutes: Option<i64>,
    pub location: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<ItineraryCategory>,
    pub created_at: String,
    pub updated_at: String,
}

/// checklist_items テーブルの1行分のデータ
#[derive(Clone, Serialize, Deserialize)]
pub struct ChecklistItem {
    pub id: i64,
    pub trip_id: i64,
    pub title: String,
    pub is_done: bool,
    pub sort_order: i64,
    pub created_at: String,
    pub updated_at: String,
}

/// trip export 用 JSON の schema バージョン
pub const TRIP_EXPORT_SCHEMA_VERSION: i32 = 1;

/// trip export の generator 名
pub const TRIP_EXPORT_GENERATOR: &str = "caglla-cli";

/// trip export 用の JSON 構造
#[derive(Serialize, Deserialize)]
pub struct TripExport {
    /// export 時に付与。旧フォーマット import では省略される。
    #[serde(default)]
    pub schema_version: Option<i32>,
    /// export 生成元（v1.0.8+）。旧フォーマット import では省略される。
    #[serde(default)]
    pub generator: Option<String>,
    /// export 生成元のバージョン（v1.0.8+）。旧フォーマット import では省略される。
    #[serde(default)]
    pub generator_version: Option<String>,
    /// export 実行時刻（RFC3339）。旧フォーマット import では省略される。
    #[serde(default)]
    pub exported_at: Option<String>,
    pub trip: Trip,
    pub itinerary_items: Vec<ItineraryItem>,
    /// 旧フォーマットでは省略可能。省略時は空配列として扱う。
    pub checklist_items: Option<Vec<ChecklistItem>>,
}

impl TripExport {
    pub fn checklist_items(&self) -> &[ChecklistItem] {
        self.checklist_items.as_deref().unwrap_or(&[])
    }
}

/// export ファイル先頭のメタデータ（表示・レポート用。valid 判定には使わない）
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TripExportMetadata {
    pub generator_present: bool,
    pub generator: Option<String>,
    pub generator_version_present: bool,
    pub generator_version: Option<String>,
    pub exported_at_present: bool,
    pub exported_at: Option<String>,
}

impl TripExportMetadata {
    pub fn from_parsed(root: &serde_json::Value, export: &TripExport) -> Self {
        Self {
            generator_present: root.get("generator").is_some(),
            generator: export.generator.clone(),
            generator_version_present: root.get("generator_version").is_some(),
            generator_version: export.generator_version.clone(),
            exported_at_present: root.get("exported_at").is_some(),
            exported_at: export.exported_at.clone(),
        }
    }

    pub fn display_generator(&self) -> &str {
        if self.generator_present {
            self.generator.as_deref().unwrap_or("-")
        } else {
            "不明"
        }
    }

    pub fn display_generator_version(&self) -> &str {
        if self.generator_version_present {
            self.generator_version.as_deref().unwrap_or("-")
        } else {
            "不明"
        }
    }

    pub fn display_exported_at(&self) -> &str {
        if self.exported_at_present {
            self.exported_at.as_deref().unwrap_or("-")
        } else {
            "不明"
        }
    }

    pub fn json_generator(&self) -> Option<String> {
        if self.generator_present {
            self.generator.clone()
        } else {
            None
        }
    }

    pub fn json_generator_version(&self) -> Option<String> {
        if self.generator_version_present {
            self.generator_version.clone()
        } else {
            None
        }
    }

    pub fn json_exported_at(&self) -> Option<String> {
        if self.exported_at_present {
            self.exported_at.clone()
        } else {
            None
        }
    }
}

/// export 検証レポート JSON の schema バージョン
pub const EXPORT_VALIDATION_REPORT_SCHEMA_VERSION: i32 = 1;

/// `trip validate-export` の構造チェック ID
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExportValidationCheckId {
    JsonFormat,
    SchemaVersion,
    Trip,
    ItineraryItems,
    ChecklistItems,
}

/// export 検証の1項目チェック結果
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExportValidationCheck {
    pub id: ExportValidationCheckId,
    pub passed: bool,
}

/// export ファイル検証結果（`trip validate-export --json`）
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExportValidationReport {
    /// レポート形式の schema バージョン
    pub schema_version: i32,
    pub file: String,
    /// import 可能か（`errors` が空）
    pub valid: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trip_name: Option<String>,
    /// 検査対象ファイル内の `schema_version`
    #[serde(skip_serializing_if = "Option::is_none")]
    pub export_schema_version: Option<i32>,
    pub itinerary_count: usize,
    pub checklist_count: usize,
    /// ファイル内 `generator`（キーなしは `null`）
    pub generator: Option<String>,
    /// ファイル内 `generator_version`（キーなしは `null`）
    pub generator_version: Option<String>,
    /// ファイル内 `exported_at`（キーなしは `null`）
    pub exported_at: Option<String>,
    pub checks: Vec<ExportValidationCheck>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
    #[serde(skip)]
    pub export_metadata: Option<TripExportMetadata>,
}

impl ExportValidationReport {
    pub fn new(file: impl Into<String>) -> Self {
        Self {
            schema_version: EXPORT_VALIDATION_REPORT_SCHEMA_VERSION,
            file: file.into(),
            valid: false,
            trip_name: None,
            export_schema_version: None,
            itinerary_count: 0,
            checklist_count: 0,
            generator: None,
            generator_version: None,
            exported_at: None,
            checks: Vec::new(),
            warnings: Vec::new(),
            errors: Vec::new(),
            export_metadata: None,
        }
    }
}

/// `trip import` 完了時のサマリー
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TripImportSummary {
    pub trip_id: i64,
    pub trip_name: String,
    pub itinerary_count: usize,
    pub checklist_count: usize,
    /// export JSON に `schema_version` キーが存在するか
    pub schema_version_present: bool,
    pub export_schema_version: Option<i32>,
    pub export_metadata: TripExportMetadata,
}

#[cfg(test)]
mod tests {
    use crate::models::{
        parse_itinerary_category, CategoryDefinition, DoctorIssue, DoctorIssueCode,
        DoctorIssueSeverity, DoctorIssueTarget, IssueTargetType, ItineraryCategory,
        DOCTOR_REPORT_SCHEMA_VERSION,
    };

    #[test]
    fn test_parse_invalid_itinerary_category() {
        assert!(parse_itinerary_category("invalid").is_err());
        assert!(parse_itinerary_category("lodging").is_err());
    }

    #[test]
    fn test_category_definition_flight() {
        let def = ItineraryCategory::Flight.definition();
        assert_eq!(def.display_name, "フライト");
        assert_eq!(
            def.default_checklist,
            &["航空券確認", "身分証明書確認", "空港到着時刻確認"]
        );
    }

    #[test]
    fn test_category_definition_hotel() {
        let def = ItineraryCategory::Hotel.definition();
        assert_eq!(def.display_name, "ホテル");
        assert_eq!(
            def.default_checklist,
            &["宿泊予約確認", "チェックイン時間確認", "住所確認"]
        );
    }

    #[test]
    fn test_category_definition_beach() {
        let def = ItineraryCategory::Beach.definition();
        assert_eq!(def.display_name, "ビーチ");
        assert_eq!(def.default_checklist, &["水着", "タオル", "日焼け止め"]);
    }

    #[test]
    fn test_all_itinerary_categories_have_definitions() {
        for category in ItineraryCategory::all() {
            let def = category.definition();
            assert!(
                !def.display_name.is_empty(),
                "display_name が空: {}",
                category.as_str()
            );
            assert!(
                !def.default_checklist.is_empty(),
                "default_checklist が空: {}",
                category.as_str()
            );
        }
    }

    #[test]
    fn test_category_definition_matches_storage_key() {
        for category in ItineraryCategory::all() {
            let parsed = parse_itinerary_category(category.as_str()).unwrap();
            assert_eq!(parsed, category);
            let _def: CategoryDefinition = parsed.definition();
        }
    }

    #[test]
    fn test_issue_json_uses_snake_case_code_and_envelope_fields() {
        let trip_id = 42;
        let issue = DoctorIssue {
            code: DoctorIssueCode::NoRestaurant,
            target: DoctorIssueTarget::Day(3),
            day: Some(3),
            itinerary_count: None,
            travel_minutes: None,
        };
        let json = issue.to_issue_json(trip_id);
        assert_eq!(json.code, DoctorIssueCode::NoRestaurant);
        assert_eq!(json.severity, DoctorIssueSeverity::Warning);
        assert_eq!(json.target.target_type, IssueTargetType::Day);
        assert_eq!(json.target.id, 3);
        assert_eq!(json.details.day, Some(3));

        let serialized = serde_json::to_value(&json).unwrap();
        assert_eq!(serialized["code"], "no_restaurant");
        assert_eq!(serialized["target"]["type"], "day");
        assert_eq!(serialized["target"]["id"], 3);
        assert_eq!(serialized["details"]["day"], 3);
    }

    #[test]
    fn test_issue_json_trip_target_uses_trip_id() {
        let issue = DoctorIssue {
            code: DoctorIssueCode::EmptyItinerary,
            target: DoctorIssueTarget::Trip,
            day: None,
            itinerary_count: None,
            travel_minutes: None,
        };
        let json = issue.to_issue_json(7);
        assert_eq!(json.target.target_type, IssueTargetType::Trip);
        assert_eq!(json.target.id, 7);
        assert!(json.details.is_empty());
        assert_eq!(json.severity, DoctorIssueSeverity::Info);

        let serialized = serde_json::to_value(&json).unwrap();
        assert!(serialized.get("details").is_none());
    }

    #[test]
    fn test_issue_json_details_for_all_codes() {
        let overloaded = DoctorIssue {
            code: DoctorIssueCode::OverloadedDay,
            target: DoctorIssueTarget::Day(2),
            day: Some(2),
            itinerary_count: Some(8),
            travel_minutes: None,
        };
        let details = overloaded.to_issue_details();
        assert_eq!(details.day, Some(2));
        assert_eq!(details.itinerary_count, Some(8));

        let travel = DoctorIssue {
            code: DoctorIssueCode::HighTravelTime,
            target: DoctorIssueTarget::Day(4),
            day: Some(4),
            itinerary_count: None,
            travel_minutes: Some(190),
        };
        let details = travel.to_issue_details();
        assert_eq!(details.day, Some(4));
        assert_eq!(details.travel_minutes, Some(190));

        let missing = DoctorIssue {
            code: DoctorIssueCode::MissingDuration,
            target: DoctorIssueTarget::Itinerary(11),
            day: None,
            itinerary_count: None,
            travel_minutes: None,
        };
        let details = missing.to_issue_details();
        assert_eq!(details.itinerary_id, Some(11));

        let report = crate::models::DoctorReportJson::new(1, vec![]);
        assert_eq!(report.schema_version, DOCTOR_REPORT_SCHEMA_VERSION);
    }
}
