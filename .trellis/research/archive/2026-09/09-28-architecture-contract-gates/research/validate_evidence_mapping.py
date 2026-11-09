"""Validate the captured AC/P1 evidence graph without granting acceptance."""
from pathlib import Path
from datetime import datetime
import base64
import hashlib
import json
import re
import subprocess

ROOT = Path(__file__).resolve().parents[4]
OUT = Path(__file__).resolve().parent
matrix = json.loads((OUT / 'requirements-evidence.json').read_text(encoding='utf-8'))
ledger = json.loads((ROOT / '.trellis/tasks/09-28-cli-tauri-architecture/research/p1-evidence-ledger.json').read_text(encoding='utf-8'))
checked = set()


def verify(entry):
    path = ROOT / entry['path']
    assert path.is_file(), entry['path']
    assert hashlib.sha256(path.read_bytes()).hexdigest() == entry['sha256'], entry['path']
    checked.add(entry['path'])


assert len(matrix['requirements']) == 39
assert len({row['id'] for row in matrix['requirements']}) == 39
assert len(ledger['findings']) == 9
ids = {row['id'] for row in matrix['requirements']}
for name, group in matrix['test_groups'].items():
    verify(group['test_source'])
    assert group['source_capture_role'] == 'current_selector_index_only', name
    for previous in group['prior_source_captures']:
        assert previous['path'] == group['test_source']['path'], name
        assert re.fullmatch(r'[0-9a-f]{64}', previous['sha256']), name
    lines = (ROOT / group['test_source']['path']).read_text(encoding='utf-8').splitlines()
    for case in group['cases']:
        assert case['selector'] in lines[case['line'] - 1], (name, case)
    for entry in group['evidence']:
        verify(entry)
        binding = entry['source_binding']
        assert binding['status'] in ('not_established_by_hash_refresh', 'not_recorded_for_test_source', 'post_run_confirmed_unchanged', 'command_start_and_finish_source_sha256', 'command_before_registered_source_map_with_post_comparison', 'command_before_and_after_registered_source_snapshot', 'command_before_and_after_registered_source_map'), (name, binding)
        if binding.get('recorded_source_sha256'):
            assert binding['current_capture_matches_recorded'] == (binding['recorded_source_sha256'] == group['test_source']['sha256']), name
        if 'execution' in entry:
            verify(entry['execution']['metadata'])
for row in matrix['requirements']:
    assert row['test_groups'] or row['artifact_evidence'], row['id']
    assert all(name in matrix['test_groups'] for name in row['test_groups'])
    for entry in row['artifact_evidence']:
        verify(entry)
for finding in ledger['findings']:
    assert finding['fix_commit'] is None
    assert finding['red_execution_status'] == 'executed_scoped_counterexample'
    assert all(ref in ids for ref in finding['requirement_refs'])
    for entry in finding['baseline_evidence']['artifacts'] + finding['source_fingerprints']:
        verify(entry)
    for mapped in finding['final_behavior_test_mapping']:
        source = matrix['test_groups'][mapped['group']]
        assert mapped['source'] == source['test_source']
        assert mapped['source_capture_role'] == source['source_capture_role']
        assert mapped['prior_source_captures'] == source['prior_source_captures']
        assert mapped['cases'] == source['cases']
        assert mapped['evidence'] == source['evidence']
for name in ('codex_settings_backend', 'codex_settings_persistence'):
    backend = matrix['test_groups'][name]
    backend_lines = [line for entry in backend['evidence'] for line in entry['selected_case_lines']]
    for case in backend['cases']:
        assert any(case['selector'] in line and line.endswith(' ... ok') for line in backend_lines), case
assert 'codex_settings_persistence' in next(row for row in matrix['requirements'] if row['id'] == 'T08.AC1')['test_groups']
assert 'codex_settings_persistence' in {entry['group'] for entry in next(finding for finding in ledger['findings'] if finding['finding'] == 'A14')['final_behavior_test_mapping']}
assert next(row for row in matrix['requirements'] if row['id'] == 'T06.AC3')['required_platform_matrix']['macOS'] == 'required_not_run'
assert next(finding for finding in ledger['findings'] if finding['finding'] == 'A09')['remaining_required_acceptance']
for row in matrix['requirements']:
    if row['id'] == 'T08.AC1':
        assert row['native_webview_csp_acceptance'] == 'not_verified'
    if row['id'] == 'T08.AC3':
        assert row['native_webview_csp_acceptance'] == 'scoped_linux_claude_raw_editor_verified'
        assert 'runtime_style_nonce' in row['test_groups']
for run in matrix['continuation_runs'].values():
    verify(run)
    verify(run['execution']['metadata'])
    if 'log_streams' in run:
        meta = json.loads((ROOT / run['execution']['metadata']['path']).read_text(encoding='utf-8'))
        assert meta.get('state') in (None, 'finished') and run['execution']['state'] == 'finished'
        assert meta['finished_at_utc'] and meta['started_at_utc']
        assert meta['exit_code'] == run['execution']['exit_code']
        assert meta['command'] == run['execution']['command']
        metadata_path = ROOT / run['execution']['metadata']['path']
        logs = meta.get('logs', {'combined': meta.get('log', metadata_path.with_suffix('.log').name)})
        hashes = meta['log_sha256'] if isinstance(meta['log_sha256'], dict) else {'combined': meta['log_sha256']}
        assert set(run['log_streams']) == set(logs) == set(hashes)
        for name, stream in run['log_streams'].items():
            verify(stream)
            assert stream['sha256'] == hashes[name]
            path = Path(logs[name])
            path = path if path.is_absolute() else (ROOT / path if path.parts[0] == '.trellis' else metadata_path.parent / path)
            assert path.relative_to(ROOT).as_posix() == stream['path']
        attribution = run['execution']['source_attribution']
        assert attribution['capture_mode'] == 'command_before_and_after_registered_source_map'
        assert attribution['source_files_sha256'] == {path: digest for path, digest in meta['sources_before'].items() if digest is not None}
        if attribution.get('source_path_base'):
            assert attribution['source_path_base'] == 'ccr-ui'
            assert attribution['resolved_source_files_sha256'] == {'ccr-ui/' + path: digest for path, digest in meta['sources_before'].items() if digest is not None}
        assert attribution['absent_paths'] == sorted(path for path, digest in meta['sources_before'].items() if digest is None)
        assert set(meta['sources_before']) == set(meta['sources_after'])
        differences = [
            {'path': path, 'before': meta['sources_before'][path], 'after': meta['sources_after'][path]}
            for path in sorted(meta['sources_before'])
            if meta['sources_before'][path] != meta['sources_after'][path]
        ]
        assert attribution['post_command_source_changes'] == differences
        if 'source_changes' in meta:
            reported = meta['source_changes']
            assert isinstance(reported, list)
            expected = [item['path'] for item in differences] if all(isinstance(item, str) for item in reported) else differences
            assert reported == expected
        assert attribution['reported_source_changes'] == meta.get('source_changes')
        assert attribution['source_stable_between_snapshots'] == (meta['sources_before'] == meta['sources_after'])
        if 'declared_freeze_reference' in run['execution']:
            reference = run['execution']['declared_freeze_reference']
            verify(reference['manifest'])
            frozen = json.loads((ROOT / reference['manifest']['path']).read_text(encoding='utf-8'))['files']
            assert reference['manifest']['sha256'] == meta['source_freeze_sha256']
            assert reference['command_start_matches_declared_freeze'] == (meta['sources_before'] == frozen)
            assert reference['command_start_differences'] == [
                {'path': path, 'freeze_sha256': frozen.get(path), 'command_start_sha256': meta['sources_before'].get(path)}
                for path in sorted(set(frozen) | set(meta['sources_before']))
                if frozen.get(path) != meta['sources_before'].get(path)
            ]
        if 'generated_comparison' in run['execution']:
            generated = run['execution']['generated_comparison']
            verify(generated['before_snapshot'])
            snapshot = json.loads((ROOT / generated['before_snapshot']['path']).read_text(encoding='utf-8'))
            assert len(snapshot) == generated['before_count'] and generated['before_bytes_verified']
            assert all(hashlib.sha256(base64.b64decode(entry['bytes_base64'], validate=True)).hexdigest() == entry['sha256'] for entry in snapshot.values())
            assert generated['before_count'] == meta['generated_before_count']
            assert generated['after_count'] == meta['generated_after_count']
            assert generated['changes'] == meta['generated_changes']
        if 'parent_freeze_reference' in run['execution']:
            parent = run['execution']['parent_freeze_reference']['manifest']
            verify(parent)
            assert parent['sha256'] == meta['parent_source_freeze_sha256']
        if 'source_epoch' in run['execution']:
            epoch = run['execution']['source_epoch']
            verify(epoch['manifest'])
            files = json.loads((ROOT / epoch['manifest']['path']).read_text(encoding='utf-8'))['files']
            assert epoch['source_before_equals_epoch'] and files == meta['sources_before']
            assert epoch['source_after_equals_epoch'] == (files == meta['sources_after'])
        if 'generated_map_comparison' in run['execution']:
            generated = run['execution']['generated_map_comparison']
            before = meta['generated_before']
            after = meta['generated_after']
            assert generated['before_count'] == len(before) and generated['after_count'] == len(after)
            assert generated['maps_equal'] == (before == after)
            differences = [
                {'path': path, 'before': before.get(path), 'after': after.get(path)}
                for path in sorted(set(before) | set(after)) if before.get(path) != after.get(path)
            ]
            assert generated['changes'] == differences
            reported = meta['generated_changes']
            assert isinstance(reported, list)
            expected = [item['path'] for item in differences] if all(isinstance(item, str) for item in reported) else differences
            assert generated['reported_changes'] == reported == expected
    if 'frozen_source_evidence' in run['execution']:
        verify(run['execution']['frozen_source_evidence']['manifest'])
    if 'command_snapshots' in run['execution']:
        verify(run['execution']['command_snapshots']['before'])
        verify(run['execution']['command_snapshots']['after'])
cases = set()
for key in ('tauri_codex', 'tauri_state', 'tauri_close-action', 'tauri_claude-read'):
    run = matrix['continuation_runs'][key]
    text = (ROOT / run['path']).read_text(encoding='utf-8')
    selected = set(re.findall(r'^test (\S+) \.\.\. ok$', text, re.MULTILINE))
    assert len(selected) == run['reported_counts']['matched_passed'], key
    assert not cases.intersection(selected), key
    cases.update(selected)
