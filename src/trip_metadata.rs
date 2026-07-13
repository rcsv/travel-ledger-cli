use anyhow::Result;

pub(crate) const TRIP_MAIN_DESTINATION_MAX_LEN: usize = 200;

/// trim し、空なら None。長さ超過時はエラー。
pub(crate) fn normalize_main_destination(input: Option<&str>) -> Result<Option<String>> {
    let Some(raw) = input else {
        return Ok(None);
    };
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }
    if trimmed.chars().count() > TRIP_MAIN_DESTINATION_MAX_LEN {
        anyhow::bail!(
            "main destination exceeds maximum length ({TRIP_MAIN_DESTINATION_MAX_LEN} characters)"
        );
    }
    Ok(Some(trimmed.to_string()))
}

pub(crate) fn validate_main_destination_country_code(
    input: Option<&str>,
) -> Result<Option<String>> {
    match input {
        None => Ok(None),
        Some(code) => crate::geo::validate_cli_write_country_code(code).map(Some),
    }
}

pub(crate) fn validate_default_currency(input: Option<&str>) -> Result<Option<String>> {
    match input {
        None => Ok(None),
        Some(code) => crate::money::validate_cli_write_currency_code(code).map(Some),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_main_destination_empty_to_none() {
        assert_eq!(normalize_main_destination(None).unwrap(), None);
        assert_eq!(normalize_main_destination(Some("")).unwrap(), None);
        assert_eq!(normalize_main_destination(Some("   ")).unwrap(), None);
    }

    #[test]
    fn normalize_main_destination_trims() {
        assert_eq!(
            normalize_main_destination(Some("  Okinawa  ")).unwrap(),
            Some("Okinawa".to_string())
        );
    }

    #[test]
    fn normalize_main_destination_rejects_over_max() {
        let too_long = "x".repeat(TRIP_MAIN_DESTINATION_MAX_LEN + 1);
        assert!(normalize_main_destination(Some(&too_long)).is_err());
    }
}
