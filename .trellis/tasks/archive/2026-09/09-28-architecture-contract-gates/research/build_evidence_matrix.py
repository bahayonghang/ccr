"""Capture scoped architecture evidence; never change acceptance or task status."""
from pathlib import Path
import base64
import hashlib
import json
import re
import subprocess
import sys

sys.stdout.reconfigure(encoding='utf-8')
ROOT = Path(__file__).resolve().parents[4]
TASKS = ROOT / '.trellis/tasks'
OUT = Path(__file__).resolve().parent
PARENT = TASKS / '09-28-cli-tauri-architecture'
LEDGER = json.loads((PARENT / 'execution-ledger.json').read_text(encoding='utf-8'))
TASK = {row['key']: TASKS / row['task'] for row in LEDGER['tasks']}
GROUPS = {}
PREVIOUS_PATH = OUT / 'requirements-evidence.json'
PREVIOUS = json.loads(PREVIOUS_PATH.read_text(encoding='utf-8')) if PREVIOUS_PATH.exists() else {}
RUNS = {}
REMAINING_SOURCE_EPOCHS = ('remaining-source-freeze.json', 'remaining-final-source-freeze.json', 'remaining-permissions-source-freeze.json', 'remaining-portability-source-freeze.json', 'remaining-platform-tests-source-freeze.json')


def relative(path):
    return path.relative_to(ROOT).as_posix()


def artifact(path):
    path = Path(path)
    assert path.is_file(), path
    data = path.read_bytes()
    return {'path': relative(path), 'sha256': hashlib.sha256(data).hexdigest(), 'bytes': len(data)}


def capture_freeze(path, role):
    frozen = json.loads(path.read_text(encoding='utf-8'))
    differences = []
    for name, digest in frozen['files'].items():
        current = ROOT / name
        observed = hashlib.sha256(current.read_bytes()).hexdigest() if current.is_file() else ('non_file_path' if current.exists() else None)
        if observed != digest:
            differences.append({'path': name, 'captured_sha256': digest, 'current_sha256': observed})
    return {
        **artifact(path), 'captured_at': frozen['captured_at'],
        'files': len(frozen['files']),
        'present_files': sum(digest is not None for digest in frozen['files'].values()),
        'absent_paths': sorted(name for name, digest in frozen['files'].items() if digest is None),
        'matches_current_sources': not differences, 'current_differences': differences,
        'role': role + '; files counts all registered paths, including explicit null deletion markers',
    }


def group(key, source, selectors, task, logs, level, limits):
    path = ROOT / source
    lines = path.read_text(encoding='utf-8').splitlines()
    locations = []
    for selector in selectors.split(';'):
        matches = [i for i, line in enumerate(lines, 1) if selector in line]
        assert matches, (key, selector, source)
        locations.append({'selector': selector, 'line': matches[0]})
    evidence = []
    for log in logs.split(';'):
        p = TASK[task] / log
        entry = artifact(p)
        text = p.read_text(encoding='utf-8', errors='replace')
        if p.suffix == '.json':
            obj = json.loads(text)
            text = obj.get('output', text)
            if 'exit_code' in obj:
                entry['exit_code'] = obj['exit_code']
        entry['result_lines'] = [line.strip() for line in text.splitlines() if any(token in line for token in ('test result:', 'Test Files', 'Tests ', 'failed (', 'passed (')) or line.strip().startswith('Ran ') or line.strip() == 'OK'][-12:]
        entry['selected_case_lines'] = [line.strip() for line in text.splitlines() if any(s in line for s in selectors.split(';')) and (' ... ' in line or ' > ' in line)][:20]
        evidence.append(entry)
    captured = artifact(path)
    prior = PREVIOUS.get('test_groups', {}).get(key, {})
    history = list(prior.get('prior_source_captures', []))
    previous_source = prior.get('test_source')
    if previous_source and previous_source != captured and previous_source not in history:
        history.append(previous_source)
    GROUPS[key] = {'test_source': captured, 'source_capture_role': 'current_selector_index_only', 'prior_source_captures': history, 'cases': locations, 'level': level, 'evidence': evidence, 'limits': limits}
    for entry in evidence:
        prior_log = next((old for old in prior.get('evidence', []) if old['path'] == entry['path']), None)
        if prior_log and Path(entry['path']).suffix == '.log':
            assert prior_log['sha256'] == entry['sha256'], ('historical raw log changed', entry['path'])
        entry['source_binding'] = {'status': 'not_established_by_hash_refresh', 'note': 'Current source SHA locates selectors. It does not establish which source bytes produced this historical log.'}


def metadata_artifact_path(name, metadata_path):
    path = Path(name)
    return path if path.is_absolute() else (ROOT / path if path.parts[0] == '.trellis' else metadata_path.parent / path)


def recorded_mapped_run(key, metadata_path, record, groups=()):
    """Add a finished run with preserved logs and its actual source map."""
    assert record.get('state') in (None, 'finished'), ('unfinished command', key)
    assert isinstance(record.get('exit_code'), int), key
    assert record.get('started_at_utc') and record.get('finished_at_utc'), key
    logs = record.get('logs', {'combined': record.get('log', metadata_path.with_suffix('.log').name)})
    hashes = record['log_sha256'] if isinstance(record['log_sha256'], dict) else {'combined': record['log_sha256']}
    assert set(logs) == set(hashes), key
    streams = {}
    for name, filename in logs.items():
        entry = artifact(metadata_artifact_path(filename, metadata_path))
        assert entry['sha256'] == hashes[name], (key, name)
        streams[name] = entry
    primary = 'combined' if 'combined' in streams else 'stdout'
    assert primary in streams, key
    before = record['sources_before']
    after = record['sources_after']
    assert isinstance(before, dict) and isinstance(after, dict) and before, key
    assert set(before) == set(after), (key, 'registered source paths changed')
    assert all(digest is None or re.fullmatch(r'[0-9a-f]{64}', digest) for digest in [*before.values(), *after.values()]), key
    changes = [
        {'path': path, 'before': before.get(path), 'after': after.get(path)}
        for path in sorted(set(before) | set(after))
        if before.get(path) != after.get(path)
    ]
    if 'source_changes' in record:
        reported = record['source_changes']
        assert isinstance(reported, list), (key, 'source comparison type')
        expected = [item['path'] for item in changes] if all(isinstance(item, str) for item in reported) else changes
        assert expected == reported, (key, 'source comparison')
    attribution = {
        'capture_mode': 'command_before_and_after_registered_source_map',
        'source_files_sha256': {path: digest for path, digest in before.items() if digest is not None},
        'absent_paths': sorted(path for path, digest in before.items() if digest is None),
        'source_paths': len(before), 'source_end_paths': len(after),
        'post_command_source_changes': changes,
        'source_comparison_basis': 'recorded comparison checked against maps' if 'source_changes' in record else 'recomputed from recorded before and after maps',
        'reported_source_changes': record.get('source_changes'),
        'source_stable_between_snapshots': not changes,
        'hash_scope': 'Actual inline command-start and command-finish registered maps. A nominal older freeze reference cannot replace these maps or reattribute earlier runs.',
    }
    if metadata_path.name in ('remaining-csp-focused-final.json', 'remaining-csp-types-final.json', 'remaining-csp-lint.json'):
        assert all(path.startswith(('src/', 'tests/')) for path in before), key
        attribution['source_path_base'] = 'ccr-ui'
        attribution['resolved_source_files_sha256'] = {'ccr-ui/' + path: digest for path, digest in before.items() if digest is not None}
        attribution['source_path_basis'] = 'These frontend command receipts record paths relative to ccr-ui; raw metadata keys remain unchanged.'
    execution = {
        'metadata': artifact(metadata_path), 'label': key,
        'command': record['command'], 'command_argv': record.get('argv', record['command']),
        'cwd': record.get('cwd'), 'state': 'finished', 'exit_code': record['exit_code'],
        'completion_basis': 'explicit finished state' if record.get('state') else 'recorded exit code and finish timestamp; metadata has no state field',
        'log_path_basis': 'recorded path' if 'logs' in record or 'log' in record else 'same-stem sibling log verified against recorded SHA256',
        'started_at': record['started_at_utc'], 'completed_at': record['finished_at_utc'],
        'environment_overrides': record.get('environment_overrides'),
        'platform': record.get('platform'),
        'rustc_version': record.get('rustc_version', record.get('rustc')),
        'git_head_before': record.get('head_before', record.get('git_head')),
        'git_head_after': record.get('head_after', record.get('git_head')),
        'source_attribution': attribution,
        'exit_code_source': 'finished command metadata',
    }
    if record.get('source_freeze_reference'):
        path = metadata_artifact_path(record['source_freeze_reference'], metadata_path)
        captured = artifact(path)
        assert captured['sha256'] == record['source_freeze_sha256'], path
        frozen = json.loads(path.read_text(encoding='utf-8'))['files']
        delta = [
            {'path': name, 'freeze_sha256': frozen.get(name), 'command_start_sha256': before.get(name)}
            for name in sorted(set(frozen) | set(before))
            if frozen.get(name) != before.get(name)
        ]
        execution['declared_freeze_reference'] = {
            'manifest': captured, 'command_start_matches_declared_freeze': before == frozen,
            'command_start_differences': delta,
            'role': 'Metadata reference retained verbatim. Only an equal files map binds this run to the referenced freeze; differences remain explicit.',
        }
    if record.get('parent_source_freeze'):
        parent = artifact(metadata_artifact_path(record['parent_source_freeze'], metadata_path))
        assert parent['sha256'] == record['parent_source_freeze_sha256'], key
        execution['parent_freeze_reference'] = {
            'manifest': parent, 'role': 'Ancestry only; this parent is not the execution source map.',
        }
    for epoch_name in REMAINING_SOURCE_EPOCHS:
        epoch_path = OUT / epoch_name
        if not epoch_path.exists():
            continue
        epoch = json.loads(epoch_path.read_text(encoding='utf-8'))
        if before == epoch['files']:
            execution['source_epoch'] = {
                'manifest': artifact(epoch_path), 'source_before_equals_epoch': True,
                'source_after_equals_epoch': after == epoch['files'],
                'role': 'Explicit equality between this run inline maps and the new remaining source epoch; earlier runs keep their original source identities.',
            }
            break
    if record.get('generated_before_snapshot'):
        snapshot_path = metadata_artifact_path(record['generated_before_snapshot'], metadata_path)
        captured = artifact(snapshot_path)
        snapshot = json.loads(snapshot_path.read_text(encoding='utf-8'))
        assert len(snapshot) == record['generated_before_count'], key
        for name, entry in snapshot.items():
            assert hashlib.sha256(base64.b64decode(entry['bytes_base64'], validate=True)).hexdigest() == entry['sha256'], (key, name)
        execution['generated_comparison'] = {
            'before_snapshot': captured,
            'before_count': record['generated_before_count'],
            'after_count': record['generated_after_count'],
            'changes': record['generated_changes'],
            'before_bytes_verified': True,
            'role': 'Saved pre-command bytes plus command metadata comparison. No separate complete post-command map is asserted.',
        }
    if 'generated_before' in record and 'generated_after' in record:
        before_generated = record['generated_before']
        after_generated = record['generated_after']
        generated_changes = [
            {'path': path, 'before': before_generated.get(path), 'after': after_generated.get(path)}
            for path in sorted(set(before_generated) | set(after_generated))
            if before_generated.get(path) != after_generated.get(path)
        ]
        reported = record['generated_changes']
        assert isinstance(reported, list), (key, 'generated comparison type')
        expected = [item['path'] for item in generated_changes] if all(isinstance(item, str) for item in reported) else generated_changes
        assert expected == reported, (key, 'generated comparison')
        execution['generated_map_comparison'] = {
            'before_count': len(before_generated), 'after_count': len(after_generated),
            'maps_equal': before_generated == after_generated, 'changes': generated_changes,
            'reported_changes': reported,
            'role': 'Complete generated-file hash maps preserved in command metadata.',
        }
    text = (ROOT / streams[primary]['path']).read_text(encoding='utf-8', errors='replace')
    raw = {
        **streams[primary], 'log_streams': streams,
        'combined_log_order': record.get('combined_log_order'), 'execution': execution,
        'result_lines': [line.strip() for line in text.splitlines() if any(token in line for token in ('test result:', 'Test Files', 'Tests ', 'failed (', 'passed (')) or line.strip().startswith('Ran ') or line.strip() == 'OK'][-20:],
    }
    RUNS[key] = {**raw, 'reported_counts': {}, 'acceptance_role': 'recorded_scope_only_no_full_gate_inference'}
    for name in groups:
        target = GROUPS[name]
        selectors = [case['selector'] for case in target['cases']]
        source_sha = attribution.get('resolved_source_files_sha256', attribution['source_files_sha256']).get(target['test_source']['path'])
        entry = {
            **raw,
            'selected_case_lines': [line.strip() for line in text.splitlines() if any(selector in line for selector in selectors) and (' ... ' in line or ' > ' in line)][:20],
            'source_binding': {
                'status': attribution['capture_mode'] if source_sha else 'not_recorded_for_test_source',
                'recorded_source_sha256': source_sha,
                'current_capture_matches_recorded': source_sha == target['test_source']['sha256'] if source_sha else None,
                'note': 'Source attribution uses only this command inline map. Earlier evidence is preserved separately.',
            },
        }
        target['evidence'] = [old for old in target['evidence'] if old['path'] != entry['path']] + [entry]


def recorded_run(key, metadata_path, record, groups=(), source_path=None):
    """Attach an existing run without promoting its source capture or result."""
    if isinstance(record.get('sources_before'), dict):
        assert source_path is None, key
        recorded_mapped_run(key, metadata_path, record, groups)
        return
    log = Path(record.get('log', metadata_path.with_suffix('.log').name))
    log = log if log.is_absolute() else (ROOT / log if log.parts[0] == '.trellis' else metadata_path.parent / log)
    raw = artifact(log)
    if record.get('log_sha256'):
        assert raw['sha256'] == record['log_sha256'], log
    attribution = record.get('source_attribution', {})
    if not attribution and isinstance(record.get('source_before_sha256'), dict):
        source_map = record['source_before_sha256']
        attribution = {
            'capture_mode': 'command_before_registered_source_map_with_post_comparison',
            'source_files_sha256': {path: digest for path, digest in source_map.items() if digest is not None},
            'absent_paths': sorted(path for path, digest in source_map.items() if digest is None),
            'post_command_source_changes': record.get('source_changes'),
            'hash_scope': 'Only the recorded command-start registered source map; post-command differences remain explicit. This does not reattribute earlier runs.',
        }
    if source_path:
        before = record['source_before_sha256']
        after = record['source_after_sha256']
        attribution = {'capture_mode': 'command_start_and_finish_source_sha256', 'source_files_sha256': {source_path: before}, 'source_end_sha256': after, 'source_stable_during_command': before == after, 'hash_scope': 'named source only, not the complete dependency graph'}
    raw['execution'] = {
        'metadata': artifact(metadata_path), 'label': key,
        'command': record.get('command_argv', record.get('command')),
        'cwd': record.get('cwd'), 'exit_code': record.get('exit_code'),
        'started_at': record.get('started_at', record.get('started_at_utc')),
        'completed_at': record.get('completed_at', record.get('finished_at_utc')),
        'rustc_version': record.get('rustc_version'),
        'environment_overrides': record.get('command_environment_override', record.get('environment_overrides', record.get('environment_override'))),
        'scope': record.get('scope'),
        'exit_code_source': record.get('exit_code_source', 'saved command metadata'),
        'source_attribution': attribution,
        'declared_source_sha': record.get('source_sha'),
        'declared_lock_sha': record.get('lock_sha'),
    }
    if record.get('source_freeze'):
        frozen_path = Path(record['source_freeze'])
        frozen_path = frozen_path if frozen_path.is_absolute() else ROOT / frozen_path
        frozen = artifact(frozen_path)
        assert frozen['sha256'] == record['source_freeze_sha256'], frozen_path
        raw['execution']['frozen_source_evidence'] = {
            'manifest': frozen,
            'source_changes_since_freeze': record.get('source_changes'),
            'role': 'Recorded pre-run freeze reference. A post-run source comparison is supplied only when source_changes is present. It does not bind unrelated historical runs.',
        }
    capture_fields = {field: record[field] for field in ('capture_wrapper_exit_code', 'capture_wrapper_error') if field in record}
    if capture_fields:
        raw['execution']['capture_wrapper'] = capture_fields
    if record.get('before_snapshot') and record.get('after_snapshot'):
        snapshots = {}
        snapshot_data = {}
        for phase in ('before', 'after'):
            captured_path = Path(record[f'{phase}_snapshot'])
            captured_path = captured_path if captured_path.is_absolute() else ROOT / captured_path
            snapshots[phase] = artifact(captured_path)
            assert snapshots[phase]['sha256'] == record[f'{phase}_snapshot_sha256'], captured_path
            snapshot_data[phase] = json.loads(captured_path.read_text(encoding='utf-8'))
        before = snapshot_data['before']
        after = snapshot_data['after']
        sources_equal = before['freeze_files'] == after['freeze_files']
        generated_equal = before['generated_types'] == after['generated_types']
        assert sources_equal and generated_equal and not before['freeze_mismatches'] and not after['freeze_mismatches'], key
        raw['execution']['command_snapshots'] = {
            **snapshots, 'source_paths': len(before['freeze_files']),
            'generated_files': len(before['generated_types']),
            'sources_equal': sources_equal, 'generated_equal': generated_equal,
        }
        if not attribution:
            attribution = {
                'capture_mode': 'command_before_and_after_registered_source_snapshot',
                'source_files_sha256': {path: digest for path, digest in before['freeze_files'].items() if digest is not None},
                'absent_paths': sorted(path for path, digest in before['freeze_files'].items() if digest is None),
                'source_stable_between_snapshots': sources_equal,
                'hash_scope': 'Only registered frozen source paths. Current fingerprints do not bind the preceding crashed process.',
            }
            raw['execution']['source_attribution'] = attribution
    raw['result_lines'] = [line.strip() for line in log.read_text(encoding='utf-8', errors='replace').splitlines() if any(token in line for token in ('test result:', 'Test Files', 'Tests ', 'failed (', 'passed (')) or line.strip().startswith('Ran ') or line.strip() == 'OK'][-12:]
    RUNS[key] = {**raw, 'reported_counts': {field: record[field] for field in ('matched_passed', 'test_files_passed', 'locale_leaf_keys', 'passed_count', 'matched_test_count', 'aggregate_passed_count', 'complete_named_pass_selector_count') if field in record}, 'acceptance_role': 'recorded_scope_only_no_full_gate_inference'}
    if 'accepted_as_behavior_evidence' in record:
        RUNS[key]['accepted_as_behavior_evidence'] = record['accepted_as_behavior_evidence']
        if not record['accepted_as_behavior_evidence']:
            RUNS[key]['acceptance_role'] = 'not_behavior_evidence_zero_matching_tests'
            RUNS[key]['selector_issue'] = record.get('selector_issue')
    for name in groups:
        target = GROUPS[name]
        selectors = [case['selector'] for case in target['cases']]
        entry = dict(raw)
        entry['selected_case_lines'] = [line.strip() for line in log.read_text(encoding='utf-8', errors='replace').splitlines() if any(selector in line for selector in selectors) and (' ... ' in line or ' > ' in line)][:20]
        source_sha = attribution.get('source_files_sha256', {}).get(target['test_source']['path'])
        entry['source_binding'] = {
            'status': attribution.get('capture_mode', 'not_recorded_for_test_source') if source_sha else 'not_recorded_for_test_source',
            'recorded_source_sha256': source_sha,
            'current_capture_matches_recorded': source_sha == target['test_source']['sha256'] if source_sha else None,
            'note': 'Only the declared capture_mode and named source hashes bind this run. The capture is not a complete dependency snapshot. A missing or different recorded source is not repaired by capturing a new SHA.',
        }
        target['evidence'] = [old for old in target['evidence'] if old['path'] != entry['path']] + [entry]


