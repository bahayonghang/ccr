#!/usr/bin/env python3
import datetime,hashlib,json,os,platform,re,subprocess,sys,time
from pathlib import Path
ROOT=Path(__file__).resolve().parents[4]
OUT=Path(__file__).resolve().parent
PREFIX='resume-2026-10-04-formal-'
TARGET=ROOT/'target/evergreen-linux-coverage'
TOOLS=ROOT/'target/evergreen-linux-tools/bin'
ENV=os.environ.copy()
ENV.update({'PATH':str(TOOLS)+':'+str(Path.home()/'.cargo/bin')+':'+ENV.get('PATH',''),'RUSTUP_TOOLCHAIN':'1.98.0','CARGO_TARGET_DIR':str(TARGET)})
OVERRIDES={k:ENV[k] for k in ['PATH','RUSTUP_TOOLCHAIN','CARGO_TARGET_DIR']}

def now():return datetime.datetime.now(datetime.timezone.utc).isoformat()
def sha(path):return hashlib.sha256(path.read_bytes()).hexdigest() if path.exists() else None
def head():
    marker=ROOT/'.git'
    if marker.is_file():marker=(ROOT/marker.read_text().strip().removeprefix('gitdir: ')).resolve()
    value=(marker/'HEAD').read_text().strip()
    if value.startswith('ref: '):
        reference=value.removeprefix('ref: ')
        if (marker/reference).exists():return (marker/reference).read_text().strip()
        for line in (marker/'packed-refs').read_text().splitlines():
            if line.endswith(' '+reference):return line.split()[0]
    return value

def inputs():
    paths=set()
    for ext in ['*.rs','*.toml']:
        paths.update(path for path in (ROOT/'crates').rglob(ext) if 'target' not in path.relative_to(ROOT).parts)
    for name in ['Cargo.toml','Cargo.lock','rust-toolchain.toml','justfile','scripts/quality/check_coverage_thresholds.py','scripts/common.py']:
        paths.add(ROOT/name)
    paths.update((ROOT/'.cargo').rglob('*.toml'))
    return {str(path.relative_to(ROOT)):sha(path) for path in sorted(paths)}

def run(name,argv):
    stdout=OUT/(PREFIX+name+'.stdout.log');stderr=OUT/(PREFIX+name+'.stderr.log')
    before=inputs();started=now();t=time.monotonic();exit_code=None
    with stdout.open('wb') as a,stderr.open('wb') as b:
        child=subprocess.Popen(argv,cwd=ROOT,env=ENV,stdout=a,stderr=b)
        child.wait();exit_code=child.returncode
    after=inputs()
    receipt={'argv':argv,'cwd':str(ROOT),'startedUtc':started,'finishedUtc':now(),'durationSeconds':round(time.monotonic()-t,3),'os':platform.platform(),'distribution':'WSL Ubuntu-24.04','head':head(),'envOverrides':OVERRIDES,'ambientBuildEnvironment':{key:ENV.get(key) for key in ['RUSTFLAGS','RUSTDOCFLAGS','RUST_TEST_THREADS','CARGO_BUILD_JOBS','LLVM_PROFILE_FILE','CARGO_ENCODED_RUSTFLAGS']},'runnerSha256':sha(Path(__file__)),'nativeExit':exit_code,'stdout':stdout.name,'stdoutSha256':sha(stdout),'stdoutBytes':stdout.stat().st_size,'stderr':stderr.name,'stderrSha256':sha(stderr),'stderrBytes':stderr.stat().st_size,'inputSourceHashesBefore':before,'inputSourceHashesAfter':after,'sourceHashesUnchanged':before==after}
    (OUT/(PREFIX+name+'-receipt.json')).write_text(json.dumps(receipt,indent=2)+'\n')
    return receipt

if '--preflight' in sys.argv:
    receipts=[]
    for name,argv in [('rustc-version',['rustc','--version','--verbose']),('cargo-version',['cargo','--version']),('llvm-cov-version',['cargo','llvm-cov','--version']),('rust-components',['rustup','component','list','--installed']),('python-version',['python3','--version'])]:
        receipt=run('linux-'+name,argv);receipts.append(receipt)
        print(name,receipt['nativeExit'],(OUT/receipt['stdout']).read_text().strip(),flush=True)
    (OUT/(PREFIX+'linux-preflight.json')).write_text(json.dumps({'recordedUtc':now(),'toolsPath':str(TOOLS),'targetDir':str(TARGET),'toolsBinaryExists':(TOOLS/'cargo-llvm-cov').exists(),'commands':[{'argv':r['argv'],'nativeExit':r['nativeExit'],'stdout':r['stdout']} for r in receipts]},indent=2)+'\n')
    sys.exit(0 if all(r['nativeExit']==0 for r in receipts) else 1)

pre=json.loads((OUT/(PREFIX+'linux-preflight.json')).read_text())
if not pre['toolsBinaryExists'] or any(r['nativeExit']!=0 for r in pre['commands']):raise SystemExit('Preflight prerequisites missing; no coverage started')
report='target/resume-2026-10-04-linux-workspace-coverage.json'
argv=['cargo','llvm-cov','--workspace','--all-features','--json','--output-path',report,'--','--skip','export_bindings']
receipt=run('linux-workspace-coverage',argv)
combined=(OUT/receipt['stdout']).read_text(errors='replace')+'\n'+(OUT/receipt['stderr']).read_text(errors='replace')
counts={name:sum(int(n) for n in re.findall(r'(\d+) '+name+r'\b',combined)) for name in ['passed','failed','ignored','filtered out']}
receipt.update({'counts':counts,'report':report,'reportSha256':sha(ROOT/report),'toolVersions':pre['commands']})
(OUT/(PREFIX+'linux-workspace-coverage-receipt.json')).write_text(json.dumps(receipt,indent=2)+'\n')
print(json.dumps({'phase':'coverage','nativeExit':receipt['nativeExit'],'counts':counts,'report':report,'sourceHashesUnchanged':receipt['sourceHashesUnchanged']}),flush=True)
if receipt['nativeExit']!=0:sys.exit(receipt['nativeExit'])
threshold=run('linux-workspace-thresholds',['python3','scripts/quality/check_coverage_thresholds.py',report,'--overall','70','--gateway','85','--gateway-pattern','crates/ccr-core/src/core/process_gateway.rs'])
print((OUT/threshold['stdout']).read_text(),flush=True)
print(json.dumps({'phase':'thresholds','nativeExit':threshold['nativeExit'],'sourceHashesUnchanged':threshold['sourceHashesUnchanged']}),flush=True)
sys.exit(threshold['nativeExit'])

