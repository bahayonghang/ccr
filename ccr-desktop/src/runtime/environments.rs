//! Environment reads stay on the blocking worker. Render does not spawn processes.
use super::Services;
use ccr_app::environments::{EnvironmentReport, InspectOptions};
use ccr_app::runtime::{AdmissionError, RequestKey, TaskSubscription};

pub fn load(
    services: &Services,
    key: RequestKey,
) -> Result<TaskSubscription<EnvironmentReport, ()>, AdmissionError> {
    let live = !services.is_fixture();
    services.executor.read(key, move |cancel, _| {
        let options = if live {
            InspectOptions::live()
        } else {
            InspectOptions::fixture()
        };
        Ok(ccr_app::environments::inspect(options, &|| {
            cancel.is_cancelled()
        }))
    })
}
