#!/usr/bin/env python3
"""Summarize independent process medians; never pool batches across processes."""
import argparse
from collections import defaultdict
import json
from pathlib import Path
import statistics


def summary(values):
    values = sorted(values)
    q1, _, q3 = statistics.quantiles(values, n=4, method='inclusive')
    return {'median': statistics.median(values), 'q1': q1, 'q3': q3, 'min': min(values), 'max': max(values), 'count': len(values)}


def summarize(data):
    batches = defaultdict(list)
    for row in data['records']:
        if row['kind'] == 'sample':
            batches[(row['runtime'], row['case'], row['round'])].append(row['nsPerOp'])
    processes = defaultdict(list)
    expected_rounds = data['metadata']['rounds']
    for (runtime, case, _), values in batches.items():
        if len(values) != 3: raise ValueError(f'Expected three batches: {runtime}/{case}')
        processes[(runtime, case)].append(statistics.median(values))
    result = {}
    for key, values in processes.items():
        if len(values) != expected_rounds: raise ValueError(f'Missing process rounds: {key}')
        result[key] = summary(values)
    return result

def startup_summary(records):
    probes = [row for row in records if row['kind'] == 'startup-probe' and row['mode'] == 'normal']
    if not probes: return None
    if len(probes) != 30: raise ValueError('Expected 30 normal instrumented startup probes')
    return {phase: statistics.median(row['phases'].get(phase, 0) for row in probes)
            for phase in ['publicImport', 'load', 'compile', 'instantiate', 'Instance', 'firstOperation']}


