import { loadNwords } from '@y4le/nwords/node';

const words = await loadNwords();
const shape = { lists: ['adjective', 'animal'] };
const phrase = words.encodeId(42n, shape);
console.log(phrase); // able cardinal
console.log(words.decodePhrase(phrase, shape)); // 42n
console.log(words.describeShape(shape).range); // 249417n
