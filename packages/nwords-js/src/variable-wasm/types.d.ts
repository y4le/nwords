export interface Wordset {
  readonly name: string;
  readonly words: readonly string[];
}
export interface VariableDefinition {
  readonly scheme: 'variable-v1';
  readonly pattern: readonly [{ readonly list: Wordset; readonly repeat: { readonly min: 0 | 1 } }, Wordset, ...Wordset[]];
  /** Inclusive total phrase bound, from the minimum shape through 64 words. */
  readonly maxWords: number;
  /** Optional exclusive accepted range. Omission uses all word-bounded capacity. */
  readonly range?: bigint | string;
}
export interface VariableCodec {
  encodeId(id: bigint | string): string;
  decodePhrase(phrase: string): bigint;
  encodeBits(bits: string): string;
  decodeBits(phrase: string): string;
  encodeBytes(bytes: Uint8Array): string;
  decodeBytes(phrase: string): Uint8Array;
  encodeText(text: string): string;
  decodeText(phrase: string): string;
  describe(): Readonly<{
    scheme: 'variable-v1'; minimum: 0 | 1; maxWords: number; range: bigint;
    requiredWords: number; capacity: Readonly<{ kind: 'exact'; value: bigint }>;
    maxBits: number; repeat: string; suffix: readonly string[];
  }>;
  dispose(): void;
}
export interface VariableFactory {
  defineVariable(definition: VariableDefinition): VariableCodec;
}