group('repository', 'crates/ccr-config/src/managers/config/repository_tests.rs', 'platform_and_service_processes_keep_both_updates;desktop_and_service_processes_keep_both_updates;windows_verbatim_and_service_processes_keep_both_updates;reads_do_not_initialize_autofix_or_lock_files;explicit_platform_and_legacy_claude_ignore_registry_order;strict_patch_rejects_invalid_missing_duplicate_and_stale_without_writes;patch_preserves_unedited_fields_and_renames_current_and_default', 'T02', 'check-repository-alias-final.log;root-linux-repository-final-alias.json', 'real filesystem and independent subprocess adapters', 'Windows plus Linux Rust 1.95 MSRV. Repository desktop fixture uses ConfigService; actual Tauri handler is config_handlers. No real UNC share or symlink-alias guarantee.')
group('profile_cli', 'crates/ccr-cli/src/application/profile_contract.rs', 'profile_preflight_contract;profile_each_write_contract;profile_success_replay_contract;profile_ancillary_contract;profile_external_version_contract;profile_rename_each_write_contract;profile_adapters_keep_application_boundary', 'T02', 'check-cli-alias-final.log', 'shared application with temporary filesystem and injected write faults', 'Claude/Codex/Grok application requests. Not interactive terminal acceptance or OS multi-file atomicity.')
group('profile_tui', 'crates/ccr-tui/src/tui/profile_backend.rs', 'apply_preflight_contract;apply_each_write_contract;apply_ancillary_contract;apply_success_replay_contract', 'T02', 'check-tui-alias-final.log', 'actual TUI application adapter and presenter', 'No interactive TUI acceptance. Calls the shared contract harness with an isolated root.')
group('profile_desktop', 'ccr-ui/src-tauri/src/commands/profile_lifecycle.rs', 'profile_preflight_contract;profile_success_replay_contract;profile_ancillary_contract;profile_each_write_contract', 'T02', 'check-desktop-alias-final.log', 'actual synchronous desktop service adapter', 'No native IPC/WebView. Rename helper coverage is also listed in the independent report.')
group('profile_warning_binary', 'crates/ccr/tests/commands/claude_profile.rs', 'claude_profile_switch_reports_committed_history_warning', 'T02', 'check-cli-warning-output.log', 'actual CLI binary, output and filesystem', 'Windows synthetic credentials; one committed activation and history_failed record; no real account.')
group('journal', 'crates/ccr-core/src/core/write_journal.rs', 'journal_restores_post_publish_failure;journal_rollback_obeys_windows_alias_leaf_lock', 'T02', 'check-journal-alias-final.log;root-linux-write-journal-faults-final-alias.json', 'actual guarded writes, metadata and CAS faults', 'Windows DACL/read-only and Linux mode branches. No process-crash recovery transaction guarantee.')
group('config_handlers', 'ccr-ui/src-tauri/src/commands/config/contract_tests.rs', 'actual_handlers_switch_and_enable_commit_runtime_and_policy;legacy_and_unsupported_platform_requests_never_write;strict_patch_rejects_unknown_wrong_types_missing_collision_without_write;patch_retains_secret_unknown_fields_inactive_marker_and_checks_version;desktop_and_service_processes_preserve_independent_changes', 'T03', 'check-backend-final.log;check-enable-final.log', 'actual Tauri handlers and temporary filesystem, plus six worker processes', 'Generic /configs explicitly Claude-only; platform enable compensation separately covers Claude/Codex/Grok. No native WebView.')
group('config_page', 'ccr-ui/tests/configs/configs-actions.smoke.test.tsx', 'switches an enabled row with an explicit platform;exposes an enable action for a disabled row;refreshes a committed warning;updates tabs, summaries and mounted cards', 'T09', 'research/check-frontend-smoke.log', 'actual ConfigsView, domain wrapper, generated client and invokeRuntime', 'Bottom Tauri invoke, unrelated queries and confirmation decisions are mocked. Filesystem behavior is covered separately by config_handlers, not native end-to-end.')
group('diagnostics', 'crates/ccr/tests/diagnostics_contract.rs', 'validate_binary_six_state_matrix_is_read_only;validate_binary_applied_api_and_subscription_profiles_follow_domain_rules;validate_binary_native_denied_read_is_distinct_from_missing;doctor_binary_default_scope_includes_grok_and_preserves_simplified_profiles;doctor_binary_grok_is_supported_and_legacy_adapters_are_explicit;validate_binary_claude_current_uses_valid_marker_precedence_without_repair', 'T04', 'check-binary-final.log', 'actual CLI binary, exit codes, stdout and unchanged file inventory', 'Windows three platforms and API-key/subscription fixtures; Linux/macOS binary acceptance unrun.')
group('backups', 'crates/ccr-core/src/core/guarded_write.rs', 'test_fixed_clock_preserves_each_preimage_and_legacy_backup;test_fixed_clock_multiprocess_backups_preserve_every_version;test_rotation_ties_are_stable_for_old_and_new_names;test_backup_preserves_windows_readonly_attribute;test_backup_preserves_private_windows_dacl;test_backup_preserves_private_unix_mode;test_dir_backup_rotation_keeps_ten', 'T02', 'check-guarded-alias-final.log;root-linux-guarded-write-final-alias.json', 'real filesystem, native metadata and independent child processes', 'Windows/Linux; Linux consumers are not a complete CLI-reader matrix; macOS unrun.')
group('pending', 'crates/ccr-codex/src/services/codex_oauth_pending_store.rs', 'oauth_pending_create_replace_cancel_and_expiry_never_make_copies;oauth_pending_permission_failure_preserves_old_or_missing_target;oauth_pending_errors_and_default_output_redact_sentinels;oauth_pending_preserves_owner_only_permissions', 'T05', 'check-linux-oauth-pending.json', 'real guarded pending store with temporary files and injected permission denial', 'Linux Rust 1.95 five tests. Simulated denial is not native OS denial. Windows DACL evidence remains in T05 check-report and T11 adapter logs; macOS unrun.')
group('usage_registry', 'ccr-ui/src-tauri/src/usage_jobs.rs', 'cancelled_result_rejects_late_failure;failed_result_rejects_cancel_and_late_progress;cancel_before_runner_start_prevents_spawn_and_keeps_admission;all_terminal_results_reject_duplicate_cancel_progress_and_completion;running_cancel_keeps_token_and_reports_cleanup_failure', 'T11', 'check-usage.log', 'production registry, cancellation tokens and controlled runner barrier', 'Not a full Tauri App click. T11 log supersedes the T06 intermediate control-policy failure.')
group('usage_executor', 'ccr-ui/src-tauri/src/llmusage_adapter/cli_lifecycle_tests.rs', 'silent_process_obeys_descriptor_deadline;stdout_eof_does_not_disable_deadline_while_child_lives;cleanup_error_and_hung_reap_override_cancelled;externally_held_stderr_has_bounded_join_and_cleanup_error;stdout_flood_without_eof_fails_at_one_mebibyte;stderr_retains_at_most_sixty_four_capped_lines;native_windows_silent_child_times_out_and_is_reaped;native_windows_running_cancel_reaps_the_child', 'T11', 'check-usage.log', 'production executor with controlled fake processes plus two Windows native child tests', 'Fake-process deadlines do not prove native tree cleanup; use process_tree. Old native deadline omission has a scoped red with only descriptor timeout shortened. Old cleanup-error precedence and new descendant fixtures remain source-only for the baseline.')
group('process_tree', 'crates/ccr-core/src/core/process_gateway.rs', 'live_tree_confirmation_times_out_without_claiming_cleanup;reaped_windows_parent_still_cleans_its_descendant_tree;termination_escalates_after_unix_parent_exits_on_term;dropping_reaped_unix_parent_still_terminates_descendants', 'T06', 'check-core-process-linux-final.log;check-process-smoke-final.log', 'native process-tree and managed cleanup tests', 'Windows 10 Tauri plus 7 core; Linux 9 core on MSRV 1.95; no macOS or original-binary red run for the newly added descendant fixtures.')
group('command_page', 'ccr-ui/tests/commands/command-workbench-lifecycle.smoke.test.tsx', 'restores the running job and output after a real route unmount;blocks synchronous duplicate submission;records terminal history once;shows a failed terminal subscription;keeps a failed history summary visible;keeps recovery suspended when a pending start resolves', 'T07', 'research/check-final-regression.log', 'real route/shell/store lifecycle with controlled API promises and events', 'API facade is mocked in this route fixture. Actual backend command owner and runtime policy are separately tested. History is one write attempt per job, not cross-IPC exactly-once; failed writes are visible and not blindly retried.')
group('command_listener', 'ccr-ui/tests/shell/event-bridge-leak.smoke.test.tsx', '卸载之后 listen 才 resolve;ignores command callbacks after shell cleanup', 'T07', 'research/check-final-regression.log', 'real event bridge, controlled listen/unlisten promises', 'Tauri event transport is mocked; native event delivery is not accepted by this fixture.')
group('command_owner', 'ccr-ui/src-tauri/src/commands/command_exec.rs', 'command_owner_receives_cancel_before_cleanup_and_keeps_start_admission;foreground_barrier_does_not_block_existing_command_owner_controls;terminal_snapshot_keeps_non_zero_exit_as_failed_with_output', 'T11', 'check-command-exec.log', 'production command owner with barriers, tokens and snapshots', 'Windows Rust behavior; paired with renderer tests, not full native UI end-to-end.')
group('settings_mapping', 'ccr-ui/tests/platforms/settings-lossless.smoke.test.ts', 'changes only Codex model and preserves notifications array;does not resend stale Codex sibling values;keeps OpenCode notify;does not issue any mutation when no field is dirty', 'T09', 'research/check-frontend-smoke.log', 'actual mapper/domain/generated wrapper to mocked bottom invoke', 'Transport is mocked. The codex_settings_backend group supplies real Rust projection/merge evidence; combined coverage remains separate-layer tests. OpenCode real filesystem end-to-end is not covered.')
group('codex_settings_backend', 'ccr-ui/src-tauri/src/commands/codex.rs', 'read_codex_config_accepts_tui_notifications_event_array;apply_codex_settings_update_preserves_tui_notifications_event_array;apply_codex_settings_update_clears_nested_fields_with_nulls', 'T10', '../09-28-cli-tauri-architecture/research/root-acceptance-after-tui-tauri.log', 'one real temporary-file read/projection plus two real in-memory merge tests', 'Windows Codex backend: file-read fixture checks notifications JSON array and unknown notification_condition in the parsed config; partial-merge fixture checks serialization preserves array/condition/method/status_line; null-clear fixture checks in-memory None fields. No disk-save roundtrip, invoke, native WebView or real accounts.')
group('settings_page', 'ccr-ui/tests/platforms/settings-capabilities.smoke.test.tsx', 'disables managed model inputs;shows Codex unknown enums and notification arrays;preserves the raw draft on conflict;waits for plaintext confirmation;does not read source files in a remote environment', 'T09', 'research/check-frontend-smoke.log', 'real platform routes, mappers, generated wrappers, CodeMirror and Toast', 'Bottom invoke/confirmation and DOM geometry mocked. Real Grok filesystem/CAS/no-backup fixture paired; native WebView/CSP and real SSH/WSL unrun.')
group('grok_settings', 'ccr-ui/src-tauri/src/commands/grok.rs', 'settings_and_raw_saves_preserve_the_configured_backup_inventory', 'T08', 'grok-rust.log', 'real temporary filesystem typed/raw handlers and backup sentinels', 'Windows fixture, no real account; check-report separately records final independent 16-test execution.')
group('auth_query', 'ccr-ui/tests/platforms/auth-query-contract.smoke.test.tsx', 'preserves authoritative backend unsupported;keeps pending distinct;keeps confirmed session data marked stale;claims the off action before confirmation', 'T09', 'research/check-frontend-smoke.log', 'real BaseAuth and Grok adapter with controlled domain responses', 'Grok domain transport and confirmation mocked; no real OAuth login.')
group('auth_cache', 'ccr-ui/tests/platforms/auth-probe-recovery-check.smoke.test.tsx', 'it(', 'T09', 'research/check-auth-probe-production-cache.log;research/check-frontend-smoke.log', 'real Query cache with production staleTime and controlled auth responses', 'A successful probe is not treated as fresh session data; native transport unrun.')
group('settings_session', 'ccr-ui/tests/platforms/settings-session.smoke.test.tsx', 'freezes synchronously from the real shell environment event;keeps the original baseline after refetch;rejects a late initial response;does not apply an old save completion', 'T09', 'research/check-frontend-smoke.log', 'real shell event bridge, Query and form session with deferred adapters', 'Environment API and settings adapter are controlled. Backend captured-target fixture supplies separate filesystem-owner evidence.')
group('settings_raw_session', 'ccr-ui/tests/platforms/settings-source-session.smoke.test.tsx', 'retains the same raw editor and token;does not discard a raw draft until', 'T09', 'research/check-frontend-smoke.log', 'real CodeMirror local draft and environment session', 'Controlled source callbacks/confirmation; no real remote host.')
group('settings_environment', 'ccr-ui/src-tauri/src/commands/claude_settings_tests.rs', 'settings_expected_environment_mismatch_rejects_read_and_update_before_io;settings_expected_environment_update_keeps_original_arc_after_a_b_a_replacement;settings_matching_local_environment_keeps_atomic_settings_manager', 'T09', 'research/check-backend-behavior.log', 'actual command-side admission and captured environment targets', 'Controlled environments plus local temporary files; real SSH/WSL sessions unrun.')
group('control_policy', 'ccr-ui/src-tauri/src/commands/runtime_policy.rs', 'control_matrix_command_and_install_delivery;control_matrix_usage_delivery;control_matrix_oauth_delivery;background_permit_transfer_is_single_use_and_preserves_shared_process_exclusion;usage_owner_controls_reach_existing_job_during_each_foreground_import;install_owner_cancel_and_recent_reach_attempt_before_cleanup', 'T11', 'check-runtime_policy.log', 'production runtime policies plus real owner/token/attempt barriers', 'C01-C06/23 IDs in T11 report. No global Parallel conversion; Windows scoped execution.')
group('oauth', 'ccr-ui/src-tauri/src/commands/codex_auth/oauth_tests.rs', 'bind_race_and_save_failure_never_publish_active_login;start_owns_listener_and_storage_before_return_and_reuses_live_id;silent_socket_times_out_and_releases_listener_without_success;slow_http_body_is_cancelled_and_server_observes_closed_connection;successful_completion_is_unique_and_does_not_expose_sentinels;stale_cleanup_retry_cannot_clear_a_replacement_login', 'T11', 'check-oauth-green.log', 'actual loopback sockets, pending store and controlled HTTP exchange/commit', 'Windows isolated credentials/network; real provider login and Linux/macOS OAuth unrun.')
group('registry', 'ccr-ui/src-tauri/src/commands/handler_registry.rs', 'command_registry_shape_matches_current_handler_surface;command_inventory_document_matches_registry', 'T09', 'research/check-backend-registry.log', 'registry, generated inventory and ACL contract tests', '21 passed; full DTO bindings guard remains separate. Snapshot metadata count is not behavior acceptance.')
group('usage_ui', 'ccr-ui/tests/usage/usage-import-lifecycle.smoke.test.tsx', 'it(', 'T06', 'check-frontend-lifecycle.log', 'actual usage hook/reducer with controlled IPC events', '18 tests including normalization; no native WebView or real llmusage command.')
group('aggregate', 'scripts/ci/test_architecture_contract_gates.py', 'test_all_local_platforms_reuse_complete_desktop_gate;test_real_aggregate_propagates_desktop_test_failure;test_ui_behavior_entries_do_not_write_bindings', 'T10', 'research/gates-governance-final.log', 'real just aggregate recipe graph executed against temporary controlled cargo leaves', 'Windows execution; all three recipe definitions checked. The failure fixture does not run real Cargo tests; actual final full gates remain root-owned.')
group('bindings', 'ccr-ui/tests/quality/bindings-transaction.smoke.test.ts', 'restores exact bytes after direct;check preserves original bytes;reports restoration failure', 'T10', 'research/gates-frontend-final.log;research/gates-full-bindings-final.json', 'real temporary directories and injected export/normalizer/spawn failures, plus actual full bindings command', '25 includes calendar tests. Full guard exit 0 preserved all 230 generated type files byte-for-byte; earlier unexplained drift remains recorded.')
group('calendar', 'ccr-ui/tests/usage/usage-date-window.smoke.test.ts', 'counts calendar days across DST', 'T10', 'research/gates-frontend-final.log', 'real helper in subprocesses with UTC/America Chicago/Asia Shanghai TZ', 'Calendar-day semantics only; no time-series network data needed.')

