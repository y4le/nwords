export * from './index.js';
import type { Bip39 } from './index.js';
/** Loads the packaged English BIP-39 WASM asset on first call. */
export function loadBip39(): Promise<Bip39>;
