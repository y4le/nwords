import { readFile } from 'node:fs/promises';
import { initialize, sourceInput } from './load.js';
import { NwordsLoadError } from './errors.js';
export { NwordsError, NwordsLoadError } from './errors.js';

export async function loadNwords(...args) {
  if (args.length !== 0) {
    throw new NwordsLoadError('LOAD_FAILED', 'The Node loader takes no source options.');
  }
  return initialize(sourceInput(), url => readFile(url));
}
