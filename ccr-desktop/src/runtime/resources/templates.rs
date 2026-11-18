use crate::runtime::remote_resources::{self as remote, Remote};
use crate::runtime::{ConfigSource, Services};
use ccr_app::environments::EnvironmentPolicy;
use ccr_app::resources::environment::Family;
use ccr_app::{
    resources::prompt_presets,
    runtime::{AdmissionError, Completion, EnvironmentId, Platform, RequestKey, TaskSubscription},
};
use std::sync::Arc;

pub struct Catalog {
    remote: Remote,
    pub builtin: Vec<ccr_skills::BuiltinPrompt>,
    pub saved: prompt_presets::Catalog,
    environment: EnvironmentId,
    platform: Platform,
}
pub enum Command {
    List,
    Commit {
        catalog: Arc<Catalog>,
        mutation: prompt_presets::Mutation,
    },
}
pub enum Outcome {
    Listed(Arc<Catalog>),
    Committed,
}

fn execute(
    source: &ConfigSource,
    key: &RequestKey,
    command: Command,
    remote: Remote,
) -> Result<Outcome, prompt_presets::Error> {
    use prompt_presets::Error;
    if remote.is_none()
        && EnvironmentPolicy::LocalOnly
            .admit(&key.environment)
            .is_err()
    {
        return Err(Error::Invalid);
    }
    if let Command::Commit { catalog, mutation } = command {
        if catalog.environment != key.environment || catalog.platform != key.platform {
            return Err(Error::Invalid);
        }
        catalog.saved.commit(mutation)?;
        return Ok(Outcome::Committed);
    }
    let platform = match key.platform {
        Platform::Claude => ccr_config::Platform::Claude,
        Platform::Codex => ccr_config::Platform::Codex,
        Platform::Gemini => ccr_config::Platform::Gemini,
        _ => return Err(Error::Invalid),
    };
    let roots = remote::roots(source, &remote).map_err(|_| Error::Storage)?;
    let saved = prompt_presets::Catalog::read(
        platform,
        &roots.home,
        &roots.ccr.join("backups/prompt-presets"),
        &roots.locks,
    )?;
    Ok(Outcome::Listed(Arc::new(Catalog {
        remote,
        builtin: ccr_skills::get_builtin_prompts(),
        saved,
        environment: key.environment.clone(),
        platform: key.platform,
    })))
}
fn run(
    source: &ConfigSource,
    key: &RequestKey,
    command: Command,
    io: Result<Option<ccr_app::environments::io::EnvironmentIo>, ccr_app::environments::io::Error>,
    cancel: tokio_util::sync::CancellationToken,
) -> Result<Outcome, prompt_presets::Error> {
    let remote = match &command {
        Command::Commit { catalog, .. } => catalog.remote.clone(),
        _ => remote::capture(
            io,
            Family::Presets(key.platform),
            Default::default(),
            cancel.clone(),
        )
        .map_err(prompt_presets::Error::Remote)?,
    };
    let writing = matches!(command, Command::Commit { .. });
    let result = execute(source, key, command, remote.clone())?;
    if writing {
        remote::publish(&remote, cancel).map_err(prompt_presets::Error::Remote)?;
    }
    Ok(result)
}
impl Services {
    pub fn template_request(
        &self,
        key: RequestKey,
        command: Command,
    ) -> Result<TaskSubscription<Outcome, prompt_presets::Error>, AdmissionError> {
        let source = self.source.clone();
        let run_key = key.clone();
        let io = self.resource_io(&key.environment);
        if matches!(command, Command::Commit { .. }) {
            self.executor.job(key, move |cancel, _| {
                if cancel.is_cancelled() {
                    return Completion::Cancelled;
                }
                match run(&source, &run_key, command, io, cancel) {
                    Ok(value) => Completion::Success(value),
                    Err(error) => Completion::Failed(error),
                }
            })
        } else {
            self.executor.read(key, move |cancel, _| {
                run(&source, &run_key, command, io, cancel)
            })
        }
    }
}
