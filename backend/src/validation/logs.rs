use crate::error::AppError;

const MAX_RAW_TEXT_LENGTH: usize = 10_000;

pub fn validate_raw_text(raw_text: &str) -> Result<(), AppError> {
    if raw_text.trim().is_empty() {
        return Err(AppError::Validation("raw_text cannot be empty".to_string()));
    }

    if raw_text.chars().count() > MAX_RAW_TEXT_LENGTH {
        return Err(AppError::Validation(format!(
            "raw_text is too long, max length is {MAX_RAW_TEXT_LENGTH}"
        )));
    }

    Ok(())
}

pub fn validate_context_date(context_date: Option<&str>) -> Result<(), AppError> {
    if let Some(value) = context_date {
        chrono::NaiveDate::parse_from_str(value, "%Y-%m-%d")
            .map_err(|error| AppError::Validation(format!("invalid context_date: {error}")))?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::error::AppError;
    use crate::validation::logs::{validate_context_date, validate_raw_text};

    #[test]
    fn rejects_empty_raw_text() {
        let error = validate_raw_text("").expect_err("empty text should fail");

        match error {
            AppError::Validation(message) => assert!(message.contains("raw_text")),
            other => panic!("expected validation error, got {other:?}"),
        }
    }

    #[test]
    fn rejects_raw_text_over_limit() {
        let input = "a".repeat(super::MAX_RAW_TEXT_LENGTH + 1);

        let error = validate_raw_text(&input).expect_err("overlong text should fail");

        match error {
            AppError::Validation(message) => assert!(message.contains("too long")),
            other => panic!("expected validation error, got {other:?}"),
        }
    }

    #[test]
    fn accepts_valid_raw_text() {
        validate_raw_text("今天 9:40 起床").expect("valid text should pass");
    }

    #[test]
    fn rejects_invalid_context_date() {
        let error = validate_context_date(Some("2026-99-99"))
            .expect_err("invalid context_date should fail");

        match error {
            AppError::Validation(message) => assert!(message.contains("context_date")),
            other => panic!("expected validation error, got {other:?}"),
        }
    }

    #[test]
    fn accepts_missing_context_date() {
        validate_context_date(None).expect("missing context_date should pass");
    }
}
