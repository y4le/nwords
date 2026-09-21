import { loadVariable } from '@y4le/nwords/variable/web';
import { adjective } from '@y4le/nwords/wordsets/adjective';
import { animal } from '@y4le/nwords/wordsets/animal';

let names;
let namesSource = 'value';
const select = selector => document.querySelector(selector);
const namesValue = select('#names-value');
const namesPhrase = select('#names-phrase');
const namesView = select('#names-view');
const namesStatus = select('#names-status');
const entropyValue = select('#entropy-value');
const mnemonicValue = select('#mnemonic-value');
const bitcoinStatus = select('#bitcoin-status');
let bitcoinSource = 'entropy';
let bip39Promise;
let randomUnavailable = false;

function status(target, error) {
  target.classList.toggle('error', Boolean(error));
  target.textContent = error ? `${error.code ?? 'INVALID_INPUT'}: ${error.message}` : 'Round trip ready';
}
function fromValue() {
  if (!names) return;
  try {
    const value = namesValue.value;
    let phrase;
    switch (namesView.value) {
      case 'number': phrase = names.encodeId(value.trim()); break;
      case 'bits': phrase = names.encodeBits(value.trim()); break;
      case 'hex': phrase = names.encodeBytes(parseHex(value)); break;
      case 'text': phrase = names.encodeText(value); break;
    }
    namesPhrase.value = phrase;
    status(namesStatus);
  } catch (error) { status(namesStatus, error); }
}
function fromPhrase() {
  if (!names) return;
  try {
    const phrase = namesPhrase.value;
    let value;
    switch (namesView.value) {
      case 'number': value = names.decodePhrase(phrase).toString(); break;
      case 'bits': value = names.decodeBits(phrase); break;
      case 'hex': value = toHex(names.decodeBytes(phrase)); break;
      case 'text': value = names.decodeText(phrase); break;
    }
    namesValue.value = value;
    status(namesStatus);
  } catch (error) { status(namesStatus, error); }
}
function parseHex(text) {
  const hex = text.trim();
  if (hex.length % 2 || !/^[0-9a-fA-F]*$/.test(hex)) throw new Error('Enter whole bytes as pairs of hex digits.');
  return Uint8Array.from(hex.match(/../g) ?? [], pair => parseInt(pair, 16));
}
function toHex(bytes) { return Array.from(bytes, byte => byte.toString(16).padStart(2, '0')).join(''); }
try {
  entropyValue.value = toHex(crypto.getRandomValues(new Uint8Array(16)));
} catch {
  randomUnavailable = true;
  status(bitcoinStatus, { code: 'RANDOM_UNAVAILABLE', message: 'Secure randomness unavailable; enter entropy manually.' });
}
const examples = {
  number: "const phrase = codec.encodeId(42n);\nconst id = codec.decodePhrase(phrase);",
  bits: "const phrase = codec.encodeBits('01011');\nconst bits = codec.decodeBits(phrase);",
  hex: "const bytes = new Uint8Array([0, 42]);\nconst phrase = codec.encodeBytes(bytes);\nconst bytesAgain = codec.decodeBytes(phrase);",
  text: "const phrase = codec.encodeText('hello');\nconst text = codec.decodeText(phrase);",
};
function codeExample() {
  const code = `import { loadVariable } from '@y4le/nwords/variable/web';\nimport { adjective } from '@y4le/nwords/wordsets/adjective';\nimport { animal } from '@y4le/nwords/wordsets/animal';\n\nconst { defineVariable } = await loadVariable();\nconst codec = defineVariable({\n  scheme: 'variable-v1',\n  pattern: [{ list: adjective, repeat: { min: 1 } }, animal],\n  maxWords: 32,\n});\n\n${examples[namesView.value]}`;
  select('#names-code').textContent = code;
  select('#names-value-label').textContent = namesView.selectedOptions[0].textContent;
}
function showTab(which) {
  const bitcoin = which === 'bitcoin';
  for (const [name, selected] of [['names', !bitcoin], ['bitcoin', bitcoin]]) {
    select(`#${name}-panel`).hidden = !selected;
    select(`#${name}-tab`).classList.toggle('active', selected);
    select(`#${name}-tab`).setAttribute('aria-selected', String(selected));
  }
  if (bitcoin && !randomUnavailable) loadBitcoin();
}
async function loadBitcoin() {
  if (!bip39Promise) bip39Promise = import('@y4le/nwords/bip39/web').then(module => module.loadBip39());
  try {
    const codec = await bip39Promise;
    if (bitcoinSource === 'entropy') fromEntropy(codec);
    else fromMnemonic(codec);
  } catch (error) {
    bip39Promise = undefined;
    status(bitcoinStatus, error);
  }
}
function fromEntropy(codec) {
  try {
    mnemonicValue.value = codec.encodeEntropy(parseHex(entropyValue.value));
    status(bitcoinStatus);
  } catch (error) { status(bitcoinStatus, error); }
}
function fromMnemonic(codec) {
  try {
    entropyValue.value = toHex(codec.decodeMnemonic(mnemonicValue.value));
    status(bitcoinStatus);
  } catch (error) { status(bitcoinStatus, error); }
}

namesValue.addEventListener('input', () => { namesSource = 'value'; fromValue(); });
namesPhrase.addEventListener('input', () => { namesSource = 'phrase'; fromPhrase(); });
namesView.addEventListener('change', () => { codeExample(); fromPhrase(); });
select('#names-tab').addEventListener('click', () => showTab('names'));
select('#bitcoin-tab').addEventListener('click', () => showTab('bitcoin'));
for (const [buttonSelector, codeSelector] of [['#copy-code', '#names-code'], ['#copy-bitcoin-code', '#bitcoin-code']]) {
  const button = select(buttonSelector);
  button.addEventListener('click', async () => {
    try { await navigator.clipboard.writeText(select(codeSelector).textContent); button.textContent = 'Copied'; }
    catch { button.textContent = 'Copy failed'; }
  });
}
entropyValue.addEventListener('input', () => {
  randomUnavailable = false;
  bitcoinSource = 'entropy';
  if (bip39Promise || !select('#bitcoin-panel').hidden) void loadBitcoin();
});
mnemonicValue.addEventListener('input', () => {
  randomUnavailable = false;
  bitcoinSource = 'mnemonic';
  if (bip39Promise || !select('#bitcoin-panel').hidden) void loadBitcoin();
});
codeExample();
namesStatus.textContent = 'Loading Names codec…';
loadVariable().then(({ defineVariable }) => {
  names = defineVariable({ scheme: 'variable-v1', pattern: [{ list: adjective, repeat: { min: 1 } }, animal], maxWords: 32 });
  select('#names-capacity').textContent = `Up to ${names.describe().maxWords} words · ${names.describe().maxBits} guaranteed bits.`;
  if (namesSource === 'phrase') fromPhrase();
  else fromValue();
}).catch(error => status(namesStatus, error));
