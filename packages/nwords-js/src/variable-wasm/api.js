import { NwordsError } from '../errors.js';

const finalizer = typeof FinalizationRegistry === 'function'
  ? new FinalizationRegistry(handle => handle.free()) : undefined;
const encoder = new TextEncoder();

function fail(code, message, field, position) { throw new NwordsError(code, message, { field, position }); }
function invoke(fn) {
  try { return fn(); }
  catch (error) {
    if (typeof error === 'string') {
      let detail;
      try { detail = JSON.parse(error); } catch { throw error; }
      if (!detail || typeof detail.code !== 'string') throw error;
      const messages = {
        INVALID_SHAPE: 'Invalid variable-v1 definition.', INVALID_INPUT: 'Invalid input.',
        INVALID_PHRASE: 'Invalid phrase.', OUT_OF_RANGE: 'Value exceeds the accepted range.',
        NOT_BYTE_ALIGNED: 'Phrase does not represent whole bytes.', INVALID_UTF8: 'Phrase bytes are not valid UTF-8.',
      };
      throw new NwordsError(detail.code, messages[detail.code] ?? 'Conversion failed.', { field: detail.field, position: detail.position });
    }
    throw error;
  }
}
function idBytes(value, range) {
  let id;
  if (typeof value === 'bigint' && value >= 0n) id = value;
  else if (typeof value === 'string' && /^(0|[1-9][0-9]*)$/u.test(value)) {
    if (value.length > range.toString().length) fail('OUT_OF_RANGE', 'Value exceeds the accepted range.', 'id');
    id = BigInt(value);
  } else fail('INVALID_INPUT', 'Expected a nonnegative bigint or canonical decimal string.', 'id');
  if (id >= range) fail('OUT_OF_RANGE', 'Value exceeds the accepted range.', 'id');
  if (id === 0n) return new Uint8Array();
  const digits = id.toString(16);
  const hex = digits.length % 2 ? `0${digits}` : digits;
  return Uint8Array.from(hex.match(/../gu), pair => Number.parseInt(pair, 16));
}
function bytesId(bytes) {
  if (!bytes.length) return 0n;
  return BigInt(`0x${Array.from(bytes, byte => byte.toString(16).padStart(2, '0')).join('')}`);
}
function definitionParts(definition) {
  if (!definition || definition.scheme !== 'variable-v1' || definition.lists !== undefined ||
      !Array.isArray(definition.pattern) || definition.pattern.length < 2 || definition.pattern.length > 32 ||
      !definition.pattern[0] || !definition.pattern[0].repeat ||
      ![0, 1].includes(definition.pattern[0].repeat.min) ||
      Object.keys(definition.pattern[0].repeat).some(key => key !== 'min') ||
      !Number.isInteger(definition.maxWords) || definition.maxWords < definition.pattern.length - 1 + definition.pattern[0].repeat.min ||
      definition.maxWords > 64) fail('INVALID_SHAPE', 'Variable-v1 needs one repeat, a suffix, and maxWords up to 64.', 'shape');
  const lists = [definition.pattern[0].list, ...definition.pattern.slice(1)];
  const unique = new Map();
  for (const [position, list] of lists.entries()) {
    if (!list || typeof list.name !== 'string' || !/^[a-z][a-z0-9_-]{0,63}$/u.test(list.name) ||
        !Array.isArray(list.words) ||
        (list.role !== undefined && !['modifier', 'head', 'either'].includes(list.role))) {
      fail('INVALID_SHAPE', 'Invalid wordset.', 'shape', position);
    }
    for (const word of list.words) {
      if (typeof word !== 'string' || !word.isWellFormed() || word.includes('\t') || word.includes('\n')) {
        fail('INVALID_SHAPE', 'Invalid wordset token.', 'shape', position);
      }
    }
    const previous = unique.get(list.name);
    if (previous && (previous.length !== list.words.length || previous.some((word, index) => word !== list.words[index]))) {
      throw new NwordsError('INVALID_SHAPE', 'Conflicting definitions for a wordset name.', { field: 'shape', position });
    }
    if (!previous) unique.set(list.name, [...list.words]);
  }
  let range;
  if (definition.range !== undefined) {
    range = definition.range;
    if (typeof range === 'bigint' && range >= 0n) range = range.toString();
    else if (typeof range !== 'string' || !/^(0|[1-9][0-9]*)$/u.test(range)) fail('INVALID_INPUT', 'Range must be a canonical integer.', 'range');
    if (range === '0') fail('INVALID_SHAPE', 'Range must be positive and within capacity.', 'range');
  }
  return {
    definitions: [...unique].map(([name, words]) => [name, ...words].join('\t')).join('\n'),
    pattern: lists.map(list => list.name).join(','),
    minimum: definition.pattern[0].repeat.min,
    maxWords: definition.maxWords,
    range,
    repeat: lists[0].name,
    suffix: lists.slice(1).map(list => list.name),
  };
}

