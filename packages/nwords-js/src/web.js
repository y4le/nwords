import { initialize, sourceInput } from './load.js';
import { NwordsLoadError } from './errors.js';
export { NwordsError, NwordsLoadError } from './errors.js';

export async function loadNwords(options = {}) {
  if (options === null || typeof options !== 'object') {
    throw new NwordsLoadError('LOAD_FAILED', 'Expected loader options.');
  }
  return initialize(sourceInput(options.source), async url => {
    const response = await fetch(url);
    if (!response.ok) throw new NwordsLoadError('LOAD_FAILED', 'WASM asset request failed.');
    return response.arrayBuffer();
  });
}
