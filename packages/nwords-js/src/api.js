import { NwordsError } from './errors.js';

const MAX_U128 = (1n << 128n) - 1n;
const names = new Set(['adjective', 'animal', 'color', 'object', 'descriptor', 'mood', 'material', 'shape', 'weather', 'plant', 'food', 'eff-long', 'bip39-en']);
let utf8;
function fail(code, message, field, position) { throw new NwordsError(code, message, { field, position }); }
function decimal(value, field) {
  if (typeof value === 'bigint') {
    if (value < 0n || value > MAX_U128) fail('INVALID_INPUT', 'Integer must be within u128.', field);
    return value.toString();
  }
  if (typeof value !== 'string' || value.length === 0 || value.length > 39 || /[^0-9]/u.test(value) || (value.length > 1 && value[0] === '0') || BigInt(value) > MAX_U128) fail('INVALID_INPUT', 'Expected a bigint or canonical decimal string within u128.', field);
  return value;
}
function object(value) { return value !== null && typeof value === 'object' && !Array.isArray(value); }
function encodedLength(value) { return (utf8 ??= new TextEncoder()).encode(value).length; }
function listInput(lists) {
  if (!Array.isArray(lists) || lists.length === 0 || lists.length > 32) fail('INVALID_SHAPE', 'Format must contain 1 to 32 lists.', 'shape');
  const custom = new Map(), snapshots = new WeakMap();
  let totalWords = 0, totalBytes = 0;
  const resolved = lists.map((list, position) => {
    if (typeof list === 'string') {
      if (!names.has(list)) fail('UNKNOWN_LIST', 'Unsupported canonical word-list name.', 'shape', position);
      return list;
    }
    if (object(list) && snapshots.has(list)) return snapshots.get(list);
    if (!object(list)) fail('UNKNOWN_LIST', 'Expected a list name or custom list.', 'shape', position);
    const { name, words: providedWords, role = 'either' } = list;
    if (typeof name !== 'string' || !/^[a-z][a-z0-9_-]{0,63}$/u.test(name) || names.has(name) || !Array.isArray(providedWords) || providedWords.length < 2 || providedWords.length > 65536 || !['modifier', 'head', 'either'].includes(role)) fail('INVALID_SHAPE', 'Invalid custom wordlist.', 'shape', position);
    const words = providedWords.slice();
    let bytes = 0;
    for (const word of words) {
      if (typeof word !== 'string' || word.length === 0 || word.length > 64 || !word.isWellFormed() || /[\s\p{Cc}]/u.test(word)) fail('INVALID_SHAPE', 'Custom tokens must be 1 to 64 UTF-8 bytes without whitespace or controls.', 'shape', position);
      const size = encodedLength(word); bytes += size;
      if (size > 64) fail('INVALID_SHAPE', 'Custom token exceeds 64 UTF-8 bytes.', 'shape', position);
    }
    if (new Set(words).size !== words.length) fail('INVALID_SHAPE', 'Duplicate custom token.', 'shape', position);
    const row = [name, role, ...words].join('\t');
    if (custom.has(name)) {
      if (custom.get(name) !== row) fail('INVALID_SHAPE', 'Conflicting definitions for a custom list name.', 'shape', position);
    } else {
      totalWords += words.length; totalBytes += bytes;
      if (totalWords > 65536 || totalBytes > 1048576) fail('INVALID_SHAPE', 'Custom dictionary budget exceeded.', 'shape');
      custom.set(name, row);
    }
    const snapshot = Object.freeze({ name, role, words: Object.freeze(words) });
    snapshots.set(list, snapshot);
    return snapshot;
  });
  return { csv: resolved.map(list => typeof list === 'string' ? list : list.name).join(','), custom: [...custom.values()].join('\n'), lists: Object.freeze(resolved) };
}
// Retain the inexpensive path for the original built-in fixed descriptors.
function builtinInput(format) {
  if (!object(format) || (format.scheme !== undefined && format.scheme !== 'positional-v1') || format.pattern !== undefined || format.maxWords !== undefined || !Array.isArray(format.lists) || format.lists.length === 0 || format.lists.length > 32) return;
  for (const name of format.lists) if (!names.has(name)) return;
  return [format.lists.join(','), format.range === undefined ? undefined : decimal(format.range, 'range')];
}
function formatInput(format, bytes = false) {
  if (!object(format)) fail('INVALID_SHAPE', 'Expected a format descriptor.', 'shape');
  if (bytes) {
    if (format.scheme !== 'radix-bytes-v1' || format.pattern !== undefined || format.range !== undefined || format.maxWords !== undefined) fail('INVALID_SHAPE', 'Byte formats require scheme radix-bytes-v1 and lists.', 'shape');
    return listInput(format.lists);
  }
  if (format.scheme === 'variable-v1') {
    const pattern = format.pattern;
    if (format.lists !== undefined || !Array.isArray(pattern) || pattern.length < 2 || pattern.length > 32 || !object(pattern[0]) || !object(pattern[0].repeat) || ![0, 1].includes(pattern[0].repeat.min) || Object.keys(pattern[0].repeat).some(key => key !== 'min')) fail('INVALID_SHAPE', 'Expected one leading repeat and a nonempty fixed suffix.', 'shape');
    const max = format.maxWords;
    if (max !== undefined && (!Number.isInteger(max) || max < 1 || max > 32)) fail('INVALID_SHAPE', 'maxWords must be an integer from 1 to 32.', 'shape');
    const input = listInput([pattern[0].list, ...pattern.slice(1)]);
    const range = format.range === undefined ? undefined : decimal(format.range, 'range');
    if (range === undefined && max === undefined) fail('INVALID_SHAPE', 'Variable formats require range or maxWords.', 'shape');
    return { ...input, range, minimum: pattern[0].repeat.min, maxWords: max, pattern: Object.freeze([Object.freeze({ list: input.lists[0], repeat: Object.freeze({ min: pattern[0].repeat.min }) }), ...input.lists.slice(1)]) };
  }
  if ((format.scheme !== undefined && format.scheme !== 'positional-v1') || format.pattern !== undefined || format.maxWords !== undefined) fail('INVALID_SHAPE', 'Ambiguous or unsupported integer format.', 'shape');
  const input = listInput(format.lists);
  const range = format.range === undefined ? undefined : decimal(format.range, 'range');
  return { ...input, range };
}
function result(json) {
  const envelope = JSON.parse(json);
  if (!envelope.ok) { const { code, message, field, position } = envelope.error; throw new NwordsError(code, message, { field, position }); }
  return envelope.value;
}
function phraseInput(phrase, max = 4096) {
  if (typeof phrase !== 'string' || phrase.length > max || !phrase.isWellFormed() || (phrase.length > Math.floor(max / 3) && encodedLength(phrase) > max)) fail('INVALID_PHRASE', `Phrase must be a string of at most ${max} UTF-8 bytes.`, 'phrase');
}
function rethrow(error) { if (typeof error === 'string') result(error); throw error; }
function convertInfo(info) { return { ...info, range: BigInt(info.range), capacity: info.capacity.kind === 'exact' ? { kind: 'exact', value: BigInt(info.capacity.value) } : info.capacity }; }
function invoke(fn) { try { return fn(); } catch (error) { rethrow(error); } }
function prepared(wasm, format, resolved) {
  const input = resolved ?? formatInput(format);
  const flexible = input.custom !== '' || input.minimum !== undefined;
  let codec = invoke(() => flexible ? new wasm.FlexibleCodec(input.csv, input.custom, input.range, input.minimum, input.maxWords) : new wasm.PreparedCodec(input.csv, input.range));
  const alive = () => { if (!codec) fail('DISPOSED', 'Prepared codec has been disposed.', 'codec'); };
  const describe = () => {
    alive();
    const info = convertInfo(result(flexible ? codec.describe_json() : wasm.describe_shape_json(input.csv, input.range)));
    return input.minimum === undefined ? { ...info, lists: input.lists } : { ...info, scheme: 'variable-v1', pattern: input.pattern, maxWords: input.maxWords };
  };
  const encode = id => { alive(); return invoke(() => codec.encode_id(decimal(id, 'id'))); };
  return Object.freeze({
    encodeId: encode,
    decodePhrase(phrase) { alive(); phraseInput(phrase); return invoke(() => codec.decode_phrase(phrase)); },
    describe,
    generatePhrase() { alive(); return encode(randomBelow(describe().range)); },
    dispose() { if (codec) { const old = codec; codec = undefined; old.free(); } },
  });
}
function randomBytes(length) {
  if (!Number.isInteger(length) || length < 0 || length > 4096) fail('INVALID_INPUT', 'Byte length must be an integer from 0 to 4096.', 'bytes');
  const crypto = globalThis.crypto;
  if (typeof crypto?.getRandomValues !== 'function') fail('RANDOM_UNAVAILABLE', 'Platform cryptographic randomness is unavailable.', 'bytes');
  return crypto.getRandomValues(new Uint8Array(length));
}
function randomBelow(range) {
  if (range < 1n) fail('INVALID_SHAPE', 'Range must be positive.', 'range');
  if (range === 1n) return 0n;
  const bits = (range - 1n).toString(2).length;
  const length = Math.ceil(bits / 8), mask = 255 >>> (length * 8 - bits);
  for (;;) {
    const bytes = randomBytes(length); bytes[0] &= mask;
    let id = 0n;
    for (const byte of bytes) id = (id << 8n) | BigInt(byte);
    if (id < range) return id;
  }
}
function byteInput(bytes) { if (!(bytes instanceof Uint8Array) || bytes.length > 4096) fail('INVALID_INPUT', 'Expected a Uint8Array of at most 4096 bytes.', 'bytes'); }
function textInput(text) { if (typeof text !== 'string' || text.length > 4096 || !text.isWellFormed() || encodedLength(text) > 4096) fail('INVALID_INPUT', 'Expected well-formed UTF-8 text of at most 4096 bytes.', 'text'); }
function preparedBytes(wasm, format) {
  const input = formatInput(format, true);
  let codec = invoke(() => new wasm.ByteCodec(input.csv, input.custom));
  const alive = () => { if (!codec) fail('DISPOSED', 'Prepared codec has been disposed.', 'codec'); };
  return Object.freeze({
    encodeBytes(bytes) { alive(); byteInput(bytes); return invoke(() => codec.encode_bytes(bytes)); },
    decodeBytes(phrase) { alive(); phraseInput(phrase, 8388608); return invoke(() => codec.decode_bytes(phrase)); },
    encodeText(text) { alive(); textInput(text); return invoke(() => codec.encode_text(text)); },
    decodeText(phrase) { alive(); phraseInput(phrase, 8388608); return invoke(() => codec.decode_text(phrase)); },
    generatePassphrase(byteLength) { alive(); return invoke(() => codec.encode_bytes(randomBytes(byteLength))); },
    describe() { alive(); return { ...result(codec.describe_json()), lists: input.lists }; },
    dispose() { if (codec) { const old = codec; codec = undefined; old.free(); } },
  });
}
function withCodec(codec, fn) { try { return fn(codec); } finally { codec.dispose(); } }
export function createApi(wasm) {
  return Object.freeze({
    prepare(format) { return prepared(wasm, format); },
    prepareBytes(format) { return preparedBytes(wasm, format); },
    lists() { return result(wasm.lists_json()).map(list => ({ ...list, size: BigInt(list.size) })); },
    describeShape(format) { return withCodec(prepared(wasm, format), codec => codec.describe()); },
    encodeId(id, format) {
      const checked = decimal(id, 'id'), simple = builtinInput(format);
      if (simple) return invoke(() => wasm.encode_id(checked, ...simple));
      const input = formatInput(format);
      if (input.custom === '' && input.minimum === undefined) return invoke(() => wasm.encode_id(checked, input.csv, input.range));
      return withCodec(prepared(wasm, format, input), codec => codec.encodeId(checked));
    },
    decodePhrase(phrase, format) {
      phraseInput(phrase);
      const simple = builtinInput(format);
      if (simple) return invoke(() => wasm.decode_phrase(phrase, ...simple));
      const input = formatInput(format);
      if (input.custom === '' && input.minimum === undefined) return invoke(() => wasm.decode_phrase(phrase, input.csv, input.range));
      return withCodec(prepared(wasm, format, input), codec => codec.decodePhrase(phrase));
    },
    generatePhrase(format) { return withCodec(prepared(wasm, format), codec => codec.generatePhrase()); },
    encodeBytes(bytes, format) { byteInput(bytes); return withCodec(preparedBytes(wasm, format), codec => codec.encodeBytes(bytes)); },
    decodeBytes(phrase, format) { phraseInput(phrase, 8388608); return withCodec(preparedBytes(wasm, format), codec => codec.decodeBytes(phrase)); },
    encodeText(text, format) { textInput(text); return withCodec(preparedBytes(wasm, format), codec => codec.encodeText(text)); },
    decodeText(phrase, format) { phraseInput(phrase, 8388608); return withCodec(preparedBytes(wasm, format), codec => codec.decodeText(phrase)); },
    generatePassphrase(byteLength, format) { return withCodec(preparedBytes(wasm, format), codec => codec.generatePassphrase(byteLength)); },
  });
}
