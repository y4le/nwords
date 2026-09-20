import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { cp, mkdtemp, readFile, rm, writeFile } from 'node:fs/promises';
import { createServer } from 'node:http';
import { tmpdir } from 'node:os';
import { extname, join, resolve, sep } from 'node:path';
import { fileURLToPath } from 'node:url';
import { chromium } from 'playwright';

const root = fileURLToPath(new URL('../../../', import.meta.url));
const project = fileURLToPath(new URL('../', import.meta.url));
const dev = process.argv.slice(2).includes('--dev');
assert(process.argv.slice(2).every(arg => arg === '--dev'), 'Usage: npm test -- [--dev]');
const report = JSON.parse(await readFile(join(root, 'dist/package-build.json'), 'utf8'));
assert(dev || (!report.dirty && !report.development), 'Qualification requires a clean build');
const hash = bytes => createHash('sha256').update(bytes).digest('hex');
assert.equal(hash(await readFile(report.tarball)), report.tarballSha256);
const assertSource = () => {
  if (dev) return;
  const git = args => execFileSync('git', args, { cwd: root, encoding: 'utf8' }).trim();
  assert.equal(git(['rev-parse', 'HEAD']), report.sourceCommit, 'Rebuild the package after changing HEAD');
  assert.equal(git(['status', '--porcelain=v1', '--untracked-files=all']), '', 'Qualification tests require a clean tree');
};
assertSource();
await rm(join(root, 'dist/package-qualification.json'), { force: true });
const temporary = await mkdtemp(join(tmpdir(), 'nwords-packed-'));
let server;
let browser;
const run = (command, args, options = {}) => execFileSync(command, args, { cwd: temporary, encoding: 'utf8', stdio: ['ignore', 'pipe', 'inherit'], ...options }).trim();
try {
  await writeFile(join(temporary, 'package.json'), JSON.stringify({ private: true, type: 'module' }));
  run('npm', ['install', '--ignore-scripts', '--no-audit', '--no-fund', report.tarball]);
  const packed = join(temporary, 'node_modules/@y4le/nwords');
  const manifest = JSON.parse(await readFile(join(packed, 'package.json'), 'utf8'));
  assert.equal(manifest.private, true); assert.equal(manifest.scripts, undefined);
  assert.deepEqual(Object.keys(manifest.exports), ['./node', './web', './wasm']);
  assert.equal(JSON.parse(await readFile(join(packed, 'build.json'), 'utf8')).sourceCommit, report.sourceCommit);
  const wasm = await readFile(join(packed, 'wasm/nwords_js_bg.wasm'));
  assert.equal(hash(wasm), report.wasm.sha256);
  for (const name of ['LICENSE-MIT', 'LICENSE-APACHE', 'NOTICE.md', 'notices/rust-1.94.0-stdlib.txt', 'notices/dependencies.json', 'notices/wordlists/licenses/unique-names-generator-MIT.LICENSE', 'notices/wordlists/licenses/glitch-friendly-words-MIT.LICENSE', 'examples/node.mjs', 'examples/browser.html']) assert((await readFile(join(packed, name))).length > 0, name);
  for (const file of ['consumer.mjs', 'contract.mjs', 'types.ts']) await cp(join(project, 'test', file), join(temporary, file));
  await cp(join(root, 'tests/vectors/js/named-shapes.tsv'), join(temporary, 'vectors.tsv'));
  const isolated = { env: { ...process.env, PATH: join(temporary, 'no-tools') } };
  for (const mode of ['contract', 'bytes', 'no-web-globals']) run(process.execPath, [join(temporary, 'consumer.mjs'), mode], isolated);
  await writeFile(join(temporary, 'consumer.cjs'), "const assert = require('node:assert/strict'); import('@y4le/nwords/node').then(async ({loadNwords}) => { assert.equal((await loadNwords()).encodeId(42n, {lists:['adjective','animal']}), 'able cardinal'); }).catch(error => { console.error(error); process.exitCode = 1; });\n");
  run(process.execPath, [join(temporary, 'consumer.cjs')], isolated);
  const cold = Array.from({ length: 5 }, () => JSON.parse(run(process.execPath, [join(temporary, 'consumer.mjs'), 'measure'], isolated)));
  for (const [module, resolution] of [['NodeNext', 'NodeNext'], ['ESNext', 'Bundler']]) run(process.execPath, [join(project, 'node_modules/typescript/bin/tsc'), '--noEmit', '--strict', '--skipLibCheck', 'false', '--target', 'ES2022', '--module', module, '--moduleResolution', resolution, '--lib', 'ES2022,DOM', '--types', 'node', '--typeRoots', join(project, 'node_modules/@types'), 'types.ts']);
  run('wasm-pack', ['build', 'crates/nwords-web', '--target', 'web', '--release', '--no-opt', '--no-pack', '--out-dir', '../../site/pkg', '--locked'], { cwd: root, env: { ...process.env, RUSTUP_TOOLCHAIN: '1.94.0' } });
  let requests = 0;
  server = createServer(async (request, response) => {
    try {
      const path = new URL(request.url, 'http://localhost').pathname;
      if (path === '/') { response.setHeader('Content-Type', 'text/html'); response.end('<!doctype html><title>packed consumer</title>'); return; }
      if (path.endsWith('.wasm')) requests++;
      if (path === '/missing.wasm') { response.writeHead(404); response.end(); return; }
      if (path === '/corrupt.wasm' || path === '/empty.wasm') { response.setHeader('Content-Type', 'application/wasm'); response.end(path === '/corrupt.wasm' ? Buffer.from([0, 1, 2]) : Buffer.from([0, 97, 115, 109, 1, 0, 0, 0])); return; }
      let base;
      let relative;
      if (path.startsWith('/package/')) { base = packed; relative = path.slice(9); }
      else if (path.startsWith('/demo/')) { base = join(root, 'site'); relative = path.slice(6) || 'index.html'; }
      else { base = temporary; relative = path.slice(1); }
      const file = resolve(base, relative);
      if (!file.startsWith(base + sep)) throw new Error('Invalid path');
      response.setHeader('Content-Type', ({ '.js': 'text/javascript', '.mjs': 'text/javascript', '.wasm': 'application/wasm', '.html': 'text/html', '.css': 'text/css' })[extname(file)] ?? 'text/plain');
      response.end(await readFile(file));
    } catch { response.writeHead(404); response.end(); }
  });
  await new Promise(resolve => server.listen(0, '127.0.0.1', resolve));
  const origin = `http://127.0.0.1:${server.address().port}`;
  browser = await chromium.launch({ headless: true });
  const vectors = (await readFile(join(temporary, 'vectors.tsv'), 'utf8')).trim().split('\n').filter(row => !row.startsWith('#')).map(row => row.split('\t'));
  const warnings = [];
  for (const mode of ['contract', 'retry', 'bytes']) {
    const page = await browser.newPage();
    page.on('console', message => { if (message.type() === 'warning') warnings.push(message.text()); });
    await page.goto(origin);
    const beforeImport = requests;
    await page.evaluate(() => import('/package/src/web.js'));
    assert.equal(requests, beforeImport, 'Import must not fetch WASM');
    await page.evaluate(async ({ mode, vectors }) => {
      const { loadNwords, NwordsError, NwordsLoadError } = await import('/package/src/web.js');
      const { check, verifyApi } = await import('/contract.mjs');
      const fails = async (promise, code) => {
        let error; try { await promise; } catch (caught) { error = caught; }
        check(error instanceof NwordsLoadError && error.code === code, `Expected loader error ${code}`);
      };
      const asset = new URL('/package/wasm/nwords_js_bg.wasm', location.href);
      let api;
      if (mode === 'retry') {
        for (const source of ['/missing.wasm', '/corrupt.wasm', '/empty.wasm']) await fails(loadNwords({ source }), 'LOAD_FAILED');
        api = await loadNwords({ source: asset });
        check(await loadNwords({ source: asset.href }) === api, 'URL/string identity');
      } else if (mode === 'bytes') {
        const bytes = new Uint8Array(await (await fetch(asset)).arrayBuffer());
        const copied = new Uint8Array(bytes);
        const pending = loadNwords({ source: bytes });
        bytes.fill(0);
        await fails(loadNwords({ source: copied }), 'ASSET_CONFLICT');
        api = await pending;
        check(await loadNwords({ source: bytes }) === api, 'Byte object identity');
      } else {
        const apis = await Promise.all(Array.from({ length: 8 }, () => loadNwords()));
        api = apis[0]; check(apis.every(value => value === api), 'Concurrent initialization');
      }
      check(await loadNwords() === api, 'Omitted source reuses instance');
      await fails(loadNwords({ source: '/different.wasm' }), 'ASSET_CONFLICT');
      verifyApi(api, NwordsError, vectors);
    }, { mode, vectors });
    await page.close();
  }
  assert.deepEqual(warnings, [], 'Generated initialization must not emit deprecated-argument warnings');
  const demo = await browser.newPage();
  await demo.goto(origin + '/demo/');
  await demo.waitForFunction(() => document.querySelector('#encode-button')?.disabled === false);
  await demo.selectOption('#preset-select', 'dec6');
  await demo.fill('#id-input', '42');
  await demo.click('#encode-button');
  assert.equal(await demo.inputValue('#phrase-input'), 'abandon aim');
  await demo.click('#decode-button');
  assert.equal(await demo.locator('#id-output strong').first().textContent(), '42');
  await demo.fill('#text-input', 'hello');
  await demo.click('#text-encode-button');
  await demo.fill('#text-input', '');
  await demo.click('#text-decode-button');
  assert.equal(await demo.inputValue('#text-input'), 'hello');
  const qualification = { schema: 'nwords.qualification.v1', sourceCommit: report.sourceCommit, dirty: report.dirty, development: report.development, tarballSha256: report.tarballSha256, wasm: report.wasm, packedBytes: report.packedBytes, unpackedBytes: report.unpackedBytes, runtime: { node: process.version, platform: process.platform, arch: process.arch, chromium: browser.version() }, checks: ['Node loader without lazy web globals', 'installed ESM', 'CJS dynamic import', 'tool-free consumers', 'direct WASM ABI', 'NodeNext and Bundler declarations', 'Chromium contract and loader recovery', 'existing demo'], coldNodeSamples: cold };
  assertSource();
  await writeFile(join(root, 'dist/package-qualification.json'), JSON.stringify(qualification, null, 2) + '\n');
  console.log(JSON.stringify(qualification, null, 2));
} finally {
  await browser?.close();
  if (server) await new Promise(resolve => server.close(resolve));
  await rm(temporary, { recursive: true, force: true });
}
