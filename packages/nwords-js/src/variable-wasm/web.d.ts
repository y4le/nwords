import type { VariableFactory } from './types.js';
export type { Wordset, VariableDefinition, VariableCodec, VariableFactory } from './types.js';
export function loadVariable(): Promise<VariableFactory>;
export { NwordsError, NwordsLoadError } from '../index.js';
