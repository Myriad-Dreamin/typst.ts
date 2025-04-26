import type { CompileArgs, NodeHtmlOutputExecResult } from '@myriaddreamin/typst-ts-node-compiler';
import * as path from 'path';
import type { ResolvedConfig, Plugin as VitePlugin } from 'vite';
import {
  CompileProvider,
  CompileProviderConstructor,
  HtmlOutputExecResult,
  OnCompileCallback,
  PartialCallback,
  TypstHTMLCompiler,
} from './compiler.js';
import type { CliCompileProvider } from './compiler/cli.js';
import type { NodeCompileProvider } from './compiler/node.js';
import { InputChecker, ResolvedTypstInput, ResolvedTypstInputs } from './input.js';

// Constantly known compiler providers
interface Providers {
  'typst-cli': CliCompileProvider;
  '@myriaddreamin/typst-ts-node-compiler': NodeCompileProvider;
}

// Known provider kind
type ProviderKind = keyof Providers;
type Provider<K> = K extends undefined
  ? NodeCompileProvider
  : K extends ProviderKind
    ? Providers[K]
    : K extends CompileProviderConstructor
      ? InstanceType<K>
      : never;

// Common options for the plugin
export interface TypstPluginOptionsBase extends TypstDocumentOptions {
  /**
   * The index document to be compiled.
   * If not provided, the plugin will try to find `index.typ` in the root directory.
   * When `index` is set to true, if "index.typ" is not found, the plugin will not compile any document.
   * By vite's convention, it will continuing finding a static "index.html".
   *
   * When `index` is set to false, the plugin will not compile any document.
   *
   * @default true
   */
  index?: DocumentInput | boolean;
  /**
   * The documents to be compiled.
   *
   * See also {@link TypstPlugin} and {@link DocumentInput}.
   *
   * @default []
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
   * A callback to be called when the inputs are resolved.
   */
  onInputs?: (typstInputs: ResolvedTypstInputs) => void;

  /**
   * Whether to override the route in `vite.configureServer`.
   * @default true
   */
  overrideRoute?: boolean;
  /**
   * Provides `sys.inputs` for the document.
   */
  fontArgs?: CompileArgs['fontArgs'];
}