# Each row adds the mechanism and observable boundary to the full ledger requirement.
group('codex_settings_persistence', 'ccr-ui/src-tauri/src/commands/codex_settings.rs', 'model_only_update_preserves_notifications_and_untouched_fields_on_disk', 'T10', 'research/continuation-codex-tests.log', 'production private persistence helper with real temporary-file write and TOML reread', 'Model-only patch preserves the entire fixture TOML value, including notification event arrays and unknown root/profile/MCP values. No invoke, State/cache invalidation, native WebView, comment/order preservation, or universal TOML-losslessness claim.')
group('lint_main_tests', 'ccr-ui/src-tauri/src/main.rs', 'close_action_hides_main_window_to_tray_when_enabled;close_action_requests_quit_when_close_to_tray_and_confirm_are_disabled', 'T10', 'research/continuation-close-action-tests.log', 'unchanged close-action cases after test-module relocation', 'Two Windows-executable tests only; the cfg(macos) chrome test was not run.')
group('lint_state_tests', 'ccr-ui/src-tauri/src/state.rs', 'desktop_shell_preferences_defaults_when_file_is_missing;desktop_shell_preferences_load_legacy_files_with_default_tray_panel_state;desktop_shell_preferences_round_trip_via_json_file', 'T10', 'research/continuation-state-tests.log', 'three preferences regressions after item relocation', 'Does not rerun all previous usage lifecycle behavior.')
group('lint_claude_test', 'ccr-ui/src-tauri/src/commands/claude.rs', 'read_claude_settings_from_env_reads_top_level_env', 'T10', 'research/continuation-claude-read-tests.log', 'equivalent boolean assertion in the existing environment-read fixture', 'One targeted test; earlier Claude environment architecture changes are not re-reviewed by this increment.')
group('lint_codex_profile_test', 'ccr-ui/src-tauri/src/commands/codex.rs', 'codex_list_profiles_reads_only_ccr_profiles_source', 'T10', 'research/continuation-codex-tests.log', 'synchronous harness with current-thread runtime and retained env guard', 'Included in the same 56-case Codex run, not an extra run or additional unique passed test.')
group('settings_visible_i18n', 'ccr-ui/tests/platforms/settings-visible-i18n.smoke.test.tsx', 'translates Codex save and known effort options in both locales while preserving an unknown value', 'T10', 'research/continuation-settings-smoke.log', 'actual translator and Settings controls with bottom invoke mock', 'One source case in the four-file, 45-case run; compact Vitest log reports aggregate counts. Separate 24-check i18n script counts 4523 locale leaf keys per locale, not 4523 behavior tests.')
group('doctor_deadline', 'crates/ccr-cli/src/commands/codex/fix.rs', 'doctor_timeout_terminates_parent_and_grandchild;doctor_deadline_applies_before_fixture_is_ready', 'T10', 'research/continuation-doctor-focused-retry.log;research/continuation-doctor-final-suite.log', 'actual Windows and Linux managed-process fixtures and production deadline entrypoint', 'Ready parent/grandchild cleanup and unready production deadline are separate cases. The controlled red proves a readiness/deadline race mechanism, not the original host delay cause. Initial focused startup STATUS_ACCESS_VIOLATION remains recorded. Linux Rust 1.98 adds three existing Doctor selectors, not full Linux CI; macOS remains unrun.')
group('tauri_process_gateway', 'ccr-ui/src-tauri/src/process/gateway.rs', 'descriptor_catalog_is_closed_and_resolves_expected_tools;oauth_url_policy_accepts_authorize_and_fixed_loopback_callback;oauth_url_policy_rejects_unsafe_schemes_and_host_confusion;owned_process_registry_filters_ports_and_requests_cancellation;sidecar_hash_mismatch_is_rejected;release_sidecar_resolution_requires_hash_and_never_falls_back_to_path;development_sidecar_and_hash_validation_are_deterministic;foreground_stdin_is_written_and_streams_are_collected', 'T10', 'research/continuation-linux-tauri-process.log', 'Linux native foreground process plus Tauri gateway catalog, URL and ownership fixtures', 'Eight Linux cases on the original freeze passed with an unused test-import warning. Zero-case guard target is excluded. The descriptor and URL cases are not native tree-cleanup or WebView/CSP acceptance; macOS remains unrun.')
group('oauth_windows_pending_failure', 'ccr-ui/src-tauri/src/commands/codex_auth.rs', 'oauth_pending_storage_failure_preserves_memory_and_redacts_output', 'T10', 'research/continuation-windows-oauth-corrected-selector.log', 'actual Windows-only pending-store adapter fixture after cfg-only import correction', 'One exact selector passed on the final freeze. This does not establish provider login, all OAuth behavior, or native WebView acceptance. The preceding zero-match selector run remains separately excluded.')

owner_path = OUT / 'continuation-tauri-lint-implementation.json'
owner = json.loads(owner_path.read_text(encoding='utf-8'))
owner_groups = {
    'tauri_all_targets_clippy': (),
    'tauri_codex': ('codex_settings_backend', 'codex_settings_persistence', 'lint_codex_profile_test'),
    'tauri_state': ('lint_state_tests',),
    'tauri_close-action': ('lint_main_tests',),
    'tauri_claude-read': ('lint_claude_test',),
    'settings_smoke': ('settings_page', 'settings_session', 'settings_raw_session', 'settings_visible_i18n'),
    'i18n': (), 'typescript': (), 'scoped_eslint': (),
}
for run in owner['validation']:
    if run['label'] in owner_groups:
        recorded_run(run['label'], owner_path, run, owner_groups[run['label']])
for key, filename, groups in [
    ('linux_198_process', 'continuation-linux-process-online.json', ('process_tree',)),
    ('linux_198_codex_clippy', 'continuation-linux-codex-clippy-online.json', ()),
    ('security_root_audit', 'continuation-security-root-audit-after.json', ()),
    ('security_tauri_audit', 'continuation-security-tauri-audit-after.json', ()),
    ('security_dependency_governance', 'continuation-security-dependency-governance.json', ()),
]:
    metadata_path = OUT / filename
    recorded_run(key, metadata_path, json.loads(metadata_path.read_text(encoding='utf-8')), groups)
RUNS['linux_198_process']['source_scope_note'] = 'Metadata source_sha identifies codex_process_service.rs, not the tested ccr-core process_gateway.rs. The log proves 9 Linux/Rust 1.98 scoped tests; no command-start core-source hash is supplied.'
RUNS['security_tauri_audit']['warning_boundary'] = 'Exit 0 retains nine pre-existing warnings; no advisory ignore was added.'
GROUPS['process_tree']['limits'] += ' Continuation adds a separate Linux Rust 1.98 9-case run. The planned macOS native matrix is still required and not run.'
for key, filename, groups in [
    ('doctor_original_narrow', 'continuation-doctor-baseline.json', ()),
    ('doctor_controlled_red', 'continuation-doctor-controlled-red.json', ()),
    ('doctor_startup_failure', 'continuation-doctor-focused.json', ()),
    ('doctor_focused_retry', 'continuation-doctor-focused-retry.json', ('doctor_deadline',)),
    ('doctor_package', 'continuation-doctor-final-suite.json', ('doctor_deadline',)),
    ('doctor_clippy', 'continuation-doctor-final-clippy.json', ()),
    ('linux_198_doctor', 'continuation-linux-doctor.json', ('doctor_deadline',)),
]:
    metadata_path = OUT / filename
    recorded_run(key, metadata_path, json.loads(metadata_path.read_text(encoding='utf-8')), groups, 'crates/ccr-cli/src/commands/codex/fix.rs')
RUNS['doctor_package']['test_partitions'] = {'unit_passed': 345, 'integration_passed': 12, 'doctest_passed': 1, 'doctest_ignored': 1}
RUNS['doctor_controlled_red']['cause_boundary'] = 'Original production and timeout test body unchanged; controlled fixture keeps child startup blocked. The original host delay cause remains unconfirmed.'
RUNS['doctor_startup_failure']['cause_boundary'] = 'STATUS_ACCESS_VIOLATION before test execution; cause unconfirmed. Successful list/retry does not erase this failure.'
metadata_path = PARENT / 'research/root-continuation-ci.json'
recorded_run('root_full_ci_before_doctor', metadata_path, json.loads(metadata_path.read_text(encoding='utf-8')))
metadata_path = PARENT / 'research/root-continuation-final-ci.json'
recorded_run('root_full_ci_after_doctor', metadata_path, json.loads(metadata_path.read_text(encoding='utf-8')))
RUNS['root_full_ci_after_doctor']['failure_boundary'] = {
    'aggregate_stage': 'Tauri CI',
    'command_stage': 'tauri-bindings-check: CLI export_bindings',
    'process_exit': '0xc0000005 / STATUS_ACCESS_VIOLATION',
    'cause': 'unconfirmed',
    'test_count_banner_observed_for_failed_process': False,
    'formal_frontend_stage_reached': False,
    'tauri_behavior_passed': 407,
    'tauri_behavior_ignored': 1,
    'tauri_guard_passed': 2,
    'later_scoped_success_does_not_replace_aggregate_result': True,
}
metadata_path = PARENT / 'research/root-continuation-frontend.json'
recorded_run('formal_frontend_gate', metadata_path, json.loads(metadata_path.read_text(encoding='utf-8')))
RUNS['formal_frontend_gate']['failure_boundary'] = {
    'rule': 'no-console', 'errors': 5,
    'protected_paths': ['ccr-ui/.tmp-desktop-probe.mjs', 'ccr-ui/.tmp-insights-visual.mjs'],
    'independent_of_final_root_ci': True,
    'scoped_eslint_does_not_replace_formal_gate': True,
}
for key, metadata_path in (
    ('linux_195_workspace_msrv', OUT / 'continuation-linux-msrv-workspace.json'),
    ('windows_195_tauri_msrv', OUT / 'continuation-windows-msrv-tauri.json'),
    ('ui_production_build', PARENT / 'research/root-continuation-ui-build.json'),
    ('bindings_followup', OUT / 'continuation-bindings-startup-bindings-check.json'),
    ('standalone_tauri_ci', OUT / 'continuation-bindings-startup-tauri-ci.json'),
):
    recorded_run(key, metadata_path, json.loads(metadata_path.read_text(encoding='utf-8')))
RUNS['bindings_followup']['test_partitions'] = {'cli_binding_exports': 24, 'usage_binding_exports': 9, 'tauri_binding_exports': 197, 'zero_case_targets_not_counted': True}
RUNS['standalone_tauri_ci']['test_partitions'] = {'tauri_behavior_passed': 407, 'tauri_behavior_ignored': 1, 'tauri_guard_passed': 2, 'cli_binding_exports': 24, 'usage_binding_exports': 9, 'tauri_binding_exports': 197, 'inventory_passed': 1, 'inventory_overlaps_behavior_suite': True, 'zero_case_targets_not_counted': True}
for key in ('bindings_followup', 'standalone_tauri_ci'):
    RUNS[key]['failure_history_boundary'] = 'Later independent execution passed without source or generated-file drift. Original root CI remains exit 1; startup crash cause is unconfirmed. Binary hashes were captured after the failure, not during the failed process.'
for key, filename, groups in (
    ('linux_198_tauri_process_before_import_fix', 'continuation-linux-tauri-process.json', ('tauri_process_gateway',)),
    ('linux_198_core_process_before_import_fix', 'continuation-linux-core-process-final.json', ('process_tree',)),
):
    metadata_path = OUT / filename
    recorded_run(key, metadata_path, json.loads(metadata_path.read_text(encoding='utf-8')), groups)
    text = (ROOT / RUNS[key]['path']).read_text(encoding='utf-8')
    RUNS[key]['observed_unique_passed_selectors'] = sorted({line[5:-7] for line in text.splitlines() if line.startswith('test ') and line.endswith(' ... ok')})
RUNS['linux_198_tauri_process_before_import_fix']['warning_boundary'] = 'Exit 0 with eight passed cases. The unused Mutex test-import warning motivated a later cfg-only correction; this run was not a failed test run.'
FINAL_PLATFORM_RUNS = (
    ('linux_198_tauri_strict_final', 'continuation-linux-tauri-strict-final.json', ()),
    ('linux_oauth_zero_match', 'continuation-linux-tauri-oauth-final.json', ()),
    ('linux_198_tauri_process_final', 'continuation-linux-tauri-process-after-import.json', ('tauri_process_gateway',)),
    ('windows_195_tauri_msrv_final', 'continuation-windows-msrv-tauri-after-import.json', ()),
    ('windows_oauth_zero_match', 'continuation-windows-oauth-after-import.json', ()),
    ('tauri_fmt_final', 'continuation-tauri-fmt-after-import.json', ()),
    ('linux_oauth_corrected', 'continuation-linux-oauth-corrected-selector.json', ('oauth',)),
    ('windows_oauth_corrected', 'continuation-windows-oauth-corrected-selector.json', ('oauth_windows_pending_failure',)),
)
for key, filename, groups in FINAL_PLATFORM_RUNS:
    metadata_path = OUT / filename
    record = json.loads(metadata_path.read_text(encoding='utf-8'))
    recorded_run(key, metadata_path, record, groups)
    if 'test' in record['command']:
        text = (ROOT / RUNS[key]['path']).read_text(encoding='utf-8')
        RUNS[key]['observed_unique_passed_selectors'] = sorted({line[5:-7] for line in text.splitlines() if line.startswith('test ') and line.endswith(' ... ok')})
        aggregate = sum(int(value) for value in re.findall(r'test result: ok\. (\d+) passed;', text))
        intact = len(RUNS[key]['observed_unique_passed_selectors'])
        RUNS[key]['observed_test_counts'] = {'aggregate_passed': aggregate, 'intact_named_selector_count': intact, 'named_capture_complete': aggregate == intact, 'overlaps_other_runs': True}
        if aggregate != intact:
            RUNS[key]['named_capture_limit'] = 'Captured stdout/stderr contains interleaved or truncated case lines. Aggregate counts come from Rust summaries; only intact names are retained, with no reconstruction of missing selectors.'
GROUPS['oauth']['limits'] = 'Original Windows isolated credentials/network evidence is retained. Final-freeze Linux Rust summaries report 20 passed; only 18 intact named pass selectors can be extracted from the lossy capture. Missing names are not reconstructed. Real provider login and macOS OAuth remain unrun; zero-match attempts are excluded.'
GROUPS['tauri_process_gateway']['limits'] += ' The final-freeze rerun reports eight passed in Rust summaries but only five intact named selectors because capture lines are interleaved or truncated. Missing names are not reconstructed; overlapping runs are not summed as unique cases.'
for key, filename in (('frontend_full_tests', 'root-continuation-frontend-tests.json'), ('frontend_coverage', 'root-continuation-coverage.json')):
    metadata_path = PARENT / 'research' / filename
    recorded_run(key, metadata_path, json.loads(metadata_path.read_text(encoding='utf-8')))
    raw_text = (ROOT / RUNS[key]['path']).read_text(encoding='utf-8')
    files = re.search(r'Test Files\s+(\d+) passed', raw_text)
    tests = re.search(r'\bTests\s+(\d+) passed', raw_text)
    assert files and tests, key
    RUNS[key]['observed_test_counts'] = {'files': int(files[1]), 'tests': int(tests[1]), 'overlaps_other_runs': True}
    if key == 'frontend_coverage':
        metrics = re.findall(r'^(Statements|Branches|Functions|Lines)\s*:\s*([0-9.]+)%\s*\(\s*(\d+)/(\d+)\s*\)', raw_text, re.MULTILINE)
        assert len(metrics) == 4
        RUNS[key]['coverage'] = {name: {'percent': float(percent), 'covered': int(covered), 'total': int(total)} for name, percent, covered, total in metrics}
