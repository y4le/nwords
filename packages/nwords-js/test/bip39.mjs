import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import { loadBip39, NwordsError } from '@y4le/nwords/bip39/node';

const vectors = JSON.parse(await readFile(new URL('./bip39-vectors.json', import.meta.url), 'utf8')).english;
const bip39 = await loadBip39();
assert.equal(await loadBip39(), bip39);
for (const [hex, phrase] of vectors) {
  const entropy = Uint8Array.from(Buffer.from(hex, 'hex'));
  assert.equal(bip39.encodeEntropy(entropy), phrase);
  assert.equal(Buffer.from(bip39.decodeMnemonic(phrase)).toString('hex'), hex);
}
const first = vectors[0][1];
const fails = (fn, code, field, position) => assert.throws(fn, error =>
  error instanceof NwordsError && error.code === code && error.field === field &&
  (position === undefined || error.position === position) &&
  !error.message.includes('missing'));
fails(() => bip39.encodeEntropy(new Uint8Array(15)), 'INVALID_ENTROPY_LENGTH', 'entropy');
fails(() => bip39.decodeMnemonic('abandon'), 'INVALID_WORD_COUNT', 'mnemonic');
fails(() => bip39.decodeMnemonic(42), 'INVALID_INPUT', 'mnemonic');
fails(() => bip39.decodeMnemonic('\ud800'), 'INVALID_INPUT', 'mnemonic');
fails(() => bip39.decodeMnemonic('a'.repeat(4097)), 'INVALID_INPUT', 'mnemonic');
fails(() => bip39.decodeMnemonic(first.replace('about', 'missing')), 'UNKNOWN_WORD', 'mnemonic', 11);
fails(() => bip39.decodeMnemonic(first.replace('about', 'abandon')), 'INVALID_CHECKSUM', 'mnemonic');