assert len(cases) == 62 and sorted(cases) == matrix['continuation_unique_tauri_selectors']
assert any('9 passed; 0 failed;' in line for line in matrix['continuation_runs']['linux_198_process']['result_lines'])
assert any(re.search(r'Tests\s+45 passed \(45\)', line) for line in matrix['continuation_runs']['settings_smoke']['result_lines'])
i18n = matrix['continuation_runs']['i18n']
assert i18n['reported_counts'] == {'matched_passed': 24, 'locale_leaf_keys': 4523}
i18n_text = re.sub(r'\x1b\[[0-9;]*m', '', (ROOT / i18n['path']).read_text(encoding='utf-8'))
assert re.search(r'Passed:\s+24', i18n_text)
assert all(f'{locale}.ts has 4523 leaf keys' in i18n_text for locale in ('zh-CN', 'en-US'))
for key in ('frontend_full_tests', 'frontend_coverage'):
    run = matrix['continuation_runs'][key]
    assert run['observed_test_counts'] == {'files': 168, 'tests': 901, 'overlaps_other_runs': True}
    assert any(re.search(r'Test Files\s+168 passed \(168\)', line) for line in run['result_lines'])
    assert any(re.search(r'Tests\s+901 passed \(901\)', line) for line in run['result_lines'])
coverage_run = matrix['continuation_runs']['frontend_coverage']
coverage_text = (ROOT / coverage_run['path']).read_text(encoding='utf-8')
for name, value in coverage_run['coverage'].items():
    matched = re.search(r'^' + name + r'\s*:\s*([0-9.]+)%\s*\(\s*(\d+)/(\d+)\s*\)', coverage_text, re.MULTILINE)
    assert matched and value == {'percent': float(matched[1]), 'covered': int(matched[2]), 'total': int(matched[3])}
    assert 0 <= value['covered'] <= value['total'] and value['total'] > 0
    assert abs(value['percent'] - 100 * value['covered'] / value['total']) < 0.011
web = matrix['continuation_web_evidence']
verify(web['report'])
assert web['evidence_type'] == 'synthetic_ipc_web_interaction' and web['native_webview_csp_acceptance'] == 'not_verified'
assert len(web['receipts']) == 7
for receipt in web['receipts']:
    verify(receipt)
assert web['checks']['patches'] == 1 and web['checks']['dirty_fields'] == ['model']
assert web['checks']['notifications_preserved'] and web['checks']['unknown_reasoning_value_preserved']
doctor = matrix['test_groups']['doctor_deadline']
doctor_lines = [line for entry in doctor['evidence'] for line in entry['selected_case_lines']]
assert all(any(case['selector'] in line and line.endswith(' ... ok') for line in doctor_lines) for case in doctor['cases'])
assert matrix['continuation_runs']['root_full_ci_before_doctor']['execution']['exit_code'] == 1
final_ci = matrix['continuation_runs']['root_full_ci_after_doctor']
assert final_ci['execution']['exit_code'] == 1
final_ci_text = (ROOT / final_ci['path']).read_text(encoding='utf-8')
failed_binding_excerpt = final_ci_text.split('just tauri-bindings-check')[-1]
assert 'export_bindings' in failed_binding_excerpt and '0xc0000005, STATUS_ACCESS_VIOLATION' in failed_binding_excerpt
assert not re.search(r'^running [0-9]+ tests', failed_binding_excerpt, re.MULTILINE)
assert '407 passed; 0 failed; 1 ignored;' in final_ci_text and '2 passed; 0 failed; 0 ignored;' in final_ci_text
assert final_ci['failure_boundary']['formal_frontend_stage_reached'] is False
assert final_ci['failure_boundary']['later_scoped_success_does_not_replace_aggregate_result'] is True
formal_frontend = matrix['continuation_runs']['formal_frontend_gate']
assert formal_frontend['execution']['exit_code'] == 1
assert formal_frontend['execution']['command'] == ['just', 'frontend-check']
formal_frontend_text = (ROOT / formal_frontend['path']).read_text(encoding='utf-8')
assert sum('Unexpected console statement' in line and 'no-console' in line for line in formal_frontend_text.splitlines()) == 5
assert '5 problems (5 errors, 0 warnings)' in formal_frontend_text
assert formal_frontend['failure_boundary']['independent_of_final_root_ci'] is True
assert formal_frontend['failure_boundary']['scoped_eslint_does_not_replace_formal_gate'] is True
for protected_path in formal_frontend['failure_boundary']['protected_paths']:
    assert Path(protected_path).name in formal_frontend_text
assert any(entry['path'] == formal_frontend['execution']['metadata']['path'] for entry in next(row for row in matrix['requirements'] if row['id'] == 'T10.AC4')['artifact_evidence'])
assert matrix['continuation_runs']['doctor_controlled_red']['execution']['exit_code'] == 101
assert matrix['continuation_runs']['doctor_startup_failure']['execution']['exit_code'] == 5
for key in ('doctor_focused_retry', 'doctor_package', 'doctor_clippy', 'linux_198_doctor'):
    run = matrix['continuation_runs'][key]
    assert run['execution']['exit_code'] == 0
    attribution = run['execution']['source_attribution']
    assert attribution['capture_mode'] == 'command_start_and_finish_source_sha256' and attribution['source_stable_during_command']
linux_doctor = matrix['continuation_runs']['linux_198_doctor']
linux_doctor_meta = json.loads((ROOT / linux_doctor['execution']['metadata']['path']).read_text(encoding='utf-8'))
linux_doctor_lines = (ROOT / linux_doctor['path']).read_text(encoding='utf-8').splitlines()
linux_doctor_selectors = sorted({line[5:-7] for line in linux_doctor_lines if line.startswith('test ') and line.endswith(' ... ok')})
assert len(linux_doctor_selectors) == linux_doctor['reported_counts']['passed_count'] == 3
assert linux_doctor_selectors == sorted(linux_doctor_meta['passed_selectors'])
package_lines = matrix['continuation_runs']['doctor_package']['result_lines']
for expected in ('345 passed; 0 failed;', '12 passed; 0 failed;', '1 passed; 0 failed; 1 ignored;'):
    assert any(expected in line for line in package_lines)
verify(matrix['final_source_freeze'])
current_freeze = json.loads((ROOT / matrix['final_source_freeze']['path']).read_text(encoding='utf-8'))
platform_capture = next(item for item in matrix['superseded_source_freezes'] if item['path'].endswith('/continuation-platform-final-source-freeze.json'))
remaining_capture = next(item for item in [*matrix['superseded_source_freezes'], matrix['final_source_freeze']] if item['path'].endswith('/remaining-source-freeze.json'))
remaining_freeze = json.loads((ROOT / remaining_capture['path']).read_text(encoding='utf-8'))
csp_capture = next(item for item in [*matrix['superseded_source_freezes'], matrix['final_source_freeze']] if item['path'].endswith('/remaining-final-source-freeze.json'))
csp_freeze = json.loads((ROOT / csp_capture['path']).read_text(encoding='utf-8'))
freeze = json.loads((ROOT / platform_capture['path']).read_text(encoding='utf-8'))
freeze_by_sha = {}
for captured in [*matrix['superseded_source_freezes'], matrix['final_source_freeze']]:
    verify(captured)
    captured_data = json.loads((ROOT / captured['path']).read_text(encoding='utf-8'))
    freeze_by_sha[captured['sha256']] = captured_data
    differences = []
    for path, digest in captured_data['files'].items():
        current = ROOT / path
        observed = hashlib.sha256(current.read_bytes()).hexdigest() if current.is_file() else ('non_file_path' if current.exists() else None)
        if observed != digest:
            differences.append({'path': path, 'captured_sha256': digest, 'current_sha256': observed})
    assert captured['current_differences'] == differences
    assert captured['matches_current_sources'] == (not differences)