web_path = OUT / 'continuation-web-review.json'
web = json.loads(web_path.read_text(encoding='utf-8'))
web_receipts = []
for receipt in web['receipts']:
    captured = artifact(ROOT / receipt['path'])
    assert captured['sha256'] == receipt['sha256'], receipt['path']
    web_receipts.append(captured)
WEB = {'report': artifact(web_path), 'evidence_type': 'synthetic_ipc_web_interaction', 'status': web['status'], 'checks': web['checks'], 'boundaries': web['boundaries'], 'receipts': web_receipts, 'native_webview_csp_acceptance': 'not_verified'}

# Remaining work adds runs. The preceding 41 run objects remain unchanged.
group('runtime_style_nonce', 'ccr-ui/tests/ui/runtime-style-nonce.smoke.test.tsx', 'applies the page style nonce to the real confirmation scroll lock and releases it on unmount;uses the script nonce when the page has no nonced style;keeps ordinary web scroll locking without introducing a nonce', 'T10', 'research/remaining-csp-focused-final.log', 'actual ConfirmModal and scroll-lock styles in DOM tests, with synthetic bootstrap nonce', 'Three target files passed seven tests. The log reports aggregate counts without individual case names. Native WebKitGTK/CSP confirmation is recorded separately by attempts6 and7 under their own source epochs. Native post-save style DOM removal and body scroll-lock restoration were not measured.')
group('permission_noop_repository', 'crates/ccr-config/src/managers/config/repository_tests.rs', 'no_op_mutations_preserve_content_time_and_backups;no_op_mutations_reject_a_leaf_replacement;no_op_mutations_enforce_secret_mode_without_replacing_the_file;rejected_mutations_and_queries_preserve_existing_permissions;no_op_hardening_survives_uncommitted_journal_rollback;no_op_hardening_rejects_an_active_journal_version_conflict', 'T10', 'research/remaining-permissions-linux-config-final.log', 'real repository and temporary files with Unix mode, inode, mtime, backup, CAS and journal assertions', 'Linux focused execution. The journal test has no content entries: it proves that this metadata-only repair creates no rollback entry. Existing content compensation still restores its recorded metadata under the original contract. No new Windows DACL or macOS execution is inferred.')
group('permission_noop_guard', 'crates/ccr-core/src/core/guarded_write.rs', 'test_secret_permission_no_op_requires_an_existing_matching_version;test_secret_permission_no_op_waits_for_the_guarded_leaf_lock;test_secret_permission_no_op_policy_fault_preserves_metadata', 'T10', 'research/remaining-permissions-linux-core-guarded.log', 'actual guarded file handle, existing version, held leaf lock and injected pre-change policy fault', 'The policy fault is injected before the metadata change; it is not a measured OS chmod denial. These checks do not extend OAuth pending lifecycle or existing content journal compensation guarantees.')
group('permission_base_regression', 'crates/ccr-config/src/platforms/base.rs', 'test_profile_structured_writes_keep_owner_only_permissions', 'T10', 'research/remaining-linux-permissions-red.log;research/remaining-permissions-linux-config-final.log', 'unchanged original platform regression run against pre-fix and repaired owners', 'The exact Linux red selector exits 101 with zero source drift. The unchanged selector then passes in the 104-case ccr-config library run. Distinct epochs and all original failed coverage maps remain visible.')
group('permission_atomic_policy', 'crates/ccr-core/src/core/atomic_writer.rs', 'test_async_secret_mode_ignores_process_umask;test_async_secret_preserves_only_stricter_existing_mode', 'T10', 'research/remaining-permissions-linux-core-atomic.log', 'existing asynchronous atomic writer Unix mode policy tests after shared mode selection extraction', 'The nine-test focused atomic writer suite is a regression check for the shared policy. It is separate from metadata-only no-op enforcement and does not prove native Windows DACL behavior.')
group('store_path_platform_contract', 'crates/ccr-store/src/sessions/providers.rs', 'restores_codex_live_jsonl_under_tempfile_with_windows_path_shapes', 'T10', 'research/remaining-linux-store-path-repro.log', 'actual provider restore-source regression with native temporary files and platform-specific path assertions', 'The exact pre-fix Linux selector fails. Separate Linux and Windows exact runs each pass the repaired selector; Linux library execution overlaps that case. Each focused receipt captures four source paths, not a full 269-path epoch. Native and canonical paths succeed in both platform runs; backslash success is retained on Windows and absence/rejection is asserted on Unix. macOS is unrun. Prior 268-path runs are not rebound to this source.')
REMAINING_RUNS = (
    ('remaining_windows_frontend', 'remaining-windows-frontend-check.json'),
    ('remaining_windows_ci_before_fixture', 'remaining-windows-ci.json'),
    ('remaining_linux_tauri_ci', 'remaining-linux-tauri-ci.json'),
    ('remaining_linux_coverage', 'remaining-linux-coverage-tauri.json'),
    ('remaining_linux_native_build', 'remaining-linux-native-build.json'),
    ('remaining_windows_ci_after_fixture', 'remaining-windows-ci-after-doctor.json'),
    ('remaining_csp_focused', 'remaining-csp-focused-final.json'),
    ('remaining_csp_types', 'remaining-csp-types-final.json'),
    ('remaining_csp_lint', 'remaining-csp-lint.json'),
    ('remaining_csp_ui_build', 'remaining-csp-ui-build.json'),
    ('remaining_linux_native_after_csp', 'remaining-linux-native-after-csp.json'),
    ('remaining_linux_tauri_ci_final', 'remaining-linux-tauri-ci-final.json'),
    ('remaining_linux_coverage_final', 'remaining-linux-coverage-tauri-final.json'),
    ('remaining_windows_ci_final', 'remaining-windows-ci-final.json'),
)
OPTIONAL_REMAINING_RUNS = (
    ('remaining_linux_workspace_quality_final', 'remaining-linux-workspace-quality-final.json'),
    ('remaining_linux_workspace_coverage_final', 'remaining-linux-workspace-coverage-final.json'),
    ('remaining_linux_permissions_red', 'remaining-linux-permissions-red.json'),
    ('remaining_linux_workspace_quality_after_python', 'remaining-linux-workspace-quality-after-python.json'),
    ('remaining_linux_store_path_red', 'remaining-linux-store-path-repro.json'),
)
PERMISSIONS_RUNS = (
    ('remaining_windows_ci_after_permissions', 'remaining-windows-ci-after-permissions.json'),
    ('remaining_linux_tauri_ci_after_permissions', 'remaining-linux-tauri-ci-after-permissions.json'),
    ('remaining_linux_coverage_after_permissions', 'remaining-linux-coverage-tauri-after-permissions.json'),
    ('remaining_linux_native_after_permissions', 'remaining-linux-native-after-permissions.json'),
    ('remaining_linux_workspace_quality_after_permissions', 'remaining-linux-workspace-quality-after-permissions.json'),
    ('remaining_linux_workspace_coverage_after_permissions', 'remaining-linux-workspace-coverage-after-permissions.json'),
)
PORTABILITY_RUNS = (
    ('remaining_windows_ci_after_octal', 'remaining-windows-ci-after-octal.json'),
    ('remaining_linux_tauri_ci_after_octal', 'remaining-linux-tauri-ci-after-octal.json'),
    ('remaining_linux_coverage_after_octal', 'remaining-linux-coverage-tauri-after-octal.json'),
    ('remaining_linux_native_after_octal', 'remaining-linux-native-after-octal.json'),
    ('remaining_linux_workspace_quality_after_octal', 'remaining-linux-workspace-quality-after-octal.json'),
    ('remaining_linux_workspace_coverage_after_octal', 'remaining-linux-workspace-coverage-after-octal.json'),
)
PLATFORM_TEST_RUNS = (
    ('remaining_windows_ci_after_path', 'remaining-windows-ci-after-path.json'),
    ('remaining_linux_tauri_ci_after_path', 'remaining-linux-tauri-ci-after-path.json'),
    ('remaining_linux_coverage_after_path', 'remaining-linux-coverage-tauri-after-path.json'),
    ('remaining_linux_native_after_path', 'remaining-linux-native-after-path.json'),
    ('remaining_linux_workspace_quality_after_path', 'remaining-linux-workspace-quality-after-path.json'),
    ('remaining_linux_workspace_coverage_after_path', 'remaining-linux-workspace-coverage-after-path.json'),
    ('remaining_linux_workspace_audit_after_path', 'remaining-linux-workspace-audit-after-path.json'),
    ('remaining_linux_workspace_msrv_after_path', 'remaining-linux-workspace-msrv-after-path.json'),
)
active_epoch_runs = PLATFORM_TEST_RUNS if (OUT / 'remaining-platform-tests-source-freeze.json').exists() else (PORTABILITY_RUNS if (OUT / 'remaining-portability-source-freeze.json').exists() else (PERMISSIONS_RUNS if (OUT / 'remaining-permissions-source-freeze.json').exists() else ()))
required_remaining_runs = REMAINING_RUNS + active_epoch_runs
remaining_pending = []
for key, filename in (*REMAINING_RUNS, *PERMISSIONS_RUNS, *PORTABILITY_RUNS, *PLATFORM_TEST_RUNS, *OPTIONAL_REMAINING_RUNS):
    path = OUT / filename
    if not path.exists():
        if (key, filename) in required_remaining_runs:
            remaining_pending.append(filename)
        continue
    record = json.loads(path.read_text(encoding='utf-8'))
    if record.get('state') not in (None, 'finished') or not record.get('finished_at_utc') or 'exit_code' not in record:
        remaining_pending.append(filename)
        continue
    attached_groups = {'remaining_csp_focused': ('runtime_style_nonce',), 'remaining_linux_permissions_red': ('permission_base_regression',), 'remaining_linux_store_path_red': ('store_path_platform_contract',)}.get(key, ())
    recorded_run(key, path, record, attached_groups)
    if key in ('remaining_linux_native_build', 'remaining_linux_native_after_csp', 'remaining_linux_native_after_permissions', 'remaining_linux_native_after_octal', 'remaining_linux_native_after_path'):
        RUNS[key]['native_build_artifacts'] = {field: record[field] for field in ('frontend_dist', 'csp_config_sha256', 'binary', 'binary_sha256', 'frontend_dist_after', 'frontend_dist_changed') if field in record}
        RUNS[key]['scope_boundary'] = 'Binary build and embedded asset hashes only; no native interaction or CSP acceptance inferred.'
audit_install_path = OUT / 'remaining-linux-audit-tool-install.json'
audit_install = json.loads(audit_install_path.read_text(encoding='utf-8'))
audit_install_log = artifact(metadata_artifact_path(audit_install['log'], audit_install_path))
assert audit_install['state'] == 'finished' and audit_install['exit_code'] == 0
assert audit_install_log['sha256'] == audit_install['log_sha256']
assert audit_install['command'][audit_install['command'].index('--version') + 1] == '0.22.2'
AUDIT_TOOL_SETUP = {
    'metadata': artifact(audit_install_path), 'log': audit_install_log,
    'command': audit_install['command'], 'exit_code': audit_install['exit_code'],
    'started_at': audit_install['started_at_utc'], 'completed_at': audit_install['finished_at_utc'],
    'version': 'cargo-audit 0.22.2',
    'boundary': 'Task-local cargo-audit installation only. This receipt has no repository source snapshots and does not count as a repository gate or behavior test.',
}
for key, filename in PLATFORM_TEST_RUNS:
    if key not in RUNS:
        continue
    run = RUNS[key]
    record = json.loads((OUT / filename).read_text(encoding='utf-8'))
    if 'workspace_' in key:
        run['scope_boundary'] = 'Separate local WSL Linux workflow lane with its own command, source epoch and result. These lanes are not one complete Linux just ci invocation or a hosted GitHub required status.'
    if key == 'remaining_linux_workspace_audit_after_path':
        assert record['cargo_audit_version'] == AUDIT_TOOL_SETUP['version']
        run['audit_tool'] = {field: record[field] for field in ('cargo_audit_version', 'cargo_audit_path', 'cargo_audit_sha256')}
        run['audit_tool']['installation_metadata'] = AUDIT_TOOL_SETUP['metadata']
        run['audit_tool']['boundary'] = 'Tool identity is recorded by this command receipt. The installation has a separate receipt; no current-path executable hash is substituted for the recorded hash.'
    if key == 'remaining_linux_workspace_msrv_after_path':
        run['scope_boundary'] += ' Actual rustc_version establishes the observed compiler version; a cargo check result does not prove test execution.'
for key, filename, groups in (
    ('remaining_doctor_empty_path_red', 'remaining-doctor-empty-path-red.json', ()),
    ('remaining_doctor_focused', 'remaining-doctor-focused.json', ('doctor_deadline',)),
    ('remaining_doctor_library', 'remaining-doctor-lib.json', ('doctor_deadline',)),
    ('remaining_doctor_clippy', 'remaining-doctor-clippy.json', ()),
):
    path = OUT / filename
    record = json.loads(path.read_text(encoding='utf-8'))
    source = 'crates/ccr-cli/src/commands/codex/fix.rs' if 'source_before_sha256' in record else None
    recorded_run(key, path, record, groups, source)
    if key == 'remaining_doctor_empty_path_red':
        RUNS[key]['declared_fixture_source_sha256'] = record['source_sha256']
        RUNS[key]['observed_test_binary_sha256'] = record['binary_sha256']
        RUNS[key]['cause_boundary'] = 'Empty PATH reproduces two spawn failures in the same observed failed CI binary. The exact concurrent PATH state and lock owner during the original CI failure were not captured; historical exporter AV remains separate and unexplained.'
for key, filename, groups in (
    ('remaining_permissions_config', 'remaining-permissions-linux-config-final.json', ('permission_noop_repository', 'permission_base_regression')),
    ('remaining_permissions_guarded', 'remaining-permissions-linux-core-guarded.json', ('permission_noop_guard',)),
    ('remaining_permissions_atomic', 'remaining-permissions-linux-core-atomic.json', ('permission_atomic_policy',)),
    ('remaining_permissions_clippy', 'remaining-permissions-linux-clippy.json', ()),
    ('remaining_permissions_check', 'remaining-permissions-linux-check.json', ()),
):
    path = OUT / filename
    record = json.loads(path.read_text(encoding='utf-8'))
    recorded_run(key, path, record, groups)
    RUNS[key]['scope_boundary'] = 'Focused Linux permission-owner delta check. Inline source maps precede the later freeze capture and bind to it by exact file-map equality, not by claiming that the freeze existed before this command.'
STORE_PATH_RUNS = (
    ('remaining_store_path_linux_exact', 'remaining-store-path-linux-exact.json'),
    ('remaining_store_path_windows_exact', 'remaining-store-path-windows-exact.json'),
    ('remaining_store_path_linux_lib', 'remaining-store-path-linux-lib.json'),
    ('remaining_store_path_linux_clippy', 'remaining-store-path-linux-clippy.json'),
    ('remaining_store_path_format', 'remaining-store-path-format.json'),
    ('remaining_store_path_diff_check', 'remaining-store-path-diff-check.json'),
)
for key, filename in STORE_PATH_RUNS:
    path = OUT / filename
    record = json.loads(path.read_text(encoding='utf-8'))
    groups = ('store_path_platform_contract',) if key in ('remaining_store_path_linux_exact', 'remaining_store_path_windows_exact', 'remaining_store_path_linux_lib') else ()
    recorded_run(key, path, record, groups)
    RUNS[key]['scope_boundary'] = 'Focused test-only portability verification with four recorded source paths. Source bytes outside the one test are unchanged; this command does not execute a full source-epoch aggregate.'
for key, filename in (
    ('remaining_cli_versions_diagnostic', 'remaining-cli-versions-empty-path-diagnostic.json'),
    ('remaining_cli_versions_focused', 'remaining-cli-versions-focused.json'),
):
    path = OUT / filename
    record = json.loads(path.read_text(encoding='utf-8'))
    source = 'ccr-ui/src-tauri/src/commands/system.rs' if 'source_before_sha256' in record else None
    recorded_run(key, path, record, (), source)
    RUNS[key]['scope_boundary'] = 'CLI-version test fixture isolation only. Empty-PATH diagnostic passed on the original binary; the specific historical host delay remains undetermined and is unrelated to the unexplained exporter AV.'
    if 'source_sha256' in record:
        RUNS[key]['declared_fixture_source_sha256'] = record['source_sha256']
        RUNS[key]['observed_test_binary_sha256'] = record['binary_sha256']
if 'remaining_windows_ci_after_fixture' in RUNS:
    RUNS['remaining_windows_ci_after_fixture']['failure_boundary'] = {
        'stage': 'Tauri behavior tests',
        'selector': 'commands::system::tests::cli_versions_fast_mode_returns_expected_shape',
        'passed': 406, 'failed': 1, 'ignored': 1,
        'specific_host_delay_cause': 'undetermined',
        'historical_exporter_av_resolution_proven': False,
    }
