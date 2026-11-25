//! Confirm stable identities against the saved account projection before dispatch.
use super::*;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JobAction {
    Checkin,
    Balance,
}

#[derive(Clone, Debug)]
pub enum JobScope {
    AllEnabled,
    Selected(BTreeSet<String>),
    Single(String),
}

pub const MAX_JOB_TARGETS: usize = 1000;

impl JobScope {
    fn eligible_rows<'a>(
        &self,
        action: JobAction,
        rows: &'a [CheckinAccountRow],
    ) -> Result<Vec<&'a CheckinAccountRow>, OperationError> {
        let mut selected = 0;
        let mut eligible = Vec::new();
        for row in rows {
            let included = match self {
                Self::AllEnabled => row.enabled,
                Self::Selected(ids) => ids.contains(&row.id),
                Self::Single(id) => id == &row.id,
            };
            if included {
                selected += 1;
                if row.enabled || action == JobAction::Checkin || matches!(self, Self::Single(_)) {
                    eligible.push(row);
                }
            }
        }
        if matches!(self, Self::Selected(ids) if ids.len() != selected)
            || matches!(self, Self::Single(_) if selected == 0)
        {
            return Err(OperationError::TargetsChanged);
        }
        if eligible.is_empty() || eligible.len() > MAX_JOB_TARGETS {
            return Err(OperationError::Invalid);
        }
        Ok(eligible)
    }

    /// Single-account controls are attached to an existing row. Revalidate its ID on click.
    pub fn can_start(&self, action: JobAction, rows: &[CheckinAccountRow]) -> bool {
        matches!(self, Self::Single(_)) || self.eligible_rows(action, rows).is_ok()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JobTarget {
    pub id: String,
    pub name: String,
    pub provider: String,
    provider_id: String,
    provider_revision: Option<String>,
    enabled: bool,
    provider_enabled: Option<bool>,
}

#[derive(Clone)]
pub struct AccountJob {
    pub action: JobAction,
    scope: JobScope,
    pub targets: Vec<JobTarget>,
}

impl AccountJob {
    pub fn prepare(
        data: &CheckinData,
        action: JobAction,
        scope: JobScope,
    ) -> Result<Self, OperationError> {
        let providers: BTreeMap<_, _> = data.providers.iter().map(|p| (&p.id, p)).collect();
        let rows = scope.eligible_rows(action, &data.accounts)?;
        let mut targets = Vec::with_capacity(rows.len());
        for row in rows {
            let provider = providers.get(&row.provider_id);
            targets.push(JobTarget {
                id: row.id.clone(),
                name: row.name.clone(),
                provider_id: row.provider_id.clone(),
                provider: row
                    .provider_name
                    .clone()
                    .unwrap_or_else(|| row.provider_id.clone()),
                provider_revision: provider.and_then(|p| p.updated_at.clone()),
                enabled: row.enabled,
                provider_enabled: provider.map(|p| p.enabled),
            });
        }
        targets.sort_by(|a, b| a.id.cmp(&b.id));
        Ok(Self {
            action,
            scope,
            targets,
        })
    }

    pub fn current_targets_match(&self, data: &CheckinData) -> Result<bool, OperationError> {
        Ok(Self::prepare(data, self.action, self.scope.clone())?.targets == self.targets)
    }

    pub(super) fn revalidate(&self, source: &ConfigSource) -> Result<(), OperationError> {
        let CheckinSnapshot::Accounts(data) =
            read_source(source).map_err(|_| OperationError::Storage)?
        else {
            return Err(OperationError::TargetsChanged);
        };
        if !self.current_targets_match(&data).unwrap_or(false) {
            return Err(OperationError::TargetsChanged);
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JobStatus {
    Checked(CheckinStatus),
    Updated,
    Recent,
    Failed,
}

#[derive(Clone, Debug)]
pub struct JobItem {
    pub target: JobTarget,
    pub status: JobStatus,
    pub error_code: Option<String>,
    pub skip_reason: Option<String>,
    pub message: Option<String>,
    pub admission_error: Option<OperationError>,
}

#[derive(Clone, Debug)]
pub struct JobReport {
    pub action: JobAction,
    pub items: Vec<JobItem>,
}

fn safe_message(raw: &str) -> Option<String> {
    let scrubbed = ccr_checkin::services::checkin_service::redact_provider_feedback(raw);
    let message: String = scrubbed
        .trim()
        .chars()
        .filter(|c| !c.is_control())
        .take(240)
        .collect();
    (!message.is_empty()).then_some(message)
}

pub(super) async fn run_job(
    storage: &Storage,
    source: &ConfigSource,
    owner: &CheckinOperations,
    job: AccountJob,
) -> JobReport {
    let throttle = job.action == JobAction::Balance && !matches!(job.scope, JobScope::Single(_));
    let mut items = Vec::with_capacity(job.targets.len());
    // Sequential dispatch also serializes accounts sharing a provider origin.
    for target in job.targets {
        let mut item = JobItem {
            target,
            status: JobStatus::Failed,
            error_code: None,
            skip_reason: None,
            message: None,
            admission_error: None,
        };
        match service_for_item(storage, source, owner, &item.target, throttle) {
            Err(error) => item.admission_error = Some(error),
            Ok(None) => item.status = JobStatus::Recent,
            Ok(Some(service)) => {
                let result = match job.action {
                    JobAction::Checkin => service.checkin(&item.target.id).await.map(|r| {
                        item.status = JobStatus::Checked(r.status);
                        item.error_code = r.error_code;
                        item.skip_reason = r.skip_reason;
                        item.message = r.message.as_deref().and_then(safe_message);
                    }),
                    JobAction::Balance => service
                        .query_balance(&item.target.id)
                        .await
                        .map(|_| item.status = JobStatus::Updated),
                };
                if let Err(error) = result {
                    item.error_code = Some(error.error_code().to_owned());
                    item.message = safe_message(&error.to_string());
                }
            }
        }
        items.push(item);
    }
    JobReport {
        action: job.action,
        items,
    }
}

// Recheck the next item after the preceding provider request has finished. Another
// client can disable an account, move it, or persist a newer sample during that wait.
fn service_for_item(
    storage: &Storage,
    source: &ConfigSource,
    owner: &CheckinOperations,
    target: &JobTarget,
    throttle: bool,
) -> Result<Option<CheckinService>, OperationError> {
    let (account, provider) = network_target(storage, source, &target.id)?;
    if account.enabled != target.enabled
        || account.provider_id != target.provider_id
        || redact_log_text(&account.name) != target.name
        || redact_log_text(&provider.name) != target.provider
        || Some(provider.enabled) != target.provider_enabled
        || provider.updated_at.map(|at| at.to_rfc3339()) != target.provider_revision
    {
        return Err(OperationError::TargetsChanged);
    }
    if throttle
        && account.last_balance_check_at.is_some_and(|at| {
            chrono::Utc::now().signed_duration_since(at) < chrono::Duration::seconds(30)
        })
    {
        return Ok(None);
    }
    service_for_provider(storage, source, owner, provider).map(Some)
}
