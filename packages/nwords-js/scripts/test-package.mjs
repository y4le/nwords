import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { gzipSync } from 'node:zlib';
import { cp, mkdtemp, readFile, readdir, rm, writeFile } from 'node:fs/promises';
import { createServer } from 'node:http';
import { tmpdir } from 'node:os';
import { extname, join, resolve, sep } from 'node:path';
import { fileURLToPath } from 'node:url';
import { pathToFileURL } from 'node:url';
import { chromium } from 'playwright';

const root = fileURLToPath(new URL('../../../', import.meta.url));
const project = fileURLToPath(new URL('../', import.meta.url));
const dev = process.argv.slice(2).includes('--dev');
assert(process.argv.slice(2).every(arg => arg === '--dev'), 'Usage: npm test -- [--dev]');
const report = JSON.parse(await readFile(join(root, 'dist/package-build.json'), 'utf8'));
assert(dev || (!report.dirty && !report.development), 'Qualification requires a clean build');
const hash = bytes => createHash('sha256').update(bytes).digest('hex');
async function assetSizes(directory) {
  const names = (await readdir(directory, { recursive: true })).filter(name => /\.(js|wasm|css)$/u.test(name)).sort();
  return Promise.all(names.map(async name => {
    const bytes = await readFile(join(directory, name));
    return { name, bytes: bytes.length, gzipBytes: gzipSync(bytes, { level: 9 }).length };
  }));
}
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
  for (const name of ['./node', './web', './variable/node', './variable/web', './variable/wasm', './bip39/node', './bip39/web', './bip39/wasm', './wasm']) {
    assert(manifest.exports[name], `Missing public export ${name}`);
  }
  assert.equal(JSON.parse(await readFile(join(packed, 'build.json'), 'utf8')).sourceCommit, report.sourceCommit);
  const wasm = await readFile(join(packed, 'wasm/nwords_js_bg.wasm'));
  assert.equal(hash(wasm), report.wasm.sha256);
  const bip39Wasm = await readFile(join(packed, 'wasm/bip39/nwords_js_bip39_bg.wasm'));
  assert.equal(hash(bip39Wasm), report.bip39Wasm.sha256);
  const variableWasm = await readFile(join(packed, 'wasm/variable/nwords_js_variable_bg.wasm'));
  assert.equal(hash(variableWasm), report.variableWasm.sha256);
  const snapshots = {
    adjective: 'adjective-animal/nwords-adjectives.txt', animal: 'adjective-animal/nwords-animals.txt',
    color: 'adjective-animal/unique-names-generator-colors.txt', object: 'friendly-words/nwords-objects.txt',
    descriptor: 'friendly-words/nwords-descriptors.txt', mood: 'semantic-wordlists/mood/nwords-moods.txt',
    material: 'semantic-wordlists/material/nwords-materials.txt', shape: 'semantic-wordlists/shape/nwords-shapes.txt',
    weather: 'semantic-wordlists/weather/nwords-weather.txt', plant: 'semantic-wordlists/plant/nwords-plants.txt',
    food: 'semantic-wordlists/food/nwords-foods.txt', 'eff-long': 'eff-long/words.txt',
  };
  const wordsetNames = Object.keys(snapshots).sort();
  assert.deepEqual((await readdir(join(project, 'src/wordsets'))).filter(name => name.endsWith('.js')).map(name => name.slice(0, -3)).sort(), wordsetNames);
  assert.deepEqual(Object.keys(manifest.exports).filter(name => name.startsWith('./wordsets/')).map(name => name.slice(11)).sort(), wordsetNames);
  for (const [name, source] of Object.entries(snapshots)) {
    const identifier = name.replace(/-([a-z])/g, (_, letter) => letter.toUpperCase());
    const module = await import(pathToFileURL(join(packed, 'src/wordsets', `${name}.js`)).href);
    const expected = (await readFile(join(root, 'tests/vectors', source), 'utf8')).trimEnd().split('\n');
    assert.equal(module[identifier]?.name, name, `${name} canonical name`);
    assert.deepEqual(module[identifier]?.words, expected, `${name} encoding order`);
  }
  for (const name of ['LICENSE-MIT', 'LICENSE-APACHE', 'NOTICE.md', 'notices/rust-1.94.0-stdlib.txt', 'notices/dependencies.json', 'notices/wordlists/licenses/unique-names-generator-MIT.LICENSE', 'notices/wordlists/licenses/glitch-friendly-words-MIT.LICENSE', 'notices/wordlists/licenses/eff-CC-BY-4.0.LICENSE', 'notices/wordlists/eff-long/README.md', 'notices/wordlists/licenses/python-mnemonic-MIT.LICENSE', 'examples/node.mjs', 'examples/browser.html']) assert((await readFile(join(packed, name))).length > 0, name);
  for (const file of ['consumer.mjs', 'contract.mjs', 'bip39.mjs', 'codec.mjs', 'types.ts']) await cp(join(project, 'test', file), join(temporary, file));
  await cp(join(root, 'tests/vectors/bip39/trezor-python-mnemonic-vectors.json'), join(temporary, 'bip39-vectors.json'));
  await cp(join(root, 'tests/vectors/js/named-shapes.tsv'), join(temporary, 'vectors.tsv'));
  await cp(join(root, 'tests/vectors/variable/variable-v1.tsv'), join(temporary, 'variable-vectors.tsv'));
  const isolated = { env: { ...process.env, PATH: join(temporary, 'no-tools') } };
  for (const mode of ['contract', 'bytes', 'no-web-globals']) run(process.execPath, [join(temporary, 'consumer.mjs'), mode], isolated);
  run(process.execPath, [join(temporary, 'bip39.mjs')], isolated);
  run(process.execPath, [join(temporary, 'codec.mjs')], isolated);
  const namesRoot = join(temporary, 'bundle-names');
  await cp(join(project, 'test/consumers/names-only'), namesRoot, { recursive: true });
  run(process.execPath, [join(project, 'node_modules/vite/bin/vite.js'), 'build', namesRoot, '--base', './', '--logLevel', 'error']);
  const namesDist = join(namesRoot, 'dist');
  const namesFiles = await readdir(namesDist, { recursive: true });
  const namesWasm = namesFiles.filter(name => name.endsWith('.wasm'));
  assert.equal(namesWasm.length, 1, 'Names-only bundle emits one WASM asset');
  assert.equal(hash(await readFile(join(namesDist, namesWasm[0]))), report.variableWasm.sha256);
  const namesJs = (await Promise.all(namesFiles.filter(name => name.endsWith('.js')).map(name => readFile(join(namesDist, name), 'utf8')))).join('\n');
  assert(namesJs.includes('aardvark') && namesJs.includes('cardinal'), 'Names-only bundle includes selected words');
  assert(!namesJs.includes('abacus') && !namesJs.includes('abandon'), 'Names-only bundle excludes EFF and BIP-39 lists');
  const bundleRoot = join(temporary, 'bundle-bip39');
  await cp(join(project, 'test/consumers/bip39-only'), bundleRoot, { recursive: true });
  run(process.execPath, [join(project, 'node_modules/vite/bin/vite.js'), 'build', bundleRoot, '--base', './', '--logLevel', 'error']);
  const bundleDist = join(bundleRoot, 'dist');
  const bundleFiles = (await readdir(bundleDist, { recursive: true })).filter(name => name.endsWith('.wasm'));
  assert.equal(bundleFiles.length, 1, 'BIP-39-only production bundle emits one WASM asset');
  assert.equal(hash(await readFile(join(bundleDist, bundleFiles[0]))), report.bip39Wasm.sha256);
  for (const file of (await readdir(bundleDist, { recursive: true })).filter(name => name.endsWith('.js'))) {
    assert(!(await readFile(join(bundleDist, file))).includes(Buffer.from('aardvark')), 'Naming wordsets must be absent from BIP-39 bundle');
  }
  await writeFile(join(temporary, 'consumer.cjs'), "const assert = require('node:assert/strict'); import('@y4le/nwords/node').then(async ({loadNwords}) => { assert.equal((await loadNwords()).encodeId(42n, {lists:['adjective','animal']}), 'able cardinal'); }).catch(error => { console.error(error); process.exitCode = 1; });\n");
  run(process.execPath, [join(temporary, 'consumer.cjs')], isolated);
  const cold = Array.from({ length: 5 }, () => JSON.parse(run(process.execPath, [join(temporary, 'consumer.mjs'), 'measure'], isolated)));
  for (const [module, resolution] of [['NodeNext', 'NodeNext'], ['ESNext', 'Bundler']]) run(process.execPath, [join(project, 'node_modules/typescript/bin/tsc'), '--noEmit', '--strict', '--skipLibCheck', 'false', '--target', 'ES2022', '--module', module, '--moduleResolution', resolution, '--lib', 'ES2022,DOM', '--types', 'node', '--typeRoots', join(project, 'node_modules/@types'), 'types.ts']);
  run(process.execPath, [join(project, 'scripts/build-site.mjs')], { cwd: root });
  const siteDist = join(root, 'dist/site');
  const siteFiles = await readdir(siteDist, { recursive: true });
  const siteWasm = siteFiles.filter(name => name.endsWith('.wasm'));
  assert.equal(siteWasm.length, 2, 'Combined site emits Names and BIP-39 WASM');
  assert.deepEqual((await Promise.all(siteWasm.map(async name => hash(await readFile(join(siteDist, name)))))).sort(),
    [report.bip39Wasm.sha256, report.variableWasm.sha256].sort());
  let requests = 0;
  let failBip39Once = false;
  let failureGate;
  let namesGate;
  server = createServer(async (request, response) => {
    try {
      const path = new URL(request.url, 'http://localhost').pathname;
      if (path === '/') { response.setHeader('Content-Type', 'text/html'); response.end('<!doctype html><title>packed consumer</title>'); return; }
      if (path.endsWith('.wasm')) requests++;
      if (namesGate && path.includes('nwords_js_variable_bg') && path.endsWith('.wasm')) {
        const gate = namesGate;
        namesGate = undefined;
        gate.started();
        await gate.release;
      }
      if (failBip39Once && path.includes('nwords_js_bip39_bg') && path.endsWith('.wasm')) {
        failBip39Once = false;
        const gate = failureGate;
        if (gate) { gate.started(); await gate.release; }
        else await new Promise(resolve => setTimeout(resolve, 150));
        response.writeHead(404); response.end(); return;
      }
      if (path === '/missing.wasm') { response.writeHead(404); response.end(); return; }
      if (path === '/corrupt.wasm' || path === '/empty.wasm') { response.setHeader('Content-Type', 'application/wasm'); response.end(path === '/corrupt.wasm' ? Buffer.from([0, 1, 2]) : Buffer.from([0, 97, 115, 109, 1, 0, 0, 0])); return; }
      let base;
      let relative;
      if (path.startsWith('/package/')) { base = packed; relative = path.slice(9); }
      else if (path.startsWith('/demo/')) { base = siteDist; relative = path.slice(6) || 'index.html'; }
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
  const coldVariableBrowserSamples = [];
  for (let sample = 0; sample < 5; sample++) {
    const page = await browser.newPage();
    await page.goto(origin);
    const timing = await page.evaluate(async () => {
      const start = performance.now();
      const [{ loadVariable, NwordsError }, { adjective }, { animal }] = await Promise.all([
        import('/package/src/variable-wasm/web.js'),
        import('/package/src/wordsets/adjective.js'),
        import('/package/src/wordsets/animal.js'),
      ]);
      const imported = performance.now();
      const { defineVariable } = await loadVariable();
      const loaded = performance.now();
      const codec = defineVariable({ scheme: 'variable-v1', pattern: [{ list: adjective, repeat: { min: 1 } }, animal], maxWords: 32 });
      if (codec.encodeId(42n) !== 'able cardinal' || codec.decodePhrase('able cardinal') !== 42n ||
          codec.decodeText(codec.encodeText('café 🦊')) !== 'café 🦊') throw new Error('Slim browser codec round trip failed');
      let invalid = false;
      try { codec.decodePhrase('able unicorn'); } catch (error) { invalid = error instanceof NwordsError && error.code === 'INVALID_PHRASE' && error.position === 1; }
      if (!invalid) throw new Error('Slim browser codec error metadata failed');
      return { importMs: imported - start, initializationMs: loaded - imported, totalMs: performance.now() - start };
    });
    coldVariableBrowserSamples.push(timing);
    await page.close();
  }
  const namesPage = await browser.newPage();
  const namesRequests = [];
  namesPage.on('request', request => namesRequests.push(new URL(request.url()).pathname));
  await namesPage.goto(origin + '/bundle-names/dist/index.html');
  await namesPage.waitForFunction(() => document.querySelector('#phrase')?.textContent === 'able cardinal');
  assert.equal(namesRequests.filter(path => path.endsWith('.wasm')).length, 1, 'Bundled Names requests one WASM asset');
  assert(namesRequests.every(path => path.startsWith('/bundle-names/dist/')), 'Names bundle only requests its own assets');
  await namesPage.close();
  const vectors = (await readFile(join(temporary, 'vectors.tsv'), 'utf8')).trim().split('\n').filter(row => !row.startsWith('#')).map(row => row.split('\t'));
  const variableVectors = (await readFile(join(temporary, 'variable-vectors.tsv'), 'utf8')).trim().split('\n').filter(row => !row.startsWith('#')).map(row => row.split('\t'));
  const warnings = [];
  for (const mode of ['contract', 'retry', 'bytes']) {
    const page = await browser.newPage();
    page.on('console', message => { if (message.type() === 'warning') warnings.push(message.text()); });
    await page.goto(origin);
    const beforeImport = requests;
    await page.evaluate(() => import('/package/src/web.js'));
    assert.equal(requests, beforeImport, 'Import must not fetch WASM');
    await page.evaluate(async ({ mode, vectors, variableVectors }) => {
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
      verifyApi(api, NwordsError, vectors, variableVectors);
    }, { mode, vectors, variableVectors });
    await page.close();
  }
  assert.deepEqual(warnings, [], 'Generated initialization must not emit deprecated-argument warnings');
  const failedMnemonicPage = await browser.newPage();
  await failedMnemonicPage.goto(origin);
  failBip39Once = true;
  await failedMnemonicPage.evaluate(async () => {
    const { loadBip39, NwordsLoadError } = await import('/package/src/bip39/web.js');
    let rejected = false;
    try { await loadBip39(); } catch (error) { rejected = error instanceof NwordsLoadError && error.code === 'LOAD_FAILED'; }
    if (!rejected) throw new Error('Expected BIP-39 load failure');
    if ((await loadBip39()).encodeEntropy(new Uint8Array(16)) !== 'abandon '.repeat(11) + 'about') {
      throw new Error('BIP-39 retry failed');
    }
  });
  await failedMnemonicPage.close();
  const mnemonicPage = await browser.newPage();
  await mnemonicPage.goto(origin);
  const beforeBip39 = requests;
  await mnemonicPage.evaluate(() => import('/package/src/bip39/web.js'));
  assert.equal(requests, beforeBip39, 'Importing BIP-39 must not fetch WASM');
  await mnemonicPage.evaluate(async () => {
    const { loadBip39 } = await import('/package/src/bip39/web.js');
    const codec = await loadBip39();
    const phrase = codec.encodeEntropy(new Uint8Array(16));
    if (phrase !== 'abandon '.repeat(11) + 'about' || codec.decodeMnemonic(phrase).length !== 16) {
      throw new Error('Browser BIP-39 vector mismatch');
    }
  });
  assert.equal(requests, beforeBip39 + 1, 'BIP-39 loads one WASM asset');
  await mnemonicPage.close();
  const bundlePage = await browser.newPage();
  const bundleRequests = [];
  bundlePage.on('request', request => bundleRequests.push(new URL(request.url()).pathname));
  await bundlePage.goto(origin + '/bundle-bip39/dist/index.html');
  await bundlePage.waitForFunction(() => document.querySelector('#phrase')?.textContent?.endsWith('about'));
  assert.equal(await bundlePage.locator('#phrase').textContent(), 'abandon '.repeat(11) + 'about');
  assert.equal(bundleRequests.filter(path => path.endsWith('.wasm')).length, 1);
  assert(bundleRequests.every(path => path.startsWith('/bundle-bip39/dist/')), 'Bundle only requests its own assets');
  await bundlePage.close();
  const namesStarted = Promise.withResolvers();
  const namesReleased = Promise.withResolvers();
  namesGate = { started: namesStarted.resolve, release: namesReleased.promise };
  const pendingNamesPage = await browser.newPage();
  try {
    await pendingNamesPage.goto(origin + '/demo/');
    await namesStarted.promise;
    await pendingNamesPage.fill('#names-phrase', 'able aardvark');
  } finally { namesReleased.resolve(); namesGate = undefined; }
  await pendingNamesPage.waitForFunction(() => document.querySelector('#names-value')?.value === '0');
  await pendingNamesPage.close();
  const failedDemo = await browser.newPage();
  const pageErrors = [];
  failedDemo.on('pageerror', error => pageErrors.push(error.message));
  await failedDemo.goto(origin + '/demo/');
  const [typedEntropy, typedMnemonic] = JSON.parse(await readFile(join(temporary, 'bip39-vectors.json'), 'utf8')).english[1];
  const started = Promise.withResolvers();
  const released = Promise.withResolvers();
  failureGate = { started: started.resolve, release: released.promise };
  failBip39Once = true;
  try {
    await failedDemo.click('#bitcoin-tab');
    let timer;
    try {
      await Promise.race([started.promise, new Promise((_, reject) => { timer = setTimeout(() => reject(new Error('BIP-39 asset request did not start')), 5000); })]);
    } finally { clearTimeout(timer); }
    await failedDemo.fill('#entropy-value', typedEntropy);
  } finally { released.resolve(); failureGate = undefined; failBip39Once = false; }
  await failedDemo.waitForFunction(() => document.querySelector('#bitcoin-status')?.textContent?.includes('LOAD_FAILED'));
  assert.deepEqual(pageErrors, [], 'Typing during BIP-39 load failure creates no unhandled rejection');
  await failedDemo.click('#names-tab');
  await failedDemo.click('#bitcoin-tab');
  await failedDemo.waitForFunction(expected => document.querySelector('#mnemonic-value')?.value === expected, typedMnemonic);
  assert.equal(await failedDemo.inputValue('#entropy-value'), typedEntropy);
  await failedDemo.close();
  const demo = await browser.newPage({ viewport: { width: 390, height: 844 } });
  const demoRequests = [];
  demo.on('request', request => demoRequests.push(request.url()));
  await demo.goto(origin + '/demo/');
  await demo.waitForFunction(() => document.querySelector('#names-phrase')?.value === 'able cardinal');
  assert.equal(demoRequests.filter(url => url.endsWith('.wasm')).length, 1, 'Opening Names fetches only Names WASM');
  await demo.fill('#names-phrase', 'able cardinal');
  assert.equal(await demo.inputValue('#names-value'), '42');
  await demo.selectOption('#names-view', 'bits');
  assert.equal(await demo.inputValue('#names-value'), '01011');
  await demo.fill('#names-value', '00000000');
  assert.equal(await demo.inputValue('#names-phrase'), 'able rooster');
  await demo.selectOption('#names-view', 'text');
  await demo.fill('#names-value', 'hello');
  await demo.fill('#names-phrase', await demo.inputValue('#names-phrase'));
  assert.equal(await demo.inputValue('#names-value'), 'hello');
  await demo.click('#bitcoin-tab');
  await demo.waitForFunction(() => document.querySelector('#mnemonic-value')?.value.endsWith('about'));
  assert.equal(await demo.inputValue('#mnemonic-value'), 'abandon '.repeat(11) + 'about');
  assert.equal(demoRequests.filter(url => url.endsWith('.wasm')).length, 2);
  await demo.fill('#mnemonic-value', 'abandon '.repeat(12).trim());
  assert.match(await demo.locator('#bitcoin-status').textContent(), /INVALID_CHECKSUM/);
  assert.equal(await demo.evaluate(() => location.search + location.hash), '');
  assert.deepEqual(await demo.evaluate(() => [localStorage.length, sessionStorage.length]), [0, 0]);
  assert(demoRequests.every(url => url.startsWith(origin + '/demo/')), 'Site makes no third-party requests');
  const qualification = { schema: 'nwords.qualification.v1', sourceCommit: report.sourceCommit, dirty: report.dirty, development: report.development, tarballSha256: report.tarballSha256, wasm: report.wasm, bip39Wasm: report.bip39Wasm, variableWasm: report.variableWasm, packedBytes: report.packedBytes, unpackedBytes: report.unpackedBytes, bundles: { namesOnly: await assetSizes(namesDist), bip39Only: await assetSizes(bundleDist), site: await assetSizes(siteDist) }, runtime: { node: process.version, platform: process.platform, arch: process.arch, chromium: browser.version() }, checks: ['Node loader without lazy web globals', 'installed ESM', 'CJS dynamic import', 'tool-free consumers', 'direct WASM ABI', 'NodeNext and Bundler declarations', 'Chromium contract and loader recovery', 'English BIP-39 reference vectors in packed Node and Chromium', 'variable-v1 parity and wide bit views', 'names-only Vite bundle', 'BIP-39-only Vite bundle', 'combined site on narrow viewport'], coldNodeSamples: cold, coldVariableBrowserSamples };
  assertSource();
  await writeFile(join(root, 'dist/package-qualification.json'), JSON.stringify(qualification, null, 2) + '\n');
  console.log(JSON.stringify(qualification, null, 2));
} finally {
  await browser?.close();
  if (server) await new Promise(resolve => server.close(resolve));
  await rm(temporary, { recursive: true, force: true });
}
