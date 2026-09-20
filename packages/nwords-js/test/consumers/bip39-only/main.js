import { loadBip39 } from '@y4le/nwords/bip39/web';

const bip39 = await loadBip39();
document.querySelector('#phrase').textContent = bip39.encodeEntropy(new Uint8Array(16));
