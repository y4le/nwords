import { NwordsError } from './errors.js';

const phraseWhitespace = /[\t\n\v\f\r \u0085\u00a0\u1680\u2000-\u200a\u2028\u2029\u202f\u205f\u3000]+/u;
let utf8;
const encodedLength = value => (utf8 ??= new TextEncoder()).encode(value).length;
function fail(code, message, field, position) {
  throw new NwordsError(code, message, { field, position });
}
function snapshot(source, position) {
  if (!source || Array.isArray(source) || typeof source.name !== 'string' || !/^[a-z][a-z0-9_-]{0,63}$/u.test(source.name) ||
      (source.role !== undefined && !['modifier', 'head', 'either'].includes(source.role)) ||
      !Array.isArray(source.words) || source.words.length < 2 || source.words.length > 65536) {
    fail('INVALID_SHAPE', 'A wordset needs a valid name and at least two ordered words.', 'shape', position);
  }
  const words = [...source.words];
  const lookup = new Map();
  let bytes = 0;
  for (const [index, word] of words.entries()) {
    if (typeof word !== 'string' || !word.isWellFormed() || !word || phraseWhitespace.test(word) ||
        /[\p{Cc}\uFEFF]/u.test(word)) {
      fail('INVALID_SHAPE', 'Wordset tokens must contain no whitespace or controls.', 'shape', position);
    }
    const size = encodedLength(word);
    bytes += size;
    if (size > 64 || lookup.has(word)) {
      fail('INVALID_SHAPE', 'Wordset tokens must be unique and at most 64 UTF-8 bytes.', 'shape', position);
    }
    lookup.set(word, index);
  }
  if (bytes > 1048576) fail('INVALID_SHAPE', 'Wordset byte budget exceeded.', 'shape', position);
  return { name: source.name, words, lookup, base: BigInt(words.length), bytes };
}
function canonical(value, field, limit) {
  if (typeof value === 'bigint' && value >= 0n) return value;
  if (typeof value === 'string' && /^(0|[1-9][0-9]*)$/u.test(value)) {
    if (value.length > limit.toString().length) fail('OUT_OF_RANGE', 'Integer exceeds this format’s range.', field);
    return BigInt(value);
  }
  fail('INVALID_INPUT', 'Expected a nonnegative bigint or canonical decimal string.', field);
}
function wordsRequired(id, leading, tail, minimum) {
  let quotient = id;
  for (let position = tail.length - 1; position >= 0; position--) quotient /= tail[position].base;
  if (minimum === 1) quotient /= leading.base;
  let count = tail.length + minimum;
  while (quotient > 0n) { quotient = (quotient - 1n) / leading.base; count++; }
  return count;
}

