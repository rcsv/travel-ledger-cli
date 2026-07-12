mod currency_registry;

use anyhow::Result;

use currency_registry::{is_iso4217_alpha3, is_travel_expense_denylisted};

/// Currency validation strictness. Existing call sites default to [`FormatOnly`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum CurrencyValidationMode {
    /// Uppercase alpha-3 format only; unknown codes allowed (legacy default).
    #[default]
    FormatOnly,
    /// ISO 4217 registry + travel-expense denylist (v4.8.13+ CLI write paths).
    IsoStrict,
}

/// CLI create/update write paths — ISO 4217 registry + travel-expense denylist.
pub(crate) fn validate_cli_write_currency_code(code: &str) -> Result<String> {
    validate_currency_code_with_mode(code, CurrencyValidationMode::IsoStrict)
}

/// 通貨コードの形式を検証し、正規化した 3 文字コードを返す。
/// 既定は [`CurrencyValidationMode::FormatOnly`]（既存互換）。
pub(crate) fn validate_currency_code(code: &str) -> Result<String> {
    validate_currency_code_with_mode(code, CurrencyValidationMode::FormatOnly)
}

/// 通貨コードを指定モードで検証し、正規化した 3 文字コードを返す。
pub(crate) fn validate_currency_code_with_mode(
    code: &str,
    mode: CurrencyValidationMode,
) -> Result<String> {
    let normalized = normalize_currency_format(code)?;
    match mode {
        CurrencyValidationMode::FormatOnly => Ok(normalized),
        CurrencyValidationMode::IsoStrict => validate_iso_strict(&normalized),
    }
}

fn normalize_currency_format(code: &str) -> Result<String> {
    let trimmed = code.trim();
    if trimmed.len() != 3 {
        anyhow::bail!("currency は 3 文字である必要があります");
    }
    let normalized: String = trimmed.chars().map(|c| c.to_ascii_uppercase()).collect();
    if !normalized.chars().all(|c| c.is_ascii_alphabetic()) {
        anyhow::bail!("currency は英字 3 文字である必要があります");
    }
    Ok(normalized)
}

fn validate_iso_strict(normalized: &str) -> Result<String> {
    if is_travel_expense_denylisted(normalized) {
        anyhow::bail!("currency は旅行費用として許可されていません: {normalized}");
    }
    if !is_iso4217_alpha3(normalized) {
        anyhow::bail!("currency は有効な ISO 4217 コードではありません");
    }
    Ok(normalized.to_string())
}

/// 通貨の最小通貨単位における小数桁数（ISO 4217 の主要通貨のみ。未知は 2）。
fn currency_minor_unit_digits(currency: &str) -> u32 {
    match currency {
        "BIF" | "CLP" | "DJF" | "GNF" | "ISK" | "JPY" | "KMF" | "KRW" | "PYG" | "RWF" | "UGX"
        | "UYI" | "VND" | "VUV" | "XAF" | "XOF" | "XPF" => 0,
        _ => 2,
    }
}

/// CLI 入力 amount を最小通貨単位の整数へ変換する（浮動小数点を使わない）。
pub(crate) fn parse_amount_for_currency(input: &str, currency: &str) -> Result<i64> {
    let currency = validate_currency_code(currency)?;
    let decimals = currency_minor_unit_digits(&currency);
    let s = input.trim();
    if s.is_empty() {
        anyhow::bail!("amount は必須です");
    }
    if s.starts_with('-') || s.starts_with('+') {
        anyhow::bail!("amount は 0 以上である必要があります");
    }

    let (whole_part, frac_part) = match s.split_once('.') {
        Some((whole, frac)) => {
            if frac.contains('.') {
                anyhow::bail!("amount の形式が不正です");
            }
            (whole, Some(frac))
        }
        None => (s, None),
    };

    if !whole_part.is_empty() && !whole_part.chars().all(|c| c.is_ascii_digit()) {
        anyhow::bail!("amount は数値である必要があります");
    }
    if let Some(frac) = frac_part {
        if !frac.chars().all(|c| c.is_ascii_digit()) {
            anyhow::bail!("amount は数値である必要があります");
        }
        if frac.len() > decimals as usize {
            anyhow::bail!("amount の小数桁が多すぎます（{currency} は {decimals} 桁まで）");
        }
    }

    let whole: i64 = if whole_part.is_empty() {
        0
    } else {
        whole_part
            .parse()
            .map_err(|_| anyhow::anyhow!("amount の形式が不正です"))?
    };

    let frac_val: i64 = match frac_part {
        None => 0,
        Some(frac) => {
            let padded = format!("{frac:0<width$}", width = decimals as usize);
            padded
                .parse()
                .map_err(|_| anyhow::anyhow!("amount の形式が不正です"))?
        }
    };

    let multiplier = 10_i64.pow(decimals);
    let amount = whole
        .checked_mul(multiplier)
        .and_then(|v| v.checked_add(frac_val))
        .ok_or_else(|| anyhow::anyhow!("amount が大きすぎます"))?;
    if amount < 0 {
        anyhow::bail!("amount は 0 以上である必要があります");
    }
    Ok(amount)
}

fn format_integer_with_commas(value: i64) -> String {
    let negative = value < 0;
    let digits = value.abs().to_string();
    let mut out = String::new();
    for (i, ch) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(ch);
    }
    if negative {
        format!("-{out}")
    } else {
        out
    }
}

