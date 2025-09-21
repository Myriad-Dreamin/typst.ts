
import * as fs from 'fs/promises';

const coreDump = await fs.readFile('target/t.txt', 'utf-8');

// decode the core dump from base64
const decodedCoreDump = Buffer.from(coreDump, 'base64');
console.log('decodedCoreDump.length', decodedCoreDump.length);


await fs.writeFile('fuzzers/corpora/skyzh-cv/debug.sir.in', decodedCoreDump);

