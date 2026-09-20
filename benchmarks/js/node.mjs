import { readFile } from 'node:fs/promises';
import { randomInt } from 'node:crypto';
import { pathToFileURL } from 'node:url';
import * as ung from 'unique-names-generator';
import niceware from 'niceware';
import { workloads, measure, sink } from './suite.mjs';
const [packagePath, caseName, roundArg = '0', duration = '100'] = process.argv.slice(2);
const { loadNwords } = await import(pathToFileURL(packagePath + '/src/node.js'));
const words = await loadNwords();
const dictionaries = await Promise.all(['adjectives', 'animals'].map(async name => (await readFile(new URL(`../../tests/vectors/adjective-animal/nwords-${name}.txt`, import.meta.url), 'utf8')).trim().split('\n')));
const values = (await readFile(new URL('../ids-u32.txt', import.meta.url), 'utf8')).trim().split('\n').map(Number);
const nativePhrases = (await readFile(new URL('../.work/native-phrases.tsv', import.meta.url), 'utf8')).trim().split('\n').map(line => line.split('\t')[1]);
const { cases, metadata } = workloads({ words, ung, niceware, Buffer, dictionaries, values, nativePhrases, cryptoRandomInt: randomInt });
if (caseName === 'list') { console.log(cases.map(([name]) => name).join('\n')); process.exit(0); }
const selected = cases.filter(([name]) => name === caseName);
if (selected.length !== 1) throw new Error('Unknown case');
console.log(JSON.stringify({ kind: 'metadata', runtime: 'node', round: Number(roundArg), ...metadata }));
for (const result of measure(selected, 'node', Number(roundArg), Number(duration))) console.log(JSON.stringify(result));
console.log(JSON.stringify({ kind: 'sink', runtime: 'node', value: sink() }));
