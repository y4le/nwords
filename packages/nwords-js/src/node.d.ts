export * from './index.js';
import type { Nwords } from './index.js';
/** Loads the packaged file or reuses the shared instance; no source options. */
export function loadNwords(): Promise<Nwords>;
