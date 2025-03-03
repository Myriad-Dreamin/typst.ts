import * as path from 'path';
import type { ResolvedConfig, Plugin as VitePlugin } from 'vite';
import { makeProvider, OnCompileCallback } from './compiler.js';
import { ResolvedTypstInputs, InputChecker } from './input.js';

type TypstCompileProvider = '@myriaddreamin/typst-ts-node-compiler';

/**
 * Vite plugin for Typst
 */
export interface TypstPluginOptions extends TypstDocumentOptions {
  /**
   * The index document to be compiled.
   * If not provided, the plugin will try to find `index.typ` in the root directory.
   * If "index.typ" is not found, the plugin will not compile any document. By vite's convention, it will continuing finding a static "index.html".
   */
  index?: DocumentInput;
  /**
   * The documents to be compiled.
   *
   * See also {@link TypstPlugin} and {@link DocumentInput}.
   *
   * @example
   * ```ts
   * TypstPlugin({ documents: 'src/*.typ' })
   * ```
   *
   * @example
   * ```ts
   * TypstPlugin({ documents: ['content/articles/*.typ', 'content/posts/*.typ'] })
   * ```
   *
   * @example
   * ```ts
   * TypstPlugin({ documents: [
   *   { input: 'content/articles/*.typ', root: 'content/articles/' },
   *   { input: 'content/posts/*.typ', root: 'content/posts/' },
   * ] })
   * ```
   */
  documents?: DocumentInput | DocumentInput[];
  /**
   * The compiler provider.
   * @default '@myriaddreamin/typst-ts-node-compiler'
   */
  compiler?: TypstCompileProvider;
  /**
   * A callback to be called when the inputs are resolved.
   */
  onInputs?: (typstInputs: ResolvedTypstInputs) => void;
  /**
   * A callback to be called when the inputs are compiling.
   */
  onCompile?: OnCompileCallback;

  /**
   * Whether to override the route in `vite.configureServer`.
   * @default true
   */
  overrideRoute?: boolean;
}

/**
 * The input glob pattern relative to vite's root directory or the grouped input with {@link TypstDocumentOptions}.
 *
 */
export type DocumentInput = string | TypstDocumentOptionsWithInput;

/**
 * Common typst document options
 */
export interface TypstDocumentOptions {
  /**
   * The root directory of the document.
   * If not provided, the plugin will use the vite's root directory.
   */
  root?: string;
}

/**
 * Typst document options with input
 */
export interface TypstDocumentOptionsWithInput extends TypstDocumentOptions {
  /**
   * The input glob pattern relative to vite's root directory.
   */
  input: string | string[];
}

/**
 * The Vite plugin for Typst.
 *
 * @param options The plugin options
 * @returns A Vite plugin for Typst
 */
export function TypstPlugin(options: TypstPluginOptions = {}): Promise<VitePlugin> {
  const inputs = new InputChecker(options);
  const compiler = createCompiler(options);

  let reload: () => void = undefined!;
  const viteReload = (conf: ResolvedConfig) => {
    compiler.inputRoot = path.resolve(conf.root || '.');
    compiler.isWatch = !!(conf.mode === 'development' || conf.build?.watch);
    compiler.args.workspace = options.root || compiler.inputRoot;

    reload = doReload;

    if (compiler.isWatch) {
      inputs.watch(reload);
    }

    function doReload() {
      if (!inputs.mutate(options, conf)) {
        return;
      }

      if (options.onInputs) {
        options.onInputs(inputs.resolved);
      }
      if (compiler.isWatch) {
        compiler.watcher().clear();
      }
      for (const input of Object.values(inputs.resolved)) {
        compiler.compileOrWatch(input.input);
      }
      if (compiler.isWatch) {
        compiler.watcher().watch();
      }
    }
  };

  return Promise.resolve({
    name: 'myriad-dreamin:vite-plugin-typst',
    enforce: 'pre',

    configResolved(conf) {
      viteReload(conf);
    },

    buildStart() {
      reload();
    },

    load(id) {
      return compiler.compiled.get(id);
    },

    resolveId(source) {
      // todo: detect shebangs
      if (!source.endsWith('.typ')) return null;
      return compiler.resolveRel(source);
    },

    config(conf) {
      viteReload(conf as unknown as ResolvedConfig);
      inputs.mutate(options, conf as unknown as ResolvedConfig)!;

      const input = inputs.asVite();
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

    closeWatcher() {
      inputs.close();
      if (compiler.isWatch) {
        compiler.watcher().clear();
      }
    },

    configureServer(server) {
      if (options.overrideRoute === false) return;

      server.middlewares.use((req, res, next) => {
        const url = req.url!;
        const toGet = url.endsWith('/') ? `${url}index.html` : url;
        const toGetWithoutPrefix = toGet.startsWith('/') ? toGet.slice(1) : toGet;
        // get cache
        const html = compiler.compiled.get(toGetWithoutPrefix);
        // console.log('middleware', req.url, !!html);
        if (html) {
          res.setHeader('Content-Type', 'text/html');
          res.end(html);
          return;
        }

        next();
      });
    },
  });
}

export default TypstPlugin;

function createCompiler(options: TypstPluginOptions) {
  return makeProvider(options, (src, project, ctx) => {
    const htmlResult = project.tryHtml({
      mainFilePath: src,
      ...ctx.args,
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
    ctx.compiled.set(ctx.resolveRel(src), htmlContent);
    console.log(` \x1b[1;32mCompiled\x1b[0m ${src}`);
  });
}
