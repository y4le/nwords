import { loadNwords, NwordsError, NwordsLoadError, type Shape, type Capacity } from '@y4le/nwords/node';
import { loadNwords as loadWeb } from '@y4le/nwords/web';

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
