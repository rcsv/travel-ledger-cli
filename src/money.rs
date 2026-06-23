use anyhow::Result;

/// 通貨コードの形式を検証し、正規化した 3 文字コードを返す。
/// v1.x: 大文字化 + 英字 3 文字チェックのみ（未知コードは許可）。
pub(crate) fn validate_currency_code(code: &str) -> Result<String> {
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
    fn test_validate_currency_code_normalizes_lowercase() {
        assert_eq!(validate_currency_code("jpy").unwrap(), "JPY");
        assert_eq!(validate_currency_code("Usd").unwrap(), "USD");
    }

    #[test]
    fn test_validate_currency_code_allows_unknown_codes() {
        assert_eq!(validate_currency_code("XXX").unwrap(), "XXX");
    }

    #[test]
    fn test_validate_currency_code_rejects_invalid_format() {
        assert!(validate_currency_code("JP").is_err());
        assert!(validate_currency_code("JPYY").is_err());
        assert!(validate_currency_code("JP1").is_err());
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