export function createVariable(wasm) {
  return Object.freeze({
    defineVariable(definition) {
      const parts = definitionParts(definition);
      const rust = invoke(() => new wasm.VariableCodec(parts.definitions, parts.pattern, parts.minimum, parts.maxWords, parts.range));
      const info = JSON.parse(rust.describe_json());
      const summary = Object.freeze({ scheme: 'variable-v1', minimum: parts.minimum, maxWords: info.maxWords,
        range: BigInt(info.range), capacity: Object.freeze({ kind: 'exact', value: BigInt(info.capacity) }),
        requiredWords: info.requiredWords, maxBits: info.maxBits, repeat: parts.repeat, suffix: Object.freeze(parts.suffix) });
      let disposed = false;
      let codec;
      const maxPhraseBytes = Math.max(4096, parts.maxWords * 65 - 1);
      const ready = () => { if (!codec || disposed) fail('DISPOSED', 'Codec has been disposed.', 'codec'); };
      const checkPhrase = phrase => {
        if (typeof phrase !== 'string' || !phrase.isWellFormed()) fail('INVALID_PHRASE', 'Phrase must be well-formed text.', 'phrase');
        if (phrase.length > maxPhraseBytes || encoder.encode(phrase).length > maxPhraseBytes) {
          fail('INVALID_PHRASE', 'Phrase is too long.', 'phrase');
        }
      };
      codec = Object.freeze({
        encodeId(value) { ready(); return invoke(() => rust.encode_id(idBytes(value, summary.range))); },
        decodePhrase(phrase) { ready(); checkPhrase(phrase); return bytesId(invoke(() => rust.decode_phrase(phrase))); },
        encodeBits(bits) { ready(); if (typeof bits !== 'string' || bits.length > 4096) fail('INVALID_INPUT', 'Bits must be a string within 4096 digits.', 'bits'); return invoke(() => rust.encode_bits(bits)); },
        decodeBits(phrase) { ready(); checkPhrase(phrase); return invoke(() => rust.decode_bits(phrase)); },
        encodeBytes(bytes) { ready(); if (!(bytes instanceof Uint8Array)) fail('INVALID_INPUT', 'Bytes must be a Uint8Array.', 'bytes'); return invoke(() => rust.encode_bytes(bytes)); },
        decodeBytes(phrase) { ready(); checkPhrase(phrase); return new Uint8Array(invoke(() => rust.decode_bytes(phrase))); },
        encodeText(text) { ready(); if (typeof text !== 'string' || !text.isWellFormed() || encoder.encode(text).length > 512) fail('INVALID_INPUT', 'Text must be well-formed UTF-8 within 512 bytes.', 'text'); return invoke(() => rust.encode_text(text)); },
        decodeText(phrase) { ready(); checkPhrase(phrase); return invoke(() => rust.decode_text(phrase)); },
        describe() { ready(); return summary; },
        dispose() { if (!disposed) { disposed = true; finalizer?.unregister(codec); rust.free(); } },
      });
      finalizer?.register(codec, rust, codec);
      return codec;
    },
  });
}
