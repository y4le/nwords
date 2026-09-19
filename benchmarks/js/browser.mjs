import { createServer } from 'node:http';
import { readFile } from 'node:fs/promises';
import { resolve, extname, sep } from 'node:path';
import { fileURLToPath } from 'node:url';
import { chromium } from 'playwright';
const [packagePath, caseName, roundArg = '0', duration = '100'] = process.argv.slice(2);
const project = fileURLToPath(new URL('./', import.meta.url));
const dictionaries = await Promise.all(['adjectives', 'animals'].map(async name => (await readFile(new URL(`../../tests/vectors/adjective-animal/nwords-${name}.txt`, import.meta.url), 'utf8')).trim().split('\n')));
const values = (await readFile(new URL('../ids-u32.txt', import.meta.url), 'utf8')).trim().split('\n').map(Number);
const nativePhrases = (await readFile(new URL('../.work/native-phrases.tsv', import.meta.url), 'utf8')).trim().split('\n').map(line => line.split('\t')[1]);
const files = [];
const server = createServer(async (request, response) => {
  try {
    const path = new URL(request.url, 'http://localhost').pathname;
    if (path === '/') { response.setHeader('content-type', 'text/html'); response.end('<!doctype html><title>Benchmark</title><script src="/deps/niceware/browser/niceware.js"></script>'); return; }
    const base = path.startsWith('/package/') ? resolve(packagePath) : path.startsWith('/deps/') ? resolve(project, 'node_modules') : resolve(project);
    const relative = path.startsWith('/package/') ? path.slice(9) : path.startsWith('/deps/') ? path.slice(6) : path.slice(1);
    const file = resolve(base, relative);
    if (!file.startsWith(base + sep)) throw new Error('Invalid path');
    const bytes = await readFile(file);
    files.push({ path, bytes: bytes.length });
    response.setHeader('content-type', ({ '.js': 'text/javascript', '.mjs': 'text/javascript', '.wasm': 'application/wasm' })[extname(file)] ?? 'text/plain');
    response.end(bytes);
  } catch { response.writeHead(404); response.end(); }
});
let browser;
try {
  await new Promise(resolve => server.listen(0, '127.0.0.1', resolve));
  browser = await chromium.launch({ headless: true });
  const page = await browser.newPage();
  await page.goto(`http://127.0.0.1:${server.address().port}`);
  const result = await page.evaluate(async ({ dictionaries, values, nativePhrases, caseName, round, targetMs }) => {
    const { loadNwords } = await import('/package/src/web.js');
    const ung = await import('/deps/unique-names-generator/dist/index.m.js');
    const { workloads, measure, sink } = await import('/suite.mjs');
    const words = await loadNwords();
    const niceware = window.niceware;
    // The published browser bundle carries its own Buffer polyfill.
    const Buffer = niceware.passphraseToBytes([]).constructor;
    const { cases, metadata } = workloads({ words, ung, niceware, Buffer, dictionaries, values, nativePhrases });
    const selected = cases.filter(([name]) => name === caseName);
    if (selected.length !== 1) throw new Error('Unknown case');
    return { metadata, samples: measure(selected, 'chromium', round, targetMs), sink: sink() };
  }, { dictionaries, values, nativePhrases, caseName, round: Number(roundArg), targetMs: Number(duration) });
  console.log(JSON.stringify({ kind: 'metadata', runtime: 'chromium', round: Number(roundArg), version: browser.version(), assets: files, ...result.metadata }));
  for (const sample of result.samples) console.log(JSON.stringify(sample));
  console.log(JSON.stringify({ kind: 'sink', runtime: 'chromium', value: result.sink }));
} finally {
  await browser?.close();
  await new Promise(resolve => server.close(resolve));
}
