import * as path from 'path';
import type { ResolvedConfig, Plugin as VitePlugin } from 'vite';
import { NodeCompiler } from '@myriaddreamin/typst-ts-node-compiler';

type TypstCompileProvider = '@myriaddreamin/typst-ts-node-compiler';

/**
 * Vite plugin for Typst
 */
export interface TypstPluginOptions {
  root?: string;
  // uriPrefix?: string;
  compiler?: TypstCompileProvider;
}

type Inputs = Record<string, string>;

// https://github.com/vbenjs/vite-plugin-html/blob/4a32df2b416b161663904de51530f462a6219fd5/packages/core/src/htmlPlugin.ts#L179
function resolveInputs(viteConfig: ResolvedConfig): Inputs {
  const input: Record<string, string> = {
    index: path.resolve(viteConfig.root || '.', 'index.typ'),
  };

  return input;
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
      const input = resolveInputs(conf as unknown as ResolvedConfig);
      // console.log('config', input);

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
