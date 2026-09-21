import { loadVariable } from '@y4le/nwords/variable/web';
import { adjective } from '@y4le/nwords/wordsets/adjective';
import { animal } from '@y4le/nwords/wordsets/animal';
const { defineVariable } = await loadVariable();
const codec = defineVariable({ scheme: 'variable-v1', pattern: [{ list: adjective, repeat: { min: 1 } }, animal], maxWords: 32 });
document.querySelector('#phrase').textContent = codec.encodeId(42n);