assert matrix['superseded_source_freezes'][0]['sha256'] != matrix['final_source_freeze']['sha256']
verify(platform_capture['provenance_correction'])
provenance = json.loads((ROOT / platform_capture['provenance_correction']['path']).read_text(encoding='utf-8'))
assert provenance['status'] == 'append_only_provenance_correction_manifest_bytes_unchanged'
assert provenance['frozen_manifest_sha256'] == platform_capture['sha256']
assert provenance['authoritative_parent_source_freeze_sha256'] == freeze['parent_source_freeze_sha256'] == matrix['superseded_source_freezes'][0]['sha256']
parent_freeze = freeze_by_sha[freeze['parent_source_freeze_sha256']]
parent_delta = sorted(path for path in set(parent_freeze['files']) | set(freeze['files']) if parent_freeze['files'].get(path) != freeze['files'].get(path))
assert parent_delta == freeze['changes_from_parent'] == provenance['authoritative_changes_from_parent'] == ['ccr-ui/src-tauri/src/commands/codex_auth.rs']
inherited_previous = json.loads((OUT / provenance['inherited_previous_freeze']).read_text(encoding='utf-8'))
inherited_delta = sorted(path for path in set(inherited_previous['files']) | set(freeze['files']) if inherited_previous['files'].get(path) != freeze['files'].get(path))
assert inherited_delta == provenance['actual_changes_since_inherited_previous']
assert len(inherited_delta) == provenance['actual_changes_count'] == 4
assert len(freeze['changes_since_previous']) == 3
assert len(current_freeze['files']) == matrix['final_source_freeze']['files']
assert all(not (ROOT / path).exists() if digest is None else (ROOT / path).is_file() and hashlib.sha256((ROOT / path).read_bytes()).hexdigest() == digest for path, digest in current_freeze['files'].items())
assert matrix['final_source_freeze']['present_files'] == sum(digest is not None for digest in current_freeze['files'].values())
assert matrix['final_source_freeze']['absent_paths'] == sorted(path for path, digest in current_freeze['files'].items() if digest is None)
assert remaining_capture['files'] == 263 and remaining_capture['present_files'] == 261
assert remaining_freeze['parent_source_freeze_sha256'] == platform_capture['sha256']
assert remaining_freeze['changes_from_parent'] == [
    {'path': path, 'before': freeze['files'][path], 'after': remaining_freeze['files'][path]}
    for path in sorted(freeze['files']) if freeze['files'][path] != remaining_freeze['files'][path]
]
assert [item['path'] for item in remaining_freeze['changes_from_parent']] == ['ccr-ui/.tmp-desktop-probe.mjs', 'ccr-ui/.tmp-insights-visual.mjs', 'crates/ccr-cli/src/commands/codex/fix.rs']
assert csp_freeze['parent_source_freeze_sha256'] == remaining_capture['sha256']
assert len(csp_freeze['files']) == 268 and csp_capture['present_files'] == 266
assert csp_freeze['changes_from_parent'] == [
    {'path': path, 'before': remaining_freeze['files'].get(path), 'after': csp_freeze['files'].get(path), 'newly_tracked_in_snapshot': path not in remaining_freeze['files']}
    for path in sorted(set(remaining_freeze['files']) | set(csp_freeze['files']))
    if remaining_freeze['files'].get(path) != csp_freeze['files'].get(path)
]
permission_capture = next(item for item in [*matrix['superseded_source_freezes'], matrix['final_source_freeze']] if item['path'].endswith('/remaining-permissions-source-freeze.json'))
permission_freeze = json.loads((ROOT / permission_capture['path']).read_text(encoding='utf-8'))
assert permission_freeze['parent_source_freeze_sha256'] == csp_capture['sha256']
assert len(permission_freeze['files']) == 268 and permission_capture['present_files'] == 266
assert permission_freeze['changes_from_parent'] == [
    {'path': path, 'before': csp_freeze['files'][path], 'after': permission_freeze['files'][path]}
    for path in sorted(csp_freeze['files']) if csp_freeze['files'][path] != permission_freeze['files'][path]
]
assert len(permission_freeze['changes_from_parent']) == 6
portability_capture = next(item for item in [*matrix['superseded_source_freezes'], matrix['final_source_freeze']] if item['path'].endswith('/remaining-portability-source-freeze.json'))
portability_freeze = json.loads((ROOT / portability_capture['path']).read_text(encoding='utf-8'))
assert portability_freeze['parent_source_freeze_sha256'] == permission_capture['sha256']
assert len(portability_freeze['files']) == 268 and portability_capture['present_files'] == 266
assert portability_freeze['changes_from_parent'] == [
    {'path': path, 'before': permission_freeze['files'][path], 'after': portability_freeze['files'][path]}
    for path in sorted(permission_freeze['files']) if permission_freeze['files'][path] != portability_freeze['files'][path]
]
assert [item['path'] for item in portability_freeze['changes_from_parent']] == ['crates/ccr/tests/diagnostics_contract.rs']
octal_evidence = matrix['portability_literal_repair']
verify(octal_evidence['report'])
octal = json.loads((ROOT / octal_evidence['report']['path']).read_text(encoding='utf-8'))
before = base64.b64decode(octal['before_bytes_base64'], validate=True)
assert hashlib.sha256(before).hexdigest() == octal['before_sha256'] == permission_freeze['files'][octal['path']]
assert before.count(b'Permissions::from_mode(0)') == 1
assert hashlib.sha256(before.replace(b'Permissions::from_mode(0)', b'Permissions::from_mode(0o0)')).hexdigest() == octal['after_sha256'] == portability_freeze['files'][octal['path']]
assert octal_evidence['semantic_change'] is octal['semantic_change'] is False
assert octal_evidence['before_bytes_verified'] and octal_evidence['single_literal_replacement_verified']
assert octal_evidence['replacement'] == octal['replacement']
platform_test_capture = next(item for item in [*matrix['superseded_source_freezes'], matrix['final_source_freeze']] if item['path'].endswith('/remaining-platform-tests-source-freeze.json'))
platform_test_freeze = json.loads((ROOT / platform_test_capture['path']).read_text(encoding='utf-8'))
assert platform_test_freeze['parent_source_freeze_sha256'] == portability_capture['sha256']
assert len(platform_test_freeze['files']) == 269 and platform_test_capture['present_files'] == 267
assert all(platform_test_freeze['files'][path] == digest for path, digest in portability_freeze['files'].items())
assert platform_test_freeze['changes_from_parent'] == []
additions = platform_test_freeze['scope_additions']
assert [entry['path'] for entry in additions] == ['crates/ccr-store/src/sessions/providers.rs']
assert set(platform_test_freeze['files']) - set(portability_freeze['files']) == {entry['path'] for entry in additions}
for entry in additions:
    assert entry['previously_in_parent_scope'] is False
    assert entry['path'] not in portability_freeze['files']
    assert entry['current_sha256'] == platform_test_freeze['files'][entry['path']]
    assert hashlib.sha256((OUT / entry['prior_source_evidence']).read_bytes()).hexdigest() == entry['prior_source_evidence_sha256']
store = matrix['store_path_portability']
for field in ('before_capture', 'current_source_index', 'implementation_report', 'independent_review', 'diff'):
    verify(store[field])
store_before = json.loads((ROOT / store['before_capture']['path']).read_text(encoding='utf-8'))
original = base64.b64decode(store_before['source_bytes_base64'], validate=True)
assert len(original) == store_before['source_size']
assert hashlib.sha256(original).hexdigest() == store_before['source_sha256'] == store_before['head_sha256']
assert subprocess.run(['git', 'show', store_before['head'] + ':' + store_before['source_path']], cwd=ROOT, capture_output=True, check=True).stdout == original
current = (ROOT / store['current_source_index']['path']).read_bytes()
start = b'    #[test]\n    fn restores_codex_live_jsonl_under_tempfile_with_windows_path_shapes()'
end = b'    #[cfg(windows)]\n    #[test]\n    fn windows_verbatim_prefix_does_not_break_root_containment()'
assert original.count(start) == current.count(start) == original.count(end) == current.count(end) == 1
old_start, old_end = original.index(start), original.index(end)
new_start, new_end = current.index(start), current.index(end)
assert original[:old_start] == current[:new_start] and original[old_end:] == current[new_end:]
old_test, new_test = original[old_start:old_end], current[new_start:new_end]
assert old_test.partition(b'        let backslash = ')[0] == new_test.partition(b'        let backslash = ')[0]
assert b'#[ignore]' not in new_test
assert b'#[cfg(unix)]\n        assert!(!backslash.exists());' in new_test
assert b'#[cfg(windows)]\n        assert!(restored.is_ok());' in new_test
assert b'#[cfg(unix)]\n        assert!(restored.is_err());' in new_test
assert store['single_test_body_changed'] and store['original_and_canonical_path_assertions_unchanged']
assert store['ignored_or_skipped_test_added'] is False
assert store['historical_identity_boundary'] == store_before['historical_identity_boundary']
assert store['current_source_index']['sha256'] == platform_test_freeze['files'][store_before['source_path']]
implementation = json.loads((ROOT / store['implementation_report']['path']).read_text(encoding='utf-8'))
review = json.loads((ROOT / store['independent_review']['path']).read_text(encoding='utf-8'))
assert implementation['before_sha256'] == store_before['source_sha256']
assert implementation['after_sha256'] == review['source_sha256'] == store['current_source_index']['sha256']
assert review['author'] != review['reviewer'] and review['source_edited_by_reviewer'] is False
assert implementation['production_bytes_unchanged'] and review['production_source_unchanged']
assert review['test_contract']['macos_execution'] == 'not_run'
red = matrix['continuation_runs'][store['red_run_ref']]
assert red['execution']['exit_code'] == 101 and red['execution']['source_attribution']['source_stable_between_snapshots']
assert red['execution']['source_attribution']['source_files_sha256'][store_before['source_path']] == store_before['source_sha256']
assert matrix['continuation_runs'][store['earlier_unbound_run_ref']]['failure_boundary']['source_sha256_in_command_map'] is None
for key in store['focused_run_refs']:
    run = matrix['continuation_runs'][key]
    assert run['execution']['exit_code'] == 0
    assert run['execution']['source_attribution']['source_paths'] == 4
    assert run['execution']['source_attribution']['source_stable_between_snapshots']
    assert run['execution']['source_attribution']['source_files_sha256'][store_before['source_path']] == store['current_source_index']['sha256']
    assert 'source_epoch' not in run['execution']
for key, platform in [('remaining_store_path_linux_exact', 'Linux'), ('remaining_store_path_windows_exact', 'Windows')]:
    run = matrix['continuation_runs'][key]
    assert run['execution']['platform'].startswith(platform) and '--exact' in run['execution']['command']
    text = (ROOT / run['path']).read_text(encoding='utf-8')
    assert '1 passed; 0 failed;' in text
    assert 'test sessions::providers::tests::restores_codex_live_jsonl_under_tempfile_with_windows_path_shapes ... ok' in text
assert '81 passed; 0 failed; 2 ignored;' in (ROOT / matrix['continuation_runs']['remaining_store_path_linux_lib']['path']).read_text(encoding='utf-8')
assert store['test_group'] in next(row for row in matrix['requirements'] if row['id'] == 'T10.AC1')['test_groups']
remediation = matrix['post_review_remediation']['DC-02']
assert ledger['post_review_remediation']['DC-02'] == remediation
assert next(finding for finding in ledger['findings'] if finding['finding'] == 'A02')['review_remediation_refs'] == ['DC-02']
assert remediation['finding'] == 'DC-02' and remediation['fix_commit'] is None
assert remediation['requirement_refs'] == ['T01.AC2', 'T01.AC3', 'T05.AC2']
for field in ('fix_report', 'before_snapshot', 'delta', 'independent_review', 'source_freeze', 'original_regression_source'):
    verify(remediation[field])
assert remediation['source_freeze'] == {key: permission_capture[key] for key in ('path', 'sha256', 'bytes')}
fix = json.loads((ROOT / remediation['fix_report']['path']).read_text(encoding='utf-8'))
review = json.loads((ROOT / remediation['independent_review']['path']).read_text(encoding='utf-8'))
before_files = json.loads((ROOT / remediation['before_snapshot']['path']).read_text(encoding='utf-8'))['files']
assert remediation['before_snapshot']['sha256'] == fix['before_snapshot_sha256']
assert remediation['delta']['sha256'] == fix['delta_sha256']
assert len(before_files) == 6 and set(before_files) == set(fix['source_paths']) == set(review['scope'])
for path, entry in before_files.items():
    assert entry['sha256'] == csp_freeze['files'][path]
    assert fix['current_source_sha256'][path] == permission_freeze['files'][path]
