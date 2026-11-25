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
        if matches!(&scope, JobScope::Selected(ids) if ids.len() > 1000) {
            return Err(OperationError::Invalid);
        }
        let providers: BTreeMap<_, _> = data.providers.iter().map(|p| (&p.id, p)).collect();
        let mut matched = 0;
        let mut targets = Vec::new();
        for row in &data.accounts {
            let chosen = match &scope {
                JobScope::AllEnabled => row.enabled,
                JobScope::Selected(ids) => ids.contains(&row.id),
                JobScope::Single(id) => id == &row.id,
            };
            if !chosen {
                continue;
            }
            matched += 1;
            if action == JobAction::Balance && !row.enabled && !matches!(scope, JobScope::Single(_))
            {
                continue;
            }
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
        if matches!(&scope, JobScope::Selected(ids) if ids.len() != matched) {
            return Err(OperationError::TargetsChanged);
        }
        if targets.is_empty() || targets.len() > 1000 {
            return Err(OperationError::Invalid);
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

    pub(super) fn revalidate(&self, source: &ConfigSource) -> Result<CheckinData, OperationError> {
        let CheckinSnapshot::Accounts(data) =
            read_source(source).map_err(|_| OperationError::Storage)?
        else {
            return Err(OperationError::TargetsChanged);
        };
        if !self.current_targets_match(&data).unwrap_or(false) {
            return Err(OperationError::TargetsChanged);
        }
        Ok(data)
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
    let scrubbed = redact_log_text(raw);
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
) -> Result<JobReport, OperationError> {
    let data = job.revalidate(source)?;
    let samples: BTreeMap<_, _> = data
        .accounts
        .iter()
        .filter_map(|row| {
            let at =
                chrono::DateTime::parse_from_rfc3339(row.last_balance_check_at.as_deref()?).ok()?;
            Some((&row.id, at))
        })
        .collect();
    let throttle = job.action == JobAction::Balance && !matches!(job.scope, JobScope::Single(_));
    let mut items = Vec::with_capacity(job.targets.len());
    // Sequential dispatch also serializes accounts sharing a provider origin.
    for target in job.targets {
        let recent = throttle
            && samples.get(&target.id).is_some_and(|at| {
                chrono::Utc::now().signed_duration_since(*at) < chrono::Duration::seconds(30)
            });
        let mut item = JobItem {
            target,
            status: JobStatus::Failed,
            error_code: None,
            skip_reason: None,
            message: None,
            admission_error: None,
        };
        if recent {
            item.status = JobStatus::Recent;
        } else {
            match service_for_account(storage, source, owner, &item.target.id) {
                Err(error) => item.admission_error = Some(error),
                Ok(service) => {
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
        }
        items.push(item);
    }
    Ok(JobReport {
        action: job.action,
        items,
    })
}
