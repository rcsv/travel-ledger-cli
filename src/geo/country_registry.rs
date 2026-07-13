//! ISO 3166-1 alpha-2 country code registry for CLI strict validation.

/// Currently assigned ISO 3166-1 alpha-2 codes (maintenance snapshot).
/// Excludes reserved, user-assigned, and withdrawn codes (e.g. `AN`, `UK`).
/// Sorted for `binary_search`.
/// Source: ISO 3166-1 MA officially assigned alpha-2 list.
/// Snapshot: 2026-07 (v4.9.1). Excludes reserved, user-assigned, and withdrawn codes.
static ISO3166_ALPHA2_CODES: &[&str] = &[
    "AD", "AE", "AF", "AG", "AI", "AL", "AM", "AO", "AQ", "AR", "AS", "AT", "AU", "AW", "AX", "AZ",
    "BA", "BB", "BD", "BE", "BF", "BG", "BH", "BI", "BJ", "BL", "BM", "BN", "BO", "BQ", "BR", "BS",
    "BT", "BV", "BW", "BY", "BZ", "CA", "CC", "CD", "CF", "CG", "CH", "CI", "CK", "CL", "CM", "CN",
    "CO", "CR", "CU", "CV", "CW", "CX", "CY", "CZ", "DE", "DJ", "DK", "DM", "DO", "DZ", "EC", "EE",
    "EG", "EH", "ER", "ES", "ET", "FI", "FJ", "FK", "FM", "FO", "FR", "GA", "GB", "GD", "GE", "GF",
    "GG", "GH", "GI", "GL", "GM", "GN", "GP", "GQ", "GR", "GS", "GT", "GU", "GW", "GY", "HK", "HM",
    "HN", "HR", "HT", "HU", "ID", "IE", "IL", "IM", "IN", "IO", "IQ", "IR", "IS", "IT", "JE", "JM",
    "JO", "JP", "KE", "KG", "KH", "KI", "KM", "KN", "KP", "KR", "KW", "KY", "KZ", "LA", "LB", "LC",
    "LI", "LK", "LR", "LS", "LT", "LU", "LV", "LY", "MA", "MC", "MD", "ME", "MF", "MG", "MH", "MK",
    "ML", "MM", "MN", "MO", "MP", "MQ", "MR", "MS", "MT", "MU", "MV", "MW", "MX", "MY", "MZ", "NA",
    "NC", "NE", "NF", "NG", "NI", "NL", "NO", "NP", "NR", "NU", "NZ", "OM", "PA", "PE", "PF", "PG",
    "PH", "PK", "PL", "PM", "PN", "PR", "PS", "PT", "PW", "PY", "QA", "RE", "RO", "RS", "RU", "RW",
    "SA", "SB", "SC", "SD", "SE", "SG", "SH", "SI", "SJ", "SK", "SL", "SM", "SN", "SO", "SR", "SS",
    "ST", "SV", "SX", "SY", "SZ", "TC", "TD", "TF", "TG", "TH", "TJ", "TK", "TL", "TM", "TN", "TO",
    "TR", "TT", "TV", "TW", "TZ", "UA", "UG", "UM", "US", "UY", "UZ", "VA", "VC", "VE", "VG", "VI",
    "VN", "VU", "WF", "WS", "YE", "YT", "ZA", "ZM", "ZW",
];

pub(crate) fn is_iso3166_alpha2(code: &str) -> bool {
    ISO3166_ALPHA2_CODES.binary_search(&code).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_is_sorted() {
        for window in ISO3166_ALPHA2_CODES.windows(2) {
            assert!(
                window[0] < window[1],
                "registry not sorted at {} vs {}",
                window[0],
                window[1]
            );
        }
    }

    #[test]
    fn registry_includes_primary_travel_countries() {
        for code in ["JP", "US", "DE", "GB"] {
            assert!(is_iso3166_alpha2(code), "missing country {code}");
        }
    }

    #[test]
    fn registry_excludes_unknown_reserved_and_historical() {
        for code in ["ZZ", "EU", "UK", "AN", "CS", "SU", "DD"] {
            assert!(!is_iso3166_alpha2(code), "unexpected country {code}");
        }
    }
}