expected_text_checks = [
    {'path': path, 'recorded_original_byte_sha256': entry['sha256'], 'stored_text_utf8_sha256': hashlib.sha256(entry['content'].encode('utf-8')).hexdigest(), 'stored_text_preserves_original_bytes': hashlib.sha256(entry['content'].encode('utf-8')).hexdigest() == entry['sha256']}
    for path, entry in before_files.items()
]
assert remediation['before_snapshot_text_checks'] == expected_text_checks
assert [item['path'] for item in expected_text_checks if not item['stored_text_preserves_original_bytes']] == ['.trellis/spec/ccr-core/backend/atomic-writer.md']
assert remediation['before_snapshot_text_boundary']
assert remediation['author'] == fix['author']
assert remediation['reviewer'] == review['reviewer'] and 'non-author' in review['reviewer']
assert remediation['review_status'] == review['status']
assert review['source_freeze_sha256'] == permission_capture['sha256']
assert remediation['journal_scope'] == fix['journal_scope']
assert 'Existing content entries still restore their recorded metadata' in remediation['journal_scope']
for group_name in remediation['test_groups']:
    group = matrix['test_groups'][group_name]
    for case in group['cases']:
        assert any(case['selector'] in line and line.endswith(' ... ok') for entry in group['evidence'] for line in entry['selected_case_lines']), (group_name, case)
for requirement_id in remediation['requirement_refs']:
    row = next(row for row in matrix['requirements'] if row['id'] == requirement_id)
    assert row['review_remediation_refs'] == ['DC-02']
    assert set(row['test_groups']).intersection(remediation['test_groups'])
permission_red = matrix['continuation_runs'][remediation['red_run_ref']]
assert permission_red['execution']['exit_code'] == 101
assert permission_red['execution']['source_attribution']['source_stable_between_snapshots']
assert permission_red['execution']['source_epoch']['manifest']['sha256'] == csp_capture['sha256']
base_path = remediation['original_regression_source']['path']
assert remediation['original_regression_source']['unchanged_from_red_run']
assert csp_freeze['files'][base_path] == permission_freeze['files'][base_path] == remediation['original_regression_source']['sha256']
for key in remediation['focused_run_refs']:
    run = matrix['continuation_runs'][key]
    assert run['execution']['exit_code'] == 0
    assert run['execution']['source_attribution']['source_stable_between_snapshots']
    assert run['execution']['source_epoch']['manifest']['sha256'] == permission_capture['sha256']
    assert run['execution']['source_epoch']['source_after_equals_epoch']
    assert run['execution']['generated_map_comparison']['maps_equal']
for key, passed, ignored in [('remaining_permissions_config', 104, 1), ('remaining_permissions_guarded', 27, 0), ('remaining_permissions_atomic', 9, 0)]:
    run = matrix['continuation_runs'][key]
    text = (ROOT / run['path']).read_text(encoding='utf-8')
    assert f'test result: ok. {passed} passed; 0 failed; {ignored} ignored;' in text
if 'remaining_linux_workspace_quality_after_python' in matrix['continuation_runs']:
    initial = matrix['continuation_runs']['remaining_linux_workspace_quality_after_permissions']
    retry = matrix['continuation_runs']['remaining_linux_workspace_quality_after_python']
    assert initial['execution']['exit_code'] == 127
    assert 'python: command not found' in (ROOT / initial['path']).read_text(encoding='utf-8')
    assert initial['failure_boundary']['rust_clippy_reached'] is False
    verify(retry['environment_preparation'])
    launcher = json.loads((ROOT / retry['environment_preparation']['path']).read_text(encoding='utf-8'))
    assert launcher['exit_code'] == 0
    assert retry['execution']['exit_code'] == 101
    assert retry['execution']['command'] == initial['execution']['command']
    assert 'non_octal_unix_permissions' in (ROOT / retry['path']).read_text(encoding='utf-8')
    assert retry['failure_boundary']['source'] == 'crates/ccr/tests/diagnostics_contract.rs'
    for run in (initial, retry):
        assert run['execution']['source_epoch']['manifest']['sha256'] == permission_capture['sha256']
        assert run['execution']['source_attribution']['source_stable_between_snapshots']
        assert run['execution']['generated_map_comparison']['maps_equal']
if 'remaining_linux_workspace_quality_after_octal' in matrix['continuation_runs']:
    run = matrix['continuation_runs']['remaining_linux_workspace_quality_after_octal']
    assert run['execution']['exit_code'] == 101
    assert run['execution']['source_epoch']['manifest']['sha256'] == portability_capture['sha256']
    text = (ROOT / run['path']).read_text(encoding='utf-8')
    assert run['failure_boundary']['selector'] in text and '80 passed; 1 failed; 2 ignored;' in text
    assert run['failure_boundary']['cause'] == 'undetermined_by_this_run'
    assert run['failure_boundary']['source_sha256_in_command_map'] == run['execution']['source_attribution']['source_files_sha256'].get(run['failure_boundary']['source'])
for key in ('linux_195_workspace_msrv', 'windows_195_tauri_msrv', 'ui_production_build'):
    run = matrix['continuation_runs'][key]
    assert run['execution']['exit_code'] == 0
    frozen = run['execution']['frozen_source_evidence']
    assert frozen['manifest']['sha256'] in freeze_by_sha
    assert frozen['source_changes_since_freeze'] == []
    assert datetime.fromisoformat(run['execution']['started_at']) >= datetime.fromisoformat(freeze_by_sha[frozen['manifest']['sha256']]['captured_at'])
msrv = matrix['continuation_runs']['linux_195_workspace_msrv']
assert msrv['execution']['rustc_version'].startswith('rustc 1.95.0 ')
assert all(arg in msrv['execution']['command'] for arg in ('check', '--locked', '--workspace', '--all-targets', '--all-features'))
tauri_msrv = matrix['continuation_runs']['windows_195_tauri_msrv']
assert tauri_msrv['execution']['rustc_version'].startswith('rustc 1.95.0 ')
assert all(arg in tauri_msrv['execution']['command'] for arg in ('+1.95', 'check', '--manifest-path', 'ccr-ui/src-tauri/Cargo.toml', '--locked', '--all-targets', '--all-features'))
assert tauri_msrv['execution']['environment_overrides'] == {'CCR_SKIP_ICON_GENERATION': '1'}
tauri_msrv_meta = json.loads((ROOT / tauri_msrv['execution']['metadata']['path']).read_text(encoding='utf-8'))
assert tauri_msrv_meta['source_before_matches_freeze'] is True and tauri_msrv_meta['source_path_count'] == 263
assert tauri_msrv_meta['generated_types_unchanged'] is True
assert tauri_msrv_meta['generated_before_count'] == tauri_msrv_meta['generated_after_count'] == 230
ui_build = matrix['continuation_runs']['ui_production_build']
assert ui_build['execution']['capture_wrapper']['capture_wrapper_exit_code'] == 1
assert 'GBK' in ui_build['execution']['capture_wrapper']['capture_wrapper_error']
assert 'built in 2.00s' in (ROOT / ui_build['path']).read_text(encoding='utf-8')
restoration_path = OUT / 'continuation-bindings-after-ci.json'
restoration = json.loads(restoration_path.read_text(encoding='utf-8'))
assert restoration['current_count'] == restoration['expected_count'] == 230
assert all(restoration[key] == [] for key in ('changed', 'missing', 'added', 'source_freeze_changes'))
for key, expected in (('linux_198_tauri_process_before_import_fix', 8), ('linux_198_core_process_before_import_fix', 9)):
    run = matrix['continuation_runs'][key]
    assert run['execution']['exit_code'] == 0
    text = (ROOT / run['path']).read_text(encoding='utf-8')
    selectors = sorted({line[5:-7] for line in text.splitlines() if line.startswith('test ') and line.endswith(' ... ok')})
    assert selectors == run['observed_unique_passed_selectors'] and len(selectors) == expected
    source_map = run['execution']['source_attribution']
    assert source_map['capture_mode'] == 'command_before_registered_source_map_with_post_comparison'
    assert source_map['post_command_source_changes'] == []
    assert run['execution']['frozen_source_evidence']['manifest']['sha256'] == matrix['superseded_source_freezes'][0]['sha256']
linux_before_import = matrix['continuation_runs']['linux_198_tauri_process_before_import_fix']
assert 'warning: unused import:' in (ROOT / linux_before_import['path']).read_text(encoding='utf-8')
import_fix = json.loads((OUT / 'continuation-linux-import-fix.json').read_text(encoding='utf-8'))
assert import_fix['original_test_run_exit_code'] == 0 and 'raw_warning_evidence' in import_fix
assert import_fix['changed_lines'] == 1 and import_fix['production_behavior_changed'] is False
assert parent_freeze['files'][import_fix['source']] == import_fix['before_sha256']
assert freeze['files'][import_fix['source']] == import_fix['after_sha256']
linux_smoke_summary = json.loads((OUT / 'continuation-linux-process-smoke-summary.json').read_text(encoding='utf-8'))
assert all(entry['exit_code'] == 0 and entry['source_changes'] == [] for entry in linux_smoke_summary['checks'])
assert linux_smoke_summary['final_source_changes'] == []
schema_backup = ROOT / linux_smoke_summary['schema_backup']
assert hashlib.sha256(schema_backup.read_bytes()).hexdigest() == linux_smoke_summary['schema_backup_sha256']
for changed in linux_smoke_summary['generated_schema_changes']:
    assert changed['before_sha256'] is None and changed['action'] == 'remove_new_build_generated_linux_schema'
    assert changed['path'] == 'ccr-ui/src-tauri/gen/schemas/linux-schema.json'
    # Historical cleanup is retained in its receipt; later Linux builds may regenerate this path.