/// 金額の数値部分を表示用に整形する（通貨コードは含めない）
pub(crate) fn format_amount_value(amount: i64, currency: &str) -> String {
    let decimals = currency_minor_unit_digits(currency);
    if decimals == 0 {
        return format_integer_with_commas(amount);
    }
    let negative = amount < 0;
    let amount_abs = amount.unsigned_abs();
    let divisor = 10_u64.pow(decimals);
    let whole = amount_abs / divisor;
    let frac = amount_abs % divisor;
    let formatted = format!(
        "{}.{:0width$}",
        format_integer_with_commas(whole as i64),
        frac,
        width = decimals as usize
    );
    if negative {
        format!("-{formatted}")
    } else {
        formatted
    }
}

pub(crate) fn format_amount_display(amount: i64, currency: &str) -> String {
    format!("{} {}", format_amount_value(amount, currency), currency)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_cli_write_currency_code_rejects_unknown_and_denylist() {
        assert!(validate_cli_write_currency_code("ZZZ").is_err());
        assert!(validate_cli_write_currency_code("XXX").is_err());
        assert_eq!(validate_cli_write_currency_code("jpy").unwrap(), "JPY");
    }

    #[test]
    fn test_validate_currency_code_normalizes_lowercase() {
        assert_eq!(validate_currency_code("jpy").unwrap(), "JPY");
        assert_eq!(validate_currency_code("Usd").unwrap(), "USD");
        assert_eq!(
            validate_currency_code_with_mode("jpy", CurrencyValidationMode::IsoStrict).unwrap(),
            "JPY"
        );
    }

    #[test]
    fn test_validate_currency_code_format_only_allows_unknown_codes() {
        assert_eq!(validate_currency_code("ZZZ").unwrap(), "ZZZ");
        assert_eq!(
            validate_currency_code_with_mode("ZZZ", CurrencyValidationMode::FormatOnly).unwrap(),
            "ZZZ"
        );
    }

    #[test]
    fn test_validate_currency_code_format_only_allows_denylisted_codes() {
        assert_eq!(validate_currency_code("XXX").unwrap(), "XXX");
        assert_eq!(
            validate_currency_code_with_mode("XXX", CurrencyValidationMode::FormatOnly).unwrap(),
            "XXX"
        );
    }

    #[test]
    fn test_validate_currency_code_iso_strict_accepts_valid_iso_codes() {
        for code in ["JPY", "USD", "EUR", "HRK", "LTL"] {
            assert_eq!(
                validate_currency_code_with_mode(code, CurrencyValidationMode::IsoStrict).unwrap(),
                code
            );
        }
    }

    #[test]
    fn test_validate_currency_code_iso_strict_rejects_unknown_codes() {
        for code in ["ZZZ", "ABC", "JPN"] {
            assert!(
                validate_currency_code_with_mode(code, CurrencyValidationMode::IsoStrict).is_err()
            );
        }
    }

    #[test]
    fn test_validate_currency_code_iso_strict_rejects_denylisted_codes() {
        for code in ["XXX", "XTS", "XAU", "XAG", "XPT", "XPD"] {
            let error = validate_currency_code_with_mode(code, CurrencyValidationMode::IsoStrict)
                .unwrap_err()
                .to_string();
            assert!(
                error.contains("旅行費用として許可されていません"),
                "unexpected error for {code}: {error}"
            );
        }
    }

    #[test]
    fn test_validate_currency_code_rejects_invalid_format() {
        for mode in [
            CurrencyValidationMode::FormatOnly,
            CurrencyValidationMode::IsoStrict,
        ] {
            assert!(validate_currency_code_with_mode("JP", mode).is_err());
            assert!(validate_currency_code_with_mode("JPYY", mode).is_err());
            assert!(validate_currency_code_with_mode("JP1", mode).is_err());
        }
    }

    #[test]
    fn test_parse_amount_jpy_integer() {
        assert_eq!(parse_amount_for_currency("1500", "JPY").unwrap(), 1500);
    }

    #[test]
    fn test_parse_amount_usd_decimal() {
        assert_eq!(parse_amount_for_currency("12.50", "USD").unwrap(), 1250);
        assert_eq!(parse_amount_for_currency("12.5", "USD").unwrap(), 1250);
    }

    #[test]
    fn test_parse_amount_rejects_too_many_decimals_for_jpy() {
        assert!(parse_amount_for_currency("10.5", "JPY").is_err());
    }

    #[test]
    fn test_parse_amount_rejects_negative() {
        assert!(parse_amount_for_currency("-100", "JPY").is_err());
    }

    #[test]
    fn test_parse_amount_unknown_currency_still_uses_format_only() {
        // ZZZ is unknown to minor-unit table → 2 decimal fallback → 100.00
        assert_eq!(parse_amount_for_currency("100", "ZZZ").unwrap(), 10_000);
    }

    #[test]
    fn test_format_amount_value_negative_decimal_currency() {
        assert_eq!(format_amount_value(-50, "USD"), "-0.50");
        assert_eq!(format_amount_value(-12345, "USD"), "-123.45");
        assert_eq!(format_amount_value(50, "USD"), "0.50");
        assert_eq!(format_amount_value(12345, "USD"), "123.45");
    }

    #[test]
    fn test_format_amount_value_negative_zero_decimal_currency() {
        assert_eq!(format_amount_value(-500, "JPY"), "-500");
        assert_eq!(format_amount_value(-50_000, "KRW"), "-50,000");
    }
}
