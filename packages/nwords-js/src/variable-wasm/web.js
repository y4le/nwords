import { initialize } from './load.js';
import { NwordsLoadError } from '../errors.js';
export { NwordsError, NwordsLoadError } from '../errors.js';

/** Loads the dictionary-free Rust variable-v1 codec on first call. */
export function loadVariable() {
  return initialize(async url => {
    const response = await fetch(url);
    if (!response.ok) throw new NwordsLoadError('LOAD_FAILED', 'Names WASM asset request failed.');
    return response.arrayBuffer();
  });
}
