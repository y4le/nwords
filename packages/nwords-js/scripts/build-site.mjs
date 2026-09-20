import { readFile } from 'node:fs/promises';
import { join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { build } from 'vite';

const root = fileURLToPath(new URL('../../../', import.meta.url));
const packageRoot = join(root, 'dist/nwords-js');
const manifest = JSON.parse(await readFile(join(packageRoot, 'package.json'), 'utf8'));
const prefix = `${manifest.name}/`;
await build({
  root: join(root, 'site'),
  base: './',
  logLevel: 'warn',
  plugins: [{
    name: 'packed-nwords-exports',
    resolveId(id) {
      if (!id.startsWith(prefix)) return null;
      const entry = manifest.exports[`./${id.slice(prefix.length)}`];
      const relative = typeof entry === 'string' ? entry : entry?.import;
      if (!relative) throw new Error(`Missing package export: ${id}`);
      return resolve(packageRoot, relative);
    },
  }],
  build: { outDir: join(root, 'dist/site'), emptyOutDir: true, target: 'es2022' },
});