startup_observation = json.loads((OUT / 'continuation-bindings-startup-before.json').read_text(encoding='utf-8'))
assert startup_observation['capture_mode'] == 'post_failure_observation_not_failure_time_hash'
assert startup_observation['ci_log_sha256'] == final_ci['sha256']
assert startup_observation['ci_metadata_sha256'] == final_ci['execution']['metadata']['sha256']
startup_list = json.loads((OUT / 'continuation-bindings-startup-list.json').read_text(encoding='utf-8'))
assert startup_list['exit_code'] == 0 and startup_list['capture_mode'] == 'post_failure_observation'
assert startup_list['binary_before_sha256'] == startup_list['binary_after_sha256'] == startup_observation['executable_sha256']
assert sum(line.endswith(': test') for line in startup_list['stdout'].splitlines()) == 24
for key in ('bindings_followup', 'standalone_tauri_ci'):
    run = matrix['continuation_runs'][key]
    assert run['execution']['exit_code'] == 0
    meta = json.loads((ROOT / run['execution']['metadata']['path']).read_text(encoding='utf-8'))
    snapshots = run['execution']['command_snapshots']
    before = json.loads((ROOT / snapshots['before']['path']).read_text(encoding='utf-8'))
    after = json.loads((ROOT / snapshots['after']['path']).read_text(encoding='utf-8'))
    assert meta['source_freeze_sha256'] in freeze_by_sha
    assert before['freeze_files'] == after['freeze_files'] == freeze_by_sha[meta['source_freeze_sha256']]['files']
    assert before['generated_types'] == after['generated_types'] and len(before['generated_types']) == 230
    assert not before['freeze_mismatches'] and not after['freeze_mismatches']
    assert snapshots['source_paths'] == 263 and snapshots['generated_files'] == 230
    assert datetime.fromisoformat(before['captured_at_utc']) <= datetime.fromisoformat(run['execution']['started_at']) <= datetime.fromisoformat(after['captured_at_utc']) <= datetime.fromisoformat(run['execution']['completed_at'])
    assert all((ROOT / path).is_file() and hashlib.sha256((ROOT / path).read_bytes()).hexdigest() == digest for path, digest in after['generated_types'].items())
    assert meta['binary_before_sha256'] == meta['binary_after_sha256'] == startup_observation['executable_sha256']
    assert meta['source_freeze_sha256'] == matrix['superseded_source_freezes'][0]['sha256']
    for count in (24, 9, 197):
        assert any(f'{count} passed; 0 failed; 0 ignored;' in line for line in run['result_lines'])
standalone_tauri = matrix['continuation_runs']['standalone_tauri_ci']
for expected in ('407 passed; 0 failed; 1 ignored;', '2 passed; 0 failed; 0 ignored;', '1 passed; 0 failed; 0 ignored;'):
    assert any(expected in line for line in standalone_tauri['result_lines'])
assert matrix['continuation_runs']['root_full_ci_after_doctor']['execution']['exit_code'] == 1
final_run_names = ('linux_198_tauri_strict_final', 'linux_oauth_zero_match', 'linux_198_tauri_process_final', 'windows_195_tauri_msrv_final', 'windows_oauth_zero_match', 'tauri_fmt_final', 'linux_oauth_corrected', 'windows_oauth_corrected')
for key in final_run_names:
    run = matrix['continuation_runs'][key]
    meta = json.loads((ROOT / run['execution']['metadata']['path']).read_text(encoding='utf-8'))
    assert run['execution']['exit_code'] == 0 and meta['source_changes'] == []
    assert meta['source_freeze_sha256'] == platform_capture['sha256']
    assert meta['source_before_sha256'] == freeze['files']
    assert meta['generated_types_unchanged'] is True
    assert run['execution']['environment_overrides'] == {'CCR_SKIP_ICON_GENERATION': '1'}
    assert datetime.fromisoformat(run['execution']['started_at']) >= datetime.fromisoformat(freeze['captured_at'])
for key, expected in {'linux_oauth_zero_match': (0, 0), 'windows_oauth_zero_match': (0, 0), 'linux_198_tauri_process_final': (8, 5), 'linux_oauth_corrected': (20, 18), 'windows_oauth_corrected': (1, 1)}.items():
    run = matrix['continuation_runs'][key]
    text = (ROOT / run['path']).read_text(encoding='utf-8')
    aggregate = sum(int(value) for value in re.findall(r'test result: ok\. (\d+) passed;', text))
    selectors = sorted({line[5:-7] for line in text.splitlines() if line.startswith('test ') and line.endswith(' ... ok')})
    assert (aggregate, len(selectors)) == expected
    assert selectors == run['observed_unique_passed_selectors']
    assert run['observed_test_counts'] == {'aggregate_passed': aggregate, 'intact_named_selector_count': len(selectors), 'named_capture_complete': aggregate == len(selectors), 'overlaps_other_runs': True}
    if aggregate != len(selectors):
        assert run['named_capture_limit']
    if aggregate == 0:
        assert run['accepted_as_behavior_evidence'] is False and run['acceptance_role'] == 'not_behavior_evidence_zero_matching_tests'
        assert not any(entry['path'] == run['path'] for group in matrix['test_groups'].values() for entry in group['evidence'])
    if key.endswith('_corrected'):
        meta = json.loads((ROOT / run['execution']['metadata']['path']).read_text(encoding='utf-8'))
        assert sorted(meta['passed_selectors']) == selectors and run['accepted_as_behavior_evidence'] is True
        assert all(name.startswith('commands::codex::auth::') for name in selectors)
platform_summary = json.loads((OUT / 'continuation-platform-final-checks.json').read_text(encoding='utf-8'))
selector_summary = json.loads((OUT / 'continuation-oauth-selector-correction.json').read_text(encoding='utf-8'))
for summary in (platform_summary, selector_summary):
    assert summary['source_freeze_sha256'] == platform_capture['sha256']
    assert summary['final_source_changes'] == [] and all(entry['exit_code'] == 0 for entry in summary['checks'])
assert selector_summary['invalid_selector_runs'] == ['continuation-linux-tauri-oauth-final.json', 'continuation-windows-oauth-after-import.json']
verify(matrix['historical_mapping_baseline'])
baseline = json.loads((ROOT / matrix['historical_mapping_baseline']['path']).read_text(encoding='utf-8'))
assert baseline['prior_run_count'] == len(baseline['prior_run_fingerprints']) == 41
assert baseline['prior_verified_artifact_count'] == 227
for key, expected in baseline['prior_run_fingerprints'].items():
    captured = json.dumps(matrix['continuation_runs'][key], ensure_ascii=False, sort_keys=True, separators=(',', ':')).encode('utf-8')
    assert hashlib.sha256(captured).hexdigest() == expected, ('historical run changed', key)
remaining_frontend = matrix['continuation_runs']['remaining_windows_frontend']
assert remaining_frontend['execution']['command'] == ['just', 'frontend-check']
assert remaining_frontend['execution']['exit_code'] == 0
frontend_text = re.sub(r'\x1b\[[0-9;]*m', '', (ROOT / remaining_frontend['path']).read_text(encoding='utf-8'))
assert re.search(r'Test Files\s+168 passed \(168\)', frontend_text)
assert re.search(r'Tests\s+901 passed \(901\)', frontend_text)
assert re.search(r'Passed:\s+24', frontend_text)
remaining_ci = matrix['continuation_runs']['remaining_windows_ci_before_fixture']
assert remaining_ci['execution']['command'] == ['just', 'ci'] and remaining_ci['execution']['exit_code'] == 1
remaining_ci_text = (ROOT / remaining_ci['path']).read_text(encoding='utf-8')
assert '343 passed; 2 failed;' in remaining_ci_text and 'program not found' in remaining_ci_text
assert '0xc0000005' not in remaining_ci_text
assert remaining_ci['failure_boundary']['historical_exporter_av_resolution_proven'] is False
assert matrix['continuation_runs']['remaining_doctor_empty_path_red']['execution']['exit_code'] == 101
for key, expected in (('remaining_doctor_focused', 3), ('remaining_doctor_library', 345)):
    run = matrix['continuation_runs'][key]
    assert run['execution']['exit_code'] == 0
    assert f'{expected} passed; 0 failed;' in (ROOT / run['path']).read_text(encoding='utf-8')
    assert run['execution']['source_attribution']['source_files_sha256']['crates/ccr-cli/src/commands/codex/fix.rs'] == current_freeze['files']['crates/ccr-cli/src/commands/codex/fix.rs']
assert matrix['continuation_runs']['remaining_doctor_clippy']['execution']['exit_code'] == 0
version_failure = matrix['continuation_runs']['remaining_windows_ci_after_fixture']
assert version_failure['execution']['exit_code'] == 1
version_failure_text = (ROOT / version_failure['path']).read_text(encoding='utf-8')
assert '406 passed; 1 failed; 1 ignored;' in version_failure_text
assert 'commands::system::tests::cli_versions_fast_mode_returns_expected_shape ... FAILED' in version_failure_text
for key in ('remaining_cli_versions_diagnostic', 'remaining_cli_versions_focused'):
    run = matrix['continuation_runs'][key]
    assert run['execution']['exit_code'] == 0
    assert '1 passed; 0 failed;' in (ROOT / run['path']).read_text(encoding='utf-8')
for key, run in matrix['continuation_runs'].items():
    if not key.startswith('remaining_'):
        continue
    lines = (ROOT / run['path']).read_text(encoding='utf-8', errors='replace').splitlines()
    expected = [
        {'status': match[1], 'passed': int(match[2]), 'failed': int(match[3]), 'ignored': int(match[4]), 'log_line': number, 'raw': line.strip()}
        for number, line in enumerate(lines, 1)
        if (match := re.search(r'test result: (ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored;', line))
    ]
    assert run['observed_rust_result_partitions'] == expected
linux_tauri = matrix['continuation_runs']['remaining_linux_tauri_ci']
assert linux_tauri['execution']['command'] == ['just', 'tauri-ci'] and linux_tauri['execution']['exit_code'] == 0
linux_text = (ROOT / linux_tauri['path']).read_text(encoding='utf-8')
partitions = re.findall(r'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored;', linux_text)
assert partitions == [('397', '0', '1'), ('2', '0', '0'), ('24', '0', '0'), ('0', '0', '0'), ('9', '0', '0'), ('197', '0', '0'), ('0', '0', '0'), ('1', '0', '0'), ('0', '0', '0')]
assert linux_tauri['test_partitions']['tauri_behavior_passed'] == 397
assert linux_tauri['test_partitions']['inventory_overlaps_behavior_suite'] is True
for key in ('remaining_linux_tauri_ci', 'remaining_linux_coverage', 'remaining_linux_native_build'):
    run = matrix['continuation_runs'][key]
    assert run['execution']['source_epoch']['manifest']['sha256'] == remaining_capture['sha256']
    assert run['execution']['source_epoch']['source_after_equals_epoch']
    assert run['execution']['generated_map_comparison']['maps_equal']
    assert run['execution']['parent_freeze_reference']['manifest']['sha256'] == platform_capture['sha256']
