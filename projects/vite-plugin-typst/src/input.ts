import * as fs from 'fs';
import { globSync } from 'glob';
import globWatch from 'glob-watcher';
import * as path from 'path';
import type { ResolvedConfig } from 'vite';
import type { DocumentInput, TypstDocumentOptionsWithInput, TypstPluginOptionsBase } from '.';

/**
 * A resolved typst input.
 */
export interface ResolvedTypstInput {
  /**
   * A single path to the input file.
   */
  mainFilePath: string;
  /**
   * The root directory of the document.
   */
  root?: string;
  /**
   * Provides `sys.inputs` for the document.
   */
  inputs?: Record<string, string>;
}

/**
 * A list of resolved typst inputs.
 */
export type ResolvedTypstInputs = Record<string, ResolvedTypstInput>;

type ResolvedViteInputs = Record<string, string>;

/**
 * The input checker manages to resolve the inputs for vite and the typst compiler.
 */
export class InputChecker {
  /**
   * The resolved inputs.
   */
  resolved: ResolvedTypstInputs = {};

  /** @internal */
  private documents: TypstDocumentOptionsWithInput[];
  /** @internal */
  private globs: string[];
  /** @internal */
  private watcher: fs.FSWatcher | undefined = undefined;
  /** @internal */
  private loadedTypstInputs: string = '';
  /** @internal */
  private prevReload: (() => void) | undefined = undefined;

  constructor(options: TypstPluginOptionsBase) {
    this.documents = normalizeDocumentInputs(options.documents || []);
    this.globs = this.documents.flatMap(doc => {
      if (typeof doc.input === 'string') {
        return [doc.input];
      }
      return doc.input;
    });
  }

  /**
   * Returns the resolved inputs for vite.
   *
   * @returns The resolved inputs for vite.
   */
  asVite(): ResolvedViteInputs | undefined {
    let input: ResolvedViteInputs | undefined;
    if (this.resolved) {
      input = input || {};
      for (const [key, { mainFilePath }] of Object.entries(this.resolved)) {
        input[key] = mainFilePath;
      }
    }

    return input;
  }

  /**
   * Mutates the input resolver with the new options.
   *
   * @param options The new options.
   * @param conf The vite config.
   *
   * @returns Whether the inputs have changed.
   */
  mutate(options: TypstPluginOptionsBase, conf: ResolvedConfig): boolean {
    this.resolved = resolveInputs(options, conf)!;
    const newLoadedTypstInputs = JSON.stringify(this.resolved);
    if (newLoadedTypstInputs === this.loadedTypstInputs) {
      return false;
    }
    this.loadedTypstInputs = newLoadedTypstInputs;
    // console.log('new', newLoadedTypstInputs);
    return true;
  }

  /**
   * Watch the inputs for changes.
   *
   * @param reload The reload callback.
   */
  watch(reload: () => void): void {
    if (this.watcher && this.prevReload) {
      this.watcher.off('add', this.prevReload);
      this.watcher.off('remove', this.prevReload);
    }
    if (!this.watcher) {
      this.watcher = globWatch(this.globs);
    }
    // When these files change, we need to reload the documents.
    this.watcher.on('add', reload);
    this.watcher.on('remove', reload);
    this.prevReload = reload;
  }

  /**
   * Close the watcher.
   */
  close(): void {
    if (this.watcher) {
      this.watcher.close();
    }
  }
}

function normalizeDocumentInputs(
  input: DocumentInput | DocumentInput[],
): TypstDocumentOptionsWithInput[] {
  if (Array.isArray(input)) {
    return input.map(normalizeDocumentInput);
  }

  return [normalizeDocumentInput(input)];
}

function normalizeDocumentInput(input: DocumentInput): TypstDocumentOptionsWithInput {
  if (typeof input === 'string') {
    return {
      input,
    };
  }

  return input;
}

// https://github.com/vbenjs/vite-plugin-html/blob/4a32df2b416b161663904de51530f462a6219fd5/packages/core/src/htmlPlugin.ts#L179
function resolveInputs(
  opts: TypstPluginOptionsBase,
  viteConfig: ResolvedConfig,
): ResolvedTypstInputs | undefined {
  const resolved: ResolvedTypstInputs = {};
  if (opts.index === false) {
  } else if (opts.index === true || opts.index === undefined) {
    const indexTyp = path.resolve(viteConfig.root || '.', 'index.typ');
    if (fs.existsSync(indexTyp)) {
      resolved['index'] = {
        mainFilePath: indexTyp,
        root: opts.root || viteConfig.root,
      };
    }
  } else {
    const index = normalizeDocumentInput(opts.index);
    if (typeof index.input === 'string') {
      resolved['index'] = {
        mainFilePath: index.input,
        root: index.root || opts.root || viteConfig.root,
      };
    } else {
      throw new Error('index.input should be a string');
    }
  }

  for (const doc of normalizeDocumentInputs(opts.documents || [])) {
    const paths = typeof doc.input === 'string' ? [doc.input] : doc.input;
    const matched = [];
    for (const p of paths) {
      const matchedP = globSync(p, {
        cwd: viteConfig.root,
      });
      for (const mp of matchedP) {
        if (mp.endsWith('.typ')) {
          matched.push(mp);
        }
      }
    }

    for (const m of matched) {
      resolved[m] = {
        mainFilePath: m,
        root: doc.root,
        inputs: doc.inputs,
      };
    }
  }

  if (Object.keys(resolved).length === 0) {
    return undefined;
  }
  return resolved;
}
