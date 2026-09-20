export type ListName = 'adjective' | 'animal' | 'color';
/** Strings must be canonical unsigned decimal integers within u128 at runtime. */
export type IntegerInput = bigint | string;
export interface Shape {
  readonly lists: readonly ListName[];
  readonly range?: IntegerInput;
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
  readonly lists: readonly ListName[];
  readonly range: bigint;
  readonly capacity: Capacity;
}
/** Immutable shape snapshot. FinalizationRegistry cleanup is best effort; dispose releases it promptly. */
export interface PreparedCodec {
  encodeId(id: IntegerInput): string;
  decodePhrase(phrase: string): bigint;
  /** Idempotent. Subsequent encode/decode calls throw DISPOSED. */
  dispose(): void;
}
export interface Nwords {
  prepare(shape: Shape): PreparedCodec;
  lists(): readonly ListInfo[];
  describeShape(shape: Shape): ShapeInfo;
  encodeId(id: IntegerInput, shape: Shape): string;
  decodePhrase(phrase: string, shape: Shape): bigint;
}
export type ErrorCode = 'INVALID_INPUT' | 'UNKNOWN_LIST' | 'INVALID_SHAPE' |
  'CAPACITY_OVERFLOW' | 'OUT_OF_RANGE' | 'INVALID_PHRASE' | 'INTERNAL_ERROR' | 'DISPOSED';
export class NwordsError extends Error {
  readonly code: ErrorCode;
  readonly field?: 'id' | 'range' | 'shape' | 'phrase' | 'codec';
  readonly position?: number;
  constructor(code: ErrorCode, message: string, details?: {
    field?: 'id' | 'range' | 'shape' | 'phrase' | 'codec'; position?: number;
  });
}
export class NwordsLoadError extends Error {
  readonly code: 'LOAD_FAILED' | 'ASSET_CONFLICT';
  constructor(code: 'LOAD_FAILED' | 'ASSET_CONFLICT', message: string);
}
export interface LoadOptions {
  /** Relative URLs resolve against the packaged WASM URL. Only the web loader accepts options. */
  readonly source?: string | URL | ArrayBuffer | Uint8Array;
}
