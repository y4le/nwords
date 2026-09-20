import { initialize } from './load.js';
import { NwordsLoadError } from '../errors.js';
export { NwordsError, NwordsLoadError } from '../errors.js';

export function loadBip39() {
  return initialize(async url => {
    const response = await fetch(url);
    if (!response.ok) throw new NwordsLoadError('LOAD_FAILED', 'BIP-39 WASM asset request failed.');
    return response.arrayBuffer();
  });
}
