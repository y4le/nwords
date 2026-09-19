#!/usr/bin/env python3
"""Optional standalone scientific plot; measurements need no plotting dependencies."""
import argparse
import json
from pathlib import Path
import matplotlib
matplotlib.use('Agg')
import matplotlib.pyplot as plt
from matplotlib.ticker import LogLocator, NullFormatter, FuncFormatter
from report import summarize

parser = argparse.ArgumentParser()
parser.add_argument('input', type=Path)
parser.add_argument('--output', type=Path, required=True)
args = parser.parse_args()
data = json.loads(args.input.read_text())
results = summarize(data)
fig, axes = plt.subplots(1, 3, figsize=(13, 4.8), layout='constrained')
for ax, runtime, libraries in zip(axes, ['rust', 'node', 'chromium'], [
    [('nwords', 'nwords · 4 animal words'), ('nwords-bip39-list', 'nwords · 3 BIP-39-list words'), ('mnemonic', 'mnemonic · 3 words')],
    [('nwords', 'nwords WASM · 4 words'), ('niceware', 'niceware · 2 words')],
    [('nwords', 'nwords WASM · 4 words'), ('niceware', 'niceware · 2 words')],
]):
    for offset, operation, color in [(-0.18, 'encode', '#2878b5'), (0.18, 'decode', '#d78327')]:
        rows = [results[(runtime, name + '/u32/' + operation)] for name, _ in libraries]
        medians = [row['median'] / 1000 for row in rows]
        errors = [[(row['median'] - row['q1']) / 1000 for row in rows], [(row['q3'] - row['median']) / 1000 for row in rows]]
        ax.errorbar(medians, [i + offset for i in range(len(libraries))], xerr=errors, fmt='o', color=color, label=operation, capsize=3)
    ax.set_yticks(range(len(libraries)), [label for _, label in libraries], fontsize=9)
    ax.invert_yaxis()
    ax.set_xscale('log')
    ax.xaxis.set_major_locator(LogLocator(base=10, subs=(1, 3)))
    ax.xaxis.set_major_formatter(FuncFormatter(lambda value, _: f'{value:g}'))
    ax.xaxis.set_minor_formatter(NullFormatter())
    rows = [results[(runtime, name + '/u32/' + op)] for name, _ in libraries for op in ['encode', 'decode']]
    ax.set_xlim(min(row['q1'] for row in rows) / 2000, max(row['q3'] for row in rows) / 500)
    ax.set_xlabel('µs/op · log scale · lower is faster', fontsize=9)
    ax.set_title({'rust': 'Native Rust', 'node': 'Node ' + data['metadata']['tools']['node'], 'chromium': 'Chromium headless shell'}[runtime])
    ax.grid(axis='x', alpha=0.2)
    ax.set_axisbelow(True)
axes[0].legend(loc='lower right')
fig.suptitle('Reversible u32: equal input domain, different word formats', fontsize=14)
fig.supxlabel(f"Compare alternatives within each panel. Points: median of process medians; whiskers: IQR.\n{data['metadata']['host']['machine']}; {data['metadata']['rounds']} fresh processes per cell; warm batches.", fontsize=9)
args.output.parent.mkdir(parents=True, exist_ok=True)
fig.savefig(args.output, bbox_inches='tight', metadata={'Creator': 'nwords benchmark, matplotlib ' + matplotlib.__version__})
plt.close(fig)
print(args.output)
