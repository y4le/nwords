export * from './index.js';
import type { LoadOptions, Nwords } from './index.js';
/** No-source calls reuse initialization. A conflicting explicit source is rejected. */
export function loadNwords(options?: LoadOptions): Promise<Nwords>;