RUNS['remaining_windows_ci_before_fixture']['failure_boundary'] = {
    'stage': 'workspace ccr-cli unit tests', 'passed': 343, 'failed': 2,
    'observed_error': 'Spawn: program not found', 'exporter_stage_reached': False,
    'tauri_stage_reached': False, 'frontend_stage_reached': False,
    'historical_exporter_av_resolution_proven': False,
}
RUNS['remaining_windows_frontend']['observed_test_counts'] = {
    'files': 168, 'tests': 901, 'i18n_checks': 24, 'overlaps_other_runs': True,
}
RUNS['remaining_linux_tauri_ci']['test_partitions'] = {
    'tauri_behavior_passed': 397, 'tauri_behavior_ignored': 1, 'tauri_guard_passed': 2,
    'cli_binding_exports': 24, 'usage_binding_exports': 9, 'tauri_binding_exports': 197,
    'inventory_passed': 1, 'inventory_overlaps_behavior_suite': True,
    'zero_case_targets_not_counted': True,
}
coverage_path = OUT / 'remaining-linux-coverage-report.json'
coverage = json.loads(coverage_path.read_text(encoding='utf-8'))['data'][0]
coverage_meta = json.loads((OUT / 'remaining-linux-coverage-tauri.json').read_text(encoding='utf-8'))
assert artifact(coverage_path)['sha256'] == coverage_meta['coverage_report_sha256']
gateway = [item for item in coverage['files'] if item['filename'].replace(chr(92), '/').endswith('ccr-ui/src-tauri/src/process/gateway.rs')]
assert len(gateway) == 1
RUNS['remaining_linux_coverage']['coverage'] = {
    'report': artifact(coverage_path), 'overall_lines': coverage['totals']['lines'],
    'gateway_lines': gateway[0]['summary']['lines'], 'gateway_path': gateway[0]['filename'],
    'gateway_threshold_percent': 85, 'overall_threshold_percent': None,
    'boundary': 'Actual coverage-tauri recipe enforces gateway >=85%; overall is reported separately without a Tauri overall threshold. Coverage execution overlaps the Linux behavior suite.',
}
RUNS['remaining_linux_coverage']['test_partitions'] = {'tauri_behavior_passed': 397, 'tauri_behavior_ignored': 1, 'tauri_guard_passed': 2, 'overlaps_linux_tauri_ci': True}
if 'remaining_linux_coverage_final' in RUNS:
    final_coverage_path = OUT / 'remaining-linux-coverage-report-final.json'
    final_coverage_meta = json.loads((OUT / 'remaining-linux-coverage-tauri-final.json').read_text(encoding='utf-8'))
    assert artifact(final_coverage_path)['sha256'] == final_coverage_meta['coverage_report_sha256']
    final_coverage = json.loads(final_coverage_path.read_text(encoding='utf-8'))['data'][0]
    final_gateway = [item for item in final_coverage['files'] if item['filename'].replace(chr(92), '/').endswith('ccr-ui/src-tauri/src/process/gateway.rs')]
    assert len(final_gateway) == 1
    RUNS['remaining_linux_coverage_final']['coverage'] = {
        **RUNS['remaining_linux_coverage']['coverage'],
        'report': artifact(final_coverage_path),
        'overall_lines': final_coverage['totals']['lines'],
        'gateway_lines': final_gateway[0]['summary']['lines'],
        'gateway_path': final_gateway[0]['filename'],
    }
if 'remaining_linux_coverage_after_path' in RUNS:
    tauri_coverage_run = RUNS['remaining_linux_coverage_after_path']
    tauri_coverage_meta = json.loads((OUT / 'remaining-linux-coverage-tauri-after-path.json').read_text(encoding='utf-8'))
    tauri_copy_path = OUT / 'remaining-linux-coverage-tauri-after-path-report-copy.json'
    tauri_copy = json.loads(tauri_copy_path.read_text(encoding='utf-8'))
    tauri_report_path = OUT / tauri_copy['copy']
    tauri_report_artifact = artifact(tauri_report_path)
    assert tauri_copy['run'] == 'remaining-linux-coverage-tauri-after-path.json'
    assert tauri_report_artifact['sha256'] == tauri_copy['sha256'] == tauri_coverage_meta['coverage_report_sha256']
    tauri_report = json.loads(tauri_report_path.read_text(encoding='utf-8'))['data']
    assert len(tauri_report) == 1
    tauri_report = tauri_report[0]
    tauri_gateway = [item for item in tauri_report['files'] if item['filename'].replace(chr(92), '/').endswith('ccr-ui/src-tauri/src/process/gateway.rs')]
    assert len(tauri_gateway) == 1
    assert tauri_copy['overall_lines'] == tauri_report['totals']['lines']
    assert tauri_copy['gateway_lines'] == tauri_gateway[0]['summary']['lines']
    assert tauri_coverage_run['execution']['exit_code'] == 0
    tauri_coverage_run['coverage'] = {
        'status': 'measured_gateway_threshold_passed', 'report': tauri_report_artifact,
        'copy_receipt': artifact(tauri_copy_path), 'reported_sha256': tauri_coverage_meta['coverage_report_sha256'],
        'overall_lines': tauri_report['totals']['lines'],
        'gateway_lines': tauri_gateway[0]['summary']['lines'], 'gateway_path': tauri_gateway[0]['filename'],
        'gateway_threshold_percent': 85, 'overall_threshold_percent': None,
        'gateway_threshold_evaluated': True, 'overall_threshold_evaluated': False,
        'boundary': 'Actual a31c coverage-tauri recipe with a separately captured immutable llvm-cov report. Only the Tauri gateway 85% threshold is enforced; the overall percentage is reported without an overall threshold. Workspace and Tauri coverage scopes remain separate.',
    }
if 'remaining_linux_workspace_coverage_after_path' in RUNS:
    workspace_coverage_run = RUNS['remaining_linux_workspace_coverage_after_path']
    workspace_coverage_meta = json.loads((OUT / 'remaining-linux-workspace-coverage-after-path.json').read_text(encoding='utf-8'))
    workspace_report_path = OUT / workspace_coverage_meta['coverage_report_copy']
    workspace_report_artifact = artifact(workspace_report_path)
    assert workspace_report_artifact['sha256'] == workspace_coverage_meta['coverage_report_sha256']
    workspace_report = json.loads(workspace_report_path.read_text(encoding='utf-8'))['data']
    assert len(workspace_report) == 1
    workspace_report = workspace_report[0]
    workspace_gateway = [item for item in workspace_report['files'] if item['filename'].replace(chr(92), '/').endswith('crates/ccr-core/src/core/process_gateway.rs')]
    assert len(workspace_gateway) == 1
    workspace_coverage_text = (ROOT / workspace_coverage_run['path']).read_text(encoding='utf-8')
    overall = workspace_report['totals']['lines']
    gateway = workspace_gateway[0]['summary']['lines']
    assert workspace_coverage_run['execution']['exit_code'] == 0
    assert f"Overall line coverage: {overall['percent']:.2f}%" in workspace_coverage_text
    assert f"Gateway line coverage: {gateway['percent']:.2f}% {workspace_gateway[0]['filename']}" in workspace_coverage_text
    workspace_coverage_run['coverage'] = {
        'status': 'measured_thresholds_passed', 'report': workspace_report_artifact,
        'reported_sha256': workspace_coverage_meta['coverage_report_sha256'],
        'overall_lines': overall, 'gateway_lines': gateway,
        'gateway_path': workspace_gateway[0]['filename'],
        'overall_threshold_percent': 70, 'gateway_threshold_percent': 85,
        'thresholds_evaluated': True,
        'boundary': 'Actual a31c workspace coverage-rust recipe and immutable llvm-cov report. The overall 70% and core gateway 85% thresholds both executed. Coverage tests overlap other workspace runs; this result does not replace the separate Tauri coverage scope.',
    }
if 'remaining_linux_workspace_coverage_final' in RUNS:
    workspace_coverage_run = RUNS['remaining_linux_workspace_coverage_final']
    workspace_coverage_meta = json.loads((OUT / 'remaining-linux-workspace-coverage-final.json').read_text(encoding='utf-8'))
    assert workspace_coverage_run['execution']['exit_code'] == 101
    assert workspace_coverage_meta['coverage_report_sha256'] is None
    workspace_coverage_text = (ROOT / workspace_coverage_run['path']).read_text(encoding='utf-8')
    assert 'platforms::base::tests::test_profile_structured_writes_keep_owner_only_permissions ... FAILED' in workspace_coverage_text
    assert 'left: 420' in workspace_coverage_text and 'right: 384' in workspace_coverage_text
    workspace_coverage_run['coverage'] = {
        'status': 'not_measured_behavior_test_failed',
        'report': None, 'reported_sha256': None,
        'overall_threshold_percent': 70, 'gateway_threshold_percent': 85,
        'thresholds_evaluated': False,
        'boundary': 'The workspace coverage command stopped in a behavior test before a coverage report or threshold result. No overall or gateway percentage is inferred from Tauri coverage.',
    }
    workspace_coverage_run['failure_boundary'] = {
        'stage': 'workspace ccr-config behavior tests under coverage instrumentation',
        'selector': 'platforms::base::tests::test_profile_structured_writes_keep_owner_only_permissions',
        'passed': 97, 'failed': 1, 'ignored': 1,
        'observed_mode_decimal': 420, 'expected_mode_decimal': 384,
        'observed_mode_octal': '0644', 'expected_mode_octal': '0600',
        'coverage_report_produced': False,
        'source_capture_boundary': 'The post-command map contains generated-file changes. Both maps and every difference remain recorded; this run does not establish generated parity.',
    }
parity_path = OUT / 'remaining-generated-after-concurrent-capture.json'
parity = json.loads(parity_path.read_text(encoding='utf-8'))
parity_windows = RUNS['remaining_windows_ci_final']['execution']
parity_failed = RUNS['remaining_linux_workspace_coverage_final']['execution']
assert parity['windows_run_sha256'] == parity_windows['metadata']['sha256']
assert parity['windows_exit_code'] == parity_windows['exit_code'] == 0
parity_before = json.loads((ROOT / parity_windows['generated_comparison']['before_snapshot']['path']).read_text(encoding='utf-8'))
assert parity['current_hashes'] == {name: entry['sha256'] for name, entry in parity_before.items()}
assert parity['generated_count'] == len(parity['current_hashes']) == 230
assert parity_windows['generated_comparison']['changes'] == []
assert parity['failed_linux_generated_after_count'] == parity_failed['generated_map_comparison']['after_count'] == 33
assert parity['failed_linux_source_changes_preserved'] == [entry['path'] for entry in parity_failed['source_attribution']['post_command_source_changes']]
GENERATED_PARITY = {
    'report': artifact(parity_path), 'captured_at': parity['captured_at_utc'],
    'status': parity['status'], 'generated_count': parity['generated_count'],
    'windows_run': parity_windows['metadata'], 'failed_linux_run': parity_failed['metadata'],
    'scope': 'Separate post-command generated-directory observation. It matches the successful Windows guard baseline at capture time and does not revise the failed Linux end snapshot or prove coverage success.',
}
permission_fix_path = OUT / 'remaining-permissions-fix.json'
permission_fix = json.loads(permission_fix_path.read_text(encoding='utf-8'))
permission_review_path = OUT / 'remaining-permissions-independent-review.json'
permission_review = json.loads(permission_review_path.read_text(encoding='utf-8'))
permission_freeze_path = OUT / 'remaining-permissions-source-freeze.json'
permission_freeze = json.loads(permission_freeze_path.read_text(encoding='utf-8'))
permission_before = artifact(OUT / permission_fix['before_snapshot'])
permission_delta = artifact(OUT / permission_fix['delta'])
assert permission_before['sha256'] == permission_fix['before_snapshot_sha256']
assert permission_delta['sha256'] == permission_fix['delta_sha256']
before_files = json.loads((ROOT / permission_before['path']).read_text(encoding='utf-8'))['files']
assert len(before_files) == len(permission_fix['source_paths']) == 6
before_text_checks = []
for name, entry in before_files.items():
    text_sha = hashlib.sha256(entry['content'].encode('utf-8')).hexdigest()
    before_text_checks.append({'path': name, 'recorded_original_byte_sha256': entry['sha256'], 'stored_text_utf8_sha256': text_sha, 'stored_text_preserves_original_bytes': text_sha == entry['sha256']})
    change = next(item for item in permission_freeze['changes_from_parent'] if item['path'] == name)
    assert change['before'] == entry['sha256'] and change['after'] == permission_fix['current_source_sha256'][name]
assert [item['path'] for item in before_text_checks if not item['stored_text_preserves_original_bytes']] == ['.trellis/spec/ccr-core/backend/atomic-writer.md']
assert permission_review['source_freeze_sha256'] == artifact(permission_freeze_path)['sha256']
assert sorted(permission_review['scope']) == sorted(permission_fix['source_paths'])
permission_base_path = 'crates/ccr-config/src/platforms/base.rs'
permission_red = RUNS['remaining_linux_permissions_red']['execution']
assert permission_red['source_attribution']['source_files_sha256'][permission_base_path] == permission_freeze['files'][permission_base_path]
assert permission_fix['original_base_test_sha256_unchanged']
PERMISSION_REMEDIATION = {
    'finding': 'DC-02', 'status': 'focused_behavior_verified_formal_epoch_results_separate',
    'fix_commit': None, 'requirement_refs': ['T01.AC2', 'T01.AC3', 'T05.AC2'],
    'test_groups': ['permission_noop_repository', 'permission_noop_guard', 'permission_base_regression', 'permission_atomic_policy'],
    'red_run_ref': 'remaining_linux_permissions_red',
    'focused_run_refs': ['remaining_permissions_config', 'remaining_permissions_guarded', 'remaining_permissions_atomic', 'remaining_permissions_clippy', 'remaining_permissions_check'],
    'fix_report': artifact(permission_fix_path), 'before_snapshot': permission_before,
    'before_snapshot_text_checks': before_text_checks,
    'before_snapshot_text_boundary': 'Five stored text values preserve the recorded original bytes. The atomic-writer spec text does not; its original byte identity remains bound to the prior freeze SHA. The reason is not established by this snapshot, and no raw-byte restoration claim is made for that text value.',
    'delta': permission_delta, 'independent_review': artifact(permission_review_path),
    'source_freeze': artifact(permission_freeze_path),
    'author': permission_fix['author'], 'reviewer': permission_review['reviewer'],
    'review_status': permission_review['status'],
    'original_regression_source': {'path': permission_base_path, 'sha256': permission_freeze['files'][permission_base_path], 'unchanged_from_red_run': True},
    'journal_scope': permission_fix['journal_scope'],
    'contract_boundary': 'Supplemental repository and shared secret-permission policy checks. No new OAuth pending lifecycle claim, no metadata rollback guarantee for journals with prior content entries, and no actual OS permission-denial or macOS execution claim.',
    'formal_epoch_run_refs': [key for key, _ in active_epoch_runs],
}
if 'remaining_linux_workspace_quality_after_permissions' in RUNS:
    missing_python = RUNS['remaining_linux_workspace_quality_after_permissions']
    text = (ROOT / missing_python['path']).read_text(encoding='utf-8')
    assert missing_python['execution']['exit_code'] == 127 and 'python: command not found' in text
    missing_python['failure_boundary'] = {'stage': 'lint-strict secret-write check launcher', 'observed_error': 'python: command not found', 'exit_code': 127, 'rust_clippy_reached': False}
if 'remaining_linux_workspace_quality_after_python' in RUNS:
    python_retry = RUNS['remaining_linux_workspace_quality_after_python']
    launcher_path = OUT / 'remaining-python-launcher.json'
    launcher = json.loads(launcher_path.read_text(encoding='utf-8'))
    assert launcher['exit_code'] == 0
    assert python_retry['execution']['command'] == RUNS['remaining_linux_workspace_quality_after_permissions']['execution']['command']
    python_retry['environment_preparation'] = artifact(launcher_path)
    text = (ROOT / python_retry['path']).read_text(encoding='utf-8')
    assert python_retry['execution']['exit_code'] == 101 and 'non_octal_unix_permissions' in text
    python_retry['failure_boundary'] = {
        'stage': 'workspace strict Clippy', 'diagnostic': 'clippy::non_octal_unix_permissions',
        'source': 'crates/ccr/tests/diagnostics_contract.rs', 'recorded_line': 357,
        'expression': 'fs::Permissions::from_mode(0)', 'suggested_octal_literal': '0o0',
        'scope': 'The environment launcher now resolves; the unchanged recipe reaches a distinct source lint failure. Later source repairs or retries keep separate identities.',
    }
if 'remaining_linux_workspace_quality_after_octal' in RUNS:
    octal_retry = RUNS['remaining_linux_workspace_quality_after_octal']
    text = (ROOT / octal_retry['path']).read_text(encoding='utf-8')
    selector = 'sessions::providers::tests::restores_codex_live_jsonl_under_tempfile_with_windows_path_shapes'
    assert octal_retry['execution']['exit_code'] == 101 and selector in text
    assert '80 passed; 1 failed; 2 ignored;' in text
    source_path = 'crates/ccr-store/src/sessions/providers.rs'
    octal_retry['failure_boundary'] = {
        'stage': 'workspace ccr-store behavior tests', 'selector': selector,
        'source': source_path, 'recorded_line': 2647, 'passed': 80, 'failed': 1, 'ignored': 2,
        'source_sha256_in_command_map': octal_retry['execution']['source_attribution']['source_files_sha256'].get(source_path),
        'observed_assertion': 'restore_source(Codex, codex-live, backslash, File, None).is_ok()',
        'cause': 'undetermined_by_this_run',
    }
