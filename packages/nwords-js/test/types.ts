import { loadNwords, NwordsError, NwordsLoadError, type Shape, type Capacity } from '@y4le/nwords/node';
import { loadNwords as loadWeb } from '@y4le/nwords/web';
import { loadBip39 } from '@y4le/nwords/bip39/node';
import { loadBip39 as loadBip39Web } from '@y4le/nwords/bip39/web';
import { loadVariable } from '@y4le/nwords/variable/node';
import { loadVariable as loadVariableWeb } from '@y4le/nwords/variable/web';
import { adjective } from '@y4le/nwords/wordsets/adjective';
import { animal } from '@y4le/nwords/wordsets/animal';

const { defineVariable } = await loadVariable();
const slim = defineVariable({ scheme: 'variable-v1', pattern: [{ list: adjective, repeat: { min: 1 } }, animal], maxWords: 32 });
const slimPhrase: string = slim.encodeBits('01011');
const slimBits: string = slim.decodeBits(slimPhrase);
const slimBytes: Uint8Array = slim.decodeBytes(slim.encodeBytes(new Uint8Array([42])));
void [slimBits, slimBytes, loadVariableWeb];
// @ts-expect-error The slim codec also rejects JS number IDs.
slim.encodeId(42);

const bip39 = await loadBip39();
const mnemonic: string = bip39.encodeEntropy(new Uint8Array(16));
const entropy: Uint8Array = bip39.decodeMnemonic(mnemonic);
void [entropy, loadBip39Web];

const words = await loadNwords();
const shape = { lists: ['adjective', 'animal'] as const } satisfies Shape;
const phrase: string = words.encodeId(42n, shape);
const id: bigint = words.decodePhrase(phrase, shape);
const size: bigint = words.lists()[0].size;
const capacity: Capacity = words.describeShape(shape).capacity;
if (capacity.kind === 'exact') { const value: bigint = capacity.value; void value; }
else { const lower: number = capacity.log2.lower; void lower; }
words.encodeId('9007199254740993', { lists: Array(7).fill('animal'), range: '454056225438947877' });
await loadWeb({ source: new Uint8Array(8) });
await loadWeb({ source: new URL('https://example.test/words.wasm') });
new NwordsError('INVALID_PHRASE', 'invalid', { position: 1, field: 'phrase' });
new NwordsLoadError('LOAD_FAILED', 'failed');
// @ts-expect-error Numbers are rejected even when currently safe integers.
words.encodeId(42, shape);
// @ts-expect-error Ranges also reject number values.
words.describeShape({ lists: ['animal'], range: 100 });
// @ts-expect-error Only the declared canonical list set is supported.
words.describeShape({ lists: ['animals'] });
// @ts-expect-error Exact ID results are bigint, never number.
const wrong: number = words.decodePhrase(phrase, shape);
// @ts-expect-error The Node entry loads the packaged file and has no source options.
await loadNwords({ source: new Uint8Array(8) });
// @ts-expect-error Single-use responses are not a supported retryable source.
await loadWeb({ source: Promise.resolve(new Response()) });
void [id, size, wrong];

const prepared = (await loadNwords()).prepare({ lists: ['adjective', 'animal'] });
const preparedPhrase: string = prepared.encodeId(42n);
const preparedId: bigint = prepared.decodePhrase(preparedPhrase);
// @ts-expect-error number inputs remain unsupported
prepared.encodeId(42);
// @ts-expect-error Exact prepared results are bigint, never number.
const wrongPrepared: number = prepared.decodePhrase(preparedPhrase);
void wrongPrepared;
prepared.dispose();
void preparedId;

const custom = { name: 'pets', words: ['猫', 'dog', '🦊'] } as const;
words.encodeId(1n, { lists: ['mood', custom, 'eff-long'] });
const variable = { scheme: 'variable-v1', pattern: [{ list: 'adjective', repeat: { min: 0 } }, custom], maxWords: 3 } as const;
const variableInfo = words.describeShape(variable);
if (variableInfo.capacity.kind === 'unbounded') void variableInfo.requiredWords;
words.prepare(variable).dispose();
const byteFormat = { scheme: 'radix-bytes-v1', lists: ['eff-long', custom] } as const;
const encodedBytes: string = words.encodeBytes(new Uint8Array([0, 1]), byteFormat);
const decodedBytes: Uint8Array = words.decodeBytes(encodedBytes, byteFormat);
const byteCodec = words.prepareBytes(byteFormat);
byteCodec.encodeText('hello 世界'); byteCodec.generatePassphrase(16); byteCodec.dispose();
words.generatePhrase({ lists: ['mood', custom] });
// @ts-expect-error Variable patterns need a range or word bound.
words.encodeId(0n, { scheme: 'variable-v1', pattern: [{list: 'adjective', repeat: {min: 0}}, 'animal'] });
// @ts-expect-error Fixed and variable descriptors are mutually exclusive.
words.encodeId(0n, {...variable, lists: ['animal']});
// @ts-expect-error Byte codecs require their explicit versioned scheme.
words.encodeBytes(decodedBytes, {lists: ['eff-long']});
