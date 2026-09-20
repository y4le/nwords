import { execFileSync } from 'node:child_process';
import { cp, mkdir, readFile, readdir, rename, rm, writeFile } from 'node:fs/promises';
import { dirname, join, resolve } from 'node:path';
import { homedir } from 'node:os';
import { fileURLToPath } from 'node:url';
import { createHash } from 'node:crypto';

const root = fileURLToPath(new URL('../../../', import.meta.url));
const project = fileURLToPath(new URL('../', import.meta.url));
const output = join(root, 'dist/nwords-js');
const tarballs = join(root, 'dist/tarballs');
let dev = false;
let profile = 'baseline';
let optimizer;
const args = process.argv.slice(2);
for (let index = 0; index < args.length; index++) {
  const flag = args[index];
  if (flag === '--dev') dev = true;
  else if (flag === '--profile' && ['baseline', 'thin', 'fat', 'size'].includes(args[index + 1])) profile = args[++index];
  else if (flag === '--wasm-opt' && args[index + 1] && !args[index + 1].startsWith('--')) optimizer = resolve(args[++index]);
  else throw new Error('Usage: npm run build -- [--dev] [--profile baseline|thin|fat|size] [--wasm-opt /path/to/wasm-opt]');
}
const cargoHome = resolve(process.env.CARGO_HOME ?? join(homedir(), '.cargo'));
const rustflags = [`--remap-path-prefix=${root}=/nwords/`, `--remap-path-prefix=${cargoHome}=/cargo`];
const env = { ...process.env, RUSTUP_TOOLCHAIN: '1.94.0', CARGO_ENCODED_RUSTFLAGS: rustflags.join('\x1f') };
delete env.RUSTFLAGS;
for (const key of ['CARGO_INCREMENTAL', 'RUSTC_WRAPPER', 'RUSTC_WORKSPACE_WRAPPER', 'RUSTC']) delete env[key];
for (const key of Object.keys(env)) if (key.startsWith('CARGO_PROFILE_')) delete env[key];
const run = (command, args, cwd = root) => execFileSync(command, args, { cwd, env, encoding: 'utf8', stdio: ['ignore', 'pipe', 'inherit'] }).trim();
const sha256 = bytes => createHash('sha256').update(bytes).digest('hex');
const commit = run('git', ['rev-parse', 'HEAD']);
const status = run('git', ['status', '--porcelain=v1', '--untracked-files=all']);
if (status && !dev) throw new Error('Qualification builds require a clean committed revision. Use --dev for an explicitly dirty development artifact.');
const rustc = run('rustc', ['--version']);
const cargo = run('cargo', ['--version']);
const wasmPack = run('wasm-pack', ['--version']);
if (!rustc.startsWith('rustc 1.94.0 ') || wasmPack !== 'wasm-pack 0.14.0') {
  throw new Error('Install Rust 1.94.0 (with wasm32-unknown-unknown) and wasm-pack 0.14.0.');
}
if (process.version !== 'v24.14.1' || run('npm', ['--version']) !== '11.11.0') throw new Error('Build with Node 24.14.1 and npm 11.11.0.');
const lock = await readFile(join(root, 'Cargo.lock'), 'utf8');
if (!/name = "wasm-bindgen"\nversion = "0\.2\.120"/u.test(lock)) throw new Error('The locked wasm-bindgen must be 0.2.120.');

const optimizationFlags = ['-O3', '--enable-bulk-memory', '--enable-mutable-globals', '--enable-sign-ext', '--enable-nontrapping-float-to-int', '--enable-multivalue', '--enable-reference-types', '--enable-bulk-memory-opt', '--enable-call-indirect-overlong'];
let optimizerInfo;
if (optimizer) {
  const version = run(optimizer, ['--version']);
  if (version !== 'wasm-opt version 131 (version_131)') throw new Error('Use pinned Binaryen 131.');
  optimizerInfo = { version, executableSha256: sha256(await readFile(optimizer)), flags: optimizationFlags };
}
const profileArgs = profile === 'baseline' ? ['--release'] : ['--profile', `wasm-${profile}`];

await rm(output, { recursive: true, force: true });
await mkdir(output, { recursive: true });
// Disable wasm-pack downloads; an explicitly selected, verified optimizer runs below.
run('wasm-pack', ['build', 'crates/nwords-js', '--target', 'web', ...profileArgs, '--no-opt', '--no-pack', '--out-dir', output + '/wasm', '--locked']);
if (optimizerInfo) {
  const input = join(output, 'wasm/nwords_js_bg.wasm');
  const optimized = join(output, 'wasm/optimized.wasm');
  run(optimizer, [input, ...optimizationFlags, '-o', optimized]);
  await rename(optimized, input);
}
// wasm-pack writes an ignore-all file that npm otherwise applies recursively.
await rm(join(output, 'wasm/.gitignore'));
for (const name of ['src', 'examples', 'notices']) await cp(join(project, name), join(output, name), { recursive: true });
for (const name of ['README.md', 'NOTICE.md']) await cp(join(project, name), join(output, name));
for (const name of ['LICENSE-MIT', 'LICENSE-APACHE']) await cp(join(root, name), join(output, name));

