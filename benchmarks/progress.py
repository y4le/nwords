"""Plot the committed stage results and the final optional prepared API."""
import argparse
import hashlib
import json
from pathlib import Path

import matplotlib
matplotlib.use('Agg')
import matplotlib.pyplot as plt

from report import summarize

ROOT = Path(__file__).resolve().parent.parent


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--results', type=Path, default=ROOT / 'benchmarks/results/performance')
    parser.add_argument('--output', type=Path, default=ROOT / 'docs/performance-progress.svg')
    args = parser.parse_args()
    labels, data, inputs = [], [], []
    for path in sorted(args.results.glob('[0-9][0-9]-*.json')):
        raw = path.read_bytes()
        measured = json.loads(raw)
        # Targeted experiments and summaries are separate from full stage runs.
        if not isinstance(measured, dict) or measured.get('metadata', {}).get('schema') != 'nwords.benchmark.v1':
            continue
        if 'completedUtc' not in measured['metadata']:
            raise ValueError(f'Incomplete measurement: {path}')
        labels.append('prepared (stateless)' if path.stem == '06-prepared' else path.stem[3:].replace('-', ' '))
        data.append(summarize(measured))
        inputs.append({'file': path.name, 'sha256': hashlib.sha256(raw).hexdigest()})
    if not data:
        raise ValueError('No completed full-stage measurements')
    if all((runtime, 'nwords-prepared/u32/encode') in data[-1] for runtime in ['node', 'chromium']):
        labels.append('prepared API (opt-in)')
        data.append({(runtime, f'nwords/u32/{op}'): data[-1][runtime, f'nwords-prepared/u32/{op}']
                     for runtime in ['node', 'chromium'] for op in ['encode', 'decode']})

    matplotlib.rcParams['svg.hashsalt'] = 'nwords-performance-progress-v1'
    matplotlib.rcParams['svg.fonttype'] = 'none'
    fig, axes = plt.subplots(1, 2, figsize=(12, 5), layout='constrained', sharey=True)
    for ax, runtime in zip(axes, ['node', 'chromium']):
        for offset, op, color in [(-.18, 'encode', '#2467a4'), (.18, 'decode', '#da8622')]:
            rows = [stage[runtime, f'nwords/u32/{op}'] for stage in data]
            xs = [index + offset for index in range(len(rows))]
            ax.bar(xs, [row['median'] for row in rows], width=.34, label=op, color=color)
            ax.errorbar(xs, [row['median'] for row in rows],
                        yerr=[[row['median'] - row['q1'] for row in rows],
                              [row['q3'] - row['median'] for row in rows]],
                        fmt='none', color='#333333', capsize=2)
        ax.set_xticks(range(len(labels)), labels, rotation=35, ha='right')
        ax.set_title(runtime.capitalize())
        ax.set_ylim(bottom=0)
        ax.grid(axis='y', alpha=.2)
        ax.set_axisbelow(True)
        ax.legend(frameon=False)
    axes[0].set_ylabel('Nanoseconds per ID (lower is faster)')
    fig.suptitle('Public API: incremental u32 performance\n'
                 'Seven process medians per cell; whiskers show IQR\n'
                 'Final bar reuses a prepared codec and excludes setup; other bars are stateless', fontsize=13)
    args.output.parent.mkdir(parents=True, exist_ok=True)
    fig.savefig(args.output, metadata={'Creator': f'nwords progress.py, matplotlib {matplotlib.__version__}',
                                     'Date': None, 'Description': json.dumps(inputs, sort_keys=True)})
    plt.close(fig)
    if args.output.suffix == '.svg':
        args.output.write_text('\n'.join(line.rstrip() for line in args.output.read_text().splitlines()) + '\n')


if __name__ == '__main__':
    main()
