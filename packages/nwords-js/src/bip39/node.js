import { readFile } from 'node:fs/promises';
import { initialize } from './load.js';
export { NwordsError, NwordsLoadError } from '../errors.js';

export function loadBip39() {
  return initialize(url => readFile(url));
}
