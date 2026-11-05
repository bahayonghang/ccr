//! Environment reads stay on the blocking worker. Render does not spawn processes.
use super::Services;
use ccr_app::environments::{EnvironmentReport, InspectOptions};
use ccr_app::runtime::{AdmissionError, RequestKey, TaskSubscription};

pub fn load(
    services: &Services,
    key: RequestKey,
) -> Result<TaskSubscription<EnvironmentReport, ()>, AdmissionError> {
    let live = !services.is_fixture();
    let delay = services.fixture_delay();
    services.executor.read(key, move |cancel, _| {
        // Fixture delay is cancellable QA only. A live read does not sleep here.
        if !delay.is_zero() {
            let deadline = std::time::Instant::now() + delay;
            while !cancel.is_cancelled() && std::time::Instant::now() < deadline {
                std::thread::sleep(
                    deadline
                        .saturating_duration_since(std::time::Instant::now())
                        .min(std::time::Duration::from_millis(10)),
                );
            }
        }
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
