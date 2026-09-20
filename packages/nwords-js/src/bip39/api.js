import { NwordsError } from '../errors.js';

const entropyLengths = new Set([16, 20, 24, 28, 32]);
const encoder = new TextEncoder();

function fail(code, message, field) {
  throw new NwordsError(code, message, { field });
}

function invoke(fn) {
  try {
    return fn();
  } catch (error) {
    if (typeof error === 'string') {
      const envelope = JSON.parse(error);
      const { code, message, field, position } = envelope.error;
      throw new NwordsError(code, message, { field, position });
    }
    throw error;
  }
}

export function createBip39(wasm) {
  return Object.freeze({
    encodeEntropy(entropy) {
      if (!(entropy instanceof Uint8Array) || !entropyLengths.has(entropy.length)) {
        fail('INVALID_ENTROPY_LENGTH', 'BIP-39 entropy must be 16, 20, 24, 28, or 32 bytes.', 'entropy');
      }
      return invoke(() => wasm.encode_entropy(entropy));
    },
    decodeMnemonic(mnemonic) {
      if (typeof mnemonic !== 'string' || !mnemonic.isWellFormed() || encoder.encode(mnemonic).length > 4096) {
        fail('INVALID_INPUT', 'BIP-39 mnemonic must be well-formed text within 4096 UTF-8 bytes.', 'mnemonic');
      }
      return new Uint8Array(invoke(() => wasm.decode_mnemonic(mnemonic)));
    },
  });
}