octal_path = OUT / 'remaining-octal-test-fix.json'
octal = json.loads(octal_path.read_text(encoding='utf-8'))
octal_before = base64.b64decode(octal['before_bytes_base64'], validate=True)
assert hashlib.sha256(octal_before).hexdigest() == octal['before_sha256']
octal_old = octal['replacement']['before'].encode('utf-8')
octal_new = octal['replacement']['after'].encode('utf-8')
assert octal_before.count(octal_old) == 1 and octal_old == b'Permissions::from_mode(0)' and octal_new == b'Permissions::from_mode(0o0)'
assert hashlib.sha256(octal_before.replace(octal_old, octal_new)).hexdigest() == octal['after_sha256']
assert octal['semantic_change'] is False
PORTABILITY_REPAIR = {
    'report': artifact(octal_path), 'path': octal['path'],
    'before_sha256': octal['before_sha256'], 'after_sha256': octal['after_sha256'],
    'before_bytes_verified': True, 'single_literal_replacement_verified': True,
    'failure_run_ref': 'remaining_linux_workspace_quality_after_python',
    'replacement': octal['replacement'], 'semantic_change': False,
    'scope': 'One equivalent Unix permission literal in an existing test. No production behavior or test assertion change.',
}
store_before_path = OUT / 'remaining-linux-store-path-before.json'
store_before = json.loads(store_before_path.read_text(encoding='utf-8'))
store_original = base64.b64decode(store_before['source_bytes_base64'], validate=True)
assert len(store_original) == store_before['source_size']
assert hashlib.sha256(store_original).hexdigest() == store_before['source_sha256'] == store_before['head_sha256']
assert store_before['equals_head_bytes'] is True
store_head = subprocess.run(['git', 'show', store_before['head'] + ':' + store_before['source_path']], cwd=ROOT, capture_output=True, check=True).stdout
assert store_head == store_original
store_source = ROOT / store_before['source_path']
store_current = store_source.read_bytes()
store_start = b'    #[test]\n    fn restores_codex_live_jsonl_under_tempfile_with_windows_path_shapes()'
store_end = b'    #[cfg(windows)]\n    #[test]\n    fn windows_verbatim_prefix_does_not_break_root_containment()'
assert store_original.count(store_start) == store_current.count(store_start) == 1
assert store_original.count(store_end) == store_current.count(store_end) == 1
old_start, old_end = store_original.index(store_start), store_original.index(store_end)
new_start, new_end = store_current.index(store_start), store_current.index(store_end)
assert store_original[:old_start] == store_current[:new_start]
assert store_original[old_end:] == store_current[new_end:]
old_test, new_test = store_original[old_start:old_end], store_current[new_start:new_end]
assert old_test.partition(b'        let backslash = ')[0] == new_test.partition(b'        let backslash = ')[0]
assert b'#[ignore]' not in new_test
assert b'#[cfg(unix)]\n        assert!(!backslash.exists());' in new_test
assert b'#[cfg(windows)]\n        assert!(restored.is_ok());' in new_test
assert b'#[cfg(unix)]\n        assert!(restored.is_err());' in new_test
store_red = RUNS['remaining_linux_store_path_red']['execution']
assert store_red['exit_code'] == 101 and store_red['source_attribution']['source_stable_between_snapshots']
assert store_red['source_attribution']['source_files_sha256'][store_before['source_path']] == store_before['source_sha256']
STORE_PATH_PORTABILITY = {
    'before_capture': artifact(store_before_path), 'current_source_index': artifact(store_source),
    'requirement_refs': ['T10.AC1'], 'test_group': 'store_path_platform_contract',
    'original_bytes_equal_recorded_head': True, 'single_test_body_changed': True,
    'original_and_canonical_path_assertions_unchanged': True,
    'ignored_or_skipped_test_added': False,
    'red_run_ref': 'remaining_linux_store_path_red',
    'earlier_unbound_run_ref': 'remaining_linux_workspace_quality_after_octal',
    'historical_identity_boundary': store_before['historical_identity_boundary'],
    'execution_boundary': 'The exact red run binds the original provider source. Separate Linux and Windows green exact receipts bind the corrected test through their four-path maps; they do not establish full-epoch gate acceptance or macOS execution.',
    'focused_run_refs': [key for key, _ in STORE_PATH_RUNS],
    'implementation_report': artifact(OUT / 'remaining-linux-store-path-implementation.json'),
    'independent_review': artifact(OUT / 'remaining-store-path-independent-review.json'),
    'diff': artifact(OUT / 'remaining-linux-store-path-change.diff'),
}
for key, run in RUNS.items():
    if not key.startswith('remaining_'):
        continue
    text = (ROOT / run['path']).read_text(encoding='utf-8', errors='replace')
    run['observed_rust_result_partitions'] = [
        {'status': match[1], 'passed': int(match[2]), 'failed': int(match[3]), 'ignored': int(match[4]), 'log_line': number, 'raw': line.strip()}
        for number, line in enumerate(text.splitlines(), 1)
        if (match := re.search(r'test result: (ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored;', line))
    ]
    run['partition_boundary'] = 'Observed result lines only; partitions and repeated runs are not summed into a unique test count.'
baseline_path = OUT / 'remaining-mapping-baseline.json'
remaining_baseline = json.loads(baseline_path.read_text(encoding='utf-8'))
for key, digest in remaining_baseline['prior_run_fingerprints'].items():
    observed = hashlib.sha256(json.dumps(RUNS[key], ensure_ascii=False, sort_keys=True, separators=(',', ':')).encode('utf-8')).hexdigest()
    assert observed == digest, ('historical run object changed', key)
native_attempts = []
for path in sorted(OUT.glob('remaining-native-webview-*.json')):
    record = json.loads(path.read_text(encoding='utf-8'))
    if record.get('state') == 'running' or not record.get('finished_at_utc'):
        remaining_pending.append(path.name)
        continue
    build_path = metadata_artifact_path(record['build_record'], path)
    build = json.loads(build_path.read_text(encoding='utf-8'))
    assert build['exit_code'] == 0 and build['state'] == 'finished', path
    assert record['binary_sha256'] == build['binary_sha256'], path
    if record.get('build_record_sha256'):
        assert artifact(build_path)['sha256'] == record['build_record_sha256'], path
    attempt = {
        'report': artifact(path), 'state': record['state'], 'scope': record['scope'],
        'build_metadata': artifact(build_path), 'binary_sha256': record['binary_sha256'],
        'checks': record['checks'], 'error': record.get('error'),
        'started_at': record['started_at_utc'], 'completed_at': record['finished_at_utc'],
        'live_process_group_members_after_cleanup': record.get('live_process_group_members_after_cleanup'),
        'build_record_sha256': record.get('build_record_sha256'),
        'editor_runtime_style': record.get('editor_runtime_style'),
        'post_save_style_csp_violations': record.get('styles_after_save'),
        'post_save_style_capture_field': 'styles_after_save',
        'post_save_style_unmeasured': ['style DOM removal', 'body scroll-lock restoration'],
        'source_binding': 'Receipt binary SHA equals the successful build metadata; source and asset identity are supplied by that build record.',
    }
    for epoch_name in REMAINING_SOURCE_EPOCHS:
        epoch_path = OUT / epoch_name
        if epoch_path.exists() and json.loads(epoch_path.read_text(encoding='utf-8'))['files'] == build['sources_before']:
            attempt['source_epoch'] = {'manifest': artifact(epoch_path), 'build_start_matches_epoch': True, 'build_finish_matches_epoch': build['sources_after'] == build['sources_before']}
            break
    log_path = path.with_suffix('.log')
    if log_path.exists():
        attempt['log'] = artifact(log_path)
    native_attempts.append(attempt)
NATIVE = {
    'evidence_type': 'native_linux_webkitgtk_custom_protocol_debug_binary',
    'attempts': native_attempts,
    'passed_attempts': [item['report']['path'] for item in native_attempts if item['state'] == 'passed'],
    'acceptance_scope': 'Linux WebKitGTK under WSLg; synthetic Claude raw editor, confirmation, CSP and local file roundtrip only when a completed receipt passes every check.',
    'not_covered': ['Windows WebView2', 'macOS WKWebView', 'release installer/package', 'physical input or display hardware', 'native Codex model-only save', 'native Grok settings', 'real provider credentials or OAuth'],
}
native_reports = []
for filename, expected_status in (
    ('remaining-native-acceptance.json', 'passed_scoped_native_linux'),
    ('remaining-native-acceptance-final.json', 'passed_scoped_native_linux_final_source'),
):
    native_acceptance_path = OUT / filename
    native_acceptance = json.loads(native_acceptance_path.read_text(encoding='utf-8'))
    assert native_acceptance['status'] == expected_status
    native_artifacts = []
    for name, digest in native_acceptance['artifacts'].items():
        entry = artifact(OUT / name)
        assert entry['sha256'] == digest, name
        native_artifacts.append(entry)
    native_reports.append({'report': artifact(native_acceptance_path), 'status': native_acceptance['status'], 'verified_artifacts': native_artifacts})
NATIVE['acceptance_report'] = native_reports[-1]['report']
NATIVE['verified_artifacts'] = native_reports[-1]['verified_artifacts']
NATIVE['prior_acceptance_reports'] = native_reports[:-1]
NATIVE['status'] = native_reports[-1]['status']
NATIVE['final_acceptance_observation'] = {field: native_acceptance[field] for field in ('source_paths', 'frontend_input_paths', 'frontend_assets', 'generated_types', 'roundtrip_semantic_match', 'live_process_group_members_after_cleanup', 'post_run_binary_and_fixture_observation', 'post_save_style_csp_violations_empty', 'visual_review', 'limits')}
style_correction_path = OUT / 'remaining-native-style-evidence-correction.json'
style_correction = json.loads(style_correction_path.read_text(encoding='utf-8'))
assert style_correction['historical_derived_report_sha256'] == native_reports[0]['report']['sha256']
assert style_correction['current_report_sha256'] == native_reports[-1]['report']['sha256']
NATIVE['style_evidence_correction'] = artifact(style_correction_path)
NATIVE['style_measurement_boundary'] = 'Raw styles_after_save captures window.__ccrNativeStyleCspEvents only. An empty list proves no captured post-save style CSP violations. Style DOM removal and body scroll-lock restoration were not measured. Native process-group cleanup is a separate measurement.'
NATIVE['style_unmeasured'] = style_correction['unmeasured']
original_harness_path = OUT / 'remaining-native-webview.py'
extended_harness_path = OUT / 'remaining-native-webview-after-path.py'
original_harness = original_harness_path.read_bytes()
extended_harness = extended_harness_path.read_bytes()
original_allowlist = b"('remaining-linux-native-build.json', 'remaining-linux-native-after-csp.json')"
extended_allowlist = b"('remaining-linux-native-build.json', 'remaining-linux-native-after-csp.json', 'remaining-linux-native-after-path.json')"
assert original_harness.count(original_allowlist) == 1
assert original_harness.count(b'\r\n') == 0
assert original_harness.replace(original_allowlist, extended_allowlist).replace(b'\n', b'\r\n') == extended_harness
NATIVE['harness_extension'] = {
    'original': artifact(original_harness_path), 'extended': artifact(extended_harness_path),
    'original_lf_count': original_harness.count(b'\n'), 'extended_crlf_count': extended_harness.count(b'\r\n'),
    'comparison': 'Exact bytes equal after one build-receipt allowlist expansion and LF-to-CRLF conversion. All other logical text, including behavioral assertions, is unchanged.',
    'byte_identical_outside_allowlist': False,
}
NATIVE['execution_binary_boundary'] = 'Use the receipt-time binary SHA bound to build metadata. Later Cargo commands can replace the same executable path; no current-path binary equality is claimed.'
review_path = OUT / 'remaining-dedicated-check.json'
review = json.loads(review_path.read_text(encoding='utf-8'))
assert review['role'] == 'trellis-check'
DEDICATED_REVIEW = {
    'reviewer': review['reviewer'], 'role': review['role'], 'dispatch_restored': True,
    'status_at_capture': review['status'], 'independence': review['independence'],
    'report_paths': [relative(review_path), relative(review_path.with_suffix('.md'))],
    'reference_rule': 'Reviewer-owned reports may incorporate this generated matrix. Keep path references without a reverse report hash to avoid a circular evidence dependency. Current result is read from the reviewer report.',
    'historical_dispatch_failures_preserved': True,
}
schema_restorations = []
for filename in ('remaining-schema-restoration.json', 'remaining-schema-restoration-final.json'):
    schema_path = OUT / filename
    schema = json.loads(schema_path.read_text(encoding='utf-8'))
    schema_copy = artifact(OUT / schema['preserved_copy'])
    assert schema_copy['sha256'] == schema['preserved_copy_sha256'] == schema['removed_generated_sha256']
    schema_baseline = artifact(OUT / schema['baseline'])
    assert schema_baseline['sha256'] == schema['baseline_sha256']
    schema_restorations.append({'report': artifact(schema_path), 'baseline': schema_baseline, 'preserved_copy': schema_copy, 'status': schema['status'], 'preexisting_files': len(schema['preexisting_schema_hashes'])})
SCHEMA_RESTORATION = {
    **schema_restorations[-1], 'historical_restorations': schema_restorations[:-1],
    'current_state_check_basis': 'Only the final restoration receipt is checked against the current schema files. The earlier restoration remains a historical observation; later native builds regenerated its removed Linux schema.',
}
context_path = PARENT / 'research/remaining-context-preservation.json'
context_preservation = json.loads(context_path.read_text(encoding='utf-8'))
context_warnings = [
    {'task': item['task'], 'warning': re.sub(r'\x1b\[[0-9;]*m', '', line).strip()}
    for item in context_preservation['validations'] for line in item['output'].splitlines()
    if 'Warning:' in line
]
CONTEXT_PRESERVATION = {
    'report': artifact(context_path), 'passed': context_preservation['passed'],
    'all_contexts_unchanged': context_preservation['all_contexts_unchanged'],
    'existing_tasks': context_preservation['existing_tasks'], 'warnings': context_warnings,
    'boundary': 'Task context validators exited zero; their warnings are preserved and are not converted to warning-free validation.',
}
final_context_path = PARENT / 'research/remaining-context-preservation-final.json'
final_context = json.loads(final_context_path.read_text(encoding='utf-8'))
assert final_context['prior_receipt'] == context_path.name
assert final_context['prior_receipt_sha256'] == CONTEXT_PRESERVATION['report']['sha256']
assert len(final_context['validations']) == 12
assert all(item['exit_code'] == 0 and item['context_unchanged'] for item in final_context['validations'])
CONTEXT_PRESERVATION['final_validation'] = {
    'report': artifact(final_context_path), 'prior_report': CONTEXT_PRESERVATION['report'],
    'passed': len(final_context['validations']),
    'context_files': sum(len(item['context_hashes']) for item in final_context['validations']),
    'all_contexts_unchanged': all(item['context_unchanged'] for item in final_context['validations']),
    'existing_tasks': final_context['insights'], 'warning': final_context['warning'],
    'warnings': [
        {'task': item['task'], 'warning': re.sub(r'\x1b\[[0-9;]*m', '', line).strip()}
        for item in final_context['validations'] for line in item['output'].splitlines() if 'Warning:' in line
    ],
}

