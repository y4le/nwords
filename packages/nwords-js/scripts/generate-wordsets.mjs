import { readFile, mkdir, writeFile } from 'node:fs/promises';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = fileURLToPath(new URL('../../../', import.meta.url));
const output = join(root, 'packages/nwords-js/src/wordsets');
const sources = {
  adjective: 'adjective-animal/nwords-adjectives.txt',
  animal: 'adjective-animal/nwords-animals.txt',
  color: 'adjective-animal/unique-names-generator-colors.txt',
  object: 'friendly-words/nwords-objects.txt',
  descriptor: 'friendly-words/nwords-descriptors.txt',
  mood: 'semantic-wordlists/mood/nwords-moods.txt',
  material: 'semantic-wordlists/material/nwords-materials.txt',
  shape: 'semantic-wordlists/shape/nwords-shapes.txt',
  weather: 'semantic-wordlists/weather/nwords-weather.txt',
  plant: 'semantic-wordlists/plant/nwords-plants.txt',
  food: 'semantic-wordlists/food/nwords-foods.txt',
  'eff-long': 'eff-long/words.txt',
};
await mkdir(output, { recursive: true });
for (const [name, source] of Object.entries(sources)) {
  const input = await readFile(join(root, 'tests/vectors', source), 'utf8');
  const words = input.trimEnd().split('\n');
  if (words.length < 2 || new Set(words).size !== words.length || words.some(word => !/^[a-z-]+$/.test(word))) {
    throw new Error(`Invalid wordset snapshot: ${source}`);
  }
  const identifier = name.replace(/-([a-z])/g, (_, letter) => letter.toUpperCase());
  const body = `// Generated from tests/vectors/${source}; order is part of the format.\n` +
    `export const ${identifier} = Object.freeze({\n  name: ${JSON.stringify(name)},\n  words: Object.freeze(${JSON.stringify(words)}),\n});\n`;
  await writeFile(join(output, `${name}.js`), body);
  await writeFile(join(output, `${name}.d.ts`),
    `import type { Wordset } from '../variable-wasm/types.js';\nexport const ${identifier}: Wordset;\n`);
}
