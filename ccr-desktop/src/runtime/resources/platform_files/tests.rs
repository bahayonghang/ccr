#![allow(clippy::unwrap_used)]
use super::*;
#[test]
fn captured_platform_directories_and_request_identity_remain_separate() {
    let temp = tempfile::tempdir().unwrap();
    let source = ConfigSource::Fixture {
        profiles: temp.path().join("profiles.toml"),
        registry: temp.path().join("registry.toml"),
        delay: Default::default(),
    };
    let roots = roots(&source).unwrap();
    let project = roots.home.join("project");
    std::fs::create_dir_all(&project).unwrap();
    for (kind, scope, expected) in [
        (
            Kind::OpenCodeAgent,
            Scope::User,
            roots.home.join(".config/opencode/agents/review.md"),
        ),
        (
            Kind::OpenCodeCommand,
            Scope::Project(project.clone()),
            project.join(".opencode/commands/review.md"),
        ),
        (
            Kind::AntigravityCommand,
            Scope::User,
            roots.home.join(".gemini/commands/review.toml"),
        ),
        (
            Kind::AntigravityCommand,
            Scope::Project(project.clone()),
            project.join(".gemini/commands/review.toml"),
        ),
    ] {
        let key = RequestKey {
            environment: EnvironmentId::Local("fixture".into()),
            platform: platform(kind),
            generation: 1,
            request_id: 1,
        };
        let name = format!("review.{}", kind.extension());
        let ReadResult::Opened(session) = read(
            &source,
            &key,
            Read::Open {
                kind,
                scope: scope.clone(),
                name,
                creating: true,
            },
        )
        .unwrap() else {
            panic!("session required")
        };
        assert_eq!(session.file.path(), expected);
        session.file.save(kind.initial_text()).unwrap();
        let mut wrong = key.clone();
        wrong.environment = EnvironmentId::Ssh("fixture".into());
        assert!(matches!(
            read(&source, &wrong, Read::List { kind, scope }),
            Err(Error::Unsupported)
        ));
    }
    assert!(matches!(
        store(&source, Kind::Extension, &Scope::Project(project)),
        Err(Error::Unsupported)
    ));
    assert!(!temp.path().join(".gemini").exists());
}