ROWS = {
'T01': [
('repository config_handlers', '资源身份统一；锁内read/validate/mutate/leaf CAS；并发不同字段均保留', 'config mutation;Claude/Codex/Grok;service/platform/Tauri'),
('repository diagnostics', '显式平台和同源current投影；读取不bootstrap/autofix', 'list/current/validate;three platforms;CLI/shared repository'),
('repository config_handlers', '严格patch与version conflict拒绝；unknown TOML/secret/marker保留', 'patch/rename;three platforms;repository/Tauri')],
'T02': [
('profile_cli profile_tui profile_desktop journal', '共享application预检与已发布写journal；逐写故障逆序补偿，外部新版保留', 'apply;Claude/Codex/Grok;CLI/TUI/Tauri'),
('profile_cli profile_tui profile_desktop', '同一contract harness比较持久状态；operation ID重放不重复计数', 'apply/replay;Claude/Codex/Grok;three adapters'),
('profile_warning_binary profile_tui config_page', 'committed outcome与ancillary warning分离；页面不自动再激活', 'history failure;Claude/Codex/Grok;binary/presenter/React'),
('profile_cli profile_tui profile_desktop', '三个adapter委托同一同步application owner；边界guard检查无终端IO/exit', 'application boundary;three platforms;three adapters'),
('profile_cli config_handlers', 'rename纳入同一journal，current/default与unknown值在逐写故障下保持可解释', 'rename;Claude/Codex;CLI/Tauri')],
'T03': [
('config_handlers config_page', 'explicit Claude配置页switch/enable进入共享application，enabled/runtime/current一致', 'switch/enable;Claude generic page;React/generated/Tauri/application'),
('config_handlers repository', '对象形状/absent/null验证；active保护在operation lock内重读；多进程合并', 'CRUD;Claude generic page;Tauri/repository'),
('config_handlers registry', '缺失平台明确失败无写；typed client/registry同步；完整bindings另列', 'wire compatibility;Claude generic page;registry/generated/Tauri')],
'T04': [
('diagnostics', 'binary统一返回诊断ExitCode；invalid/corrupt/unreadable与warning分类', 'validate;Claude/Codex/Grok;actual binary'),
('diagnostics profile_cli', 'validate/apply复用领域auth validator；原生denied read不降为missing', 'validate/apply;API-key/subscription;binary/application'),
('diagnostics repository', '只读前后路径/字节/mtime相同；Grok支持、Gemini/Droid保留adapter状态明确', 'doctor/current;five platform capability labels;binary/shared')],
'T05': [
('backups', '固定时钟下UUID no-clobber备份，每次前镜像独立，保留keep-10', 'write/backup/rotate;Windows/Linux;guarded writer'),
('backups', '旧新命名稳定reader/rotation；DACL和mode在payload前建立，readonly发布后恢复', 'restore/metadata;Windows/Linux;writer/consumer'),
('pending oauth', 'pending secret:true/BackupPolicy::None；disk-first发布、失败保留旧目标', 'create/replace/cancel/expire;Windows/Linux scoped;store/Tauri'),
('pending oauth', 'Debug/Serialize/errors/events不含合成verifier/token sentinel', 'failure/cancel/expiry;Windows/Linux scoped;store/controller')],
'T06': [
('usage_registry control_policy', '一次锁内发布snapshot/token；cancel_requested保留active直到runner cleanup完成', 'start/cancel/admit;all usage providers;Tauri owner'),
('usage_registry usage_ui', '五种terminal不可回退；取消请求与清理完成分开；诊断保留', 'cancel/progress/terminal;usage job;backend/frontend'),
('usage_executor process_tree', 'execution deadline覆盖stdout/callback/wait/stderr；cleanup共享预算并回报失败', 'stream/cancel/cleanup;Windows/Linux;adapter/core'),
('usage_executor usage_registry', 'stdout 1MiB+1、stderr 64×64KiB有界；no-crate guards验证SQL owner', 'stream bounds/DTO;usage adapter;backend')],
'T07': [
('command_page command_owner', 'shell/store持有job；路由重挂载对账snapshot并恢复取消', 'route leave/return;command jobs;React and backend owner'),
('command_page command_owner', '同步提交锁、job-ID隔离、terminal优先；history pending/saved/failed且单次写尝试', 'start/events/history;command jobs;React/store/backend'),
('command_listener command_page', 'disposed协议释放迟到listen；shell暂停隔离迟到start/cancel响应', 'mount/unmount/listen;command events;React shell')],
'T08': [
('settings_mapping settings_page codex_settings_backend codex_settings_persistence', 'snapshot与dirty leaves无损映射；只发送编辑字段，保留union/unknown/default语义；Codex helper真实磁盘往返', 'typed save;Codex/OpenCode;React/mapper/domain/Rust persistence helper'),
('settings_page grok_settings settings_visible_i18n', 'managed metadata进入可执行capability；disabled+原因+未知枚举当前值', 'managed settings;Grok/Codex;React/backend'),
('settings_page grok_settings settings_raw_session runtime_style_nonce', '真实raw editor入口、确认/CAS/invalid草稿；Local能力约束与Grok无备份；原生Linux公共编辑器CSP单列', 'raw edit;Claude/Codex/Grok;React/CodeMirror/Tauri')],
'T09': [
('auth_query auth_cache', 'probe/load状态明确；stale成功数据仍可见，off pending阻止重复并保留失败前状态', 'auth probe/load/off;platform adapters;React/Query'),
('settings_session settings_raw_session settings_environment', 'snapshot/baseline/draft分离；环境身份/世代冻结旧会话，后端捕获同一目标', 'refetch/save/environment transition;typed/raw settings;React/Query/Tauri'),
('config_page', '实际翻译订阅参与memo依赖，同一Query引用和挂载卡片语言切换', 'locale switch;Configs;React')],
'T10': [
('aggregate bindings calendar lint_main_tests lint_state_tests lint_claude_test lint_codex_profile_test doctor_deadline tauri_process_gateway oauth_windows_pending_failure', '跨workspace aggregate复用tauri-ci；已运行故障fixture非零；root正式完整gate单独验收', 'aggregate;Windows/Linux/macOS declared;local/hosted'),
('', '39AC及9P1逐项映射，旧源码/实际红例/修后行为分开；缺失红例不提升验收', 'evidence;all tasks;review'),
('', 'registry/manifest权威计数；现行路径与usage owner导航；生命周期契约无损抽取', 'spec navigation;all packages;documentation'),
('', '正式失败原样保留；保护脚本哈希和Insights状态单独核对；不创建commit', 'baseline/final gates;original workspace;root integration')],
'T11': [
('control_policy command_owner usage_registry oauth', '控制投递与执行admission分离；真实owner持permit到cleanup/join结束', 'C01-C06;23 command/install/usage/OAuth IDs;registry/runtime/owner'),
('oauth pending', 'bind和disk先成功后发布URL；单controller负责唯一terminal及active reuse', 'start/complete;Codex OAuth;loopback/controller/store'),
('oauth', 'listener/socket/body/exchange有界取消；commit_in_progress拒绝伪取消', 'cancel/deadline;Codex OAuth;loopback/HTTP/controller'),
('registry oauth pending bindings', 'ACL/confirmation元数据保持；sentinel不外泄；完整binding guard保留调用前230文件', 'wire/security;Tauri;registry/generated/controller')],
}

rows = []
for task in LEDGER['tasks']:
    entries = ROWS[task['key']]
    assert len(entries) == len(task['acceptance'])
    for ac, (groups, mechanism, coverage) in zip(task['acceptance'], entries, strict=True):
        rows.append({'id': task['key'] + '.' + ac['id'], 'requirement': ac['requirement'], 'mechanism': mechanism, 'operation_platform_entry': coverage, 'test_groups': groups.split(), 'ledger_status_at_capture': ac['status'], 'acceptance_note': 'Scoped evidence only; no task lifecycle or full-gate acceptance inferred.', 'report': relative(TASK[task['key']] / 'check-report.md') if (TASK[task['key']] / 'check-report.md').exists() else None})
assert len(rows) == 39

# These acceptance clauses concern artifacts and preservation, not executable product cases.
artifact_requirements = {
    'T08.AC1': [
        (OUT / 'continuation-web-review.json', 'Real web interaction with synthetic IPC: one model-only patch; native and actual config writes are not covered.'),
        (OUT / 'continuation-rust-check.json', 'Separate actual private-helper disk roundtrip; no invoke or State/cache execution.'),
    ],
    'T10.AC1': [
        (OUT / 'continuation-tauri-lint-implementation.json', 'Scoped Clippy, 62 Tauri, 45 Settings and 24 i18n checks; post-run source confirmation is explicit.'),
        (OUT / 'continuation-rust-check.json', 'Non-author six-file source and actual-selector review; no new Cargo execution.'),
        (OUT / 'continuation-doctor-check.json', 'Independent doctor production-sequence and test-contract review; historical failures retained.'),
        (OUT / 'continuation-security-implementation.json', 'Both lockfile audit fixes, original failures and scoped command records.'),
        (OUT / 'continuation-security-check.json', 'Independent lockfile review; no full-CI acceptance.'),
        (PARENT / 'research/root-continuation-frontend-tests.json', 'Full frontend automated suite: raw log counts 168 files and 901 tests; not native acceptance.'),
        (PARENT / 'research/root-continuation-coverage.json', 'Frontend coverage metrics with numerators and denominators; overlaps the frontend suite.'),
        (PARENT / 'research/root-continuation-final-ci.json', 'Formal root CI failed during CLI export_bindings process startup, after Tauri 407 behavior and two guard cases passed. Not a frontend-lint failure.'),
        (OUT / 'continuation-bindings-after-ci.json', 'All 230 generated type files match the previous guard baseline after the failed export; 263 frozen paths have no changes.'),
        (OUT / 'continuation-linux-msrv-workspace.json', 'Linux Rust 1.95 locked workspace/all-targets/all-features check passed; no test or full Linux CI claim.'),
        (OUT / 'continuation-windows-msrv-tauri.json', 'Windows Rust 1.95 independent Tauri locked/all-targets/all-features compile check passed; no runtime or native UI acceptance.'),
        (PARENT / 'research/root-continuation-ui-build.json', 'Actual bun run build exit 0; later capture-wrapper GBK rendering exit 1 is recorded separately.'),
        (OUT / 'continuation-bindings-startup-before.json', 'Post-failure executable observation only; no failure-time executable hash was recorded.'),
        (OUT / 'continuation-bindings-startup-list.json', 'Same observed executable lists 24 export tests successfully; discovery is not test execution.'),
        (OUT / 'continuation-bindings-startup-application-errors-initial.json', 'Initial no-event query retained before exact timestamp correction.'),
        (OUT / 'continuation-bindings-startup-application-errors.json', 'Corrected narrow Application Error query found no matching event; cause remains unconfirmed.'),
        (OUT / 'continuation-bindings-startup-bindings-check.json', 'Single subsequent bindings check passed; recorded pre/post source and generated-file snapshots are equal.'),
        (OUT / 'continuation-bindings-startup-tauri-ci.json', 'Independent complete just tauri-ci passed after the original root failure; no root aggregate acceptance inference.'),
        (OUT / 'continuation-bindings-startup-report.json', 'Startup investigation retains original failure, post-failure binary observation, absent matching event, and successful independent revalidation; cause remains unconfirmed.'),
        (OUT / 'continuation-linux-process-smoke-summary.json', 'Original-freeze Linux Tauri eight and core nine cases passed; newly generated Linux schema was precisely removed after the checks.'),
        (OUT / 'continuation-linux-import-fix.json', 'One cfg-only test-import correction responds to a compiler warning in a successful eight-case Linux run, not a test failure.'),
        (OUT / 'continuation-platform-freeze-provenance.json', 'Immutable final freeze keeps stale inherited lineage fields; explicit parent SHA and actual files delta are authoritative.'),
        (OUT / 'continuation-platform-final-checks.json', 'Six post-import commands exited zero; two zero-match OAuth commands are excluded from behavioral coverage. Linux schema cleanup remains explicit.'),
        (OUT / 'continuation-oauth-selector-correction.json', 'Corrected Linux OAuth summaries report 20 passed with 18 intact named selectors; Windows reports one passed and one intact name. Zero-match attempts and lossy-capture limits remain explicit.'),
    ],
    'T10.AC2': [
        (PARENT / 'research/p1-evidence-ledger.json', 'Nine scoped old-red/fixed-behavior maps; final root acceptance remains separate.'),
        (OUT / 'baseline-repro/evidence-index.json', 'Root index of original baseline counterexamples and instrumentation boundaries.'),
        (OUT / 'baseline-repro/cli-baseline.log', 'Original A01/A02/A03/A11: four scoped failing cases.'),
        (OUT / 'baseline-repro/a09-baseline.log', 'Original native deadline omission with shortened descriptor timeout.'),
        (OUT / 'baseline-repro/a14-baseline.log', 'Original Codex mapper notification-array loss.'),
        (OUT / 'baseline-repro/a15-baseline.log', 'Original managed-input lock loss at the real route.'),
    ],
    'T10.AC3': [
        (OUT / 'spec-validation.json', 'Current navigation, file budgets, manifest counts and protected-byte checks.'),
        (OUT / 'spec-convergence-evidence.json', 'Lossless extraction provenance for prior analytics/pricing/Insights content.'),
        (OUT / 'spec-context-09-28-usage-job-lifecycle.log', 'T06 context validation after lifecycle spec extraction.'),
        (OUT / 'spec-context-09-28-architecture-contract-gates.log', 'T10 context validation against current specs.'),
    ],
    'T10.AC4': [
        (OUT / 'spec-validation.json', 'Historical pre-authorization script hashes and original Insights task states; later authorized script changes are recorded separately.'),
        (PARENT / 'research/baseline-lint.json', 'Original formal lint outcome; no diagnostic substitution.'),
        (PARENT / 'research/baseline-lint-tracked-diagnostic.json', 'Separately labeled tracked-file diagnostic.'),
        (PARENT / 'research/root-final-frontend.json', 'Original-worktree formal frontend outcome retains protected-script failures.'),
        (PARENT / 'research/root-final-frontend.log', 'Unmodified raw formal frontend output.'),
        (PARENT / 'research/root-continuation-frontend.json', 'Historical independent formal frontend-check exited 1 with five protected-script no-console errors; the corresponding root CI did not reach frontend.'),
        (PARENT / 'research/root-continuation-frontend.log', 'Historical raw formal frontend output retained beside the earlier failure.'),
        (PARENT / 'research/isolated-final-frontend.json', 'Exact authorized diff isolated from protected unrelated scripts.'),
        (PARENT / 'research/isolated-final-frontend.log', 'Raw isolated frontend gate; does not override original-worktree failure.'),
    ],
}
artifact_requirements['T10.AC1'] += [
    (OUT / 'remaining-doctor-implementation.json', 'Test-only PowerShell path isolation, same-binary empty-PATH red reproduction and scoped checks; no exporter AV root-cause conclusion.'),
    (OUT / 'remaining-linux-tauri-ci.json', 'Actual complete Linux just tauri-ci on its inline source map: 397 behavior cases, two guards and separate exports.'),
    (OUT / 'remaining-linux-coverage-tauri.json', 'Actual coverage-tauri recipe; gateway threshold 85 is unchanged; overall Tauri coverage is reported without an overall threshold.'),
    (coverage_path, 'Immutable llvm-cov JSON copy matches the command metadata SHA; contains all numerators and denominators.'),
    (baseline_path, 'Prior 41 run fingerprints and 227 verified-artifact baseline retained before this additive integration.'),
    (OUT / 'remaining-cli-versions-implementation.json', 'Test-only host CLI isolation; empty-PATH diagnostic passed and does not identify the historical slow CLI.'),
    (OUT / 'remaining-native-post-run-binary.json', 'Post-native executable observation occurred during later Tauri compilation; current-path bytes do not replace the receipt-time binary SHA.'),
    (OUT / 'remaining-schema-restoration.json', 'Historical restoration and saved Linux schema are retained. Later Linux builds regenerated the removed path; this old receipt does not assert current absence.'),
    (schema_path, 'Final restoration preserves four preexisting schemas and separately saves the regenerated Linux schema before removing only the path absent from the original baseline.'),
]
if 'remaining_linux_coverage_final' in RUNS:
    artifact_requirements['T10.AC1'].append((final_coverage_path, 'd8ee-epoch Tauri coverage: raw line counts, gateway 85% threshold, no Tauri overall threshold. Other source epochs remain separate.'))
if 'remaining_linux_workspace_coverage_after_path' in RUNS:
    artifact_requirements['T10.AC1'].append((workspace_report_path, 'a31c workspace coverage-rust: immutable llvm-cov report with overall 70% and core gateway 85% thresholds; no Tauri scope inference.'))
if 'remaining_linux_coverage_after_path' in RUNS:
    artifact_requirements['T10.AC1'] += [
        (tauri_report_path, 'a31c coverage-tauri immutable report: Tauri gateway 85% threshold; overall percentage has no threshold in this recipe.'),
        (tauri_copy_path, 'Later report-copy observation exactly matches the original coverage command report SHA; the original command receipt is unchanged.'),
    ]
artifact_requirements['T10.AC3'] += [
    (context_path, 'Twelve task context validators exit zero; JSONL and existing Insights states are preserved; context-injection warnings remain explicit.'),
    (final_context_path, 'Repeated final validation preserves all 24 JSONL files across 12 tasks and both existing Insights statuses; the 33513-byte context warning remains explicit.'),
    (OUT / 'remaining-final-source-freeze.json', 'Final registered paths include the coordinated current install-dialog documentation path and confirmation nonce contract.'),
]
artifact_requirements['T08.AC3'] = [
    (native_acceptance_path, 'Scoped native Linux WebKitGTK Claude raw editor, confirmation scroll-lock nonce, script CSP negative control and synthetic file roundtrip; no native Codex/Grok or release-package inference.'),
    (OUT / 'remaining-native-csp-diagnosis.md', 'Original runtime style CSP failure and its separate CodeMirror and confirmation boundaries.'),
    (style_correction_path, 'Historical derived cleanup wording is corrected without changing raw receipts. Empty post-save style CSP events do not prove style DOM removal or body scroll-lock restoration.'),
]
artifact_requirements['T10.AC4'] += [
    (OUT / 'remaining-windows-script-baseline.json', 'Original two script hashes before the newly authorized five output substitutions.'),
    (OUT / 'remaining-windows-script-changes.diff', 'Authorized output substitutions preserve stream, formatting and newline behavior.'),
    (OUT / 'remaining-windows-script-verification.json', 'Syntax, scoped lint and five output-equivalence checks; the probe scripts themselves were not run.'),
    (OUT / 'remaining-windows-frontend-check.json', 'Later independent formal frontend-check exit 0 after authorized script edits; earlier failed gates remain unchanged.'),
    (OUT / 'remaining-windows-ci.json', 'New formal root CI fails in two Doctor fixture spawns before exporters, Tauri or frontend; historical AV is separate.'),
    (parity_path, 'Separate 230-file generated parity observation after the successful Windows guard; failed Linux maps remain unchanged.'),
]
for requirement_id in PERMISSION_REMEDIATION['requirement_refs']:
    artifact_requirements.setdefault(requirement_id, []).extend([
        (permission_fix_path, 'DC-02 focused repair evidence and unchanged content-journal metadata contract.'),
        (permission_review_path, 'Non-author root review of the six-file permission delta; formal epoch gates remain separate.'),
    ])
