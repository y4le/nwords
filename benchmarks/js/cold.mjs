// Fresh process per observation; excludes Node process startup and harness imports.
import { pathToFileURL } from 'node:url';
const [choice, packagePath] = process.argv.slice(2);
// Resolve each public module entry before timing, so package lookup is not mixed
// into only the bare-specifier competitors' cold measurements.
const entry = choice === 'nwords' ? pathToFileURL(packagePath + '/src/node.js').href : import.meta.resolve(choice);
const started = performance.now();
let imported;
let initialized;
let phrase;
if (choice === 'nwords') {
  const { loadNwords } = await import(entry);
  imported = performance.now();
  const words = await loadNwords();
  initialized = performance.now();
  phrase = words.encodeId(42n, { lists: ['adjective', 'animal'] });
} else if (choice === 'unique-names-generator') {
  const module = await import(entry);
  imported = performance.now(); initialized = imported;
  phrase = module.uniqueNamesGenerator({ dictionaries: [module.adjectives, module.animals], separator: '-', length: 2 });
} else if (choice === 'niceware') {
  const { default: module } = await import(entry);
  imported = performance.now(); initialized = imported;
  phrase = module.bytesToPassphrase(Buffer.from([0, 0, 0, 42])).join(' ');
} else throw new Error('Unknown package');
const end = performance.now();
if (!phrase) throw new Error('No output');
console.log(JSON.stringify({ kind: 'cold', case: choice, runtime: 'node', importMs: imported - started, initializationMs: initialized - imported, firstOperationMs: end - initialized, totalMs: end - started }));
