import { mkdirSync } from 'fs';

mkdirSync('./dist/esm', { recursive: true });
mkdirSync('./dist/cjs', { recursive: true });
