import { createApi } from './api.js';
import { NwordsLoadError } from './errors.js';

export const defaultSource = new URL('../wasm/nwords_js_bg.wasm', import.meta.url);
let pending;
let selected;
let attempt = 0;

export function sourceInput(source) {
  if (source === undefined) return { key: defaultSource.href, url: defaultSource, explicit: false };
  if (typeof source === 'string' || source instanceof URL) {
    try {
      const url = new URL(source, defaultSource);
      return { key: url.href, url, explicit: true };
    } catch {
      throw new NwordsLoadError('LOAD_FAILED', 'Invalid WASM asset URL.');
    }
  }
  if (source instanceof Uint8Array || source instanceof ArrayBuffer) {
    try {
      // Snapshot before any await; caller mutation cannot change this attempt.
      const bytes = source instanceof Uint8Array ? new Uint8Array(source) : new Uint8Array(source.slice(0));
      return { key: source, bytes, explicit: true };
    } catch {
      throw new NwordsLoadError('LOAD_FAILED', 'WASM asset bytes are unavailable.');
    }
  }
  throw new NwordsLoadError('LOAD_FAILED', 'Expected an asset URL, ArrayBuffer, or Uint8Array.');
}

// Both public entries share this state. Source identity is normalized URL href,
// or object identity for byte inputs. Even a pending attempt owns its source.
export function initialize(input, read) {
  if (pending) {
    if (input.explicit && input.key !== selected) {
      return Promise.reject(new NwordsLoadError('ASSET_CONFLICT', 'A different WASM asset was already selected.'));
    }
    return pending;
  }
  selected = input.key;
  pending = (async () => {
    // Generated glue assigns its globals before calling __wbindgen_start.
    // A fresh module per retry also recovers from failures after that assignment.
    const bytes = input.bytes ?? await read(input.url);
    // Missing/corrupt assets must not accumulate cached ESM module records.
    const compiled = await WebAssembly.compile(bytes);
    const module = new URL('../wasm/nwords_js.js', import.meta.url);
    module.searchParams.set('attempt', String(++attempt));
    const wasm = await import(module.href);
    await wasm.default({ module_or_path: compiled });
    return createApi(wasm);
  })().catch(() => {
    pending = undefined;
    selected = undefined;
    throw new NwordsLoadError('LOAD_FAILED', 'Could not initialize the nwords WASM asset.');
  });
  return pending;
}
