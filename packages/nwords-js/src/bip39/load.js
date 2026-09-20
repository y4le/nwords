import * as wasm from '../../wasm/bip39/nwords_js_bip39.js';
import { NwordsLoadError } from '../errors.js';
import { createBip39 } from './api.js';

const asset = new URL('../../wasm/bip39/nwords_js_bip39_bg.wasm', import.meta.url);
let pending;

export function initialize(read) {
  if (pending) return pending;
  pending = (async () => {
    const bytes = await read(asset);
    const module = await WebAssembly.compile(bytes);
    wasm.initSync({ module });
    return createBip39(wasm);
  })().catch(() => {
    pending = undefined;
    throw new NwordsLoadError('LOAD_FAILED', 'Could not initialize the BIP-39 WASM asset.');
  });
  return pending;
}
