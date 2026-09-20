import assert from 'node:assert/strict';
import { readFile, rename } from 'node:fs/promises';
import { spawnSync } from 'node:child_process';
import { verifyApi } from './contract.mjs';

const mode = process.argv[2];
for (const command of ['cargo', 'rustc', 'wasm-pack']) {
  assert.equal(spawnSync(command, ['--version']).error?.code, 'ENOENT', `${command} must be absent from consumer PATH`);
}
if (mode === 'no-web-globals') {
  for (const name of ['Request', 'Response']) {
    Object.defineProperty(globalThis, name, { configurable: true, get() { throw new Error(`Unexpected ${name} access`); } });
  }
}
const begin = performance.now();
const node = await import('@y4le/nwords/node');
const imported = performance.now();
const web = await import('@y4le/nwords/web');
const asset = new URL(import.meta.resolve('@y4le/nwords/wasm'));
const shape = { lists: ['adjective', 'animal'] };
const fails = async (promise, code) => assert.rejects(promise, error => error instanceof node.NwordsLoadError && error.code === code);

if (mode === 'measure') {
  const start = performance.now();
  const api = await node.loadNwords();
  const end = performance.now();
  assert.equal(api.encodeId(42n, shape), 'able cardinal');
  console.log(JSON.stringify({ importMs: imported - begin, initializationMs: end - start, totalMs: end - begin }));
} else if (mode === 'no-web-globals') {
  const api = await node.loadNwords();
  assert.equal(api.encodeId(42n, shape), 'able cardinal');
} else if (mode === 'bytes') {
  await Promise.all([
    fails(web.loadNwords({ source: new Uint8Array([0, 1, 2]) }), 'LOAD_FAILED'),
    fails(web.loadNwords(), 'LOAD_FAILED'),
    fails(node.loadNwords(), 'LOAD_FAILED'),
  ]);
  // Valid WASM with no exports reaches the generated finalizer before failing.
  await fails(web.loadNwords({ source: new Uint8Array([0, 97, 115, 109, 1, 0, 0, 0]) }), 'LOAD_FAILED');
  const bytes = new Uint8Array(await readFile(asset));
  const sameContents = new Uint8Array(bytes);
  const loading = web.loadNwords({ source: bytes });
  bytes.fill(0); // Our snapshot, not subsequent caller mutation, is instantiated.
  await fails(web.loadNwords({ source: sameContents }), 'ASSET_CONFLICT');
  const [api, concurrent, noSource] = await Promise.all([loading, web.loadNwords({ source: bytes }), node.loadNwords()]);
  assert.equal(api, concurrent); assert.equal(api, noSource);
  assert.equal(await web.loadNwords(), api);
  await fails(web.loadNwords({ source: asset }), 'ASSET_CONFLICT');
  assert.equal(api.encodeId(42n, shape), 'able cardinal');
} else if (mode === 'contract') {
  // Importing both entries succeeded without loading the asset. A missing-file
  // failure is recoverable in the same process after the file is restored.
  const moved = new URL(asset.href + '.missing');
  await rename(asset, moved);
  try { await fails(node.loadNwords(), 'LOAD_FAILED'); }
  finally { await rename(moved, asset); }
  const apis = await Promise.all([node.loadNwords(), ...Array.from({ length: 8 }, () => node.loadNwords()), web.loadNwords()]);
  for (const api of apis) assert.equal(api, apis[0]);
  assert.equal(node.NwordsError, web.NwordsError);
  assert.equal(await web.loadNwords({ source: asset.href }), apis[0]);
  await fails(web.loadNwords({ source: new URL('./different.wasm', asset) }), 'ASSET_CONFLICT');
  await fails(node.loadNwords({ source: asset }), 'LOAD_FAILED');
  const vectors = (await readFile(new URL('./vectors.tsv', import.meta.url), 'utf8')).trim().split('\n').filter(row => !row.startsWith('#')).map(row => row.split('\t'));
  verifyApi(apis[0], node.NwordsError, vectors);
  await assert.rejects(import('@y4le/nwords/src/api.js'), { code: 'ERR_PACKAGE_PATH_NOT_EXPORTED' });
  const raw = await import(new URL('./nwords_js.js', asset));
  await raw.default({ module_or_path: await readFile(asset) });
  for (const invalid of ['+1', '01', ' 1', '1 ', '1\n', '', '1e3', '-0', '-1', (1n << 128n).toString()]) {
    for (const call of [() => raw.encode_id(invalid, 'animal'), () => raw.encode_id('0', 'animal', invalid), () => raw.decode_phrase('aardvark', 'animal', invalid)]) {
      assert.throws(call, error => typeof error === 'string' && JSON.parse(error).error.code === 'INVALID_INPUT');
    }
    for (const json of [raw.encode_id_json(invalid, 'animal'), raw.describe_shape_json('animal', invalid), raw.decode_phrase_json('aardvark', 'animal', invalid)]) {
      const result = JSON.parse(json);
      assert.equal(result.ok, false); assert.equal(result.error.code, 'INVALID_INPUT');
    }
  }
  for (const [name, lists, range, id, phrase] of vectors) {
    assert.equal(JSON.parse(raw.encode_id_json(id, lists, range === '-' ? undefined : range)).value, phrase, name);
    assert.equal(raw.encode_id(id, lists, range === '-' ? undefined : range), phrase, name);
    assert.equal(raw.decode_phrase(phrase, lists, range === '-' ? undefined : range), BigInt(id), name);
    const prepared = new raw.PreparedCodec(lists, range === '-' ? undefined : range);
    try {
      assert.equal(prepared.encode_id(id), phrase, name);
      assert.equal(prepared.decode_phrase(phrase), BigInt(id), name);
      assert.throws(() => prepared.encode_id('-1'), error => typeof error === 'string' && JSON.parse(error).error.code === 'INVALID_INPUT');
    } finally { prepared.free(); }
  }
  assert.equal(JSON.parse(raw.describe_shape_json('animal,,color')).error.code, 'INVALID_SHAPE');
} else {
  throw new Error('Unknown consumer scenario');
}
