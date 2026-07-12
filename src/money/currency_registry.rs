//! ISO 4217 alpha-3 registry and travel-expense denylist for strict validation.

/// ISO 4217 alpha-3 codes: `iso_currency` 0.5.3 dataset + supplemental withdrawn codes.
/// Sorted for `binary_search`. See v4.8.12 spec for scope notes.
static ISO4217_ALPHA3_CODES: &[&str] = &[
    "ADF", "ADP", "AED", "AFA", "AFN", "ALL", "AMD", "AOA", "AOK", "AON", "AOR", "ARA", "ARP",
    "ARS", "ATS", "AUD", "AWG", "AYM", "AZM", "AZN", "BAD", "BAM", "BBD", "BDT", "BEF", "BGL",
    "BGN", "BHD", "BIF", "BMD", "BND", "BOB", "BOV", "BRC", "BRE", "BRL", "BRR", "BSD", "BTN",
    "BWP", "BYB", "BYN", "BYR", "BZD", "CAD", "CDF", "CHC", "CHE", "CHF", "CHW", "CLF", "CLP",
    "CNY", "COP", "COU", "CRC", "CSD", "CUC", "CUP", "CVE", "CYP", "CZK", "DDM", "DEM", "DJF",
    "DKK", "DOP", "DZD", "ECS", "ECV", "EEK", "EGP", "ERN", "ESP", "ETB", "EUR", "FIM", "FJD",
    "FKP", "FRF", "GBP", "GEL", "GHC", "GHP", "GHS", "GIP", "GMD", "GNF", "GRD", "GTQ", "GYD",
    "HKD", "HNL", "HRD", "HRK", "HTG", "HUF", "IDR", "IEP", "ILS", "INR", "IQD", "IRR", "ISK",
    "ITL", "JMD", "JOD", "JPY", "KES", "KGS", "KHR", "KMF", "KPW", "KRW", "KWD", "KYD", "KZT",
    "LAK", "LBP", "LKR", "LRD", "LSL", "LTL", "LUF", "LVL", "LYD", "MAD", "MDL", "MGA", "MGF",
    "MKD", "MMK", "MNT", "MOP", "MRO", "MRU", "MTL", "MUR", "MVR", "MWK", "MXN", "MXP", "MXV",
    "MYR", "MZM", "MZN", "NAD", "NGN", "NIC", "NIO", "NLG", "NOK", "NPR", "NZD", "OMR", "PAB",
    "PEH", "PEI", "PEN", "PGK", "PHP", "PKR", "PLN", "PLZ", "PTE", "PYG", "QAR", "RHD", "ROL",
    "RON", "RSD", "RUB", "RUR", "RWF", "SAR", "SBD", "SCR", "SDD", "SDG", "SDP", "SEK", "SGD",
    "SHP", "SIT", "SKK", "SLE", "SLL", "SOS", "SRD", "SRG", "SSP", "STD", "STN", "SUR", "SVC",
    "SYP", "SZL", "THB", "TJR", "TJS", "TMM", "TMT", "TND", "TOP", "TPE", "TRL", "TRY", "TTD",
    "TWD", "TZS", "UAH", "UAK", "UGX", "USD", "USM", "USN", "USS", "UYI", "UYU", "UYW", "UZS",
    "VEB", "VED", "VEF", "VES", "VNC", "VND", "VUV", "WST", "XAF", "XAG", "XAU", "XBA", "XBB",
    "XBC", "XBD", "XCD", "XCG", "XDR", "XEU", "XFO", "XFU", "XOF", "XPD", "XPF", "XPT", "XSU",
    "XTS", "XUA", "XXX", "YDD", "YER", "YUM", "YUN", "ZAL", "ZAR", "ZMK", "ZMW", "ZRN", "ZWD",
    "ZWG", "ZWL",
];

/// Travel-expense denylist: ISO codes that must not be used for trip money entries.
static TRAVEL_EXPENSE_DENYLIST: &[&str] = &[
    "XAG", "XAU", "XBA", "XBB", "XBC", "XBD", "XPD", "XPT", "XTS", "XXX",
];

pub(crate) fn is_iso4217_alpha3(code: &str) -> bool {
    ISO4217_ALPHA3_CODES.binary_search(&code).is_ok()
}

pub(crate) fn is_travel_expense_denylisted(code: &str) -> bool {
    TRAVEL_EXPENSE_DENYLIST.binary_search(&code).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_includes_primary_travel_currencies() {
        for code in [
            "JPY", "USD", "EUR", "KRW", "GBP", "TWD", "CNY", "HKD", "AUD", "CAD", "CHF", "SGD",
            "THB",
        ] {
            assert!(is_iso4217_alpha3(code), "missing travel currency {code}");
        }
    }

    #[test]
    fn registry_includes_withdrawn_codes() {
        for code in ["HRK", "LTL", "LVL", "DEM", "ITL"] {
            assert!(is_iso4217_alpha3(code), "missing withdrawn code {code}");
        }
    }

    #[test]
    fn registry_excludes_non_iso_codes() {
        for code in ["ZZZ", "ABC", "JPN"] {
            assert!(!is_iso4217_alpha3(code));
        }
    }

    #[test]
    fn denylist_includes_pseudo_and_precious_metal_codes() {
        for code in ["XXX", "XTS", "XAU", "XAG", "XPT", "XPD"] {
            assert!(is_travel_expense_denylisted(code));
        }
    }
}
