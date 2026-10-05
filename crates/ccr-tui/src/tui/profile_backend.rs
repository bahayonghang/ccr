//! TUI profile adapter. The application owns all writes and success effects.
use ccr_cli::application::profile_lifecycle::{ApplyProfileRequest, ProfileOutcome, apply_profile};
use ccr_core::Result;

pub fn apply(request: ApplyProfileRequest) -> Result<ProfileOutcome> {
    apply_profile(request)
}

pub fn presentation(outcome: &ProfileOutcome) -> super::toast::Toast {
    use super::toast::Toast;
    use ccr_cli::application::profile_lifecycle::ProfileStatus;
    if outcome.status == ProfileStatus::RecoveryRequired {
        Toast::error(crate::tui_text!(
            "Profile operation requires recovery. External changes were retained.",
            "配置操作需要恢复，外部修改已保留。"
        ))
    } else if !outcome.activation_committed {
        Toast::error(crate::tui_text!(
            "Profile was not applied. Previous files were restored.",
            "配置未激活，原文件已恢复。"
        ))
    } else if !outcome.warnings.is_empty() {
        Toast::warning(crate::tui_text!(
            "Profile applied; ancillary work needs attention. Do not repeat activation.",
            "配置已激活，附属记录未完整完成；请勿重复激活。"
        ))
    } else {
        Toast::success(crate::tui_text!("Profile applied", "配置已激活"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn profile_post_publish_failure_contract() {
        profile_contract::post_publish_failure(apply);
    }
    #[test]
    fn apply_warning_presentation_is_localized_and_never_success_only() {
        use ccr_cli::{
            application::profile_lifecycle::{ProfileStatus, ProfileWarning},
            managers::TuiLanguage,
        };
        let mut outcome = ProfileOutcome {
            operation_id: "fixture".into(),
            platform: "claude".into(),
            profile: "new".into(),
            previous_profile: None,
            status: ProfileStatus::AppliedWithWarning,
            activation_committed: true,
            warnings: vec![ProfileWarning::HistoryFailed],
            recovery_paths: vec![],
        };
        for language in [TuiLanguage::English, TuiLanguage::SimplifiedChinese] {
            super::super::i18n::set_language(language);
            let toast = presentation(&outcome);
            assert_eq!(toast.kind, super::super::toast::ToastKind::Warning);
            assert!(toast.message.contains(if language == TuiLanguage::English {
                "Do not repeat"
            } else {
                "请勿重复"
            }));
        }
        outcome.status = ProfileStatus::RecoveryRequired;
        outcome.activation_committed = false;
        assert_eq!(
            presentation(&outcome).kind,
            super::super::toast::ToastKind::Error
        );
        super::super::i18n::set_language(TuiLanguage::English);
    }
    use ccr_cli::application::profile_contract;
    #[test]
    fn profile_deleted_after_prepare_contract() {
        profile_contract::deleted_after_preparation(apply);
    }
    #[test]
    fn apply_preflight_contract() {
        profile_contract::preflight(apply);
    }
    #[test]
    fn apply_success_replay_contract() {
        profile_contract::success_and_replay(apply);
    }
    #[test]
    fn apply_ancillary_contract() {
        profile_contract::ancillary_failures(apply);
    }
    #[test]
    fn apply_each_write_contract() {
        profile_contract::failure_matrix(apply);
    }
    #[test]
    fn apply_external_version_contract() {
        profile_contract::external_change(apply);
    }
}
