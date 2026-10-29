"""Run the independent consumer against a fresh candidate extraction."""
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import tarfile
import tempfile
import tomllib

root = Path(__file__).resolve().parent
version = tomllib.loads((root / 'Cargo.toml').read_text())['package']['version']
archive = root / 'dist' / f'mnn-src-{version}.crate'
assert archive.stat().st_size < 10_000_000
work = Path(tempfile.mkdtemp(prefix='mnn-consumer-'))
print(f'Isolated consumer: {work}', flush=True)
with tarfile.open(archive) as source:
    source.extractall(work, filter='data')
crate = work / f'mnn-src-{version}'
def fingerprint():
    return {str(p.relative_to(crate)): hashlib.sha256(p.read_bytes()).hexdigest()
            for p in crate.rglob('*') if p.is_file()}


pristine = fingerprint()
project = work / 'consumer'
shutil.copytree(root / 'testcrate', project, ignore=shutil.ignore_patterns('target'))
manifest = project / 'Cargo.toml'
features = os.environ.get('MNN_TEST_FEATURES', 'static').split(',')
dep = '{ path = ' + json.dumps(str(crate)) + ', default-features = false, features = ' + json.dumps(features) + ' }'
manifest.write_text(re.sub(r'(?m)^mnn-src = .*', lambda _: 'mnn-src = ' + dep, manifest.read_text()))
env = dict(os.environ)
for key in ['MNN_ROOT', 'LD_LIBRARY_PATH', 'DYLD_LIBRARY_PATH', 'LD_PRELOAD', 'DYLD_INSERT_LIBRARIES']:
    env.pop(key, None)
env['CARGO_TARGET_DIR'] = str(work / 'target')
command = ['cargo', 'test', '--manifest-path', str(manifest), '--locked']
subprocess.run(command + ['--no-run'], env=env, check=True)
runtime = work / 'target/debug/deps'
for pattern in ['debug/build/mnn-src-*/out/bin/*.dll', 'debug/build/mnn-src-*/out/lib/*.dylib', 'debug/build/mnn-src-*/out/lib/*.so*']:
    for library in (work / 'target').glob(pattern):
        shutil.copy2(library, runtime / library.name)
if os.environ.get('MNN_BUILD_ONLY') != '1':
    subprocess.run(command + ['--offline', '--', '--test-threads=1'], env=env, check=True, timeout=120)

    relocated = work / 'relocated'
    relocated.mkdir()
    for artifact in runtime.iterdir():
        if artifact.name.endswith(('.dll', '.dylib', '.so')) or (artifact.name.startswith('mnn_src_smoke-') and (artifact.suffix == '.exe' or not artifact.suffix)):
            shutil.copy2(artifact, relocated / artifact.name)
    binary = next(p for p in relocated.iterdir() if p.name.startswith('mnn_src_smoke-'))
    subprocess.run([str(binary), '--test-threads=1'], cwd=relocated, env=env, check=True, timeout=120)

assert fingerprint() == pristine, "consumer build modified extracted crate"