def host_description(meta):
    host = meta['host']
    return (f"{host['platform']} ({host['machine']}), CPU affinity {host['cpuAffinity']}, "
            f"MIDR {host['midrEl1']}, maximum frequency {host['maxFrequencyKhz']} kHz, "
            f"governor {host['governor']}")


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('input', type=Path)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    data = json.loads(args.input.read_text())
    if 'completedUtc' not in data['metadata']: raise ValueError('Measurement did not complete')
    result = summarize(data)
    meta = data['metadata']
    lines = ['# Readable names and reversible u32: measured comparison', '',
             'All warm timings below are **nanoseconds per operation**, median of process medians; brackets show the process-median interquartile range. Lower is faster.', '',
             f"Measured {meta['completedUtc']} on {host_description(meta)}. "
             f"Rust: {meta['tools']['rustc']}; Node: {meta['tools']['node']}. "
             f"{meta['rounds']} fresh processes per cell, three measured batches per process, {meta['warmupMs']} ms warmup, {meta['targetSampleMs']} ms target batches.", '',
             f"Harness source: `{meta['sourceCommit']}` (dirty: `{str(meta['sourceDirty']).lower()}`). "
             f"WASM artifact source: `{meta['artifact']['sourceCommit']}`, SHA-256 `{meta['artifact']['sha256']}`. "
             'Exact harness/fixture/lock hashes and samples are in the raw JSON.', '']
    browser = next(row for row in data['records'] if row['kind'] == 'metadata' and row['runtime'] == 'chromium')
    lines += [f"Browser: Chromium headless shell {browser['version']}, published ESM / browserify assets over local HTTP. Compare alternatives **within a runtime**, not Rust vs Node vs Chromium.", '']
    def cell(runtime, name):
        row = result.get((runtime, name))
        return '—' if row is None else f"{row['median']:,.1f} [{row['q1']:,.1f}–{row['q3']:,.1f}]"
    lines += ['## Two-word name generation', '',
              'Shared rows use the same 749 adjectives and 333 animals (249,417 phrase states). nwords includes the harness RNG; it does not itself generate randomness. Native output uses hyphens and JavaScript output spaces. Shared RNG families are matched, but nwords makes one bounded draw and competitors two.', '',
              '| Rust operation | ns/op [IQR] |', '|---|---:|']
    for name in ['nwords/name/shared-rand08', 'names/name/shared', 'nwords/name/shared-rand10', 'petname/name/shared-rand10', 'names/name/default', 'petname/name/default-rand10']:
        lines.append(f'| {name} | {cell("rust", name)} |')
    lines += ['', '| JavaScript operation | Node ns/op [IQR] | Chromium ns/op [IQR] |', '|---|---:|---:|']
    for name in ['nwords/name/shared-math-random', 'unique-names-generator/name/shared', 'unique-names-generator/name/default', 'nwords/name/shared-crypto']:
        lines.append(f'| {name} | {cell("node", name)} | {cell("chromium", name)} |')
    native = next(row for row in data['records'] if row['kind'] == 'metadata' and row['runtime'] == 'rust')
    lines += ['', f"Default capacities differ: names {native['namesDefaultCapacity']:,}; petname {native['petnameDefaultCapacity']:,}; unique-names-generator {browser['uniqueNamesDefaultCapacity']:,}. Those rows use each library's own vocabulary. The crypto row is informational, with no equivalent UNG row.", '',
              '## Reversible u32 encoding and decoding', '',
              'Every implementation roundtrips the same 4,096-ID fixture, including zero and u32::MAX. Output grammars differ. This is an equal-input-domain tradeoff, not an interchangeable-format ranking.', '',
              '| Runtime / format | Words | Dictionary size | Encode ns/op [IQR] | Decode ns/op [IQR] |', '|---|---:|---:|---:|---:|']
    formats = [('rust', 'nwords', 4, 333), ('rust', 'nwords-bip39-list', 3, 2048), ('rust', 'mnemonic', 3, 1626), ('node', 'nwords', 4, 333), ('node', 'niceware', 2, 65536), ('chromium', 'nwords', 4, 333), ('chromium', 'niceware', 2, 65536)]
    for runtime, name, words, dictionary in formats:
        lines.append(f'| {runtime} / {name} | {words} | {dictionary:,} | {cell(runtime, name + "/u32/encode")} | {cell(runtime, name + "/u32/decode")} |')
    lengths = next(row for row in data['records'] if row['kind'] == 'lengths')
    lines += ['', f"Mean phrase lengths in this fixture: nwords animal×4 {lengths['nwords4']:.2f}; nwords positional BIP-39 list {lengths['nwords3']:.2f}; mnemonic {lengths['mnemonic']:.2f}; niceware {browser['meanChars']['niceware']:.2f} characters, including separators.", '',
              f"mnemonic has 1,626 ordinary words plus seven remainder markers; four-byte IDs use the ordinary-word alphabet. nwords uses {meta.get('nwordsLookup', 'linear dictionary lookup')} and exact case-sensitive parsing. mnemonic uses a lazily built hash map and accepts non-alphabetic separators. niceware lowercases input and binary-searches its larger dictionary. Their error/validation behavior is not equivalent.", '',
              '## Binding diagnostics', '',
              'The native JSON diagnostic includes shape resolution, codec construction and JSON output. The public JS/WASM API uses the binding shipped in the measured artifact; see the stage write-up for changes to preparation and result transport. Ratios do **not** isolate pure WASM overhead.', '',
              '| Layer (animal×4) | Encode ns/op [IQR] | Decode ns/op [IQR] |', '|---|---:|---:|',
              f'| Reused native Rust codec | {cell("rust", "nwords/u32/encode")} | {cell("rust", "nwords/u32/decode")} |',
              f'| Native Rust JSON ABI (JSON output) | {cell("rust", "nwords-abi/u32/encode")} | {cell("rust", "nwords-abi/u32/decode")} |',
              f'| Public Node JS/WASM API | {cell("node", "nwords/u32/encode")} | {cell("node", "nwords/u32/decode")} |', '',
              '## Fresh-process Node startup', '',
              '30 fresh processes per package, warm OS file cache, compile cache disabled. Initial entry resolution is excluded equally. Internal total is module loading/evaluation + explicit initialization + first operation. Whole-process wall time includes Node startup and output. These are different first operations (names versus a reversible four-byte encoding).', '',
              '| Package | Import ms | Explicit init ms | First op ms | Internal total ms [IQR] | Process wall ms |', '|---|---:|---:|---:|---:|---:|']
    for name in ['nwords', 'unique-names-generator', 'niceware']:
        rows = [row for row in data['records'] if row['kind'] == 'cold' and row['case'] == name]
        if len(rows) != 30: raise ValueError('Expected 30 cold observations')
        totals = summary([row['totalMs'] for row in rows])
        parts = [statistics.median(row[field] for row in rows) for field in ['importMs', 'initializationMs', 'firstOperationMs']]
        wall = statistics.median(row['processWallMs'] for row in rows)
        lines.append(f"| {name} | {parts[0]:.2f} | {parts[1]:.2f} | {parts[2]:.3f} | {totals['median']:.2f} [{totals['q1']:.2f}–{totals['q3']:.2f}] | {wall:.2f} |")
    empty = statistics.median(row['processWallMs'] for row in data['records'] if row['kind'] == 'cold-baseline')
    first = statistics.median(row['elapsedNs'] for row in data['records'] if row['kind'] == 'cold-native') / 1000
    lines += ['', f'Empty Node process median wall time: {empty:.2f} ms. Native mnemonic first decode, including lazy index construction: {first:.2f} µs (30 fresh processes). Neither is subtracted from other timings.', '',
              '## Actual browser payload and npm package size', '',
              'These are the bytes served by this benchmark, not minimal bundler output. niceware carries its published Buffer shim; UNG ESM includes its published dictionaries. Harness files are excluded.', '',
              '| Served implementation | Uncompressed bytes |', '|---|---:|']
    assets = {row['path']: row['bytes'] for row in browser['assets']}
    for label, prefix in [('nwords ESM + WASM', '/package/'), ('unique-names-generator ESM', '/deps/unique-names-generator/'), ('niceware browserify bundle', '/deps/niceware/')]:
        lines.append(f'| {label} | {sum(size for name, size in assets.items() if name.startswith(prefix)):,} |')
    lines += ['', f"The nwords tarball is {meta['artifact']['packedBytes']:,} compressed bytes, including licenses. Installed comparator package bytes (excluding transitives): " + ', '.join(f'{name} {size:,}' for name, size in meta['npmPackageBytes'].items()) + '. These package sizes are not comparable to the served-byte column.', '',
              '## All warm diagnostics and variability', '', '| Runtime / operation | Median ns/op | IQR | Min–max process median |', '|---|---:|---:|---:|']
    probes = startup_summary(data['records'])
    if probes:
        diagnostic = ['## Instrumented Node loader phases', '',
                      'A separate fresh-process probe observes the actual public loader. These timings include instrumentation overhead; the uninstrumented startup table remains the primary measurement. Load includes its component phases, so do not sum these medians. Zero means a path was not invoked.', '',
                      '| Phase | Median ms |', '|---|---:|']
        for phase, value in probes.items():
            diagnostic.append(f'| {phase} | {value:.3f} |')
        position = lines.index('## All warm diagnostics and variability')
        lines[position:position] = diagnostic + ['']
    for (runtime, name), row in sorted(result.items()):
        lines.append(f"| {runtime} / {name} | {row['median']:,.1f} | {row['q1']:,.1f}–{row['q3']:,.1f} | {row['min']:,.1f}–{row['max']:,.1f} |")
    noisy = [f'{runtime}/{name}' for (runtime, name), row in sorted(result.items()) if (row['q3'] - row['q1']) > 0.25 * row['median']]
    lines += ['', 'Variability flag (IQR greater than 25% of median): ' + (', '.join(noisy) if noisy else 'no cells') + '. No rounds were discarded.', '',
              '| Round | Host load average before → after (1 min) | Selected CPU busy fraction |', '|---|---:|---:|']
    for load in meta['roundLoad']:
        busy = load.get('selectedCpuBusyFraction')
        lines.append(f"| {load['round'] + 1} | {load['before'][0]:.2f} → {load['after'][0]:.2f} | {busy:.1%} |" if busy is not None else f"| {load['round'] + 1} | {load['before'][0]:.2f} → {load['after'][0]:.2f} | unavailable |")
    lines += ['', f"This is one shared {meta['host']['machine']} host. CPU affinity limits migration but does not eliminate contention, frequency changes or GC. Selected-CPU activity includes this benchmark and cannot isolate competing work. {meta['rounds']} process rounds provide a descriptive estimate, not a statistical proof or a portable performance guarantee. No measured operation includes registry collision checking or persistence.", '',
              'See benchmarks/README.md for methodology and comparator sources. Re-run on the intended deployment host before making a performance-sensitive choice.', '']
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text('\n'.join(lines))
    compact = [{'runtime': runtime, 'case': name, **row} for (runtime, name), row in sorted(result.items())]
    args.output.with_suffix('.summary.json').write_text(json.dumps(compact, indent=2) + '\n')
    print(args.output)

if __name__ == '__main__':
    main()
