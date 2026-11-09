
#[cfg(all(test, windows))]
mod architecture_baseline_deadline {
    use super::*;
    #[tokio::test]
    async fn a09_native_stream_must_obey_shortened_descriptor_deadline() {
        let temporary = tempfile::tempdir().unwrap();
        let cli = LlmusageCli { paths: AppPaths::from_root(temporary.path()), binary: std::env::var("CCR_ARCH_BASELINE_HELPER").unwrap() };
        assert_eq!(cli.command().unwrap().1.timeout(), std::time::Duration::from_millis(30));
        let token = CancellationToken::new();
        let run = run_sync_stream(&cli, SyncCommandOptions::default(), token.clone(), |_| std::future::ready(Ok(())));
        tokio::pin!(run);
        let deadline_missed = tokio::time::timeout(std::time::Duration::from_millis(300), &mut run).await.is_err();
        if deadline_missed {
            token.cancel();
            let cleanup = tokio::time::timeout(std::time::Duration::from_secs(5), &mut run).await.expect("owned test child must be cleaned up");
            assert!(cleanup.is_err());
        }
        assert!(temporary.path().join("child.pid").exists(), "real synthetic child must have started");
        assert!(!deadline_missed, "real silent child exceeded descriptor deadline; explicit cancellation was needed");
    }
}
