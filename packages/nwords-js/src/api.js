import { NwordsError } from './errors.js';

const MAX_U128 = (1n << 128n) - 1n;
const names = new Set(['adjective', 'animal', 'color']);
let utf8;

function fail(code, message, field, position) {
  throw new NwordsError(code, message, { field, position });
}

function decimal(value, field) {
  if (typeof value === 'bigint') {
    if (value < 0n || value > MAX_U128) {
      fail('INVALID_INPUT', 'Integer must be within u128.', field);
    }
    return value.toString();
  }
  // Avoid `$`, which also matches before a trailing newline in JavaScript.
  if (typeof value !== 'string' || value.length === 0 || value.length > 39 ||
      /[^0-9]/u.test(value) || (value.length > 1 && value[0] === '0') ||
      BigInt(value) > MAX_U128) {
    fail('INVALID_INPUT', 'Expected a bigint or canonical decimal string within u128.', field);
  }
  return value;
}

function shapeInput(shape) {
  if (shape === null || typeof shape !== 'object' || !Array.isArray(shape.lists) ||
      shape.lists.length === 0 || shape.lists.length > 32) {
    fail('INVALID_SHAPE', 'Shape must have 1 to 32 canonical lists.', 'shape');
  }
  for (let position = 0; position < shape.lists.length; position++) {
    if (!names.has(shape.lists[position])) {
      fail('UNKNOWN_LIST', 'Unsupported canonical word-list name.', 'shape', position);
    }
  }
  return [shape.lists.join(','), shape.range === undefined ? undefined : decimal(shape.range, 'range')];
}

function result(json) {
  const envelope = JSON.parse(json);
  if (!envelope.ok) {
    const { code, message, field, position } = envelope.error;
    throw new NwordsError(code, message, { field, position });
  }
  return envelope.value;
}

function phraseInput(phrase) {
  // At most three UTF-8 bytes per UTF-16 code unit, including lone surrogates.
  if (typeof phrase !== 'string' || phrase.length > 4096 ||
      (phrase.length > 1365 && (utf8 ??= new TextEncoder()).encode(phrase).length > 4096)) {
    fail('INVALID_PHRASE', 'Phrase must be a string of at most 4096 UTF-8 bytes.', 'phrase');
  }
}

function prepared(wasm, shape) {
  let codec;
  try { codec = new wasm.PreparedCodec(...shapeInput(shape)); }
  catch (error) { rethrow(error); }
  const alive = () => {
    if (!codec) fail('DISPOSED', 'Prepared codec has been disposed.', 'codec');
  };
  return Object.freeze({
    encodeId(id) {
      alive();
      try { return codec.encode_id(decimal(id, 'id')); }
      catch (error) { rethrow(error); }
    },
    decodePhrase(phrase) {
      alive();
      phraseInput(phrase);
      try { return codec.decode_phrase(phrase); }
      catch (error) { rethrow(error); }
    },
    dispose() {
      if (codec) { const previous = codec; codec = undefined; previous.free(); }
    },
  });
}

// Expected Rust failures throw the same structured envelope as the diagnostic ABI.
function rethrow(error) {
  if (typeof error === 'string') result(error);
  throw error;
}

export function createApi(wasm) {
  return Object.freeze({
    prepare(shape) { return prepared(wasm, shape); },
    lists() {
      return result(wasm.lists_json()).map(list => ({ ...list, size: BigInt(list.size) }));
    },
    describeShape(shape) {
      const info = result(wasm.describe_shape_json(...shapeInput(shape)));
      return {
        ...info,
        range: BigInt(info.range),
        capacity: info.capacity.kind === 'exact'
          ? { kind: 'exact', value: BigInt(info.capacity.value) }
          : info.capacity,
      };
    },
    encodeId(id, shape) {
      try { return wasm.encode_id(decimal(id, 'id'), ...shapeInput(shape)); }
      catch (error) { rethrow(error); }
    },
    decodePhrase(phrase, shape) {
      phraseInput(phrase);
      try { return wasm.decode_phrase(phrase, ...shapeInput(shape)); }
      catch (error) { rethrow(error); }
    },
  });
}
