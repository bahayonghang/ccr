use crate::runtime::{ConfigSource, Services};
use ccr_app::{
    resources::prompt_presets,
    runtime::{AdmissionError, Completion, EnvironmentId, Platform, RequestKey, TaskSubscription},
};
use std::sync::Arc;

pub struct Catalog {
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
) -> Result<Outcome, prompt_presets::Error> {
    use prompt_presets::Error;
    if !matches!(key.environment, EnvironmentId::Local(_)) {
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
    let roots = super::roots(source).map_err(|_| Error::Storage)?;
    let saved = prompt_presets::Catalog::read(
        platform,
        &roots.home,
        &roots.ccr.join("backups/prompt-presets"),
        &roots.locks,
    )?;
    Ok(Outcome::Listed(Arc::new(Catalog {
        builtin: ccr_skills::get_builtin_prompts(),
        saved,
        environment: key.environment.clone(),
        platform: key.platform,
    })))
}
impl Services {
    pub fn template_request(
        &self,
        key: RequestKey,
        command: Command,
    ) -> Result<TaskSubscription<Outcome, prompt_presets::Error>, AdmissionError> {
        let source = self.source.clone();
        let run_key = key.clone();
        if matches!(command, Command::Commit { .. }) {
            self.executor.job(key, move |cancel, _| {
                if cancel.is_cancelled() {
                    return Completion::Cancelled;
                }
                match execute(&source, &run_key, command) {
                    Ok(value) => Completion::Success(value),
                    Err(error) => Completion::Failed(error),
                }
            })
        } else {
            self.executor
                .read(key, move |_, _| execute(&source, &run_key, command))
        }
    }
}
