import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import { loadVariable, NwordsError } from '@y4le/nwords/variable/node';
import { adjective } from '@y4le/nwords/wordsets/adjective';
import { animal } from '@y4le/nwords/wordsets/animal';
import { effLong } from '@y4le/nwords/wordsets/eff-long';
import { loadNwords } from '@y4le/nwords/node';

const { defineVariable } = await loadVariable();
const slim = defineVariable({ scheme: 'variable-v1', pattern: [{ list: adjective, repeat: { min: 1 } }, animal], maxWords: 32 });
assert.equal(slim.encodeId(42n), 'able cardinal');
assert.equal(slim.encodeId(249417n), 'able able aardvark');
assert.equal(slim.decodePhrase('able cardinal'), 42n);
assert.equal(slim.describe().minimum, 1);
assert.equal(slim.describe().maxBits, 303);
const legacy = await loadNwords();
const format = { scheme: 'variable-v1', pattern: [{ list: 'adjective', repeat: { min: 1 } }, 'animal'], range: ((1n << 128n) - 1n).toString(), maxWords: 32 };
const boundaries = [0n, 1n, 42n, 249416n, 249417n, 249750n, 2n ** 64n, (1n << 127n) - 1n, (1n << 128n) - 2n];
let random = 0x9e3779b97f4a7c15n;
for (let i = 0; i < 256; i++) {
  random ^= random << 13n; random ^= random >> 7n; random ^= random << 17n;
  random &= (1n << 128n) - 1n;
  boundaries.push(random & ((1n << 128n) - 2n));
}
for (const id of boundaries) {
  const phrase = slim.encodeId(id);
  assert.equal(phrase, legacy.encodeId(id, format), `variable-v1 encode parity at ${id}`);
  assert.equal(slim.decodePhrase(phrase), id);
  assert.equal(legacy.decodePhrase(phrase, format), id);
}
for (const phrase of ['able\u00a0cardinal', 'able\u0085cardinal']) {
  assert.equal(slim.decodePhrase(phrase), legacy.decodePhrase(phrase, format));
}
for (const [phrase, position] of [
  ['\uFEFFable cardinal', 0], ['able unicorn', 1], ['Able cardinal', 0],
  ['able Cardinal', 1], ['unicorn able cardinal', 0], ['able cardinal dog', 1],
]) {
  for (const decode of [p => slim.decodePhrase(p), p => legacy.decodePhrase(p, format)]) {
    assert.throws(() => decode(phrase), error => error instanceof NwordsError && error.code === 'INVALID_PHRASE' && error.position === position);
  }
}
for (const bits of ['', '0', '00', '01011', '00000000', '1'.repeat(127), '0'.repeat(128), '1'.repeat(129), '1'.repeat(255)]) {
  assert.equal(slim.decodeBits(slim.encodeBits(bits)), bits);
}
for (const text of ['', 'hello', 'café 🦊', '\uFEFFA']) assert.equal(slim.decodeText(slim.encodeText(text)), text);
for (const bytes of [new Uint8Array(), new Uint8Array([0, 0, 42]), new Uint8Array([255, 0, 128])]) {
  assert.deepEqual(slim.decodeBytes(slim.encodeBytes(bytes)), bytes);
}
assert.throws(() => slim.decodeBytes(slim.encodeId(42n)), error => error instanceof NwordsError && error.code === 'NOT_BYTE_ALIGNED');
assert.throws(() => slim.decodeText(slim.encodeBits('11111111')), error => error instanceof NwordsError && error.code === 'INVALID_UTF8');
assert.throws(() => slim.encodeBytes(new Uint8Array(513)), error => error instanceof NwordsError && error.code === 'INVALID_INPUT');
assert.throws(() => slim.encodeBits('0'.repeat(4097)), error => error instanceof NwordsError && error.code === 'INVALID_INPUT');
assert.throws(() => slim.encodeId(42), error => error instanceof NwordsError && error.code === 'INVALID_INPUT');
assert.throws(() => slim.decodePhrase('a'.repeat(5000)), error => error instanceof NwordsError && error.code === 'INVALID_PHRASE' && error.field === 'phrase');
assert.throws(() => slim.decodeBytes('able unicorn'), error => error instanceof NwordsError && error.code === 'INVALID_PHRASE' && error.field === 'phrase' && error.position === 1);
assert.throws(() => slim.encodeId('9'.repeat(100000)), error => error instanceof NwordsError && error.code === 'OUT_OF_RANGE');
const restricted = defineVariable({ scheme: 'variable-v1', pattern: [{ list: adjective, repeat: { min: 1 } }, animal], maxWords: 32, range: 1000n });
assert.equal(restricted.encodeId(999n), slim.encodeId(999n));
assert.throws(() => restricted.decodePhrase(slim.encodeId(1000n)), error => error instanceof NwordsError && error.code === 'OUT_OF_RANGE');
assert.equal(effLong.words.length, 7776);
const twoWords = defineVariable({ scheme: 'variable-v1', pattern: [{ list: adjective, repeat: { min: 1 } }, animal], maxWords: 2 });
assert.throws(() => twoWords.decodePhrase(slim.encodeId(249417n)), error => error instanceof NwordsError && error.code === 'INVALID_PHRASE');
const longToken = word => ({ name: 'long', words: [word.repeat(64), 'y'.repeat(64)] });
const long = defineVariable({ scheme: 'variable-v1', pattern: [{ list: longToken('a'), repeat: { min: 1 } }, { name: 'tail', words: ['c'.repeat(64), 'd'.repeat(64)] }], maxWords: 64 });
const longestPhrase = long.encodeId(long.describe().range - 1n);
assert.equal(Buffer.byteLength(longestPhrase), 4159);
assert.equal(long.decodePhrase(longestPhrase), long.describe().range - 1n);
const toyShape = (list, changes = {}) => ({ scheme: 'variable-v1', pattern: [{ list, repeat: { min: 1 } }, { name: 'pet', words: ['cat', 'dog'] }], maxWords: 3, ...changes });
const validModifier = { name: 'modifier', words: ['calm', 'wild'] };
assert.throws(() => defineVariable(toyShape({ name: 'modifier', words: ['calm', 'bad word'] })),
  error => error instanceof NwordsError && error.code === 'INVALID_SHAPE' && error.position === 0);