// options depending on the kind of compiler (node or CLI)
export interface TypstPluginOptions<K extends ProviderKind | CompileProviderConstructor | undefined>
  extends TypstPluginOptionsBase {
  /**
   * The compiler provider, either a string for a built-in provider or a constructor for a custom provider.
   * @default '@myriaddreamin/typst-ts-node-compiler'
   */
  compiler?: K | CompileProviderConstructor;
  /**
   * *Override* the callback to be called when the parts is resolving.
   */
  onResolveParts?: OnCompileCallback<any, Provider<K>>;
  /**
   * *Override* the callback to be called when the inputs are compiling.
   */
  onCompile?: OnCompileCallback<void, Provider<K>>;
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

  /**
   * Provides `sys.inputs` for the document.
   */
  inputs?: Record<string, string>;
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
export async function TypstPlugin<
  K extends ProviderKind | CompileProviderConstructor | undefined = undefined,
>(options: TypstPluginOptions<K> = {}): Promise<VitePlugin> {
  if (options.index === undefined) {
    options.index = true;
  }

  const inputs = new InputChecker(options);
  const { provider, partsFunc } = await createCompiler<K>(options);
  let reload: () => void = undefined!;

  const extractOpts = (path: string) => {
    const attributes: Record<string, boolean> = {};
    if (path.endsWith('?html')) {
      path = path.slice(0, -5);
      attributes['html'] = true;
    }
    if (path.endsWith('?parts')) {
      path = path.slice(0, -6);
      attributes['parts'] = true;
    }
    return { path, attributes };
  };

  const suffixJs = '.vite-plugin-typst.js';

  return Promise.resolve<VitePlugin>({
    name: 'myriad-dreamin:vite-plugin-typst',
    enforce: 'pre',

    configResolved(conf) {
      viteReload(conf);
    },

    buildStart() {
      reload();
    },

    load(id) {
      const memoryHtml = provider.compiled.get(id);
      if (memoryHtml) {
        return memoryHtml;
      }

      let isJs = id.endsWith(suffixJs);
      if (!isJs) return null;
      id = id.slice(0, -suffixJs.length);

      const { path, attributes } = extractOpts(id);
      const input = { mainFilePath: path };

      // todo: cache js import
      this.addWatchFile(path);
      // console.log('load isWatch', path, compiler.isWatch);
      if (provider.isWatch) {
        provider.compileOrWatch(input);
      }
      // todo: remove any
      const project: TypstHTMLCompiler<any> = provider.compiler();
      const result = defaultCompile(input, project, provider);
      if (!result?.result) {
        return undefined;
      }

      const doc = result.result!;

      if (attributes.parts) {
        const userParts: any = partsFunc(input, project);
        if (typeof userParts !== 'object') {
          throw new Error('onResolveParts must return an object');
        }
        const parts = {
          title: doc.title(),
          description: doc.description(),
          body: doc.body(),
          ...userParts,
        };

        const bindingsCode = Object.keys(parts)
          .map(key => `export const ${key} = parts[${JSON.stringify(key)}];`)
          .join('\n');
        return `const parts = ${JSON.stringify(parts)};
${bindingsCode}
export default parts;`;
      }

      return `export default ${JSON.stringify(result.result!.html())}`;
    },

    resolveId(source) {
      const { path, attributes } = extractOpts(source);
      if (!path.endsWith('.typ')) return null;
      if (attributes.html || attributes.parts) {
        return source + suffixJs;
      }
      return provider.resolveRel(path);
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
      if (provider.isWatch) {
        provider.watcher().clear();
      }
    },

    configureServer(server) {
      if (options.overrideRoute === false) return;

      server.middlewares.use((req, res, next) => {
        const url = req.url!;
        const toGet = url.endsWith('/') ? `${url}index.html` : url;
        const toGetWithoutPrefix = toGet.startsWith('/') ? toGet.slice(1) : toGet;
        // get cache
        const html = provider.compiled.get(toGetWithoutPrefix);
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

  function viteReload(conf: ResolvedConfig) {
    provider.inputRoot = path.resolve(conf.root ?? '.');
    provider.isWatch = !!(conf.mode === 'development' || conf.build?.watch);
    provider.compileArgs.workspace = options.root ?? provider.inputRoot;

    reload = doReload;

    if (provider.isWatch) {
      inputs.watch(reload);
    }

    function doReload() {
      // console.log('reload c');
      if (!inputs.mutate(options, conf)) {
        return;
      }

      // console.log('reload 1');

      if (options.onInputs) {
        options.onInputs(inputs.resolved);
      }
      if (provider.isWatch) {
        provider.watcher().clear();
      }
      for (const input of Object.values(inputs.resolved)) {
        provider.compileOrWatch(input);
      }
      if (provider.isWatch) {
        provider.watcher().watch();
      }

      // console.log('reload');
    }
  }
}

export default TypstPlugin;

const defaultCompile: OnCompileCallback<HtmlOutputExecResult | undefined> = (
  input,
  project,
  ctx,
) => {
  const htmlResult = project.tryHtml(input);

  // Only print the error once
  if (htmlResult.hasError()) {
    // console.log(` \x1b[1;31mError\x1b[0m ${mainFilePath}`);
    htmlResult.printDiagnostics();

    // todo: how could we raise error if not in watch mode?
    if (!ctx.isWatch) {
      console.error(new Error(`Failed to compile ${input.mainFilePath}`));
      process.exit(1);
    }
    return;
  }

  // todo: resolveRel may override file paths.
  // todo: html is fat
  const htmlContent = htmlResult.result!.html();
  ctx.compiled.set(ctx.resolveRel(input.mainFilePath), htmlContent);
  // console.log(` \x1b[1;32mCompiled\x1b[0m ${mainFilePath}`);

  return htmlResult;
};

async function createCompiler<K extends ProviderKind | CompileProviderConstructor | undefined>(
  options: TypstPluginOptions<K>,
) {
  let TCompileProvider = (
    options.compiler === undefined || options.compiler === '@myriaddreamin/typst-ts-node-compiler'
      ? (await import('./compiler/node.js')).NodeCompileProvider
      : options.compiler === 'typst-cli'
        ? (await import('./compiler/cli.js')).CliCompileProvider
        : CompileProvider.isCons<Provider<K>>(options.compiler)
          ? options.compiler
          : undefined
  ) as CompileProviderConstructor<Provider<K>> | undefined;
  if (!TCompileProvider) throw new Error(`Unsupported compiler provider: ${options.compiler}`);

  const compileArgs: CompileArgs = {
    workspace: path.resolve(options.root ?? '.'),
    ...{ inputs: options.inputs, fontArgs: options.fontArgs },
  };
  const provider = new TCompileProvider(false, compileArgs, options.onCompile ?? defaultCompile);
  const partsFunc: PartialCallback = (mainFilePath, project) =>
    options.onResolveParts?.(mainFilePath, project, provider) ?? {};
  return {
    provider,
    partsFunc,
  };
}

export interface ExecResult<T>
  extends Pick<NodeHtmlOutputExecResult, 'hasError' | 'printErrors' | 'printDiagnostics'> {
  result: T | null;
}

export function checkExecResult<R>(
  input: ResolvedTypstInput,
  result: ExecResult<R> | undefined,
  ctx: any,
): R | undefined {
  if (!result) {
    return;
  }

  // Only print the error once
  if (result.hasError()) {
    result.printErrors();

    // todo: how could we raise error if not in watch mode?
    if (!ctx.isWatch) {
      console.error(new Error(`Failed to compile ${input}`));
      process.exit(1);
    }
    return undefined;
  }
  return result.result || undefined;
}

const _test: TypstPluginOptions<ProviderKind>['onResolveParts'] = {} as any;