coverage_run = matrix['continuation_runs']['remaining_linux_coverage']
assert coverage_run['execution']['command'] == ['just', 'coverage-tauri'] and coverage_run['execution']['exit_code'] == 0
coverage = coverage_run['coverage']
verify(coverage['report'])
report = json.loads((ROOT / coverage['report']['path']).read_text(encoding='utf-8'))['data'][0]
gateway = [entry for entry in report['files'] if entry['filename'] == coverage['gateway_path']]
assert len(gateway) == 1
assert coverage['gateway_lines'] == gateway[0]['summary']['lines']
assert coverage['overall_lines'] == report['totals']['lines']
assert (coverage['gateway_lines']['covered'], coverage['gateway_lines']['count']) == (654, 703)
assert (coverage['overall_lines']['covered'], coverage['overall_lines']['count']) == (17929, 34415)
for name in ('gateway_lines', 'overall_lines'):
    metric = coverage[name]
    assert abs(metric['percent'] - 100 * metric['covered'] / metric['count']) < 1e-10
assert coverage['gateway_threshold_percent'] == 85 and coverage['overall_threshold_percent'] is None
assert coverage['gateway_lines']['percent'] >= coverage['gateway_threshold_percent']
final_coverage_run = matrix['continuation_runs']['remaining_linux_coverage_final']
assert final_coverage_run['execution']['exit_code'] == 0
final_coverage = final_coverage_run['coverage']
verify(final_coverage['report'])
final_report = json.loads((ROOT / final_coverage['report']['path']).read_text(encoding='utf-8'))['data'][0]
assert final_coverage['overall_lines'] == final_report['totals']['lines']
assert final_coverage['gateway_lines'] == next(item['summary']['lines'] for item in final_report['files'] if item['filename'] == final_coverage['gateway_path'])
assert (final_coverage['gateway_lines']['covered'], final_coverage['gateway_lines']['count']) == (654, 703)
assert (final_coverage['overall_lines']['covered'], final_coverage['overall_lines']['count']) == (17964, 34456)
assert final_coverage['overall_threshold_percent'] is None and final_coverage['gateway_threshold_percent'] == 85
if 'remaining_linux_coverage_after_path' in matrix['continuation_runs']:
    tauri_run = matrix['continuation_runs']['remaining_linux_coverage_after_path']
    assert tauri_run['execution']['command'] == ['just', 'coverage-tauri']
    assert tauri_run['execution']['exit_code'] == 0
    assert tauri_run['execution']['source_epoch']['manifest']['sha256'] == platform_test_capture['sha256']
    meta = json.loads((ROOT / tauri_run['execution']['metadata']['path']).read_text(encoding='utf-8'))
    coverage = tauri_run['coverage']
    verify(coverage['report'])
    verify(coverage['copy_receipt'])
    copy = json.loads((ROOT / coverage['copy_receipt']['path']).read_text(encoding='utf-8'))
    assert coverage['report']['sha256'] == coverage['reported_sha256'] == meta['coverage_report_sha256'] == copy['sha256']
    assert coverage['report']['path'].endswith('/' + copy['copy'])
    assert tauri_run['execution']['metadata']['path'].endswith('/' + copy['run'])
    assert datetime.fromisoformat(copy['copied_at_utc']) >= datetime.fromisoformat(meta['finished_at_utc'])
    data = json.loads((ROOT / coverage['report']['path']).read_text(encoding='utf-8'))['data']
    assert len(data) == 1
    assert coverage['overall_lines'] == copy['overall_lines'] == data[0]['totals']['lines']
    assert coverage['gateway_lines'] == copy['gateway_lines'] == next(item['summary']['lines'] for item in data[0]['files'] if item['filename'] == coverage['gateway_path'])
    assert coverage['gateway_path'].replace(chr(92), '/').endswith('ccr-ui/src-tauri/src/process/gateway.rs')
    assert (coverage['overall_lines']['covered'], coverage['overall_lines']['count']) == (17964, 34456)
    assert (coverage['gateway_lines']['covered'], coverage['gateway_lines']['count']) == (654, 703)
    assert coverage['status'] == 'measured_gateway_threshold_passed'
    assert coverage['gateway_threshold_percent'] == copy['required_gateway_threshold'] == 85
    assert coverage['overall_threshold_percent'] is None and copy['overall_threshold_not_applied_by_this_recipe']
    assert coverage['gateway_threshold_evaluated'] is True and coverage['overall_threshold_evaluated'] is False
    for name in ('gateway', 'overall'):
        metric = coverage[name + '_lines']
        assert abs(metric['percent'] - 100 * metric['covered'] / metric['count']) < 1e-10
    assert coverage['gateway_lines']['percent'] >= 85
    text = (ROOT / tauri_run['path']).read_text(encoding='utf-8')
    assert f"Overall line coverage: {coverage['overall_lines']['percent']:.2f}%" in text
    assert f"Gateway line coverage: {coverage['gateway_lines']['percent']:.2f}% {coverage['gateway_path']}" in text
if 'remaining_linux_workspace_coverage_after_path' in matrix['continuation_runs']:
    workspace_run = matrix['continuation_runs']['remaining_linux_workspace_coverage_after_path']
    assert workspace_run['execution']['command'] == ['just', 'coverage-rust']
    assert workspace_run['execution']['exit_code'] == 0
    assert workspace_run['execution']['source_epoch']['manifest']['sha256'] == platform_test_capture['sha256']
    meta = json.loads((ROOT / workspace_run['execution']['metadata']['path']).read_text(encoding='utf-8'))
    coverage = workspace_run['coverage']
    verify(coverage['report'])
    assert coverage['report']['path'].endswith('/' + meta['coverage_report_copy'])
    assert coverage['report']['sha256'] == coverage['reported_sha256'] == meta['coverage_report_sha256']
    data = json.loads((ROOT / coverage['report']['path']).read_text(encoding='utf-8'))['data']
    assert len(data) == 1
    assert coverage['overall_lines'] == data[0]['totals']['lines']
    assert coverage['gateway_lines'] == next(item['summary']['lines'] for item in data[0]['files'] if item['filename'] == coverage['gateway_path'])
    assert coverage['gateway_path'].replace(chr(92), '/').endswith('crates/ccr-core/src/core/process_gateway.rs')
    assert (coverage['overall_lines']['covered'], coverage['overall_lines']['count']) == (74554, 99931)
    assert (coverage['gateway_lines']['covered'], coverage['gateway_lines']['count']) == (393, 409)
    assert coverage['status'] == 'measured_thresholds_passed' and coverage['thresholds_evaluated']
    assert (coverage['overall_threshold_percent'], coverage['gateway_threshold_percent']) == (70, 85)
    for name in ('gateway', 'overall'):
        metric = coverage[name + '_lines']
        assert abs(metric['percent'] - 100 * metric['covered'] / metric['count']) < 1e-10
        assert metric['percent'] >= coverage[name + '_threshold_percent']
    text = (ROOT / workspace_run['path']).read_text(encoding='utf-8')
    assert f"Overall line coverage: {coverage['overall_lines']['percent']:.2f}%" in text
    assert f"Gateway line coverage: {coverage['gateway_lines']['percent']:.2f}% {coverage['gateway_path']}" in text
if 'remaining_linux_workspace_coverage_final' in matrix['continuation_runs']:
    failed_coverage = matrix['continuation_runs']['remaining_linux_workspace_coverage_final']
    assert failed_coverage['execution']['command'] == ['just', 'coverage-rust']
    assert failed_coverage['execution']['exit_code'] == 101
    metadata = json.loads((ROOT / failed_coverage['execution']['metadata']['path']).read_text(encoding='utf-8'))
    assert metadata['coverage_report_sha256'] is None
    coverage = failed_coverage['coverage']
    assert coverage['status'] == 'not_measured_behavior_test_failed'
    assert coverage['report'] is None and coverage['reported_sha256'] is None
    assert coverage['thresholds_evaluated'] is False
    assert (coverage['overall_threshold_percent'], coverage['gateway_threshold_percent']) == (70, 85)
    text = (ROOT / failed_coverage['path']).read_text(encoding='utf-8')
    assert 'platforms::base::tests::test_profile_structured_writes_keep_owner_only_permissions ... FAILED' in text
    assert '97 passed; 1 failed; 1 ignored;' in text
    assert 'left: 420' in text and 'right: 384' in text
    boundary = failed_coverage['failure_boundary']
    assert boundary['coverage_report_produced'] is False
    assert (boundary['observed_mode_decimal'], boundary['expected_mode_decimal']) == (420, 384)
    assert (boundary['observed_mode_octal'], boundary['expected_mode_octal']) == ('0644', '0600')
    attribution = failed_coverage['execution']['source_attribution']
    assert len(attribution['post_command_source_changes']) == 7
    assert attribution['source_stable_between_snapshots'] is False
    assert failed_coverage['execution']['source_epoch']['source_before_equals_epoch']
    assert failed_coverage['execution']['source_epoch']['source_after_equals_epoch'] is False
    generated = failed_coverage['execution']['generated_map_comparison']
    assert generated['before_count'] == 230 and generated['after_count'] == 33
    assert generated['maps_equal'] is False and generated['changes']
parity_evidence = matrix['remaining_generated_parity_observation']
for field in ('report', 'windows_run', 'failed_linux_run'):
    verify(parity_evidence[field])
parity = json.loads((ROOT / parity_evidence['report']['path']).read_text(encoding='utf-8'))
windows_run = matrix['continuation_runs']['remaining_windows_ci_final']['execution']
failed_run = matrix['continuation_runs']['remaining_linux_workspace_coverage_final']['execution']
assert windows_run['source_epoch']['manifest']['sha256'] == csp_capture['sha256']
assert windows_run['source_epoch']['source_after_equals_epoch']
assert parity_evidence['windows_run'] == windows_run['metadata']
assert parity_evidence['failed_linux_run'] == failed_run['metadata']
assert parity['windows_run_sha256'] == windows_run['metadata']['sha256']
assert parity['windows_exit_code'] == windows_run['exit_code'] == 0
before = json.loads((ROOT / windows_run['generated_comparison']['before_snapshot']['path']).read_text(encoding='utf-8'))
assert parity['current_hashes'] == {path: entry['sha256'] for path, entry in before.items()}
assert parity_evidence['generated_count'] == parity['generated_count'] == len(parity['current_hashes']) == 230
assert parity_evidence['captured_at'] == parity['captured_at_utc']
assert parity['failed_linux_generated_after_count'] == failed_run['generated_map_comparison']['after_count'] == 33
assert parity['failed_linux_source_changes_preserved'] == [entry['path'] for entry in failed_run['source_attribution']['post_command_source_changes']]
for key in ('remaining_linux_coverage_final', 'remaining_linux_tauri_ci_final', 'remaining_linux_native_after_csp'):
    run = matrix['continuation_runs'][key]
    assert run['execution']['source_epoch']['manifest']['sha256'] == csp_capture['sha256']
    assert run['execution']['source_epoch']['source_after_equals_epoch']
    assert run['execution']['generated_map_comparison']['maps_equal']
    assert run['execution']['exit_code'] == 0
