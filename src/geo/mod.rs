mod country_registry;

use anyhow::Result;

use country_registry::is_iso3166_alpha2;

/// CLI write path — ISO 3166-1 alpha-2 currently assigned codes only.
pub(crate) fn validate_cli_write_country_code(code: &str) -> Result<String> {
    let normalized = normalize_country_code_format(code)?;
    if !is_iso3166_alpha2(&normalized) {
        anyhow::bail!("country code は有効な ISO 3166-1 alpha-2 コードではありません");
    }
    Ok(normalized)
}

fn normalize_country_code_format(code: &str) -> Result<String> {
    let trimmed = code.trim();
    if trimmed.len() != 2 {
        anyhow::bail!("country code は 2 文字である必要があります");
    }
    let normalized: String = trimmed.chars().map(|c| c.to_ascii_uppercase()).collect();
    if !normalized.chars().all(|c| c.is_ascii_alphabetic()) {
        anyhow::bail!("country code は英字 2 文字である必要があります");
    }
    Ok(normalized)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_normalizes_lowercase() {
        assert_eq!(validate_cli_write_country_code("jp").unwrap(), "JP");
    }

    #[test]
    fn validate_rejects_unknown_and_historical() {
        for code in ["ZZ", "EU", "UK", "AN"] {
            assert!(
                validate_cli_write_country_code(code).is_err(),
                "{code} should fail"
            );
        }
    }
}