for row in rows:
    if row['id'] in PERMISSION_REMEDIATION['requirement_refs']:
        row['review_remediation_refs'] = ['DC-02']
        supplementary = {
            'T01.AC2': ['permission_noop_repository'],
            'T01.AC3': ['permission_noop_repository', 'permission_noop_guard', 'permission_base_regression'],
            'T05.AC2': ['permission_noop_guard', 'permission_base_regression', 'permission_atomic_policy'],
        }[row['id']]
        row['test_groups'].extend(supplementary)
    row['artifact_evidence'] = [{**artifact(path), 'role': role} for path, role in artifact_requirements.get(row['id'], [])]
    if row['id'] == 'T06.AC3':
        row['required_platform_matrix'] = {'Windows': 'scoped_native_evidence', 'Linux': 'scoped_native_evidence_rust_195_and_198', 'macOS': 'required_not_run'}
    if row['id'] in ('T08.AC1', 'T08.AC3'):
        row['native_webview_csp_acceptance'] = 'not_verified'
        row['native_shared_editor_evidence'] = {'status': NATIVE['status'], 'report': relative(native_acceptance_path), 'scope': NATIVE['acceptance_scope'], 'not_covered': NATIVE['not_covered']}
    if row['id'] == 'T08.AC3':
        row['native_webview_csp_acceptance'] = 'scoped_linux_claude_raw_editor_verified'
    if row['id'] == 'T10.AC1':
        row['test_groups'].append('store_path_platform_contract')
        row['artifact_evidence'].append({**artifact(store_before_path), 'role': 'Fresh exact-run original-source capture equals recorded HEAD; it does not backfill the earlier 268-path source map.'})
        row['continuation_run_refs'] = list(RUNS)

source_paths = sorted({g['test_source']['path'] for g in GROUPS.values()})
diff = subprocess.run(['git', '-c', 'core.safecrlf=false', 'diff', '34d8a85e0e48b793733835e0304c8ed33940fcee', '--', *source_paths], cwd=ROOT, capture_output=True, check=True)
diff_fingerprint = {'scope': source_paths, 'tracked_diff_sha256': hashlib.sha256(diff.stdout).hexdigest(), 'untracked_source_note': 'Git diff omits untracked source. Each test_source SHA-256 includes its actual file bytes regardless of tracking.'}

tauri_cases = set()
for key in ('tauri_codex', 'tauri_state', 'tauri_close-action', 'tauri_claude-read'):
    text = (ROOT / RUNS[key]['path']).read_text(encoding='utf-8')
    cases = set(re.findall(r'^test (\S+) \.\.\. ok$', text, re.MULTILINE))
    assert len(cases) == RUNS[key]['reported_counts']['matched_passed'], key
    assert not tauri_cases.intersection(cases), key
    tauri_cases.update(cases)
assert len(tauri_cases) == 62
matrix = {
    'date': '2026-09-29', 'baseline_commit': '34d8a85e0e48b793733835e0304c8ed33940fcee',
    'fix_commit': None,
    'scope': '39 child ACs. Reports and raw evidence are separate; nonzero and unexecuted results remain visible.',
    'fingerprint_note': 'Current source capture indexes selectors only. Prior captures are retained, and execution source attribution is stored on each new run. Refreshing SHA cannot bind historical logs to new source.',
    'review_limit': 'Dedicated trellis-check review is restored. Its current result and authorship boundaries are in remaining-dedicated-check.md/json. Earlier thread-limit failures are historical, not the current blocker.',
    'global_limits': [], 'requirements': rows, 'test_groups': GROUPS,
    'continuation_runs': RUNS, 'continuation_unique_tauri_selectors': sorted(tauri_cases),
    'continuation_final_evidence': {'status': 'local_check_records_complete_required_acceptance_open', 'pending': [], 'lifecycle_acceptance_granted': False},
}
matrix['test_source_diff_fingerprint'] = diff_fingerprint
matrix['continuation_web_evidence'] = WEB
matrix['remaining_native_evidence'] = NATIVE
matrix['dedicated_review'] = DEDICATED_REVIEW
matrix['remaining_schema_restoration'] = SCHEMA_RESTORATION
matrix['remaining_context_preservation'] = CONTEXT_PRESERVATION
matrix['remaining_generated_parity_observation'] = GENERATED_PARITY
matrix['post_review_remediation'] = {'DC-02': PERMISSION_REMEDIATION}
matrix['portability_literal_repair'] = PORTABILITY_REPAIR
matrix['store_path_portability'] = STORE_PATH_PORTABILITY
matrix['remaining_audit_tool_setup'] = AUDIT_TOOL_SETUP
matrix['remaining_platform_test_lane_receipts'] = [
    {'key': key, 'metadata': filename, 'status': 'recorded_finished' if key in RUNS else ('pending' if (OUT / filename).exists() else 'not_run'), 'exit_code': RUNS[key]['execution']['exit_code'] if key in RUNS else None}
    for key, filename in PLATFORM_TEST_RUNS
]
matrix['remaining_optional_receipts'] = [
    {'key': key, 'metadata': filename, 'status': 'recorded_finished' if key in RUNS else ('pending' if (OUT / filename).exists() else 'not_run')}
    for key, filename in OPTIONAL_REMAINING_RUNS
]
matrix['remaining_permission_lane_receipts'] = [
    {'key': key, 'metadata': filename, 'status': 'recorded_finished' if key in RUNS else ('pending' if (OUT / filename).exists() else 'not_run_superseded_by_portability_epoch')}
    for key, filename in PERMISSIONS_RUNS
]
matrix['remaining_portability_lane_receipts'] = [
    {'key': key, 'metadata': filename, 'status': 'recorded_finished' if key in RUNS else ('pending' if (OUT / filename).exists() else ('not_run_superseded_by_platform_test_epoch' if active_epoch_runs == PLATFORM_TEST_RUNS else 'not_run'))}
    for key, filename in PORTABILITY_RUNS
]
platform_capture = capture_freeze(OUT / 'continuation-platform-final-source-freeze.json', 'Historical import-correction epoch; its runs retain this source identity')
platform_capture['provenance_correction'] = artifact(OUT / 'continuation-platform-freeze-provenance.json')
platform_capture['lineage_rule'] = 'Use immutable files and explicit parent_source_freeze SHA / changes_from_parent. Inherited previous_freeze/changes_since_previous omit the added import correction; their count is not authoritative.'
remaining_captures = [capture_freeze(OUT / name, 'Root-owned remaining source epoch; use only runs whose command-start maps equal these files') for name in REMAINING_SOURCE_EPOCHS if (OUT / name).exists()]
matrix['superseded_source_freezes'] = [capture_freeze(OUT / 'continuation-final-source-freeze.json', 'Original aggregate, standalone Tauri and earlier platform checks retain this frozen source identity'), platform_capture, *remaining_captures[:-1]]
matrix['final_source_freeze'] = remaining_captures[-1]
assert matrix['final_source_freeze']['matches_current_sources'], 'final source freeze drift'
NATIVE['passed_attempts_on_final_source_epoch'] = [item['report']['path'] for item in native_attempts if item['state'] == 'passed' and item.get('source_epoch', {}).get('manifest', {}).get('sha256') == matrix['final_source_freeze']['sha256']]
matrix['historical_mapping_baseline'] = artifact(baseline_path)
matrix['continuation_final_evidence']['pending'] = remaining_pending
matrix['continuation_final_evidence']['remaining_required_acceptance'] = ['macOS native process matrix: required_not_run', 'fix_commit remains null; T10.AC2 remains partial_mapping_fix_commit_pending']
matrix['continuation_final_evidence']['native_acceptance'] = NATIVE['status']
matrix['continuation_final_evidence']['native_receipt_on_final_source_epoch'] = 'passed_scoped_receipt' if NATIVE['passed_attempts_on_final_source_epoch'] else 'not_verified'
if not NATIVE['passed_attempts_on_final_source_epoch']:
    matrix['continuation_final_evidence']['remaining_required_acceptance'].append('Native receipt on the final source epoch is not yet verified; earlier native acceptance keeps its original epoch')
latest_root = RUNS.get('remaining_windows_ci_after_path', RUNS.get('remaining_windows_ci_after_octal', RUNS.get('remaining_windows_ci_after_permissions', RUNS.get('remaining_windows_ci_final', RUNS.get('remaining_windows_ci_after_fixture', RUNS['remaining_windows_ci_before_fixture'])))))
matrix['continuation_final_evidence']['latest_root_run'] = latest_root['execution']['label']
matrix['continuation_final_evidence']['latest_root_exit_code'] = latest_root['execution']['exit_code']
if latest_root['execution']['exit_code'] != 0:
    matrix['continuation_final_evidence']['remaining_required_acceptance'].insert(0, 'Latest completed root aggregate exited nonzero; its original failure evidence remains authoritative')
latest_linux_quality = RUNS.get('remaining_linux_workspace_quality_after_path', RUNS.get('remaining_linux_workspace_quality_after_octal', RUNS.get('remaining_linux_workspace_quality_after_python', RUNS.get('remaining_linux_workspace_quality_after_permissions'))))
if latest_linux_quality:
    matrix['continuation_final_evidence']['latest_linux_workspace_quality_run'] = latest_linux_quality['execution']['label']
    matrix['continuation_final_evidence']['latest_linux_workspace_quality_exit_code'] = latest_linux_quality['execution']['exit_code']
    if latest_linux_quality['execution']['exit_code'] != 0:
        matrix['continuation_final_evidence']['remaining_required_acceptance'].append('Latest completed Linux workspace-quality recipe sequence exited nonzero; this separate lane is not an entire Linux just ci')
matrix['global_limits'] = [
    'The planned macOS native process matrix is required and not run',
    'Native Linux WebKitGTK custom-protocol debug acceptance passed for the synthetic Claude raw editor, confirmation scroll locking, CSP and file save. Windows/macOS WebViews, native Codex/Grok and release packaging remain outside this receipt',
    'Native post-save style evidence records no captured style CSP violations. Style DOM removal and body scroll-lock restoration were not measured; process-group cleanup has separate evidence',
    'No real remote SSH/WSL accounts or provider OAuth login',
    'Historical root exporter STATUS_ACCESS_VIOLATION remains unexplained; later completed runs do not establish its cause',
    'The earlier five no-console errors remain in immutable failed logs. The two script edits were subsequently authorized, and the separate remaining formal frontend gate passed',
    'Actual Linux just tauri-ci and coverage-tauri passed on remaining source maps; neither command is the entire Linux root CI',
    'd8ee-epoch Tauri gateway line coverage is 654/703 (93.0298719772404%), with threshold 85%; its overall is 17964/34456 (52.1360575806826%) without a Tauri overall threshold; the 46f5 report and later epochs remain separate',
    'Source epochs and command-start maps remain distinct. No source hash refresh reattributes historical runs',
]
if context_warnings:
    matrix['global_limits'].append('Task context validators exited zero but retained context-injection warnings; see remaining_context_preservation.warnings for exact affected tasks and limits')
(OUT / 'requirements-evidence.json').write_text(json.dumps(matrix, ensure_ascii=False, indent=2) + chr(10), encoding='utf-8')
lines = ['# T10 逐项验收证据矩阵', '', '生成日期（UTC）：2026-09-29。基线：34d8a85e0e48b793733835e0304c8ed33940fcee。没有创建修复提交，fix_commit 为 null。', '', '本表覆盖 39 个子任务 AC。精确测试选择器、源码行、原始日志、SHA-256 和 mock/平台边界保存在 requirements-evidence.json；表内引用的是证据组，不是新增通过结论。旧失败复现仍缺失的 P1 必须保持开放。', '', '| AC | 机制 | 测试组 | operation × platform × entry |', '| --- | --- | --- | --- |']
for row in rows:
    evidence_label = ', '.join(row['test_groups']) or 'artifact_evidence：' + str(len(row['artifact_evidence'])) + ' 项'
    lines.append('| ' + ' | '.join([row['id'], row['mechanism'], evidence_label, row['operation_platform_entry']]) + ' |')
lines += ['', '## 验收限制', '', '- 使用真实前端组件与底层 transport mock 的测试，只证明所列前端调用链；Rust 临时文件、真实子进程和 loopback fixture 单独提供后端证据。两者组合不等于原生 UI 端到端。', '- A01/A02/A03/A11 原始 CLI 库红例、A09 缩短 descriptor deadline 的原生子进程红例、A14 原 mapper 红例和 A15 原组件红例均已记录。A02 为确定性旧快照交错；A03 仅覆盖已提交后的 history failure；A11 为原函数权限失败组合。各范围不外推。', '- A09 旧 cleanup-error precedence、新增 descendant 用例、A03 旧 TUI off-before-invalid 与每种 rename 故障、A15 旧 raw-editor 可达性未分别运行旧基线，保留为源码证据。A08 旧 pre-run spawn 窗口同样保留源码边界。', '- T10 AC2/AC3/AC4 的 artifact_evidence 分别保存 P1/旧红例、规范导航/抽取/上下文、正式与隔离门禁/受保护基线的路径及 SHA-256。空 test_groups 不表示空验收证据。', '- T10 专职 trellis-check 已恢复，当前报告为 remaining-dedicated-check.md/json。历史线程限制失败仍保留；T01 既有作者关系和本轮文档单行修复的独立性边界单独记录。', '- 历史原工作区 frontend-check 保留两个脚本的五个 no-console 错误。后续五处输出替换已获授权，remaining 正式 frontend-check 通过；旧失败原样保留。root CI、Linux Tauri 和原生 WebView 使用各自的原始日志及源码归属。']
lines += ['', '## 本轮补充证据', '', '| 运行 | 退出码 | 范围 |', '| --- | ---: | --- |']
for key, run in RUNS.items():
    counts = run.get('observed_test_counts') or run.get('test_partitions') or run['reported_counts']
    lines.append('| ' + key + ' | ' + str(run['execution']['exit_code']) + ' | ' + (json.dumps(counts, ensure_ascii=False) if counts else '见原始日志和命令元数据') + ' |')
lines += ['', '- 四组 Tauri 日志去重后为 62 个实际通过 selector；空 target 和 export_bindings 不计入。不同运行之间存在覆盖重叠，不累计为唯一测试总数。', '- Web 的 7 个成功 receipt 使用合成 IPC；model-only patch、通知数组、未知值、raw CodeMirror 与纯 Web 不可用状态已记录。实际磁盘写入属于独立后端 helper fixture。', '- 当前源码 SHA 只定位选择器。运行绑定单列 capture_mode、recorded_source_sha256 与一致性结果；旧日志不会因 SHA 刷新而获得新源码执行证明。', '- 覆盖率按原始日志记录 Statements、Branches、Functions、Lines 的分子与分母，不将百分比四舍五入为满足其他阈值。']
lines += ['', '- 两次 OAuth 错误选择器运行均为 exit 0 / 0 匹配，明确排除行为验收。修正后 Linux Rust 汇总为 20 项通过、完整具名记录为 18 项；Windows 汇总与完整具名记录均为 1 项。process 复验汇总 8 项通过、完整具名记录为 5 项；缺损名称不猜补，重复运行不累计。', '- a531、fa79、46f5 和 d8ee 冻结分开记录。旧 full CI、独立 tauri-ci、MSRV 和 OAuth 定向检查保留原来的源码归属；新正式 Linux 门禁与各自的命令前后 map 关联。fa79 的继承 lineage 计数偏差保留独立 provenance correction。', '- 原 exporter AV 和后续 Doctor fixture spawn 失败分属不同的 root CI 运行。后续重跑不修改历史失败，AV 原因仍未查明。']
lines += ['', '## 剩余事项续作', '',
    '- 旧 41 条运行对象逐个按 canonical SHA-256 校验保持一致；旧 227 项 artifact 的验证计数保存在 remaining-mapping-baseline.json。',
    '- Linux 完整 just tauri-ci：397 项行为测试通过、1 项 ignored、2 项 guard；导出分区为 24/9/197。inventory 的 1 项与行为测试重叠，空 target 不累计。',
    '- d8ee 阶段 gateway 行覆盖率为 654/703（93.0298719772404%），现行门槛为 85%。该阶段 Tauri 总体行覆盖率为 17964/34456（52.1360575806826%）；46f5 及后续阶段报告分别保留。现行 Tauri recipe 未设置总体门槛。',
    '- 原生 attempt6 在 Linux WebKitGTK/WSLg 中通过：确认滚动锁与 CodeMirror nonce、CSP 内联脚本拒绝、合成 Claude 配置磁盘往返及清理。attempt1–5 的失败均保留；不外推到 Windows/macOS WebView、Codex/Grok 原生保存或发布安装包。',
    '- 原生执行时二进制 SHA 与 build 记录关联。后续 Cargo 会覆盖同路径可执行文件，当前文件哈希不同不重写执行时证据。',
    '- 最新完成的 root aggregate：' + latest_root['execution']['label'] + '，exit ' + str(latest_root['execution']['exit_code']) + '。',
]
lines += ['- ' + limitation for limitation in matrix['global_limits']]
(OUT / 'requirements-evidence.md').write_text(chr(10).join(lines) + chr(10), encoding='utf-8')
print(json.dumps({'requirements': len(rows), 'test_groups': len(GROUPS), 'artifacts': ['requirements-evidence.json', 'requirements-evidence.md']}, ensure_ascii=False))