assert matrix['continuation_runs']['remaining_linux_native_build']['scope_boundary'].startswith('Binary build')
native = matrix['remaining_native_evidence']
assert native['evidence_type'] == 'native_linux_webkitgtk_custom_protocol_debug_binary'
assert native['status'] == 'passed_scoped_native_linux_final_source'
verify(native['acceptance_report'])
for entry in native['verified_artifacts']:
    verify(entry)
assert len(native['prior_acceptance_reports']) == 1
for report in native['prior_acceptance_reports']:
    verify(report['report'])
    prior = json.loads((ROOT / report['report']['path']).read_text(encoding='utf-8'))
    assert report['status'] == prior['status'] == 'passed_scoped_native_linux'
    for entry in report['verified_artifacts']:
        verify(entry)
        assert entry['sha256'] == prior['artifacts'][Path(entry['path']).name]
verify(native['style_evidence_correction'])
correction = json.loads((ROOT / native['style_evidence_correction']['path']).read_text(encoding='utf-8'))
assert correction['historical_derived_report_sha256'] == native['prior_acceptance_reports'][0]['report']['sha256']
assert correction['current_report_sha256'] == native['acceptance_report']['sha256']
assert native['style_unmeasured'] == correction['unmeasured'] == ['style DOM removal', 'body scroll-lock restoration']
assert correction['raw_receipts_unchanged'] == ['remaining-native-webview-6.json', 'remaining-native-webview-7.json']
final_native_report = json.loads((ROOT / native['acceptance_report']['path']).read_text(encoding='utf-8'))
assert final_native_report['status'] == native['status']
for entry in native['verified_artifacts']:
    assert entry['sha256'] == final_native_report['artifacts'][Path(entry['path']).name]
observation = native['final_acceptance_observation']
assert observation == {field: final_native_report[field] for field in observation}
assert observation['post_save_style_csp_violations_empty'] is True
assert 'nonce_style_cleanup' not in observation and 'nonce_style_cleanup' not in final_native_report
assert final_native_report['post_save_style_csp_violations_empty'] is True
assert any('were not measured' in value for value in observation['limits'])
assert native['passed_attempts'] == [item['report']['path'] for item in native['attempts'] if item['state'] == 'passed']
for attempt in native['attempts']:
    verify(attempt['report'])
    verify(attempt['build_metadata'])
    if 'log' in attempt:
        verify(attempt['log'])
    receipt = json.loads((ROOT / attempt['report']['path']).read_text(encoding='utf-8'))
    build = json.loads((ROOT / attempt['build_metadata']['path']).read_text(encoding='utf-8'))
    assert attempt['state'] == receipt['state'] and receipt['finished_at_utc']
    assert attempt['checks'] == receipt['checks']
    assert attempt['post_save_style_csp_violations'] == receipt.get('styles_after_save')
    assert attempt['post_save_style_capture_field'] == 'styles_after_save'
    assert attempt['post_save_style_unmeasured'] == native['style_unmeasured']
    assert attempt['binary_sha256'] == receipt['binary_sha256'] == build['binary_sha256']
    if receipt.get('build_record_sha256'):
        assert receipt['build_record_sha256'] == attempt['build_metadata']['sha256']
    assert build['state'] == 'finished' and build['exit_code'] == 0
    epoch = attempt['source_epoch']
    verify(epoch['manifest'])
    frozen = json.loads((ROOT / epoch['manifest']['path']).read_text(encoding='utf-8'))['files']
    assert epoch['build_start_matches_epoch'] and build['sources_before'] == frozen
    assert epoch['build_finish_matches_epoch'] == (build['sources_after'] == frozen)
    if attempt['state'] == 'passed':
        checks = receipt['checks']
        assert checks['native_boot']['native'] and checks['native_boot']['url'].startswith('tauri://localhost')
        assert checks['plaintext_confirmation'] == {'visible': True, 'editorCount': 0}
        editor = checks['native_editor']
        assert editor['editorHeight'] > 0 and editor['tokenSpans'] > 0 and editor['noncedStyles'] > 0
        assert editor['editorDisplay'] == 'flex' and editor['editorFontSize'] == '13px'
        assert editor['styleCspViolations'] == []
        assert checks['production_csp']['inlineExecuted'] is False
        assert any(event['directive'].startswith('script-src') for event in checks['production_csp']['events'])
        assert checks['native_save_roundtrip']['actual'] == checks['native_save_roundtrip']['expected']
        assert receipt['live_process_group_members_after_cleanup'] == []
        assert receipt['environment']['HOME'].startswith(receipt['fixture_root'] + '/')
        scroll = checks['native_scroll_lock']
        assert scroll['locked'] and scroll['sheetReadable'] and scroll['rules'] > 0
        assert scroll['nonce'] == scroll['pageNonce'] and scroll['bodyOverflow'] == 'hidden'
        style = receipt['editor_runtime_style']
        assert style['nonce'] == style['pageNonce'] and style['rules'] > 0 and style['sheetReadable']
        assert abs(style['contentTop'] - style['gutterTop']) < 1
        # The raw field contains CSP events. DOM style removal and overflow restoration were not captured.
        assert receipt['styles_after_save'] == []
final_attempt = next(item for item in native['attempts'] if item['report']['path'].endswith('/remaining-native-webview-7.json'))
final_receipt = json.loads((ROOT / final_attempt['report']['path']).read_text(encoding='utf-8'))
final_build = json.loads((ROOT / final_attempt['build_metadata']['path']).read_text(encoding='utf-8'))
ui_build = json.loads((OUT / 'remaining-csp-ui-build.json').read_text(encoding='utf-8'))
assert final_attempt['state'] == 'passed'
assert final_attempt['source_epoch']['manifest']['sha256'] == platform_test_capture['sha256']
assert observation['source_paths'] == len(final_build['sources_before']) == len(final_build['sources_after']) == 269
assert observation['frontend_input_paths'] == len(ui_build['sources_before']) == len(ui_build['sources_after']) == 1060
assert ui_build['sources_before'] == ui_build['sources_after']
assert final_build['frontend_dist'] == final_build['frontend_dist_after'] == ui_build['frontend_dist']
assert observation['frontend_assets'] == len(final_build['frontend_dist']) == 284
assert final_build['generated_before'] == final_build['generated_after']
assert observation['generated_types'] == len(final_build['generated_before']) == 230
assert observation['live_process_group_members_after_cleanup'] == final_receipt['live_process_group_members_after_cleanup'] == []
assert observation['roundtrip_semantic_match'] is True
saved = observation['post_run_binary_and_fixture_observation']
assert saved['binary_sha256'] == final_receipt['binary_sha256']
assert saved['saved_sha256'] == final_receipt['checks']['native_save_roundtrip']['sha256']
assert saved['saved'] == final_receipt['checks']['native_save_roundtrip']['actual'] == final_receipt['checks']['native_save_roundtrip']['expected']
original_harness = (OUT / 'remaining-native-webview.py').read_bytes()
extended_harness = (OUT / 'remaining-native-webview-after-path.py').read_bytes()
extension = native['harness_extension']
verify(extension['original'])
verify(extension['extended'])
original_allowlist = b"('remaining-linux-native-build.json', 'remaining-linux-native-after-csp.json')"
extended_allowlist = b"('remaining-linux-native-build.json', 'remaining-linux-native-after-csp.json', 'remaining-linux-native-after-path.json')"
assert original_harness.count(original_allowlist) == 1
assert original_harness.count(b'\r\n') == 0
assert original_harness.replace(original_allowlist, extended_allowlist).replace(b'\n', b'\r\n') == extended_harness
assert extension['original_lf_count'] == original_harness.count(b'\n') == 227
assert extension['extended_crlf_count'] == extended_harness.count(b'\r\n') == 227
assert extension['byte_identical_outside_allowlist'] is False
assert hashlib.sha256(extended_harness).hexdigest() == final_receipt['harness_sha256']
assert b"record['styles_after_save'] = script('return window.__ccrNativeStyleCspEvents')" in extended_harness
assert native['passed_attempts_on_final_source_epoch'] == [item['report']['path'] for item in native['attempts'] if item['state'] == 'passed' and item['source_epoch']['manifest']['sha256'] == matrix['final_source_freeze']['sha256']]
assert matrix['continuation_final_evidence']['native_receipt_on_final_source_epoch'] == ('passed_scoped_receipt' if native['passed_attempts_on_final_source_epoch'] else 'not_verified')
dedicated = matrix['dedicated_review']
assert dedicated['role'] == 'trellis-check' and dedicated['dispatch_restored']
assert ledger['dedicated_review'] == dedicated
current_review = json.loads((ROOT / dedicated['report_paths'][0]).read_text(encoding='utf-8'))
assert current_review['role'] == dedicated['role'] and current_review['reviewer'] == dedicated['reviewer']
assert current_review['independence'] == dedicated['independence']
schema_evidence = matrix['remaining_schema_restoration']
assert schema_evidence['report']['path'].endswith('/remaining-schema-restoration-final.json')
assert len(schema_evidence['historical_restorations']) == 1
historical_schema_evidence = schema_evidence['historical_restorations'][0]
assert historical_schema_evidence['report']['path'].endswith('/remaining-schema-restoration.json')
for field in ('report', 'baseline', 'preserved_copy'):
    verify(historical_schema_evidence[field])
historical_schema = json.loads((ROOT / historical_schema_evidence['report']['path']).read_text(encoding='utf-8'))
assert historical_schema['status'] == historical_schema_evidence['status'] == 'restored_original_schema_set'
assert historical_schema['preexisting_files_unchanged'] and historical_schema['removed_path_absent']
assert historical_schema_evidence['preexisting_files'] == len(historical_schema['preexisting_schema_hashes']) == 4
assert historical_schema_evidence['preserved_copy']['sha256'] == historical_schema['preserved_copy_sha256'] == historical_schema['removed_generated_sha256']
assert historical_schema_evidence['baseline']['sha256'] == historical_schema['baseline_sha256']
for field in ('report', 'baseline', 'preserved_copy'):
    verify(schema_evidence[field])
