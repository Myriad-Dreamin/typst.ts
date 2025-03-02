import * as path from 'path';
import * as fs from 'fs';
import { globSync } from 'glob';
import type { ResolvedConfig, Plugin as VitePlugin } from 'vite';
import { NodeCompiler } from '@myriaddreamin/typst-ts-node-compiler';

type TypstCompileProvider = '@myriaddreamin/typst-ts-node-compiler';

export interface TypstDocumentOptions {
  root?: string;
}

export interface TypstDocumentOptionsWithInput extends TypstDocumentOptions {
  input: string | string[];
}

type DocumentInput = string | TypstDocumentOptionsWithInput;

/**
 * Vite plugin for Typst
 */
export interface TypstPluginOptions extends TypstDocumentOptions {
  index?: DocumentInput;
  documents?: DocumentInput | DocumentInput[];
  // uriPrefix?: string;
  compiler?: TypstCompileProvider;
}

interface TypstInput {
  input: string;
  root?: string;
}

type ViteInputs = Record<string, string>;
type TypstInputs = Record<string, TypstInput>;

function normalizeDocumentInput(input: DocumentInput): TypstDocumentOptionsWithInput {
  if (typeof input === 'string') {
    return {
      input,
    };
  }

  return input;
}

function normalizeDocumentInputs(
  input: DocumentInput | DocumentInput[],
): TypstDocumentOptionsWithInput[] {
  if (Array.isArray(input)) {
    return input.map(normalizeDocumentInput);
  }

  return [normalizeDocumentInput(input)];
}

// https://github.com/vbenjs/vite-plugin-html/blob/4a32df2b416b161663904de51530f462a6219fd5/packages/core/src/htmlPlugin.ts#L179
function resolveInputs(
  opts: TypstPluginOptions,
  viteConfig: ResolvedConfig,
): TypstInputs | undefined {
  const viteInputs: TypstInputs = {};
  if (!opts.index) {
    const indexTyp = path.resolve(viteConfig.root || '.', 'index.typ');
    if (fs.existsSync(indexTyp)) {
      viteInputs['index'] = {
        input: indexTyp,
        root: opts.root || viteConfig.root,
      };
    }
  } else {
    const index = normalizeDocumentInput(opts.index);
    if (typeof index.input === 'string') {
      viteInputs['index'] = {
        input: index.input,
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
      viteInputs[m] = {
        input: m,
        root: doc.root || opts.root || viteConfig.root,
      };
    }
  }

  if (Object.keys(viteInputs).length === 0) {
    return undefined;
  }
  return viteInputs;
}

export function TypstPlugin(options: TypstPluginOptions = {}): VitePlugin {
  const { root } = options;
  const compilerProvider = options?.compiler || '@myriaddreamin/typst-ts-node-compiler';
  if (compilerProvider !== '@myriaddreamin/typst-ts-node-compiler') {
    throw new Error(`Unsupported compiler provider: ${compilerProvider}`);
  }

  let viteConfig: ResolvedConfig;
  let compiler: NodeCompiler;

  let inputRoot: string;
  let outputDir: string;

  let typstInputs: TypstInputs = {};

  const cache = new Map<string, string>();

  const resolveRel = (input: string) => {
    const rel = input.endsWith('.typ') ? input.slice(0, -4) : input;
    return path.relative(inputRoot, `${rel}.html`);
  };

  return {
    name: 'myriad-dreamin:vite-plugin-typst',
    enforce: 'pre',
    configResolved(resolvedConfig) {
      viteConfig = resolvedConfig;

      inputRoot = path.resolve(viteConfig.root || '.');
      outputDir = path.resolve(viteConfig.build.outDir || 'dist');

      compiler = NodeCompiler.create({
        workspace: root || inputRoot,
      });
    },
    config(conf) {
      typstInputs = resolveInputs(options, conf as unknown as ResolvedConfig)!;

      let input: ViteInputs | undefined;
      if (typstInputs) {
        input = input || {};
        for (const [key, { input: inputPath }] of Object.entries(typstInputs)) {
          input[key] = inputPath;
        }
      }

      if (input) {
        return {
          build: {
            rollupOptions: {
              input,
            },
          },
        };
      }
    },

    async buildStart(options) {
      const inputs = (
        options.input instanceof Array ? options.input : Object.values(options.input)
      ) as string[];

      for (const input of inputs) {
        const res = compiler.tryHtml({
          mainFilePath: input,
        });
        if (res.hasError()) {
          console.log(` \x1b[1;31mError\x1b[0m ${input}`);
          res.printErrors();
        } else {
          console.log(` \x1b[1;32mCompiled\x1b[0m ${input}`);
        }

        const dst = resolveRel(input);
        // todo: resolveRel may override file paths.
        // todo: html is fat
        cache.set(resolveRel(input), res.result!.html());
      }
    },

    load(id) {
      const html = cache.get(id);
      if (html) {
        return html;
      }
    },

    resolveId(source, importer, options) {
      // todo: detect shebangs
      if (!source.endsWith('.typ')) return null;

      return resolveRel(source);
    },

    configureServer(server) {
      server.middlewares.use((req, res, next) => {
        const url = req.url!;
        const toGet = url.endsWith('/') ? `${url}index.html` : url;
        const toGetWithoutPrefix = toGet.startsWith('/') ? toGet.slice(1) : toGet;
        // get cache
        const html = cache.get(toGetWithoutPrefix);
        console.log('middleware', req.url, !!html);
        if (html) {
          res.setHeader('Content-Type', 'text/html');
          res.end(html);
          return;
        }

        next();
      });
    },
  };
}