for (const invalid of [
  toyShape({ name: 'Invalid', words: ['calm', 'wild'] }),
  toyShape({ name: 'modifier', words: ['calm'] }),
  toyShape({ name: 'modifier', words: ['calm', 'calm'] }),
  toyShape({ name: 'modifier', words: ['calm', 'bad word'] }),
  toyShape({ name: 'modifier', words: ['calm', 'bad\u0000word'] }),
  toyShape({ name: 'modifier', words: ['calm', '\uFEFFbad'] }),
  toyShape({ name: 'modifier', words: ['calm', 'é'.repeat(33)] }),
  toyShape(validModifier, { maxWords: 1 }),
  toyShape(validModifier, { maxWords: 65 }),
  toyShape(validModifier, { pattern: [{ list: validModifier, repeat: { min: 2 } }, { name: 'pet', words: ['cat', 'dog'] }] }),
  toyShape(validModifier, { pattern: [{ list: validModifier, repeat: { min: 1, extra: true } }, { name: 'pet', words: ['cat', 'dog'] }] }),
  toyShape(validModifier, { lists: [validModifier] }),
  toyShape(validModifier, { pattern: [{ list: validModifier, repeat: { min: 1 } }, { name: 'modifier', words: ['calm', 'quiet'] }] }),
  toyShape({ name: 'modifier', words: Array(65537).fill('calm') }),
  toyShape({ name: 'modifier', words: Array.from({ length: 32769 }, (_, i) => i.toString(36).padStart(5, '0') + 'x'.repeat(28)) }),
]) {
  assert.throws(() => defineVariable(invalid), error => error instanceof NwordsError && error.code === 'INVALID_SHAPE');
}
const zwj = defineVariable({ scheme: 'variable-v1', pattern: [{ list: { name: 'emoji', words: ['👨‍👩‍👧', '🦊'] }, repeat: { min: 1 } }, { name: 'pet', words: ['cat', 'dog'] }], maxWords: 3 });
assert.equal(zwj.decodePhrase(zwj.encodeId(0n)), 0n);
const fixtures = (await readFile(new URL('./variable-vectors.tsv', import.meta.url), 'utf8')).trim().split('\n').filter(row => !row.startsWith('#'));
for (const [minimum, column] of [[0, 1], [1, 2]]) {
  const toy = defineVariable({ scheme: 'variable-v1', pattern: [{ list: { name: 'modifier', words: ['calm', 'wild'] }, repeat: { min: minimum } }, { name: 'pet', words: ['cat', 'dog'] }], maxWords: 4 });
  for (const row of fixtures) {
    const fields = row.split('\t');
    assert.equal(toy.encodeId(fields[0]), fields[column]);
    assert.equal(toy.decodePhrase(fields[column]), BigInt(fields[0]));
  }
  for (let id = 0n; id < toy.describe().range; id++) {
    assert.equal(toy.decodePhrase(toy.encodeId(id)), id);
  }
}
function independentRank(phrase) {
  const words = phrase.split(' ');
  const prefix = words.length - 2;
  const base = BigInt(adjective.words.length);
  const animalBase = BigInt(animal.words.length);
  let offset = 0n;
  for (let count = 0; count < prefix; count++) offset += base ** BigInt(count + 1) * animalBase;
  let rank = 0n;
  for (const word of words.slice(0, -1)) rank = rank * base + BigInt(adjective.words.indexOf(word));
  rank = rank * animalBase + BigInt(animal.words.indexOf(words.at(-1)));
  return offset + rank;
}
for (const id of [(1n << 128n) - 1n, 1n << 128n, 1n << 129n, 1n << 255n]) {
  const phrase = slim.encodeId(id);
  assert.equal(independentRank(phrase), id);
  assert.equal(slim.decodePhrase(phrase), id);
}
console.log('slim variable-v1 parity, wide bit views, and wordsets passed');
