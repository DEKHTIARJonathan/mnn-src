"""Prepare pristine sources; never download dependencies during a Cargo build."""
import hashlib
import io
import lzma
from pathlib import Path
import re
import shutil
import subprocess
import sys
import tarfile
import tomllib
import urllib.request

ROOT = Path(__file__).resolve().parent


def git(*args):
    return subprocess.check_output(['git', '-C', str(ROOT / 'mnn'), *args])


def version():
    return tomllib.loads((ROOT / 'Cargo.toml').read_text())['package']['version']


def check():
    native = '.'.join(re.findall(r'#define MNN_VERSION_(?:MAJOR|MINOR|PATCH) (\d+)', (ROOT / 'mnn/include/MNN/MNNDefine.h').read_text()))
    assert version().split('-')[0] == native
    assert git('rev-parse', 'HEAD') == git('rev-parse', native + '^{commit}')
    assert not git('status', '--porcelain', '--untracked-files=all')


def prepare():
    check()
    paths = ['CMakeLists.txt', 'LICENSE.txt', 'cmake', 'include', 'source', 'express', 'schema/current', 'schema/default', '3rd_party/flatbuffers/include', '3rd_party/flatbuffers/LICENSE.txt', '3rd_party/half', '3rd_party/imageHelper', '3rd_party/lic', '3rd_party/OpenCLHeaders', 'tools/converter/CMakeLists.txt', 'tools/audio/CMakeLists.txt']
    backends = {'cpu', 'arm82', 'metal', 'opencl', 'vulkan', 'cuda'}
    raw = io.BytesIO()
    with tarfile.open(fileobj=raw, mode='w') as output:
        with tarfile.open(fileobj=io.BytesIO(git('archive', 'HEAD', *paths))) as source:
            for member in source:
                parts = Path(member.name).parts
                if member.name.startswith(('source/internal/', 'schema/private/')):
                    continue
                if len(parts) > 2 and parts[:2] == ('source', 'backend') and parts[2] not in backends:
                    continue
                data = source.extractfile(member) if member.isfile() else None
                member.name = 'mnn/' + member.name
                output.addfile(member, data)
        kleidi = (ROOT / 'mnn/cmake/KleidiAI.cmake').read_text()
        kv = re.search(r'set\(KLEIDIAI_COMMIT_SHA "([^"]+)"', kleidi)[1]
        md5 = re.search(r'set\(KLEIDIAI_ARCHIVE_MD5 "([^"]+)"', kleidi)[1]
        cuda = (ROOT / 'mnn/source/backend/cuda/CMakeLists.txt').read_text()
        pins = re.findall(r'set\(CUTLASS_COMMIT_HASH "([^"]+)"\)\s*set\(CUTLASS_VERSION_NAME "([^"]+)"\)', cuda)
        deps = [('kleidiai', f'https://codeload.github.com/ARM-software/kleidiai/tar.gz/refs/tags/v{kv}', md5)]
        deps += [(f'cutlass/{tag}', f'https://codeload.github.com/NVIDIA/cutlass/tar.gz/{sha}', None) for sha, tag in pins]
        for name, url, expected in deps:
            payload = urllib.request.urlopen(url, timeout=120).read()
            if expected:
                assert hashlib.md5(payload).hexdigest() == expected, 'upstream checksum mismatch'
            with tarfile.open(fileobj=io.BytesIO(payload)) as source:
                for member in source:
                    rel = '/'.join(Path(member.name).parts[1:])
                    if not rel:
                        continue
                    if name.startswith('cutlass') and not (rel == 'LICENSE.txt' or rel.startswith(('include/', 'tools/util/include/'))):
                        continue
                    data = source.extractfile(member) if member.isfile() else None
                    member.name = 'dependencies/' + name + '/' + rel
                    output.addfile(member, data)
    (ROOT / 'mnn-sources.tar.xz').write_bytes(lzma.compress(raw.getvalue(), preset=9))


def package():
    prepare()
    staging = ROOT / 'target/package-src'
    if staging.exists():
        shutil.rmtree(staging)
    staging.mkdir(parents=True)
    for name in ['Cargo.toml', 'Cargo.lock', 'build.rs', 'README.md', 'LICENSE.md', 'mnn-sources.tar.xz']:
        shutil.copy2(ROOT / name, staging / name)
    shutil.copytree(ROOT / 'src', staging / 'src')
    subprocess.run(['cargo', 'package', '--manifest-path', str(staging / 'Cargo.toml'), '--locked', '--no-verify'], check=True)
    archive = staging / f'target/package/mnn-src-{version()}.crate'
    size = archive.stat().st_size
    print(f'Archive: {size:,} bytes')
    assert size < 10_000_000
    (ROOT / 'dist').mkdir(exist_ok=True)
    shutil.copy2(archive, ROOT / 'dist' / archive.name)


if __name__ == '__main__':
    {'check': check, 'prepare': prepare, 'package': package}[sys.argv[1]]()