schema = json.loads((ROOT / schema_evidence['report']['path']).read_text(encoding='utf-8'))
assert schema['status'] == schema_evidence['status'] == 'restored_original_schema_set'
assert schema['preexisting_files_unchanged'] and schema['removed_path_absent']
assert schema_evidence['preexisting_files'] == len(schema['preexisting_schema_hashes']) == 4
assert schema['preserved_copy_sha256'] == schema_evidence['preserved_copy']['sha256'] == schema['removed_generated_sha256']
assert schema_evidence['preserved_copy']['path'].endswith('/remaining-linux-generated-schema-final.json')
assert schema_evidence['baseline'] == historical_schema_evidence['baseline']
assert schema['preexisting_schema_hashes'] == historical_schema['preexisting_schema_hashes']
assert schema['removed_generated_path'] == historical_schema['removed_generated_path']
assert datetime.fromisoformat(schema['verified_at_utc']) >= datetime.fromisoformat(historical_schema['verified_at_utc'])
assert schema['previous_restoration'] == Path(historical_schema_evidence['report']['path']).name
assert schema['previous_restoration_sha256'] == historical_schema_evidence['report']['sha256']
final_windows_execution = matrix['continuation_runs']['remaining_windows_ci_after_path']['execution']
assert schema['completed_windows_gate'] == Path(final_windows_execution['metadata']['path']).name
assert schema['completed_windows_gate_sha256'] == final_windows_execution['metadata']['sha256']
assert final_windows_execution['exit_code'] == 0
assert datetime.fromisoformat(schema['verified_at_utc']) >= datetime.fromisoformat(final_windows_execution['completed_at'])
assert schema['source_freeze_sha256'] == platform_test_capture['sha256']
assert schema['source_changes'] == [] and schema['current_generated_count'] == 230
# Current filesystem retention is evaluated only after the final generation transaction and restoration.
assert not (ROOT / schema['removed_generated_path']).exists()
assert all(hashlib.sha256((ROOT / path).read_bytes()).hexdigest() == digest for path, digest in schema['preexisting_schema_hashes'].items())
contexts = matrix['remaining_context_preservation']
verify(contexts['report'])
context_report = json.loads((ROOT / contexts['report']['path']).read_text(encoding='utf-8'))
assert contexts['passed'] == context_report['passed'] == len(context_report['validations']) == 12
assert contexts['all_contexts_unchanged'] and context_report['all_contexts_unchanged']
for item in context_report['validations']:
    assert item['exit_code'] == 0 and item['context_unchanged']
    for name, digest in item['context_hashes'].items():
        assert hashlib.sha256((ROOT / '.trellis/tasks' / item['task'] / name).read_bytes()).hexdigest() == digest
assert contexts['warnings'] == [
    {'task': item['task'], 'warning': re.sub(r'\x1b\[[0-9;]*m', '', line).strip()}
    for item in context_report['validations'] for line in item['output'].splitlines() if 'Warning:' in line
]
for task, status in contexts['existing_tasks'].items():
    assert json.loads((ROOT / '.trellis/tasks' / task / 'task.json').read_text(encoding='utf-8'))['status'] == status
final_contexts = contexts['final_validation']
verify(final_contexts['report'])
assert final_contexts['prior_report'] == contexts['report']
final_context_report = json.loads((ROOT / final_contexts['report']['path']).read_text(encoding='utf-8'))
assert final_context_report['prior_receipt_sha256'] == contexts['report']['sha256']
assert final_context_report['prior_receipt'] == Path(contexts['report']['path']).name
assert final_contexts['passed'] == len(final_context_report['validations']) == 12
assert final_contexts['context_files'] == sum(len(item['context_hashes']) for item in final_context_report['validations']) == 24
assert final_contexts['all_contexts_unchanged'] is True
previous_contexts = {item['task']: item['context_hashes'] for item in context_report['validations']}
for item in final_context_report['validations']:
    assert item['exit_code'] == 0 and item['context_unchanged']
    assert item['context_hashes'] == previous_contexts[item['task']]
    for name, digest in item['context_hashes'].items():
        assert hashlib.sha256((ROOT / '.trellis/tasks' / item['task'] / name).read_bytes()).hexdigest() == digest
assert final_contexts['existing_tasks'] == final_context_report['insights'] == contexts['existing_tasks']
assert final_contexts['warning'] == final_context_report['warning']
assert '33513' in final_contexts['warning'] and '32768' in final_contexts['warning']
assert final_contexts['warnings'] == [
    {'task': item['task'], 'warning': re.sub(r'\x1b\[[0-9;]*m', '', line).strip()}
    for item in final_context_report['validations'] for line in item['output'].splitlines() if 'Warning:' in line
]
assert matrix['continuation_final_evidence']['status'] == 'local_check_records_complete_required_acceptance_open'
audit_setup = matrix['remaining_audit_tool_setup']
verify(audit_setup['metadata'])
verify(audit_setup['log'])
audit_install = json.loads((ROOT / audit_setup['metadata']['path']).read_text(encoding='utf-8'))
assert audit_setup['log']['sha256'] == audit_install['log_sha256']
assert audit_setup['command'] == audit_install['command']
assert audit_setup['exit_code'] == audit_install['exit_code'] == 0
assert audit_install['state'] == 'finished'
assert audit_setup['version'] == 'cargo-audit 0.22.2'
assert audit_install['command'][audit_install['command'].index('--version') + 1] == '0.22.2'
assert audit_setup['started_at'] == audit_install['started_at_utc']
assert audit_setup['completed_at'] == audit_install['finished_at_utc']
assert 'sources_before' not in audit_install and 'sources_after' not in audit_install
expected_platform_lanes = {
    'remaining_windows_ci_after_path', 'remaining_linux_tauri_ci_after_path',
    'remaining_linux_coverage_after_path', 'remaining_linux_native_after_path',
    'remaining_linux_workspace_quality_after_path', 'remaining_linux_workspace_coverage_after_path',
    'remaining_linux_workspace_audit_after_path', 'remaining_linux_workspace_msrv_after_path',
}
assert {item['key'] for item in matrix['remaining_platform_test_lane_receipts']} == expected_platform_lanes
for lane in matrix['remaining_platform_test_lane_receipts']:
    run = matrix['continuation_runs'].get(lane['key'])
    if run:
        assert lane['status'] == 'recorded_finished' and lane['exit_code'] == run['execution']['exit_code']
        assert run['execution']['metadata']['path'].endswith('/' + lane['metadata'])
        assert run['execution']['source_epoch']['manifest']['sha256'] == platform_test_capture['sha256']
    else:
        assert lane['status'] in ('pending', 'not_run') and lane['exit_code'] is None
        assert lane['metadata'] in matrix['continuation_final_evidence']['pending']
audit_run = matrix['continuation_runs'].get('remaining_linux_workspace_audit_after_path')
if audit_run:
    meta = json.loads((ROOT / audit_run['execution']['metadata']['path']).read_text(encoding='utf-8'))
    assert audit_run['execution']['command'] == ['cargo', 'audit']
    assert audit_run['audit_tool']['installation_metadata'] == audit_setup['metadata']
    for field in ('cargo_audit_version', 'cargo_audit_path', 'cargo_audit_sha256'):
        assert audit_run['audit_tool'][field] == meta[field]
    assert meta['cargo_audit_version'] == audit_setup['version']
    assert re.fullmatch(r'[0-9a-f]{64}', meta['cargo_audit_sha256'])
    assert datetime.fromisoformat(audit_setup['completed_at']) <= datetime.fromisoformat(meta['started_at_utc'])
msrv_run = matrix['continuation_runs'].get('remaining_linux_workspace_msrv_after_path')
if msrv_run:
    assert msrv_run['execution']['command'] == ['cargo', 'check', '--locked', '--workspace', '--all-targets', '--all-features']
    assert msrv_run['execution']['rustc_version'].startswith('rustc 1.95.0 ')
assert matrix['continuation_final_evidence']['pending'] == []
assert matrix['continuation_final_evidence']['remaining_required_acceptance']
assert matrix['continuation_final_evidence']['lifecycle_acceptance_granted'] is False
python_results = matrix['test_groups']['aggregate']['evidence'][0]['result_lines']
assert any(line.startswith('Ran 27 tests') for line in python_results)
assert 'OK' in python_results
result = {
    'date': '2026-09-29', 'status': 'passed',
    'requirements': len(ids), 'test_groups': len(matrix['test_groups']),
    'p1_groups': len(ledger['findings']), 'verified_artifacts': len(checked),
    'continuation_runs': len(matrix['continuation_runs']),
    'checks': ['current-index source and log SHA-256', 'historical captures separated from recorded execution source', 'exact selector and captured line', 'AC group or artifact coverage', 'P1 group equality', 'scoped old-red artifact hashes', 'null fix commits', 'Python result extraction', 'real Codex persistence helper selector', '62 unique Tauri selectors excluding empty targets', 'Linux 1.98 nine process cases and three Doctor cases', '45 Settings and 24 i18n checks with 4523 keys per locale', 'root CI binding startup failure retained separately from formal frontend', '230 generated-type restoration with original failed concurrent-capture differences preserved', '269 final registered paths, 267 files and two explicit deletions, with seven immutable epoch manifests', 'Linux 1.95 workspace check without test inference', 'cargo-audit 0.22.2 installation and command receipts kept separate', 'workspace overall 70 and core gateway 85 coverage thresholds with immutable raw counts', 'Tauri gateway 85 threshold without an overall Tauri threshold', 'test-only octal and store path portability changes with exact source boundaries', 'DC-02 permission-owner remediation and metadata-only rollback scope', 'actual UI build result separate from capture-wrapper encoding error', 'scoped native Linux receipts retain their build epoch and platform limitations', 'macOS required acceptance, null fix commits and task-context warnings preserved'],
    'limits': 'Evidence graph integrity and the recorded scopes only. No lifecycle transition or global acceptance. Native Linux receipts do not establish Windows/macOS WebViews, release packaging, physical input, native Codex/Grok or real provider-account behavior.',
}
(OUT / 'evidence-mapping-validation.json').write_text(json.dumps(result, ensure_ascii=False, indent=2) + chr(10), encoding='utf-8')
print(json.dumps(result, ensure_ascii=False))