// Include conservative notices for every named list compiled by the feature,
// even when optimization removes data unreachable through the selected package API.
const wordNotices = join(output, 'notices/wordlists');
for (const group of ['eff-long', 'adjective-animal', 'friendly-words', 'semantic-wordlists/mood', 'semantic-wordlists/material', 'semantic-wordlists/shape', 'semantic-wordlists/weather', 'semantic-wordlists/plant', 'semantic-wordlists/food']) {
  const target = join(wordNotices, group);
  await mkdir(target, { recursive: true });
  await cp(join(root, 'tests/vectors', group, 'README.md'), join(target, 'README.md'));
}
await mkdir(join(wordNotices, 'licenses'), { recursive: true });
for (const name of ['unique-names-generator-MIT.LICENSE', 'glitch-friendly-words-MIT.LICENSE', 'eff-CC-BY-4.0.LICENSE', 'python-mnemonic-MIT.LICENSE']) {
  await cp(join(root, 'tests/vectors/licenses', name), join(wordNotices, 'licenses', name));
}

// cargo tree selects this crate's actual feature graph; workspace metadata by
// itself can include other members' optional BIP-39 dependencies.
const graph = new Set(run('cargo', ['tree', '--locked', '-p', 'nwords-js', '--target', 'wasm32-unknown-unknown', '--edges', 'normal', '--prefix', 'none', '--format', '{p}'])
  .split('\n').map(line => line.split(' ').slice(0, 2).join(' ')));
const metadata = JSON.parse(run('cargo', ['metadata', '--locked', '--format-version', '1']));
const binding = metadata.packages.find(pkg => pkg.name === 'nwords-js');
const dependencies = [];
for (const pkg of metadata.packages) {
  if (!pkg.source || !graph.has(`${pkg.name} v${pkg.version}`)) continue;
  const source = dirname(pkg.manifest_path);
  const target = join(output, 'notices/dependencies', `${pkg.name}-${pkg.version}`);
  const files = (await readdir(source, { withFileTypes: true }))
    .filter(entry => entry.isFile() && /^(LICENSE|COPYING|NOTICE)/iu.test(entry.name)).map(entry => entry.name).sort();
  if (!files.length) throw new Error(`Missing license texts for ${pkg.name}`);
  await mkdir(target, { recursive: true });
  for (const file of files) await cp(join(source, file), join(target, file));
  dependencies.push({ name: pkg.name, version: pkg.version, license: pkg.license, repository: pkg.repository, notices: files });
}
await writeFile(join(output, 'notices/dependencies.json'), JSON.stringify(dependencies, null, 2) + '\n');
const template = JSON.parse(await readFile(join(project, 'package.template.json'), 'utf8'));
if (template.version !== binding?.version) throw new Error('Package and binding crate versions must match.');
if (template.private !== true || template.scripts) throw new Error('Development package must be private with no lifecycle scripts.');
template.repository = { type: 'git', url: 'https://github.com/y4le/nwords.git' };
const wasm = await readFile(join(output, 'wasm/nwords_js_bg.wasm'));
const provenance = {
  schema: 'nwords.build.v1', sourceCommit: commit, dirty: Boolean(status), development: dev,
  tools: { rustc, cargo, wasmPack, wasmBindgen: '0.2.120', node: process.version, npm: run('npm', ['--version']) },
  rustflags: ['--remap-path-prefix=<workspace>/=/nwords/', '--remap-path-prefix=<cargo-home>=/cargo'],
  wasm: { target: 'web', optimization: `rustc-${profile === 'baseline' ? 'release' : 'wasm-' + profile}; ${optimizerInfo ? optimizerInfo.version + ' -O3' : 'wasm-opt disabled'}`, profile, optimizer: optimizerInfo ?? null, bytes: wasm.length, sha256: sha256(wasm) },
  cargoLockSha256: sha256(lock),
};
await writeFile(join(output, 'build.json'), JSON.stringify(provenance, null, 2) + '\n');
await writeFile(join(output, 'package.json'), JSON.stringify(template, null, 2) + '\n');
if (run('git', ['rev-parse', 'HEAD']) !== commit || run('git', ['status', '--porcelain=v1', '--untracked-files=all']) !== status) {
  throw new Error('Source tree changed during assembly; rebuild before qualifying this artifact.');
}
await mkdir(tarballs, { recursive: true });
const packed = JSON.parse(run('npm', ['pack', '--ignore-scripts', '--json', '--pack-destination', tarballs], output))[0];
const tarball = resolve(tarballs, packed.filename);
const report = { ...provenance, tarball, packedBytes: packed.size, unpackedBytes: packed.unpackedSize, integrity: packed.integrity, tarballSha256: sha256(await readFile(tarball)) };
await writeFile(join(root, 'dist/package-build.json'), JSON.stringify(report, null, 2) + '\n');
console.log(JSON.stringify(report, null, 2));
