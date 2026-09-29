"""Attach verified evidence candidates. Root owns final acceptance and lifecycle."""
from pathlib import Path
import hashlib
import json
import subprocess

ROOT = Path(__file__).resolve().parents[4]
RESEARCH = Path(__file__).resolve().parent
PARENT = ROOT / '.trellis/tasks/09-28-cli-tauri-architecture'
PATH = PARENT / 'research/p1-evidence-ledger.json'
ledger = json.loads(PATH.read_text(encoding='utf-8'))
matrix = json.loads((RESEARCH / 'requirements-evidence.json').read_text(encoding='utf-8'))
baseline_hashes = {entry['path']: entry['sha256'] for finding in ledger['findings'] for entry in finding.get('baseline_evidence', {}).get('artifacts', [])}


def evidence(path):
    p = ROOT / path
    assert p.is_file(), p
    digest = hashlib.sha256(p.read_bytes()).hexdigest()
    if path in baseline_hashes:
        assert digest == baseline_hashes[path], ('historical baseline changed', path)
    return {'path': path, 'sha256': digest}


baseline = '.trellis/tasks/09-28-architecture-contract-gates/research/baseline-repro/'
old_cli = [baseline + name for name in ('architecture_baseline_tests.rs', 'cli-baseline.json', 'cli-baseline.log')]
definitions = {
    'A01': ('config_handlers config_page', ['T03.AC1', 'T03.AC3'], 'executed_original_call_target', old_cli, 'a01_desktop_legacy_switch_target_must_succeed', 'Original switch_command called by old Tauri handler returns migration error. Isolated CLI-library test invokes the real old call target; not an old native Tauri UI end-to-end run.', ['ccr-ui/src-tauri/src/commands/config.rs', 'ccr-ui/src-tauri/src/commands/config/adapter.rs', 'ccr-ui/src/features/configs/hooks/useConfigsPage.ts']),
    'A02': ('repository config_handlers', ['T01.AC1', 'T01.AC3', 'T03.AC2'], 'executed_original_deterministic_interleaving', old_cli, 'a02_stale_platform_snapshot_must_not_erase_service_update', 'Original Platform stale whole-document save erases a committed ConfigService field. Deterministic interleaving, not an old multiprocess race. Fixed-side tests run real independent child processes through adapters.', ['crates/ccr-config/src/managers/config/repository.rs', 'crates/ccr-config/src/platforms/base.rs', 'crates/ccr-config/src/services/config_service.rs']),
    'A03': ('profile_cli profile_tui profile_desktop profile_warning_binary journal', ['T02.AC1', 'T02.AC2', 'T02.AC3', 'T02.AC4', 'T02.AC5'], 'executed_original_application_history_failure', old_cli, 'a03_committed_activation_must_not_return_generic_history_failure', 'Original application actually commits runtime=new, then generic history DB failure escapes. This red fixture covers ancillary failure; the original TUI off-before-invalid-target sequence and every old rename fault were not separately executed.', ['crates/ccr-cli/src/application/profile_lifecycle.rs', 'crates/ccr-cli/src/application/profile_switch.rs', 'crates/ccr-core/src/core/write_journal.rs', 'crates/ccr-tui/src/tui/profile_backend.rs']),
    'A08': ('usage_registry usage_ui control_policy', ['T06.AC1', 'T06.AC2'], 'executed_preimplementation_registry_red', ['.trellis/tasks/09-28-usage-job-lifecycle/baseline-tests.log'], 'cancelled_result_rejects_late_failure;failed_result_rejects_cancel_and_late_progress', 'Two production registry tests failed before T06 implementation (0 passed/2 failed). The old pre-run cancellation spawn window was source-confirmed; current barrier proves zero spawns.', ['ccr-ui/src-tauri/src/usage_jobs.rs', 'ccr-ui/src-tauri/src/commands/usage.rs']),
    'A09': ('usage_executor process_tree', ['T06.AC3', 'T06.AC4'], 'executed_instrumented_original_native_deadline', [baseline + name for name in ('a09_baseline_test.rs', 'a09-instrumentation.json', 'a09-baseline.json', 'a09-baseline.log', 'silent_child.rs')], 'a09_native_stream_must_obey_shortened_descriptor_deadline', 'Original run_sync_stream/ProcessGateway with only descriptor timeout changed from 3600000ms to 30ms: a real silent child remains beyond 300ms; explicit cancellation/reap completes. Old deadline omission is behavior-proven. Old cleanup-error precedence and new descendant cleanup tests were not run against the old binary; those old paths remain source evidence only.', ['ccr-ui/src-tauri/src/llmusage_adapter/cli.rs', 'ccr-ui/src-tauri/src/process/gateway.rs', 'crates/ccr-core/src/core/process_gateway.rs']),
    'A11': ('pending oauth', ['T05.AC3', 'T05.AC4'], 'executed_original_permission_sequence', old_cli, 'a11_permission_failure_must_not_be_reported_as_success', 'An actual invalid-user icacls invocation first proves a nonzero permission-command result. Original AtomicWriter then ensure_private_permissions sequence still returns success. This uses the real original functions with synthetic bytes, not a complete old OAuth handler or proven secret disclosure.', ['crates/ccr-codex/src/services/codex_oauth_pending_store.rs', 'ccr-ui/src-tauri/src/commands/codex_auth.rs', 'crates/ccr-core/src/core/atomic_writer.rs']),
    'A13': ('command_page command_listener command_owner control_policy', ['T07.AC1', 'T07.AC2', 'T07.AC3'], 'executed_preimplementation_route_red', ['.trellis/tasks/09-28-command-workbench-lifecycle/research/initial-red.log', '.trellis/tasks/09-28-cli-tauri-architecture/research/frontend-audit.evidence.test.tsx', '.trellis/tasks/09-28-cli-tauri-architecture/research/frontend-audit.md'], 'route remount;terminal before start;late listener cleanup;duplicate submit', 'T07 initial suite 0 passed/3 failed; original audit characterizes late start and late listen. Route test mocks API responses/events; backend owner/control fixtures supply real Rust behavior. History semantics are one write attempt per job, not cross-IPC exactly-once.', ['ccr-ui/src/features/commands/useCommandsPage.ts', 'ccr-ui/src/shell/eventBridge.ts', 'ccr-ui/src/features/commands/stores.ts']),
    'A14': ('settings_mapping settings_page codex_settings_backend codex_settings_persistence', ['T08.AC1'], 'executed_original_mapper_notification_array_red', [baseline + name for name in ('a14-baseline.test.ts', 'a14-baseline.json', 'a14-baseline.log', 'settings-codex-map-baseline.ts', 'settings-helpers-baseline.ts')], 'A14 changing only model preserves notification event array', 'Original mapper/helper extracted from the recorded baseline, with only the helper import redirected locally: changing model changes notifications from the event array to false (0 passed/1 failed). No mapper logic instrumentation. Fixed renderer tests use actual mapper/domain and bottom transport mock. Rust projection/merge retains its original evidence; the continuation 56-case Codex run adds the production private persistence helper with an actual model-only file write and full TOML reread. This is separate-layer evidence and does not execute Tauri invoke, State/cache invalidation, or native WebView.', ['ccr-ui/src/configs/settings-codex-map.ts', 'ccr-ui/src/configs/settings-codex.ts', 'ccr-ui/src/configs/settings-patch.ts', 'ccr-ui/src-tauri/src/commands/codex_settings.rs']),
    'A15': ('settings_page grok_settings settings_raw_session', ['T08.AC2', 'T08.AC3'], 'executed_original_component_managed_lock_red', [baseline + name for name in ('baseline-capabilities-input.tsx', 'a15-baseline.json', 'a15-baseline.log')], 'disables managed model inputs, keeps unknown enums visible, and saves only the edited key', 'Original GrokSettingsView with real adapter and bottom IPC mock renders managed model editable (1 failed, 29 skipped). The red covers lock metadata loss; raw-editor reachability was old-source evidence. Fixed route tests exercise actual CodeMirror; separate real Grok filesystem fixture proves CAS/no-backup.', ['ccr-ui/src/features/platform/settings/BaseSettings.tsx', 'ccr-ui/src/configs/settings-grok.ts', 'ccr-ui/src/features/platform/settings/SettingsSource.tsx']),
}