/** Build the stable variable-v1 positional format over caller-supplied wordsets. */
export function defineVariable(format) {
  if (!format || format.scheme !== 'variable-v1' || format.lists !== undefined || !Array.isArray(format.pattern) ||
      format.pattern.length < 2 || format.pattern.length > 32 ||
      !format.pattern[0] || !format.pattern[0].repeat ||
      ![0, 1].includes(format.pattern[0].repeat.min) ||
      Object.keys(format.pattern[0].repeat).some(key => key !== 'min') ||
      !Number.isInteger(format.maxWords) || format.maxWords < format.pattern.length - 1 + format.pattern[0].repeat.min ||
      format.maxWords > 64) {
    fail('INVALID_SHAPE', 'Variable-v1 needs one repeat, a suffix, and maxWords up to 64.', 'shape');
  }
  const minimum = format.pattern[0].repeat.min;
  const leading = snapshot(format.pattern[0].list, 0);
  const tail = format.pattern.slice(1).map((source, position) => snapshot(source, position + 1));
  const seen = new Map();
  let totalWords = 0, totalBytes = 0;
  for (const [position, list] of [leading, ...tail].entries()) {
    const previous = seen.get(list.name);
    if (previous) {
      if (previous.words.length !== list.words.length || previous.words.some((word, index) => word !== list.words[index])) {
        fail('INVALID_SHAPE', 'Conflicting definitions for a wordset name.', 'shape', position);
      }
    } else {
      seen.set(list.name, list);
      totalWords += list.words.length; totalBytes += list.bytes;
      if (totalWords > 65536 || totalBytes > 1048576) fail('INVALID_SHAPE', 'Wordset budget exceeded.', 'shape');
    }
  }
  const base = leading.base;
  const maxWords = format.maxWords;
  const maxPhraseBytes = Math.max(4096, maxWords * 65 - 1);
  const required = tail.length + minimum;
  let tier = base ** BigInt(minimum);
  for (const list of tail) tier *= list.base;
  let capacity = 0n;
  for (let words = required; words <= maxWords; words++) { capacity += tier; tier *= base; }
  const range = format.range === undefined ? capacity : canonical(format.range, 'range', capacity);
  if (range < 1n || range > capacity) fail('INVALID_SHAPE', 'Range must be positive and within capacity.', 'range');

  function encodeId(value) {
    const id = canonical(value, 'id', range);
    if (id >= range) fail('OUT_OF_RANGE', 'ID exceeds this format’s accepted range.', 'id');
    let quotient = id;
    const tailIndexes = new Array(tail.length);
    for (let position = tail.length - 1; position >= 0; position--) {
      const list = tail[position];
      tailIndexes[position] = Number(quotient % list.base);
      quotient /= list.base;
    }
    let minimumIndex;
    if (minimum === 1) { minimumIndex = Number(quotient % base); quotient /= base; }
    const prefix = [];
    while (quotient > 0n) { quotient -= 1n; prefix.push(Number(quotient % base)); quotient /= base; }
    if (prefix.length + required > maxWords) fail('OUT_OF_RANGE', 'ID exceeds this format’s word bound.', 'id');
    const words = prefix.reverse().map(index => leading.words[index]);
    if (minimum === 1) words.push(leading.words[minimumIndex]);
    for (let position = 0; position < tail.length; position++) words.push(tail[position].words[tailIndexes[position]]);
    return words.join(' ');
  }

  function decodePhrase(phrase) {
    if (typeof phrase !== 'string' || !phrase.isWellFormed() || phrase.length > maxPhraseBytes || encodedLength(phrase) > maxPhraseBytes) {
      fail('INVALID_PHRASE', 'Phrase must be well-formed text within this format’s UTF-8 byte bound.', 'phrase');
    }
    const words = phrase.split(phraseWhitespace).filter(Boolean);
    if (words.length < required || words.length > maxWords) fail('INVALID_PHRASE', 'Phrase has the wrong number of words.', 'phrase');
    const prefixLength = words.length - required;
    let value = 0n;
    for (const [position, word] of words.entries()) {
      const isLeading = position < prefixLength + minimum;
      const list = isLeading ? leading : tail[position - prefixLength - minimum];
      const index = list.lookup.get(word);
      if (index === undefined) fail('INVALID_PHRASE', 'Phrase contains a word outside its position’s wordset.', 'phrase', position);
      value = value * list.base + BigInt(index) + BigInt(position < prefixLength);
    }
    if (value >= range) fail('OUT_OF_RANGE', 'Phrase exceeds this format’s accepted range.', 'phrase');
    return value;
  }
  const summary = Object.freeze({ scheme: 'variable-v1', minimum, maxWords, range,
    requiredWords: wordsRequired(range - 1n, leading, tail, minimum),
    capacity: Object.freeze({ kind: 'exact', value: capacity }),
    maxBits: Math.max(0, (range + 1n).toString(2).length - 2),
    repeat: leading.name, suffix: Object.freeze(tail.map(list => list.name)) });
  return Object.freeze({ encodeId, decodePhrase, describe: () => summary });
}

export { NwordsError } from './errors.js';
