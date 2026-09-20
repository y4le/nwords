#!/usr/bin/env python3
"""Serial, reproducible comparison runner. No changes to production dependencies."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import random
import shutil
import subprocess
import time

ROOT = Path(__file__).resolve().parent.parent
BENCH = ROOT / 'benchmarks'

def output(command, **kwargs):
    return subprocess.check_output(command, cwd=ROOT, text=True, **kwargs).strip()

def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()

def cpu_file(cpu, name):
    path = Path(f'/sys/devices/system/cpu/cpu{cpu}/{name}')
    return path.read_text().strip() if path.exists() else 'unavailable'

def cpu_ticks(cpu):
    if cpu is None or not Path('/proc/stat').exists(): return None
    row = next(line for line in Path('/proc/stat').read_text().splitlines() if line.startswith(f'cpu{cpu} '))
    ticks = list(map(int, row.split()[1:9]))  # Exclude guest counters, already in user/nice.
    return {'total': sum(ticks), 'idle': ticks[3] + ticks[4]}

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--tarball', type=Path, default=ROOT / 'dist/tarballs/y4le-nwords-0.1.0.tgz')
    parser.add_argument('--rounds', type=int, default=7)
    parser.add_argument('--sample-ms', type=int, default=100)
    parser.add_argument('--output', type=Path, default=BENCH / '.work/results.json')
    parser.add_argument('--cpu', type=int, default=None, help='default: first CPU in allowed affinity set')
    args = parser.parse_args()
    if not (3 <= args.rounds <= 30 and 10 <= args.sample_ms <= 1000):
        parser.error('Use 3–30 process rounds and 10–1000 ms samples')
    allowed = sorted(os.sched_getaffinity(0)) if hasattr(os, 'sched_getaffinity') else []
    selected = args.cpu if args.cpu is not None else (allowed[0] if allowed else None)
    if selected is not None:
        if selected not in allowed: parser.error('CPU is outside the allowed set')
        os.sched_setaffinity(0, {selected})
    work = BENCH / '.work'
    work.mkdir(exist_ok=True)
    # Install the exact artifact. This also catches omissions hidden by a source import.
    consumer = work / 'consumer'
    if consumer.exists(): shutil.rmtree(consumer)
    consumer.mkdir()
    (consumer / 'package.json').write_text('{"private":true,"type":"module"}\n')
    subprocess.run(['npm', 'install', '--ignore-scripts', '--no-audit', '--no-fund', str(args.tarball.resolve())], cwd=consumer, check=True, stdout=subprocess.DEVNULL)
    package = consumer / 'node_modules/@y4le/nwords'
    build = json.loads((package / 'build.json').read_text())
    if build['dirty'] or build['development']:
        raise RuntimeError('Benchmark a qualified clean artifact, not a development tarball')
    subprocess.run(['git', 'diff', '--exit-code', build['sourceCommit'], '--', 'crates', 'packages/nwords-js/src', 'packages/nwords-js/package.template.json'], cwd=ROOT, check=True, stdout=subprocess.DEVNULL)
    if output(['git', 'ls-files', '--others', '--exclude-standard', '--', 'crates', 'packages/nwords-js/src']):
        raise RuntimeError('Untracked crate or wrapper sources invalidate artifact parity')
    if digest(package / 'wasm/nwords_js_bg.wasm') != build['wasm']['sha256']:
        raise RuntimeError('WASM provenance mismatch')
    os.environ.pop('NODE_COMPILE_CACHE', None)
    os.environ.pop('NODE_OPTIONS', None)
    env = dict(os.environ, RUSTUP_TOOLCHAIN='1.94.0')
    subprocess.run(['cargo', 'build', '--locked', '--release', '--manifest-path', str(BENCH / 'rust/Cargo.toml')], cwd=ROOT, env=env, check=True)
    rust = BENCH / 'rust/target/release/nwords-comparison'
    metadata = {
        'schema': 'nwords.benchmark.v1', 'startedUtc': time.strftime('%Y-%m-%dT%H:%M:%SZ', time.gmtime()),
        'sourceCommit': output(['git', 'rev-parse', 'HEAD']),
        'nwordsLookup': 'binary search for sorted adjective/animal/color and English BIP-39 dictionaries',
        'sourceDirty': bool(output(['git', 'status', '--porcelain=v1', '--untracked-files=all'])),
        'sourceStatus': output(['git', 'status', '--porcelain=v1', '--untracked-files=all']),
        'artifact': { 'sourceCommit': build['sourceCommit'], 'sha256': digest(args.tarball), 'packedBytes': args.tarball.stat().st_size, 'wasm': build['wasm'] },
        'host': { 'platform': platform.platform(), 'machine': platform.machine(), 'cpuAffinity': selected, 'availableCpus': allowed, 'governor': (Path(f'/sys/devices/system/cpu/cpu{selected}/cpufreq/scaling_governor').read_text().strip() if Path(f'/sys/devices/system/cpu/cpu{selected}/cpufreq/scaling_governor').exists() else 'unavailable'),
                  'cpu': output(['lscpu']) if shutil.which('lscpu') else platform.processor(),
                  'maxFrequencyKhz': cpu_file(selected, 'cpufreq/cpuinfo_max_freq'),
                  'midrEl1': cpu_file(selected, 'regs/identification/midr_el1'),
                  'topology': output(['lscpu', '-e=CPU,CORE,SOCKET,MAXMHZ,MINMHZ,ONLINE']) if shutil.which('lscpu') else 'unavailable',
                  'selectedCpuInfo': next((section for section in Path('/proc/cpuinfo').read_text().split('\n\n') if section.startswith(f'processor\t: {selected}\n')), 'unavailable') if Path('/proc/cpuinfo').exists() else 'unavailable' },
        'tools': { 'rustc': output(['rustc', '--version'], env=env), 'node': output(['node', '--version']), 'npm': output(['npm', '--version']) },
        'rounds': args.rounds, 'targetSampleMs': args.sample_ms, 'warmupMs': 100, 'samplesPerProcess': 3, 'roundLoad': [],
        'fixtureSha256': digest(BENCH / 'ids-u32.txt'),
        'cargoTree': output(['cargo', 'tree', '--locked', '--manifest-path', str(BENCH / 'rust/Cargo.toml')], env=env),
        'locks': { 'cargo': digest(BENCH / 'rust/Cargo.lock'), 'npm': digest(BENCH / 'js/package-lock.json') },
        'harnessHashes': { str(path.relative_to(ROOT)): digest(path) for path in sorted(BENCH.rglob('*'))
                          if path.is_file() and not any(p in path.parts for p in ['node_modules', 'target', '.work', 'results']) and path.suffix in ['.py', '.mjs', '.rs', '.toml', '.json', '.lock'] },
    }
    metadata['npmPackageBytes'] = {name: sum(path.stat().st_size for path in (BENCH / 'js/node_modules' / name).rglob('*') if path.is_file()) for name in ['unique-names-generator', 'niceware']}
    metadata['npmTree'] = json.loads(output(['npm', '--prefix', str(BENCH / 'js'), 'ls', '--all', '--json']))
    records = []
    def collect(command):
        started = time.perf_counter()
        stdout = output(command)
        wall_ms = (time.perf_counter() - started) * 1000
        for line in stdout.splitlines():
            records.append(json.loads(line))
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(json.dumps({'metadata': metadata, 'records': records}, indent=2) + '\n')
        return wall_ms
    fixture = str(BENCH / 'ids-u32.txt')
    (work / 'native-phrases.tsv').write_text(output([str(rust), 'vectors', '0', '100', fixture]) + '\n')
    rust_cases = output([str(rust), 'list', '0', '100', fixture]).splitlines()
    node_cases = output(['node', str(BENCH / 'js/node.mjs'), str(package), 'list']).splitlines()
    browser_cases = [name for name in node_cases if name != 'nwords/name/shared-crypto']
    for round_index in range(args.rounds):
        load = {'round': round_index, 'before': os.getloadavg(), 'cpuTicksBefore': cpu_ticks(selected)}
        commands = [[str(rust), case, str(round_index), str(args.sample_ms), fixture] for case in rust_cases]
        commands += [['node', str(BENCH / 'js/node.mjs'), str(package), case, str(round_index), str(args.sample_ms)] for case in node_cases]
        commands += [['node', str(BENCH / 'js/browser.mjs'), str(package), case, str(round_index), str(args.sample_ms)] for case in browser_cases]
        random.Random(7301 + round_index).shuffle(commands)
        print(f'round {round_index + 1}/{args.rounds}: {len(commands)} isolated cells', flush=True)
        for command in commands: collect(command)
        load['after'] = os.getloadavg()
        load['cpuTicksAfter'] = cpu_ticks(selected)
        if load['cpuTicksBefore'] and load['cpuTicksAfter']:
            elapsed = load['cpuTicksAfter']['total'] - load['cpuTicksBefore']['total']
            idle = load['cpuTicksAfter']['idle'] - load['cpuTicksBefore']['idle']
            load['selectedCpuBusyFraction'] = 1 - idle / elapsed if elapsed else None
        metadata['roundLoad'].append(load)
    # Single-shot cold observations need more repetitions than warm batches.
    for cold_round in range(30):
        choices = ['nwords', 'unique-names-generator', 'niceware']
        random.Random(3107 + cold_round).shuffle(choices)
        for choice in choices:
            wall_ms = collect(['node', str(BENCH / 'js/cold.mjs'), choice, str(package)])
            records[-1]['processWallMs'] = wall_ms
        started = time.perf_counter()
        output(['node', '-e', ''])
        records.append({'kind': 'cold-baseline', 'runtime': 'node', 'processWallMs': (time.perf_counter() - started) * 1000})
        collect([str(rust), 'cold-mnemonic'])
    checksums = {record['idsChecksum'] for record in records if record['kind'] == 'metadata'}
    if len(checksums) != 1: raise RuntimeError('Languages did not benchmark the same ID dataset')
    if digest(BENCH / 'ids-u32.txt') != metadata['fixtureSha256']: raise RuntimeError('Fixture changed during measurement')
    for name, before in metadata['harnessHashes'].items():
        if digest(ROOT / name) != before: raise RuntimeError(f'Harness changed during measurement: {name}')
    metadata['completedUtc'] = time.strftime('%Y-%m-%dT%H:%M:%SZ', time.gmtime())
    args.output.write_text(json.dumps({'metadata': metadata, 'records': records}, indent=2) + '\n')
    print(args.output)

if __name__ == '__main__':
    main()
