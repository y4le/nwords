// Focused warm-operation measurements; run serially with fixed CPU affinity.
import { createRequire } from 'node:module';
import { performance } from 'node:perf_hooks';
import { readFile, writeFile } from 'node:fs/promises';
import { createHash } from 'node:crypto';
import { execFileSync } from 'node:child_process';
import { pathToFileURL } from 'node:url';
import { resolve, dirname } from 'node:path';
const root = resolve(import.meta.dirname, '..');
const { loadNwords } = await import(pathToFileURL(resolve(process.argv[2] ?? 'dist/nwords-js/src/node.js')));
const api = await loadNwords();
const rows = [];
let sink = 0;
function measure(name, operation, wordCount) {
  const iterations = name.includes('custom') && name.includes('stateless') ? 100 : 20000;
  for (let i = 0; i < Math.min(iterations, 5000); i++) operation(i);
  const ns = [];
  for (let round = 0; round < 9; round++) {
    const start = performance.now();
    for (let i = 0; i < iterations; i++) { const value = operation(i); sink ^= typeof value === 'bigint' ? Number(value & 255n) : value.length; }
    ns.push((performance.now() - start) * 1e6 / iterations);
  }
  const sorted = [...ns].sort((a,b)=>a-b);
  rows.push({name,wordCount,iterations,medianNs:sorted[4],q1Ns:sorted[2],q3Ns:sorted[6],samplesNs:ns});
}
function integers(name, format) {
  const codec = api.prepare(format);
  const ids = Array.from({length:256},(_,i)=>BigInt(i*1234567));
  const phrases = ids.map(id=>codec.encodeId(id));
  for(let i=0;i<256;i++) if(codec.decodePhrase(phrases[i])!==ids[i]) throw Error('Round trip');
  const wordCount = [...new Set(phrases.map(p=>p.split(' ').length))];
  measure(`${name}/prepared-encode`,i=>codec.encodeId(ids[i&255]),wordCount);
  measure(`${name}/prepared-decode`,i=>codec.decodePhrase(phrases[i&255]),wordCount);
  measure(`${name}/stateless-encode`,i=>api.encodeId(ids[i&255],format),wordCount);
  codec.dispose();
}
integers('animal-u32',{lists:Array(4).fill('animal'),range:1n<<32n});
if (api.lists().some(list=>list.name==='eff-long')) {
  integers('eff-u32',{lists:Array(3).fill('eff-long'),range:1n<<32n});
  integers('variable-u32',{scheme:'variable-v1',pattern:[{list:'adjective',repeat:{min:0}},'animal'],range:1n<<32n});
  const custom={name:'custom',words:Array.from({length:256},(_,i)=>`token${i}`)};
  integers('custom-u32',{lists:Array(4).fill(custom),range:1n<<32n});
  for(const lists of [['eff-long'],Array(3).fill('eff-long'),['adjective','animal']]) {
    const codec=api.prepareBytes({scheme:'radix-bytes-v1',lists});
    const payload=new Uint8Array(16).map((_,i)=>i*13), phrase=codec.encodeBytes(payload);
    if(String(codec.decodeBytes(phrase))!==String(payload))throw Error('Bytes round trip');
    measure(`bytes-16/${lists.join(',')}/encode`,()=>codec.encodeBytes(payload),phrase.split(' ').length);
    measure(`bytes-16/${lists.join(',')}/decode`,()=>codec.decodeBytes(phrase),phrase.split(' ').length);
    codec.dispose();
  }
}
const require = createRequire(new URL('./js/package.json', import.meta.url));
const niceware = require('niceware');
const nicePayload = Buffer.from(Array.from({length:16},(_,i)=>i*13));
const nicePhrase = niceware.bytesToPassphrase(nicePayload).join(' ');
if (!niceware.passphraseToBytes(nicePhrase.split(' ')).equals(nicePayload)) throw Error('Niceware round trip');
measure('niceware-bytes-16/encode',()=>niceware.bytesToPassphrase(nicePayload).join(' '),8);
measure('niceware-bytes-16/decode',()=>niceware.passphraseToBytes(nicePhrase.split(' ')),8);
const report={schema:'nwords.wordsets-benchmark.v1',node:process.version,arch:process.arch,date:new Date().toISOString(),sourceCommit:execFileSync('git',['rev-parse','HEAD'],{cwd:root,encoding:'utf8'}).trim(),dirty:execFileSync('git',['status','--porcelain'],{cwd:root,encoding:'utf8'}).trim()!=='',module:resolve(process.argv[2]??'dist/nwords-js/src/node.js'),build:JSON.parse(await readFile(resolve(dirname(resolve(process.argv[2]??'dist/nwords-js/src/node.js')),'../../package-build.json'),'utf8')),cpuAffinity:execFileSync('taskset',['-pc',String(process.pid)],{encoding:'utf8'}).trim(),wrapperSha256:createHash('sha256').update(await readFile(resolve(dirname(resolve(process.argv[2]??'dist/nwords-js/src/node.js')),'api.js'))).digest('hex'),sink,rows};
const output=process.argv[3];if(output)await writeFile(output,JSON.stringify(report,null,2)+'\n');else console.log(JSON.stringify(report,null,2));
