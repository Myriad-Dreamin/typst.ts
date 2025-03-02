import * as path from 'path';
import * as fs from 'fs';
import { globSync } from 'glob';
import globWatch from 'glob-watcher';
import type { ResolvedConfig, Plugin as VitePlugin } from 'vite';
import { NodeCompiler, ProjectWatcher } from '@myriaddreamin/typst-ts-node-compiler';

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
  onInputs?: (typstInputs: TypstInputs) => void;
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

  const documents = normalizeDocumentInputs(options.documents || []);
  const globs = documents.flatMap(doc => {
    if (typeof doc.input === 'string') {
      return [doc.input];
    }
    return doc.input;
  });

  let viteConfig: ResolvedConfig;

  const compileArgs = {
    workspace: root || '.',
    inputs: {
      'x-target': 'web-light',
      // ...(urlBase ? { 'x-url-base': urlBase } : {}),
    },
    fontArgs: [{ fontPaths: ['./assets/fonts', './assets/typst-fonts'] }],
  };

  /**
   * Lazily created compiler.
   */
  let _compiler: NodeCompiler | undefined = undefined;
  /**
   * Lazily created compiler.
   */
  const compiler = (): NodeCompiler => (_compiler ||= NodeCompiler.create(compileArgs));
  let _watcher: ProjectWatcher | undefined = undefined;
  /**
   * Lazily created watcher
   */
  const watcher = (): ProjectWatcher => (_watcher ||= ProjectWatcher.create(compileArgs));

  /**
   * Common getter for the compiler or watcher.
   */
  const compilerOrWatcher = () => _compiler || _watcher;

  /**
   * Compiles the source file to the destination file.
   *
   * @param {string} src The source file path
   *
   * @example
   * compile("src/index.typ", "dist/index.html")(compiler());
   */
  const compile = (src: string) => {
    return (compiler: import('@myriaddreamin/typst-ts-node-compiler').NodeTypstProject) => {
      const htmlResult = compiler.tryHtml({
        mainFilePath: src,
        ...compileArgs,
      });

      // Only print the error once
      if (htmlResult.hasError()) {
        console.log(` \x1b[1;31mError\x1b[0m ${src}`);
        htmlResult.printErrors();
        return;
      }

      // todo: resolveRel may override file paths.
      // todo: html is fat
      const htmlContent = htmlResult.result!.html();
      cache.set(resolveRel(src), htmlContent);

      console.log(` \x1b[1;32mCompiled\x1b[0m ${src}`);

      // Evicts the cache unused in last 30 runs
      compilerOrWatcher()?.evictCache(30);
    };
  };

  /**
   * User trigger compiles the source file to the destination file or watches the source file.
   *
   * All the errors are caught and printed to the console.
   *
   * @param {string} src The source file path
   *
   * @example
   * compileOrWatch("src/index.typ", "dist/index.html");
   */
  const compileOrWatch = (src: string) => {
    try {
      if (isWatch) {
        watcher().add([src], compile(src));
      } else {
        compile(src)(compiler());
      }
    } catch (e) {
      console.error(e);
      return;
    }
  };

  let inputRoot: string;
  let outputDir: string;
  let inputWatcher: fs.FSWatcher | undefined = undefined;

  let typstInputs: TypstInputs = {};
  let loadedTypstInputs: string = '';

  let isWatch = false;
  let reload = () => {};
  let firstReload = true;

  const createReload = (conf: ResolvedConfig) => {
    return () => {
      typstInputs = resolveInputs(options, conf)!;
      const newLoadedTypstInputs = JSON.stringify(typstInputs);
      if (newLoadedTypstInputs === loadedTypstInputs) {
        return;
      }
      loadedTypstInputs = newLoadedTypstInputs;
      // console.log('new', newLoadedTypstInputs);

      if (options.onInputs) {
        options.onInputs(typstInputs);
      }
      if (isWatch) {
        watcher().clear();
      }
      for (const input of Object.values(typstInputs)) {
        compileOrWatch(input.input);
      }
      if (isWatch) {
        watcher().watch();
      }
    };
  };

  const viteReload = (conf: ResolvedConfig) => {
    viteConfig = conf;
    inputRoot = path.resolve(viteConfig.root || '.');
    outputDir = path.resolve(viteConfig.build?.outDir || 'dist');
    // isWatch = !!(conf.server || conf.build?.watch);
    // console.log('isWatch', isWatch, conf.mode);
    isWatch = !!(conf.mode === 'development' || conf.build?.watch);
    // console.log('isWatch', isWatch, conf.server, conf.build?.watch);
    compileArgs.workspace = root || conf.root || '.';
    if (inputWatcher && !firstReload) {
      inputWatcher.off('add', reload);
      inputWatcher.off('remove', reload);
    }
    firstReload = false;
    reload = createReload(conf);
    if (isWatch) {
      // When these files change, we need to reload the documents.
      inputWatcher = globWatch(globs);
      inputWatcher.on('add', reload);
      inputWatcher.on('remove', reload);
    }
  };

  const cache = new Map<string, string>();

  const resolveRel = (input: string) => {
    const rel = input.endsWith('.typ') ? input.slice(0, -4) : input;
    return path.relative(inputRoot, `${rel}.html`);
  };

  return {
    name: 'myriad-dreamin:vite-plugin-typst',
    enforce: 'pre',
    configResolved(resolvedConfig) {
      viteReload(resolvedConfig);
    },
    config(conf) {
      viteReload(conf as unknown as ResolvedConfig);
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

    async buildStart(_options) {
      reload();
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
        // console.log('middleware', req.url, !!html);
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

export default TypstPlugin;
