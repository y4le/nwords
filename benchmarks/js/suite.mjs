// Shared warm workload for Node and actual Chromium. No initialization is timed here.
export function workloads({ words, ung, niceware, Buffer, dictionaries, values, nativePhrases, cryptoRandomInt }) {
  const big = values.map(BigInt);
  const pair = { lists: ['adjective', 'animal'] };
  const four = { lists: Array(4).fill('animal'), range: 1n << 32n };
  const pairIds = values.map(value => BigInt(value % 249417));
  const shared = { dictionaries, length: 2, separator: ' ' };
  const defaults = { ...shared, dictionaries: [ung.adjectives, ung.animals] };
  const niceEncode = value => {
    const bytes = Buffer.allocUnsafe(4); bytes.writeUInt32BE(value);
    return niceware.bytesToPassphrase(bytes).join(' ');
  };
  const niceDecode = phrase => niceware.passphraseToBytes(phrase.split(' ')).readUInt32BE(0);
  const phrases = big.map(value => words.encodeId(value, four));
  const nicePhrases = values.map(niceEncode);
  for (let i = 0; i < values.length; i++) {
    if (words.decodePhrase(phrases[i], four) !== big[i] || niceDecode(nicePhrases[i]) !== values[i] ||
        nativePhrases[i] !== phrases[i]) throw new Error(`Roundtrip/parity failed at ${i}`);
  }
  if (words.encodeId(42n, pair) !== 'able cardinal') throw new Error('Pinned pair vector changed');
  const cases = [
    ['nwords/name/shared-math-random', () => words.encodeId(BigInt(Math.floor(Math.random() * 249417)), pair)],
    ['unique-names-generator/name/shared', () => ung.uniqueNamesGenerator(shared)],
    ['unique-names-generator/name/default', () => ung.uniqueNamesGenerator(defaults)],
    ['nwords/pair/encode', i => words.encodeId(pairIds[i], pair)],
    ['nwords/u32/encode', i => words.encodeId(big[i], four)],
    ['niceware/u32/encode', i => niceEncode(values[i])],
    ['nwords/u32/decode', i => Number(words.decodePhrase(phrases[i], four))],
    ['niceware/u32/decode', i => niceDecode(nicePhrases[i])],
    ['rng/math-random/one-draw', () => Math.floor(Math.random() * 249417)],
    ['rng/math-random/two-draws', () => Math.floor(Math.random() * 749) + Math.floor(Math.random() * 333)],
  ];
  if (cryptoRandomInt) cases.push(['nwords/name/shared-crypto', () => words.encodeId(BigInt(cryptoRandomInt(249417)), pair)]);
  return { cases, metadata: { idsChecksum: values.reduce((sum, value) => sum + value, 0), uniqueNamesDefaultCapacity: ung.adjectives.length * ung.animals.length,
    dictionarySizes: dictionaries.map(list => list.length), u32Words: { nwords: 4, niceware: 2 }, meanChars: { nwords: phrases.reduce((n, s) => n + s.length, 0) / values.length, niceware: nicePhrases.reduce((n, s) => n + s.length, 0) / values.length }, example: { nwords: phrases[2], niceware: nicePhrases[2] } } };
}
let consumed = 0;
function batch(run, count) {
  let sink = 0;
  const start = performance.now();
  for (let i = 0; i < count; i++) {
    const value = run(i & 4095);
    sink += typeof value === 'string' ? value.length + value.charCodeAt(0) : value;
  }
  const ns = (performance.now() - start) * 1e6;
  consumed = (consumed + sink) % 0x100000000;
  return { ns, checksum: sink };
}
export function measure(cases, runtime, round, targetMs = 100) {
  if (!(targetMs >= 10 && targetMs <= 1000)) throw new Error('Invalid duration');
  const offset = round % cases.length;
  const ordered = [...cases.slice(offset), ...cases.slice(0, offset)];
  if (round % 2) ordered.reverse();
  return ordered.flatMap(([name, run]) => {
    const warm = performance.now();
    while (performance.now() - warm < 100) batch(run, 4096);
    let count = 4096;
    while (batch(run, count).ns < targetMs * 1e6 && count < 2 ** 24) count *= 2;
    return Array.from({ length: 3 }, (_, sampleIndex) => {
    const { ns, checksum } = batch(run, count);
    return { kind: 'sample', runtime, round, case: name, sampleIndex, iterations: count, elapsedNs: ns, nsPerOp: ns / count, checksum };
    });
  });
}
export const sink = () => consumed;
