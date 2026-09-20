export function check(condition, message) {
  if (!condition) throw new Error(message);
}

export function verifyApi(api, NwordsError, vectors) {
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
  check(lists.length === 3, 'Only the supported lists are advertised');
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
  for (const name of ['animals', 'Animal', 'bip39-en', 'descriptor', 'animal,color', '', null]) {
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
}
