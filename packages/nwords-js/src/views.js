import { NwordsError } from './errors.js';

let encoder;
let decoder;
function fail(code, message, field) { throw new NwordsError(code, message, { field }); }

/** Map a finite binary string to one natural number, preserving leading zeros. */
export function bitsToId(bits) {
  if (typeof bits !== 'string' || !/^[01]*$/u.test(bits) || bits.length > 4096) {
    fail('INVALID_INPUT', 'Bits must be a binary string of at most 4096 digits.', 'bits');
  }
  return BigInt(`0b1${bits}`) - 1n;
}
/** Recover the exact bitstring represented by an ID. */
export function idToBits(id) {
  if (typeof id !== 'bigint' || id < 0n) fail('INVALID_INPUT', 'ID must be a nonnegative bigint.', 'id');
  return (id + 1n).toString(2).slice(1);
}
export function bytesToBits(bytes) {
  if (!(bytes instanceof Uint8Array) || bytes.length > 512) fail('INVALID_INPUT', 'Bytes must be a Uint8Array of at most 512 bytes.', 'bytes');
  return Array.from(bytes, byte => byte.toString(2).padStart(8, '0')).join('');
}
export function bitsToBytes(bits) {
  if (typeof bits !== 'string' || !/^[01]*$/u.test(bits) || bits.length > 4096) fail('INVALID_INPUT', 'Bits must be a binary string of at most 4096 digits.', 'bits');
  if (bits.length % 8) fail('NOT_BYTE_ALIGNED', 'This phrase is not aligned to whole bytes.', 'bytes');
  return Uint8Array.from(bits.match(/.{8}/gu) ?? [], binary => parseInt(binary, 2));
}
export function textToBits(text) {
  if (typeof text !== 'string' || !text.isWellFormed()) fail('INVALID_INPUT', 'Text must be well-formed Unicode.', 'text');
  const bytes = (encoder ??= new TextEncoder()).encode(text);
  if (bytes.length > 512) fail('INVALID_INPUT', 'Text exceeds the 512-byte view limit.', 'text');
  return bytesToBits(bytes);
}
export function bitsToText(bits) {
  const bytes = bitsToBytes(bits);
  try { return (decoder ??= new TextDecoder('utf-8', { fatal: true, ignoreBOM: true })).decode(bytes); }
  catch { fail('INVALID_UTF8', 'Phrase bytes are not valid UTF-8.', 'text'); }
}
/** Optional view helpers keep the variable-v1 ID codec itself view-free. */
export function encodeBits(codec, bits) { return codec.encodeId(bitsToId(bits)); }
export function decodeBits(codec, phrase) { return idToBits(codec.decodePhrase(phrase)); }
export function encodeBytes(codec, bytes) { return encodeBits(codec, bytesToBits(bytes)); }
export function decodeBytes(codec, phrase) { return bitsToBytes(decodeBits(codec, phrase)); }
export function encodeText(codec, text) { return encodeBits(codec, textToBits(text)); }
export function decodeText(codec, phrase) { return bitsToText(decodeBits(codec, phrase)); }
