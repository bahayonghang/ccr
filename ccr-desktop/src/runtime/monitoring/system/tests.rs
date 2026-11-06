use super::*;

#[test]
fn synthetic_and_remote_refuse_before_host_probe() {
    for environment in [
        EnvironmentId::Local("fixture".into()),
        EnvironmentId::Ssh("fixture".into()),
        EnvironmentId::Wsl("fixture".into()),
    ] {
        assert!(
            permitted(&environment, true, || -> Result<(), Error> {
                panic!("must not probe host")
            })
            .is_err()
        );
    }
    assert_eq!(
        permitted(
            &EnvironmentId::Ssh("fixture".into()),
            false,
            || -> Result<(), Error> { panic!("must not probe host") }
        ),
        Err(Error::LocalOnly)
    );
    assert_eq!(
        permitted(
            &EnvironmentId::Local("fixture".into()),
            true,
            || -> Result<(), Error> { panic!("must not probe host") }
        ),
        Err(Error::Fixture)
    );
}

#[test]
fn host_measurements_keep_missing_capacity_and_cpu_distinct_from_zero() {
    assert_eq!(
        Memory::measured(0, 0),
        Memory {
            total_bytes: None,
            used_bytes: None
        }
    );
    assert_eq!(
        Memory::measured(8_589_934_592, 2_147_483_648).used_percent(),
        Some(25.)
    );
    assert_eq!(Memory::measured(100, 101).used_percent(), None);
    assert_eq!(Memory::measured(100, 0).used_percent(), Some(0.));
    assert_eq!(average_cpu(&[]), None);
    assert_eq!(average_cpu(&[f32::NAN]), None);
    assert_eq!(average_cpu(&[0., 100.]), Some(50.));
    assert_eq!(average_cpu(&[0.]), Some(0.));
    assert_eq!(average_cpu(&[-1., 20.]), None);
    assert_eq!(average_cpu(&[101.]), None);
    assert_eq!(text(Some(" \n ".into())), None);
}

#[tokio::test]
async fn actual_requests_deny_fixture_and_remote_sources() {
    use crate::runtime::{ConfigSource, Services};
    use ccr_app::runtime::{Completion, Platform, RequestKey, RuntimeOwner, TaskEvent};
    let owner = RuntimeOwner::new().unwrap_or_else(|e| panic!("runtime: {e}"));
    let services = Services::new(
        owner.handle(),
        ConfigSource::Fixture {
            profiles: "invalid-relative-profiles".into(),
            registry: "invalid-relative-registry".into(),
            delay: Duration::ZERO,
        },
    );
    for (environment, expected) in [
        (EnvironmentId::Local("fixture".into()), Error::Fixture),
        (EnvironmentId::Ssh("fixture".into()), Error::LocalOnly),
        (EnvironmentId::Wsl("fixture".into()), Error::LocalOnly),
    ] {
        let mut subscription = services
            .system_info(RequestKey {
                environment,
                platform: Platform::Claude,
                generation: 1,
                request_id: 1,
            })
            .unwrap_or_else(|e| panic!("admission: {e:?}"));
        loop {
            match subscription.next().await {
                Some(TaskEvent::Terminal(result)) => {
                    assert!(
                        matches!(result.as_ref(),Completion::Failed(error) if *error==expected)
                    );
                    break;
                }
                Some(_) => {}
                None => panic!("missing terminal"),
            }
        }
    }
    tokio::task::spawn_blocking(move || owner.shutdown())
        .await
        .unwrap_or_else(|e| panic!("shutdown: {e}"));
}
