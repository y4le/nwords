import { readFile } from 'node:fs/promises';
import { initialize } from './load.js';
export { NwordsError, NwordsLoadError } from '../errors.js';

/** Loads the packaged dictionary-free Rust variable-v1 codec on first call. */
export function loadVariable() {
  return initialize(url => readFile(url));
}
