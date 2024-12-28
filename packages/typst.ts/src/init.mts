import { BeforeBuildMark, InitOptions } from './options.init.mjs';
import { LazyWasmModule } from './wasm.mjs';
import * as idb from 'idb';

/** @internal */
export interface TypstCommonBuilder<T> {
  free(): void;

  add_raw_font(font_buffer: Uint8Array): Promise<void>;

  add_web_fonts(font: any[]): Promise<void>;

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

/** @internal */
export const globalFontPromises: Promise<{ buffer: ArrayBuffer; idx: number }>[] = [];

async function addPartialFonts<T>({ builder, hooks }: InitContext<T>): Promise<void> {
  const t = performance.now();

  if ('queryLocalFonts' in window) {
    const fonts: any[] = await (window as any).queryLocalFonts();
    console.log('local fonts count:', fonts.length);

    const db = await idb.openDB('typst-ts-store', 1, {
      upgrade(db) {
        db.createObjectStore('font-information', {
          keyPath: 'postscriptName',
        });
      },
    });

    const information = await Promise.all(
      fonts.map(async font => {
        const postscriptName = font.postscriptName;

        return (await db.get('font-information', postscriptName))?.info;
      }),
    );

    const get_font_info = (builder as any).handler_for_font_info();

    await builder.add_web_fonts(
      fonts.map((font, font_idx) => {
        let gettingBuffer = false;
        let readyBuffer: ArrayBuffer | undefined = undefined;
        const fullName = font.fullName;
        const postscriptName = font.postscriptName;

        const prev = information[font_idx];
        if (prev) {
          console.log('prev', postscriptName, prev);
        }
        return {
          family: font.family,
          style: font.style,
          fullName: fullName,
          postscriptName: postscriptName,
          ref: font,
          info: information[font_idx],
          blob: (idx: number) => {
            console.log(this, font, idx);
            if (readyBuffer) {
              return readyBuffer;
            }
            if (gettingBuffer) {
              return;
            }
            gettingBuffer = true;
            globalFontPromises.push(
              (async () => {
                const blob: Blob = await font.blob();
                const buffer = await blob.arrayBuffer();
                readyBuffer = buffer;
                const realFontInfo = get_font_info(new Uint8Array(buffer));
                console.log(realFontInfo);

                db.put('font-information', {
                  fullName,
                  postscriptName,
                  info: realFontInfo,
                });

                return { buffer, idx };
              })(),
            );
          },
        };
      }),
    );
  }

  const t2 = performance.now();
  console.log('addPartialFonts time used:', t2 - t);
}

class ComponentBuilder<T> {
  loadedFonts = new Set<string>();
  fetcher?: typeof fetch = fetch;

  setFetcher(fetcher: typeof fetch): void {
    this.fetcher = fetcher;
  }

  async loadFonts(builder: TypstCommonBuilder<T>, fonts: (string | Uint8Array)[]): Promise<void> {
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
      if (font instanceof Uint8Array) {
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

    const component = await builder.build();

    return component;
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
