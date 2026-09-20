import { NwordsError } from './errors.js';

const MAX_U128 = (1n << 128n) - 1n;
const names = new Set(['adjective', 'animal', 'color']);

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

export function createApi(wasm) {
  return Object.freeze({
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
      return result(wasm.encode_id_json(decimal(id, 'id'), ...shapeInput(shape)));
    },
    decodePhrase(phrase, shape) {
      if (typeof phrase !== 'string' || phrase.length > 4096 || new TextEncoder().encode(phrase).length > 4096) {
        fail('INVALID_PHRASE', 'Phrase must be a string of at most 4096 UTF-8 bytes.', 'phrase');
      }
      return BigInt(result(wasm.decode_phrase_json(phrase, ...shapeInput(shape))));
    },
  });
}
