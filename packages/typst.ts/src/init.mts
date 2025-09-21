import { BeforeBuildMark, InitOptions, LazyFont } from './options.init.mjs';
import { LazyWasmModule } from './wasm.mjs';

/** @internal */
export interface TypstCommonBuilder<T> {
  free(): void;

  add_raw_font(font_buffer: Uint8Array): Promise<void>;

  add_lazy_font<C extends LazyFont>(
    info: C,
    blob: (this: C, index: number) => Uint8Array,
  ): Promise<void>;

  build(): Promise<T>;
}

/** @internal */
export interface ComponentBuildHooks {
  latelyBuild?: (ctx: unknown) => void;
}

interface InitContext<T> {
  ref: {
    loadFonts(builder: TypstCommonBuilder<T>, fonts: (string | Uint8Array)[]): Promise<void>;
  };
  builder: TypstCommonBuilder<T>;
  hooks: ComponentBuildHooks;
}

class ComponentBuilder<T> {
  loadedFonts = new Set<string>();
  fetcher?: typeof fetch = fetch;

  setFetcher(fetcher: typeof fetch): void {
    this.fetcher = fetcher;
  }

  async loadFonts(
    builder: TypstCommonBuilder<T>,
    fonts: (string | Uint8Array | LazyFont)[],
  ): Promise<void> {
    const escapeImport = new Function('m', 'return import(m)');
    const fetcher = (this.fetcher ||= await (async function () {
      const { fetchBuilder, FileSystemCache } = await escapeImport('node-fetch-cache');
      const cache = new FileSystemCache({
        /// By default, we don't have a complicated cache policy.
        cacheDirectory: '.cache/typst/fonts',
      });

      const cachedFetcher = fetchBuilder.withCache(cache);

      return function (input: RequestInfo | URL, init?: RequestInit) {
        const timeout = setTimeout(() => {
          console.warn('font fetching is stucking:', input);
        }, 15000);
        return cachedFetcher(input, init).finally(() => {
          clearTimeout(timeout);
        });
      };
    })());

    const fontsToLoad = fonts.filter(font => {
      if (font instanceof Uint8Array || (typeof font === 'object' && 'info' in font)) {
        return true;
      }

      if (this.loadedFonts.has(font)) {
        return false;
      }

      this.loadedFonts.add(font);
      return true;
    });

    const fontLists = await Promise.all(
      fontsToLoad.map(async font => {
        if (font instanceof Uint8Array) {
          await builder.add_raw_font(font);
          return;
        }
        if (typeof font === 'object' && 'info' in font) {
          await builder.add_lazy_font(font, 'blob' in font ? font.blob : loadFontSync(font));
          return;
        }

        return new Uint8Array(await (await fetcher(font)).arrayBuffer());
      }),
    );

    for (const font of fontLists) {
      if (!font) {
        continue;
      }
      await builder.add_raw_font(font);
    }
  }

  async build(
    options: Partial<InitOptions> | undefined,
    builder: TypstCommonBuilder<T>,
    hooks: ComponentBuildHooks,
  ): Promise<T> {
    /// build typst component
    const buildCtx: InitContext<T> = { ref: this, builder, hooks };

    for (const fn of options?.beforeBuild ?? []) {
      await fn(undefined as unknown as BeforeBuildMark, buildCtx);
    }
    // await addPartialFonts(buildCtx);

    if (hooks.latelyBuild) {
      hooks.latelyBuild(buildCtx);
    }

    return await builder.build();
  }
}

/** @internal */
export async function buildComponent<T>(
  options: Partial<InitOptions> | undefined,
  gModule: LazyWasmModule,
  Builder: { new (): TypstCommonBuilder<T> },
  hooks: ComponentBuildHooks,
): Promise<T> {
  /// init typst wasm module
  await gModule.init(options?.getModule?.());

  return await new ComponentBuilder<T>().build(options, new Builder(), hooks);
}

/**
 * Loads a font by a lazy font synchronously, which is required by the compiler.
 * @param font
 */
export function loadFontSync(font: LazyFont & { url: string }): (index: number) => Uint8Array {
  return () => {
    const xhr = new XMLHttpRequest();
    xhr.overrideMimeType('text/plain; charset=x-user-defined');
    xhr.open('GET', font.url, false);
    xhr.send(null);

    if (
      xhr.status === 200 &&
      (xhr.response instanceof String || typeof xhr.response === 'string')
    ) {
      return Uint8Array.from(xhr.response, (c: string) => c.charCodeAt(0));
    }
    return new Uint8Array();
  };
}
