import { fileURLToPath } from 'url';
import * as path from 'path';
import { existsSync } from 'fs';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

function findGitRoot() {
  let p = __dirname,
    lastP = '';
  while (p !== lastP) {
    if (existsSync(path.resolve(p, '.git/HEAD'))) {
      return p;
    }
    lastP = p;
    p = path.resolve(p, '..');
  }
  throw new Error('git root not found');
}

export const projectRoot = findGitRoot();
