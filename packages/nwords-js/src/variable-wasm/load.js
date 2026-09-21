import * as wasm from '../../wasm/variable/nwords_js_variable.js';
import { NwordsLoadError } from '../errors.js';
import { createVariable } from './api.js';

const asset = new URL('../../wasm/variable/nwords_js_variable_bg.wasm', import.meta.url);
let pending;

export function initialize(read) {
  if (pending) return pending;
  pending = (async () => {
    const bytes = await read(asset);
    const module = await WebAssembly.compile(bytes);
    wasm.initSync({ module });
    return createVariable(wasm);
  })().catch(cause => {
    pending = undefined;
    throw new NwordsLoadError('LOAD_FAILED', 'Could not initialize the Names WASM asset.', { cause });
  });
  return pending;
}
