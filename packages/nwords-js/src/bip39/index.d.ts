export { NwordsError, NwordsLoadError } from '../index.js';

/** Standards-compliant English BIP-39 entropy and mnemonic conversion. */
export interface Bip39 {
  encodeEntropy(entropy: Uint8Array): string;
  decodeMnemonic(mnemonic: string): Uint8Array;
}