for finding in ledger['findings']:
    groups, refs, kind, old_paths, selector, boundary, source_paths = definitions[finding['finding']]
    finding['final_behavior_test_mapping'] = []
    for name in groups.split():
        group = matrix['test_groups'][name]
        finding['final_behavior_test_mapping'].append({'group': name, 'source': group['test_source'], 'source_capture_role': group['source_capture_role'], 'prior_source_captures': group['prior_source_captures'], 'cases': group['cases'], 'evidence': group['evidence'], 'evidence_level': group['level'], 'limits': group['limits']})
    finding['requirement_refs'] = refs
    finding['baseline_evidence'] = {'kind': kind, 'selector': selector, 'artifacts': [evidence(path) for path in old_paths], 'boundary': boundary}
    finding['red_execution_status'] = 'executed_scoped_counterexample'
    finding['final_mapping_status'] = 'reviewed_scoped_behavior_mapping_root_gate_reported_separately'
    finding['fix_commit'] = None
    current_sources = [evidence(path) for path in source_paths]
    history = list(finding.get('prior_source_captures', []))
    for old in finding.get('source_fingerprints', []):
        if old not in current_sources and old not in history:
            history.append(old)
    finding['prior_source_captures'] = history
    finding['source_fingerprints'] = current_sources
    diff = subprocess.run(['git', '-c', 'core.safecrlf=false', 'diff', ledger['baseline_commit'], '--', *source_paths], cwd=ROOT, capture_output=True, check=True)
    finding['tracked_diff_sha256'] = hashlib.sha256(diff.stdout).hexdigest()
    finding['fingerprint_note'] = 'Current source capture and tracked diff only. Earlier captures are retained; execution attribution belongs to each evidence entry. Current SHA does not bind an old log to new source. Root owns final source parity and aggregate acceptance; no fix commit exists.'
    finding['candidate_note'] = 'Candidate locations have been verified and promoted to scoped test/log mappings. This field does not grant whole-task, native or full-gate acceptance.'
    if finding['finding'] == 'A09':
        finding['remaining_required_acceptance'] = ['macOS native process cleanup matrix: required_not_run']
    if finding['finding'] == 'A02':
        finding['review_remediation_refs'] = ['DC-02']
    if finding['finding'] in ('A14', 'A15'):
        finding['native_webview_csp_acceptance'] = 'not_verified'
        finding['native_shared_editor_evidence'] = {
            'status': matrix['remaining_native_evidence']['status'],
            'report': matrix['remaining_native_evidence']['acceptance_report'],
            'scope': matrix['remaining_native_evidence']['acceptance_scope'],
            'style_measurement_boundary': matrix['remaining_native_evidence']['style_measurement_boundary'],
            'boundary': 'Native Claude shared-editor/CSP evidence does not execute the native Codex model-only save or Grok settings flow required by this specific finding.',
        }
ledger['scope'] = 'Nine original P1 groups mapped to original scoped counterexamples, fixed production/adapter/renderer behavior tests, raw evidence, source/diff fingerprints and limits. Final root acceptance is separate.'
ledger['mapping_report'] = '.trellis/tasks/09-28-architecture-contract-gates/research/requirements-evidence.json'
ledger['review_limit'] = 'Dedicated trellis-check review is restored. Current result and authorship boundaries are in remaining-dedicated-check.md/json; earlier dispatch failures remain historical.'
ledger['dedicated_review'] = matrix['dedicated_review']
ledger['post_review_remediation'] = matrix['post_review_remediation']
PATH.write_text(json.dumps(ledger, ensure_ascii=False, indent=2) + chr(10), encoding='utf-8')
print(json.dumps({'p1_groups': len(ledger['findings']), 'fix_commits': sorted({str(f['fix_commit']) for f in ledger['findings']})}))
