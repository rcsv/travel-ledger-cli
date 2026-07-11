//! Fragment apply structured errors — internal model (v4.8.5).
//!
//! Stable codes for API/GUI readiness. Human `errors: Vec<String>` remains the
//! external contract until v4.8.6+ JSON exposure.

use serde::{Deserialize, Serialize};

/// Stable machine-readable error code (registry).
#[allow(clippy::enum_variant_names)] // `Apply*` prefix matches `APPLY_*` registry strings
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ApplyErrorCode {
    ApplyUnsupportedIntent,
    ApplyUnsupportedTarget,
    ApplyTargetUnresolved,
    ApplyTargetAmbiguous,
    ApplyRequiredDecision,
    ApplyFieldInvalid,
    ApplyMutationFieldRejected,
    ApplyEntityNotFound,
    ApplyEntityParentMismatch,
    ApplyBaselineMismatch,
    ApplyPreviewMismatch,
    ApplyToctouDrift,
    ApplyScopedWriteZeroRows,
    ApplyScopedWriteMultipleRows,
    ApplyTransactionFailed,
}

impl ApplyErrorCode {
    #[allow(dead_code)] // unit tests + reserved for v4.8.6 JSON `code` field
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ApplyUnsupportedIntent => "APPLY_UNSUPPORTED_INTENT",
            Self::ApplyUnsupportedTarget => "APPLY_UNSUPPORTED_TARGET",
            Self::ApplyTargetUnresolved => "APPLY_TARGET_UNRESOLVED",
            Self::ApplyTargetAmbiguous => "APPLY_TARGET_AMBIGUOUS",
            Self::ApplyRequiredDecision => "APPLY_REQUIRED_DECISION",
            Self::ApplyFieldInvalid => "APPLY_FIELD_INVALID",
            Self::ApplyMutationFieldRejected => "APPLY_MUTATION_FIELD_REJECTED",
            Self::ApplyEntityNotFound => "APPLY_ENTITY_NOT_FOUND",
            Self::ApplyEntityParentMismatch => "APPLY_ENTITY_PARENT_MISMATCH",
            Self::ApplyBaselineMismatch => "APPLY_BASELINE_MISMATCH",
            Self::ApplyPreviewMismatch => "APPLY_PREVIEW_MISMATCH",
            Self::ApplyToctouDrift => "APPLY_TOCTOU_DRIFT",
            Self::ApplyScopedWriteZeroRows => "APPLY_SCOPED_WRITE_ZERO_ROWS",
            Self::ApplyScopedWriteMultipleRows => "APPLY_SCOPED_WRITE_MULTIPLE_ROWS",
            Self::ApplyTransactionFailed => "APPLY_TRANSACTION_FAILED",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ApplyErrorKind {
    Blocking,
    Warning,
    DecisionRequired,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ApplyErrorPhase {
    FragmentValidate,
    Gate,
    ConfirmScope,
    Simulate,
    ConfirmTransaction,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StructuredApplyError {
    pub code: ApplyErrorCode,
    pub kind: ApplyErrorKind,
    pub message: String,
    pub phase: ApplyErrorPhase,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub field_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub intent: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_type: Option<String>,
}

impl StructuredApplyError {
    pub fn blocking(
        code: ApplyErrorCode,
        phase: ApplyErrorPhase,
        message: impl Into<String>,
    ) -> Self {
        Self {
            code,
            kind: ApplyErrorKind::Blocking,
            message: message.into(),
            phase,
            field_path: None,
            intent: None,
            target_type: None,
        }
    }

    pub fn with_field_path(mut self, field_path: impl Into<String>) -> Self {
        self.field_path = Some(field_path.into());
        self
    }

    pub fn with_intent(mut self, intent: impl Into<String>) -> Self {
        self.intent = Some(intent.into());
        self
    }

    pub fn with_target_type(mut self, target_type: impl Into<String>) -> Self {
        self.target_type = Some(target_type.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn apply_error_code_strings_are_stable() {
        assert_eq!(
            ApplyErrorCode::ApplyUnsupportedIntent.as_str(),
            "APPLY_UNSUPPORTED_INTENT"
        );
        assert_eq!(
            ApplyErrorCode::ApplyUnsupportedTarget.as_str(),
            "APPLY_UNSUPPORTED_TARGET"
        );
        assert_eq!(
            ApplyErrorCode::ApplyTargetUnresolved.as_str(),
            "APPLY_TARGET_UNRESOLVED"
        );
        assert_eq!(
            ApplyErrorCode::ApplyTargetAmbiguous.as_str(),
            "APPLY_TARGET_AMBIGUOUS"
        );
        assert_eq!(
            ApplyErrorCode::ApplyRequiredDecision.as_str(),
            "APPLY_REQUIRED_DECISION"
        );
        assert_eq!(
            ApplyErrorCode::ApplyBaselineMismatch.as_str(),
            "APPLY_BASELINE_MISMATCH"
        );
        assert_eq!(
            ApplyErrorCode::ApplyScopedWriteMultipleRows.as_str(),
            "APPLY_SCOPED_WRITE_MULTIPLE_ROWS"
        );
    }
}
