// Instrument the actual public loader in a fresh Node process. Normal cold
// timings remain authoritative; this probe adds observation overhead.
import { pathToFileURL } from 'node:url';
import { performance } from 'node:perf_hooks';

const base = pathToFileURL(process.argv[2] + '/');
const phases = {};
const counts = {};
function record(name, start) {
  phases[name] = (phases[name] ?? 0) + performance.now() - start;
  counts[name] = (counts[name] ?? 0) + 1;
}
async function timed(name, fn) {
  const start = performance.now();
  try { return await fn(); } finally { record(name, start); }
}
// Inspecting Node's lazy global property descriptors can itself initialize them.
// The normal probe leaves them untouched; a separate control can prewarm them.
const mode = process.argv[3] ?? 'normal';
if (!['normal', 'prewarm-web'].includes(mode)) throw new Error('Unknown probe mode');
if (mode === 'prewarm-web') await timed('webGlobals', () => [globalThis.Request, globalThis.Response]);
for (const name of ['compile', 'instantiate']) {
  const original = WebAssembly[name];
  WebAssembly[name] = (...args) => timed(name, () => original(...args));
}
const Instance = WebAssembly.Instance;
WebAssembly.Instance = new Proxy(Instance, {
  construct(target, args, newTarget) {
    const start = performance.now();
    try { return Reflect.construct(target, args, newTarget); }
    finally { record('Instance', start); }
  },
});
const { loadNwords } = await timed('publicImport', () => import(new URL('src/node.js', base)));
const api = await timed('load', () => loadNwords());
const phrase = await timed('firstOperation', () => api.encodeId(42n, { lists: ['adjective', 'animal'] }));
if (phrase !== 'able cardinal') throw new Error('Probe parity failure');
if (['publicImport', 'load', 'compile', 'firstOperation'].some(name => counts[name] !== 1) ||
    (counts.instantiate ?? 0) + (counts.Instance ?? 0) !== 1) throw new Error('Unexpected loader invocation counts');
console.log(JSON.stringify({ kind: 'startup-probe', runtime: 'node', mode, phases, counts }));
