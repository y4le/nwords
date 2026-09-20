export type ListName = 'adjective' | 'animal' | 'color' | 'object' | 'descriptor' |
  'mood' | 'material' | 'shape' | 'weather' | 'plant' | 'food' | 'eff-long' | 'bip39-en';
/** Exact ordered Unicode vocabulary; tokens have 1–64 UTF-8 bytes, no whitespace or controls. */
export interface CustomWordList {
  readonly name: string;
  readonly words: readonly string[];
  readonly role?: 'modifier' | 'head' | 'either';
}
export type ListSource = ListName | CustomWordList;
/** Strings must be canonical unsigned decimal integers within u128 at runtime. */
export type IntegerInput = bigint | string;
export interface Shape {
  readonly scheme?: 'positional-v1';
  readonly lists: readonly ListSource[];
  readonly range?: IntegerInput;
  readonly pattern?: never;
  readonly maxWords?: never;
}
export interface RepeatSlot {
  readonly list: ListSource;
  readonly repeat: { readonly min: 0 | 1 };
}
/** One leading repeat and a nonempty fixed suffix; at least one bound is required. */
export type VariableFormat = {
  readonly scheme: 'variable-v1';
  readonly pattern: readonly [RepeatSlot, ListSource, ...ListSource[]];
  readonly lists?: never;
} & ({ readonly range: IntegerInput; readonly maxWords?: number } |
     { readonly range?: IntegerInput; readonly maxWords: number });
export type IdFormat = Shape | VariableFormat;
/** Length-framed mixed-radix bytes; distinct from legacy word-bytes-v1. */
export interface ByteFormat {
  readonly scheme: 'radix-bytes-v1';
  readonly lists: readonly ListSource[];
  readonly range?: never;
  readonly pattern?: never;
  readonly maxWords?: never;
}
export interface ListInfo {
  readonly name: ListName;
  readonly size: bigint;
  readonly role: 'modifier' | 'head' | 'either';
}
export type Capacity =
  | { readonly kind: 'exact'; readonly value: bigint }
  | { readonly kind: 'beyond-u128'; readonly log2: { readonly lower: number; readonly upper: number } };
export interface ShapeInfo {
  readonly lists: readonly ListSource[];
  readonly range: bigint;
  readonly capacity: Capacity;
}
export interface VariableInfo {
  readonly scheme: 'variable-v1';
  readonly pattern: readonly [RepeatSlot, ListSource, ...ListSource[]];
  readonly range: bigint;
  readonly maxWords?: number;
  readonly requiredWords: number;
  readonly capacity: Capacity | { readonly kind: 'unbounded' };
}
export type FormatInfo = ShapeInfo | VariableInfo;
export interface ByteInfo {
  readonly scheme: 'radix-bytes-v1';
  readonly lists: readonly ListSource[];
  readonly minBlockWords: number;
  readonly maxBlockWords: number;
  readonly blockBytes: number;
  readonly maxBytes: number;
}
/** Immutable snapshot. Dispose releases WASM storage promptly; finalization is best effort. */
export interface PreparedCodec {
  encodeId(id: IntegerInput): string;
  decodePhrase(phrase: string): bigint;
  describe(): FormatInfo;
  /** Uniformly samples an accepted ID using platform cryptographic randomness. */
  generatePhrase(): string;
  dispose(): void;
}
export interface PreparedBytes {
  encodeBytes(bytes: Uint8Array): string;
  decodeBytes(phrase: string): Uint8Array;
  encodeText(text: string): string;
  decodeText(phrase: string): string;
  /** Encodes byteLength cryptographically random bytes. Maximum 4096 bytes. */
  generatePassphrase(byteLength: number): string;
  describe(): ByteInfo;
  dispose(): void;
}
export interface Nwords {
  prepare(format: IdFormat): PreparedCodec;
  prepareBytes(format: ByteFormat): PreparedBytes;
  lists(): readonly ListInfo[];
  describeShape(shape: Shape): ShapeInfo;
  describeShape(format: VariableFormat): VariableInfo;
  describeShape(format: IdFormat): FormatInfo;
  encodeId(id: IntegerInput, format: IdFormat): string;
  decodePhrase(phrase: string, format: IdFormat): bigint;
  generatePhrase(format: IdFormat): string;
  encodeBytes(bytes: Uint8Array, format: ByteFormat): string;
  decodeBytes(phrase: string, format: ByteFormat): Uint8Array;
  encodeText(text: string, format: ByteFormat): string;
  decodeText(phrase: string, format: ByteFormat): string;
  generatePassphrase(byteLength: number, format: ByteFormat): string;
}
export type ErrorCode = 'INVALID_INPUT' | 'UNKNOWN_LIST' | 'INVALID_SHAPE' |
  'CAPACITY_OVERFLOW' | 'OUT_OF_RANGE' | 'INVALID_PHRASE' | 'INTERNAL_ERROR' |
  'DISPOSED' | 'NUMERIC_OVERFLOW' | 'INVALID_UTF8' | 'RANDOM_UNAVAILABLE' |
  'INVALID_ENTROPY_LENGTH' | 'INVALID_WORD_COUNT' | 'UNKNOWN_WORD' | 'INVALID_CHECKSUM' |
  'NOT_BYTE_ALIGNED';
export type ErrorField = 'id' | 'range' | 'shape' | 'phrase' | 'codec' | 'bytes' | 'text' |
  'entropy' | 'mnemonic' | 'bits';
export class NwordsError extends Error {
  readonly code: ErrorCode;
  readonly field?: ErrorField;
  readonly position?: number;
  constructor(code: ErrorCode, message: string, details?: { field?: ErrorField; position?: number });
}
export class NwordsLoadError extends Error {
  readonly code: 'LOAD_FAILED' | 'ASSET_CONFLICT';
  constructor(code: 'LOAD_FAILED' | 'ASSET_CONFLICT', message: string);
}
export interface LoadOptions {
  /** Relative URLs resolve against the packaged WASM URL. Only the web loader accepts options. */
  readonly source?: string | URL | ArrayBuffer | Uint8Array;
}
