export function check(condition, message) {
  if (!condition) throw new Error(message);
}

export function verifyApi(api, NwordsError, vectors, variableVectors = []) {
  const shape = { lists: ['adjective', 'animal'] };
  const fails = (fn, code, field, position) => {
    let error;
    try { fn(); } catch (caught) { error = caught; }
    check(error instanceof NwordsError, `Expected NwordsError: ${code}`);
    check(error.code === code, `Expected ${code}, got ${error.code}`);
    if (field !== undefined) check(error.field === field, `Wrong field for ${code}`);
    if (position !== undefined) check(error.position === position, `Wrong position for ${code}`);
    return error;
  };
  const lists = api.lists();
  check(lists.length === 13, 'Only the supported lists are advertised');
  for (const [index, name, size, role] of [[0, 'adjective', 749n, 'modifier'], [1, 'animal', 333n, 'head'], [2, 'color', 52n, 'modifier']]) {
    check(lists[index].name === name && lists[index].size === size && lists[index].role === role, 'List metadata mismatch');
  }
  for (const [name, csv, range, id, phrase] of vectors) {
    const input = { lists: csv.split(',') };
    if (range !== '-') input.range = range;
    check(api.encodeId(id, input) === phrase, `${name}: decimal ID parity`);
    check(api.encodeId(BigInt(id), input) === phrase, `${name}: bigint ID parity`);
    check(api.decodePhrase(phrase, input) === BigInt(id), `${name}: decode parity`);
    const prepared = api.prepare(input);
    check(Object.isFrozen(prepared), 'Prepared facade is immutable');
    check(prepared.encodeId(id) === phrase && prepared.encodeId(BigInt(id)) === phrase, `${name}: prepared encode parity`);
    check(prepared.decodePhrase(phrase) === BigInt(id), `${name}: prepared decode parity`);
    prepared.dispose(); prepared.dispose();
    fails(() => prepared.encodeId(id), 'DISPOSED', 'codec');
    fails(() => prepared.decodePhrase(phrase), 'DISPOSED', 'codec');
  }
  const mutable = { lists: ['adjective', 'animal'], range: 249417n };
  const snapshot = api.prepare(mutable);
  mutable.lists.reverse(); mutable.range = 1n;
  check(snapshot.encodeId(42n) === 'able cardinal', 'Prepared codec snapshots caller shape');
  fails(() => api.encodeId(42n, mutable), 'OUT_OF_RANGE', 'id');
  fails(() => snapshot.encodeId('01'), 'INVALID_INPUT', 'id');
  fails(() => snapshot.encodeId(1n << 128n), 'INVALID_INPUT', 'id');
  fails(() => snapshot.encodeId(249417n), 'OUT_OF_RANGE', 'id');
  fails(() => snapshot.decodePhrase('able secret-token'), 'INVALID_PHRASE', 'phrase', 1);
  fails(() => snapshot.decodePhrase('\u2003'.repeat(1361) + 'able aardvark '), 'INVALID_PHRASE', 'phrase');
  check(snapshot.decodePhrase('\t able\u2003aardvark\n') === 0n, 'Prepared Rust whitespace grammar');
  check(snapshot.decodePhrase('\u2003'.repeat(1361) + 'able aardvark') === 0n, 'Prepared 4096 byte boundary');
  check(snapshot.encodeId(42n) === 'able cardinal', 'Prepared errors do not poison codec');
  let cloneFailed = false;
  try { structuredClone(snapshot); } catch { cloneFailed = true; }
  check(cloneFailed, 'Prepared facade is not structured-cloneable');
  snapshot.dispose();
  fails(() => snapshot.encodeId('invalid'), 'DISPOSED', 'codec');
  fails(() => api.prepare({ lists: ['animal'], range: 0n }), 'INVALID_SHAPE', 'range');
  fails(() => api.prepare({ lists: ['animals'] }), 'UNKNOWN_LIST', 'shape', 0);
  const info = api.describeShape(shape);
  check(info.capacity.kind === 'exact' && info.capacity.value === 249417n && info.range === 249417n, 'Exact capacity/range');
  check(info.lists.join(',') === 'adjective,animal', 'Resolved order');
  const narrowed = api.describeShape({ ...shape, range: 100n });
  check(narrowed.capacity.value === 249417n && narrowed.range === 100n, 'Narrowing must not change capacity');
  const repeated = { lists: Array(7).fill('animal') };
  check(api.describeShape(repeated).capacity.value === 454056225438947877n, 'Precision capacity');
  const max = (1n << 128n) - 1n;
  const large = { lists: Array(16).fill('animal'), range: max };
  const largeInfo = api.describeShape(large);
  check(largeInfo.capacity.kind === 'beyond-u128' && !('value' in largeInfo.capacity) && largeInfo.range === max, 'Do not substitute range for capacity');
  const exactBits = (333n ** 16n).toString(2).length;
  check(largeInfo.capacity.log2.lower <= exactBits && largeInfo.capacity.log2.upper >= exactBits - 1, 'Capacity estimate brackets actual size');
  fails(() => api.describeShape({ lists: large.lists }), 'CAPACITY_OVERFLOW', 'range');
  fails(() => api.encodeId(0n, { lists: large.lists }), 'CAPACITY_OVERFLOW', 'range');
  fails(() => api.decodePhrase(Array(16).fill('aardvark').join(' '), { lists: large.lists }), 'CAPACITY_OVERFLOW', 'range');
  fails(() => api.encodeId(max, large), 'OUT_OF_RANGE', 'id');
  fails(() => api.decodePhrase(Array(16).fill('zebra').join(' '), large), 'OUT_OF_RANGE', 'phrase');
  fails(() => api.encodeId(249417n, shape), 'OUT_OF_RANGE', 'id');
  fails(() => api.decodePhrase(api.encodeId(100n, shape), { ...shape, range: 100n }), 'OUT_OF_RANGE', 'phrase');
  for (const invalid of ['+1', '01', ' 1', '1 ', '1\n', '', '1e3', '-0', (1n << 128n).toString(), -1n, 1n << 128n, 1, 1.5, NaN, Infinity, {}, null, undefined]) {
    fails(() => api.encodeId(invalid, shape), 'INVALID_INPUT', 'id');
    if (invalid !== undefined) fails(() => api.describeShape({ ...shape, range: invalid }), 'INVALID_INPUT', 'range');
  }
  fails(() => api.describeShape({ ...shape, range: 0n }), 'INVALID_SHAPE', 'range');
  fails(() => api.describeShape({ ...shape, range: 249418n }), 'INVALID_SHAPE', 'range');
  for (const lists of [[], Array(33).fill('animal'), 'animal', null]) {
    fails(() => api.describeShape({ lists }), 'INVALID_SHAPE', 'shape');
  }
  for (const invalid of [null, undefined, 42]) fails(() => api.describeShape(invalid), 'INVALID_SHAPE');
  for (const name of ['animals', 'Animal', 'missing', 'invalid', 'animal,color', '', null]) {
    fails(() => api.describeShape({ lists: ['adjective', name] }), 'UNKNOWN_LIST', 'shape', 1);
  }
  for (const phrase of ['Able aardvark', '\ufeffable aardvark', 'able-aardvark', 'aardvark', '', 'a'.repeat(4097), '\u2003'.repeat(2000), null, 42]) {
    fails(() => api.decodePhrase(phrase, shape), 'INVALID_PHRASE', 'phrase');
  }
  check(api.decodePhrase('\t able\u2003aardvark\n', shape) === 0n, 'Rust Unicode whitespace parsing');
  check(api.decodePhrase('\u2003'.repeat(1352) + 'able aardvark', shape) === 0n, 'Short UTF-16 bound admits valid Unicode');
  check(api.decodePhrase('\u2003'.repeat(1361) + 'able aardvark', shape) === 0n, 'Exactly 4096 UTF-8 bytes');
  fails(() => api.decodePhrase('\u2003'.repeat(1361) + 'able aardvark ', shape), 'INVALID_PHRASE', 'phrase');
  fails(() => api.decodePhrase('\u2003'.repeat(1361) + 'able aardvark ', null), 'INVALID_PHRASE', 'phrase');
  fails(() => api.decodePhrase('able aardvark', { ...shape, range: 0n }), 'INVALID_SHAPE', 'range');
  fails(() => api.encodeId(0n, { ...shape, range: 0n }), 'INVALID_SHAPE', 'range');
  fails(() => api.encodeId(0n, { ...shape, range: 249418n }), 'INVALID_SHAPE', 'range');
  const error = fails(() => api.decodePhrase('able secret-token', shape), 'INVALID_PHRASE', 'phrase', 1);
  check(!error.message.includes('secret-token') && !JSON.stringify(error).includes('secret-token'), 'Never echo raw input');
  check(api.encodeId(42n, shape) === 'able cardinal', 'Ordinary codec errors do not poison the instance');
  for (const name of ['eff-long', 'object', 'descriptor', 'mood', 'material', 'shape', 'weather', 'plant', 'food', 'bip39-en']) {
    const format = {lists: [name, 'animal']};
    check(api.decodePhrase(api.encodeId(42n, format), format) === 42n, `Expanded catalog: ${name}`);
  }
  const mods = {name: 'mods', words: ['calm', 'wild']};
  const pets = {name: 'pets', words: ['cat', 'dog']};
  const variable = {scheme: 'variable-v1', pattern: [{list: mods, repeat: {min: 0}}, pets], maxWords: 3};
  const tier = api.prepare(variable);
  const expected = ['cat', 'dog', 'calm cat', 'calm dog', 'wild cat', 'wild dog', 'calm calm cat', 'calm calm dog', 'calm wild cat', 'calm wild dog', 'wild calm cat', 'wild calm dog', 'wild wild cat', 'wild wild dog'];
  expected.forEach((phrase, id) => {
    check(tier.encodeId(BigInt(id)) === phrase, `Variable vector ${id}`);
    check(api.decodePhrase(phrase, variable) === BigInt(id), `Variable decode ${id}`);
  });
  check(tier.describe().capacity.value === 14n && tier.describe().range === 14n, 'Cumulative capacity');
  fails(() => tier.encodeId(14n), 'OUT_OF_RANGE');
  const narrowedVariable = {...variable, range: 7n};
  fails(() => api.decodePhrase('calm calm dog', narrowedVariable), 'OUT_OF_RANGE');
  check(api.describeShape({...variable, maxWords: undefined, range: 1000n}).capacity.kind === 'unbounded', 'Range is not capacity');
  const plus = {...variable, pattern: [{list: mods, repeat: {min: 1}}, pets]};
  check(api.encodeId(0n, plus) === 'calm cat' && api.encodeId(4n, plus) === 'calm calm cat', 'Minimum changes mapping');
  for (const [id, zero, one] of variableVectors) {
    const unbounded = {...variable, range: 1000n, maxWords: undefined};
    check(api.encodeId(id, unbounded) === zero, 'Shared variable-v1 zero vector');
    check(api.encodeId(id, {...unbounded, pattern:[{list:mods,repeat:{min:1}},pets]}) === one, 'Shared variable-v1 one vector');
  }
  mods.words.reverse(); pets.words[0] = 'changed';
  check(tier.encodeId(2n) === 'calm cat', 'Variable snapshot owns custom dictionaries');
  check(Object.isFrozen(tier.describe().pattern[0].list.words), 'Snapshot metadata is immutable');
  tier.dispose(); fails(() => tier.describe(), 'DISPOSED');
  const tokens = {name: 'unicode', words: ['🦊', '猫', 'é', 'e\u0301', '"quote"', 'back\\slash']};
  const arbitrary = {lists: [tokens, 'mood', tokens]};
  for (const id of [0n, 1n, 6n, 100n, 2303n]) check(api.decodePhrase(api.encodeId(id, arbitrary), arbitrary) === id, 'Exact Unicode and punctuation tokens');
  const snapshotTokens = api.prepare(arbitrary);
  const saved = snapshotTokens.encodeId(0n); tokens.words.reverse();
  check(snapshotTokens.encodeId(0n) === saved, 'Fixed custom snapshot'); snapshotTokens.dispose();
  for (const words of [[], ['one'], ['one', 'one'], ['a b', 'c'], ['a\t', 'b'], ['\ufeff', 'b'], ['\u0000', 'b'], ['\ud800', 'b'], ['x'.repeat(65), 'b']]) fails(() => api.prepare({lists: [{name: 'bad', words}]}), 'INVALID_SHAPE');
  fails(() => api.prepare({lists: [{name: 'pets', words: ['a','b']}, {name: 'pets', words: ['b','a']}]}), 'INVALID_SHAPE');
  for (const invalid of [
    {scheme:'variable-v1',pattern:[{list:'animal',repeat:{min:0}},'animal']},
    {...variable,lists:['animal']}, {...variable,maxWords:33}, {...variable,maxWords:1,range:10n},
    {...variable,pattern:[{list:'animal',repeat:{min:2}},'animal']},
    {...variable,pattern:['animal',{list:'animal',repeat:{min:0}}]},
    {...variable,pattern:[{list:'animal',repeat:{min:0}},{list:'animal',repeat:{min:0}}]},
    {lists:['animal'],scheme:'guessed'}, {lists:['animal'],maxWords:2},
  ]) fails(() => api.prepare(invalid),'INVALID_SHAPE');
  const replacement = {lists:[{name:'replacement',words:['�','ok']}]};
  fails(() => api.decodePhrase('\ud800',replacement),'INVALID_PHRASE');
  const phased = api.prepareBytes({scheme:'radix-bytes-v1',lists:[
    {name:'three',words:['a','b','c']},
    {name:'threehundred',words:Array.from({length:300},(_,i)=>`word${i}`)},
  ]});
  check(phased.describe().minBlockWords === 13 && phased.describe().maxBlockWords === 14, 'Expose phase-dependent block widths');
  phased.dispose();
  const effBytes = {scheme:'radix-bytes-v1',lists:['eff-long']};
  for (const [input, phrase] of [[[], 'abacus'], [[0], 'abdomen'], [[255], 'appealing'], [[102,111], 'abdominal harddisk'], [[104,101,108,108,111], 'ventricle reference fester']]) check(api.encodeBytes(new Uint8Array(input), effBytes) === phrase, 'Independent byte oracle');
  const formats = [
    {scheme:'radix-bytes-v1',lists:['eff-long']},
    {scheme:'radix-bytes-v1',lists:['adjective','animal']},
    {scheme:'radix-bytes-v1',lists:[{name:'binary',words:['zero','one']}]},
    {scheme:'radix-bytes-v1',lists:[tokens,'food']},
  ];
  for (const format of formats) {
    const codec = api.prepareBytes(format);
    for (const bytes of [new Uint8Array(),new Uint8Array([0]),new Uint8Array([255]),new Uint8Array([0,255,0]),new Uint8Array(128).map((_,i)=>i)]) {
      const phrase=codec.encodeBytes(bytes), recovered=codec.decodeBytes(phrase);
      check(recovered instanceof Uint8Array && String(recovered)===String(bytes),'Byte frame round trip');
      check(api.encodeBytes(bytes,format)===phrase && String(api.decodeBytes(phrase,format))===String(bytes),'Stateless byte parity');
      fails(()=>codec.decodeBytes(phrase+' not-in-list'),'INVALID_PHRASE');
    }
    for (const text of ['', 'hello 世界 🦊', 'é', 'e\u0301']) check(codec.decodeText(codec.encodeText(text))===text,'UTF-8 exact');
    fails(()=>codec.decodeText(codec.encodeBytes(new Uint8Array([255]))),'INVALID_UTF8');
    fails(()=>codec.encodeText('\ud800'),'INVALID_INPUT');
    fails(()=>codec.encodeBytes(new Uint8Array(4097)),'INVALID_INPUT');
    fails(()=>codec.encodeBytes([0]),'INVALID_INPUT');
    for (const length of [-1,1.5,4097,NaN]) fails(()=>codec.generatePassphrase(length),'INVALID_INPUT');
    check(codec.decodeBytes(codec.generatePassphrase(16)).length===16,'Cryptographic byte generation');
    codec.dispose(); codec.dispose(); fails(()=>codec.encodeBytes(new Uint8Array()),'DISPOSED');
  }
  const single = {lists:[{name:'tiny',words:['a','b','c']}],range:1n};
  check(api.generatePhrase(single)==='a','Singleton random domain');
  for(let i=0;i<50;i++) {const format={...single,range:3n};check(api.decodePhrase(api.generatePhrase(format),format)<3n,'Random accepted IDs');}

  const enormous = {scheme:'variable-v1',pattern:[{list:'adjective',repeat:{min:0}},'animal'],range:max,maxWords:32};
  const highest = api.encodeId(max-1n,enormous);
  check(api.decodePhrase(highest,enormous)===max-1n,'Variable u128 boundary');
  check(api.describeShape(enormous).capacity.kind==='beyond-u128','Variable capacity exceeds range');
  fails(()=>api.decodePhrase([...Array(31).fill('zippy'),'zebra'].join(' '),enormous),'NUMERIC_OVERFLOW');
  const customBound = {name:'wide',words:Array.from({length:65536},(_,i)=>`w${i}`)};
  const wide = api.prepareBytes({scheme:'radix-bytes-v1',lists:[customBound]});
  check(wide.encodeBytes(new Uint8Array([0,255]))==='w512','16-bit dictionary independent byte vector');
  fails(()=>wide.decodeBytes('w65535 w65535 w65535 w65535'),'INVALID_PHRASE');
  wide.dispose();
  fails(()=>api.prepare({lists:[{name:'excess',words:Array(65537).fill('a')}]}),'INVALID_SHAPE');

  const originalCrypto = Object.getOwnPropertyDescriptor(globalThis, 'crypto');
  let draws = 0;
  try {
    Object.defineProperty(globalThis, 'crypto', {configurable:true,value:{getRandomValues(bytes){bytes.fill(draws++ === 0 ? 255 : 0);return bytes;}}});
    check(api.generatePhrase({lists:[{name:'three',words:['a','b','c']}]}) === 'a' && draws === 2, 'Rejection sampling excludes masked slack');
    Object.defineProperty(globalThis, 'crypto', {configurable:true,value:undefined});
    fails(()=>api.generatePassphrase(1,effBytes),'RANDOM_UNAVAILABLE');
  } finally {
    if(originalCrypto)Object.defineProperty(globalThis,'crypto',originalCrypto);else delete globalThis.crypto;
  }

}
