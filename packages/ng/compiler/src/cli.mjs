import { mkdtemp, readFile, rm, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { basename, join } from 'node:path';
import { spawn } from 'node:child_process';
import { normalizeInput, unsupported } from './shared.mjs';

export async function createCliCompiler(options = {}) {
  return new CliCompilerFacade(options.cli || options);
}

export class CliCompilerFacade {
  backend = 'cli';

  constructor(options = {}) {
    this.command = options.command || 'typst';
    this.vectorCommand = options.vectorCommand || options.typstTsCommand || 'typst-ts-cli';
    this.cwd = options.cwd || process.cwd();
    this.env = options.env;
  }

  compile(input) {
    return this.vector(input);
  }

  vector(input) {
    return this.runFormat('vector', input, 'artifact.sir.in', true);
  }

  pdf(input) {
    return this.runFormat('pdf', input, 'pdf', true);
  }

  plainSvg(input) {
    return this.runFormat('svg', input, 'artifact.svg', false);
  }

  svg(input) {
    return this.plainSvg(input);
  }

  html(input) {
    return this.runFormat('html', input, 'html', false);
  }

  setFontProvider() {
    unsupported('cli', 'setFontProvider');
  }

  setAccessModel() {
    unsupported('cli', 'setAccessModel');
  }

  setPackageProvider() {
    unsupported('cli', 'setPackageProvider');
  }

  setPackageRegistry() {
    unsupported('cli', 'setPackageRegistry');
  }

  async runFormat(format, input, extension, binary) {
    if (format === 'vector') {
      return this.runTypstTsFormat(format, input, extension, binary);
    }

    return this.runOfficialFormat(format, input, extension, binary);
  }

  async runOfficialFormat(format, input, extension, binary) {
    const opts = normalizeInput(input);
    const dir = await mkdtemp(join(tmpdir(), 'typst-compiler-'));
    const entry = opts.mainFileContent == null ? opts.mainFilePath : '-';
    const outputBase = entry === '-' ? 'main' : basename(entry).replace(/\.[^.]*$/, '');
    const outputPath = join(dir, `${outputBase}.${extension}`);

    if (!entry) {
      throw new Error('cli backend requires mainFilePath or mainFileContent');
    }

    const args = ['compile'];
    const root = opts.workspace || opts.root;
    if (root) {
      args.push('--root', root);
    }

    for (const [key, value] of Object.entries(opts.inputs || {})) {
      args.push('--input', `${key}=${value}`);
    }

    args.push(entry, outputPath);

    try {
      await runProcess(this.command, args, {
        cwd: this.cwd,
        env: this.env,
        input: opts.mainFileContent,
      });

      const data = await readFile(outputPath);
      return binary ? new Uint8Array(data.buffer, data.byteOffset, data.byteLength) : data.toString();
    } finally {
      await rm(dir, { recursive: true, force: true });
    }
  }

  async runTypstTsFormat(format, input, extension, binary) {
    const opts = normalizeInput(input);
    const dir = await mkdtemp(join(tmpdir(), 'typst-compiler-'));
    const sourcePath = join(dir, 'main.typ');
    const entry = opts.mainFileContent == null ? opts.mainFilePath : sourcePath;
    const outputBase = basename(entry || sourcePath).replace(/\.[^.]*$/, '');
    const outputPath = join(dir, `${outputBase}.${extension}`);

    if (!entry) {
      throw new Error('cli backend requires mainFilePath or mainFileContent');
    }

    if (opts.mainFileContent != null) {
      await writeFile(sourcePath, opts.mainFileContent);
    }

    const args = [
      'compile',
      '--workspace',
      opts.workspace || opts.root || (opts.mainFileContent == null ? this.cwd : dir),
      '--entry',
      entry,
      '--output',
      dir,
      '--format',
      format,
    ];

    for (const [key, value] of Object.entries(opts.inputs || {})) {
      args.push('--input', `${key}=${value}`);
    }

    try {
      await runProcess(this.vectorCommand, args, {
        cwd: this.cwd,
        env: this.env,
      });

      const data = await readFile(outputPath);
      return binary ? new Uint8Array(data.buffer, data.byteOffset, data.byteLength) : data.toString();
    } finally {
      await rm(dir, { recursive: true, force: true });
    }
  }
}

function runProcess(command, args, options) {
  return new Promise((resolve, reject) => {
    const child = spawn(command, args, {
      cwd: options.cwd,
      env: options.env,
      stdio: ['pipe', 'pipe', 'pipe'],
    });

    const stdout = [];
    const stderr = [];

    child.stdout.on('data', chunk => stdout.push(chunk));
    child.stderr.on('data', chunk => stderr.push(chunk));
    child.on('error', reject);
    child.on('close', code => {
      if (code === 0) {
        resolve(Buffer.concat(stdout));
        return;
      }

      reject(
        new Error(
          `${command} ${args.join(' ')} failed with exit code ${code}\n` +
            Buffer.concat(stderr).toString(),
        ),
      );
    });

    if (options.input != null) {
      child.stdin.end(options.input);
    } else {
      child.stdin.end();
    }
  });
}

export { createCliCompiler as createCompiler };
