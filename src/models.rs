use anyhow::Result;
use serde::{Deserialize, Serialize};

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
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DoctorIssueCode {
    EmptyItinerary,
    OverloadedDay,
    NoRestaurant,
    HighTravelTime,
    MissingDuration,
}

/// trip doctor / advisor が扱う1件の問題
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DoctorIssue {
    pub code: DoctorIssueCode,
    pub day: Option<i64>,
    pub itinerary_count: Option<usize>,
    pub missing_duration_count: Option<usize>,
    pub travel_minutes: Option<i64>,
}

impl DoctorIssue {
    /// 警告・Info 表示用の1行メッセージ
    pub fn warning_message(&self) -> String {
        match self.code {
            DoctorIssueCode::EmptyItinerary => "No itinerary found.".to_string(),
            DoctorIssueCode::OverloadedDay => format!(
                "Day {} has many itineraries ({})",
                self.day.unwrap_or(0),
                self.itinerary_count.unwrap_or(0)
            ),
            DoctorIssueCode::NoRestaurant => {
                format!("Day {} has no restaurant", self.day.unwrap_or(0))
            }
            DoctorIssueCode::HighTravelTime => format!(
                "Day {} has high travel time ({})",
                self.day.unwrap_or(0),
                crate::stats::format_minutes_duration(self.travel_minutes.unwrap_or(0))
            ),
            DoctorIssueCode::MissingDuration => {
                let count = self.missing_duration_count.unwrap_or(0);
                if count == 1 {
                    "1 itinerary has no duration estimate".to_string()
                } else {
                    format!("{count} itineraries have no duration estimate")
                }
            }
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
#[derive(Clone)]
pub struct ChecklistItem {
    pub id: i64,
    pub trip_id: i64,
    pub title: String,
    pub is_done: bool,
    pub sort_order: i64,
    pub created_at: String,
    pub updated_at: String,
}

/// trip export 用の JSON 構造
#[derive(Serialize, Deserialize)]
pub struct TripExport {
    pub trip: Trip,
    pub itinerary_items: Vec<ItineraryItem>,
}

#[cfg(test)]
mod tests {
    use crate::models::{parse_itinerary_category, CategoryDefinition, ItineraryCategory};

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
}
