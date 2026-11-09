# CLI version fixture correction

The recorded Windows just ci run remaining-windows-ci-after-doctor failed in cli_versions_fast_mode_returns_expected_shape at the existing 5000ms assertion. The run did not record per-tool timings. The specific source of the delay remains unknown.

The old test called compute_cli_versions with no detected-status fixture and could execute installed host CLIs. The original executable passed its exact test in 0.00s under an empty PATH; that diagnostic does not reproduce or explain the earlier delay.

The test now uses the existing TestProcessEnv lock and restoration protocol. PATH points to an empty temporary directory before controlled scripts are written. Claude, Codex and Gemini use explicit absolute fixture paths. Windows scripts use cmd built-in echo; Unix scripts use /bin/sh and built-in printf. The test exercises actual ProcessGateway execution, verifies stable order, versions, success, missing fallback and not-installed states, and keeps the 3500ms per-tool deadline, parallelism 4 and total 5000ms assertion.

Only the test module changed. The exact Windows test passed in 0.05s; command compilation and execution took 59.976s. Formal aggregate validation and the Unix branch remain separate checks. This fixture correction does not explain the historical exporter 0xc0000005.

Before and empty-PATH evidence were collected by remaining_gates. Root implemented and validated the fixture after ownership transfer. The dedicated checker independently reviewed the test-only diff.
