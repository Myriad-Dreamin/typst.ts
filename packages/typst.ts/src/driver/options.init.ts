// @ts-ignore
import type * as typst from '../../pkg/typst_renderer_ts';

export type StagedOptFn<S extends symbol, T = any> = (s: S, t: T) => Promise<void>;

/// this mark is used to identify the beforeBuild stage
const BeforeBuildSymbol = Symbol('beforeBuild');
export type BeforeBuildMark = typeof BeforeBuildSymbol;
export type BeforeBuildFn = StagedOptFn<BeforeBuildMark, any>;

export interface TypstRendererInitOptions {
  /// before build stage, the registered functions will be executed in order
  /// possible options:
  /// - preload remote fonts: preloadRemoteFonts
  /// - preload system fonts: preloadSystemFonts
  beforeBuild: BeforeBuildFn[];
}

export function preloadRemoteFonts(fonts: string[]): BeforeBuildFn {
  return async (_, { ref, builder }: InitContext) => {
    await Promise.all(fonts.map(font => ref.loadFont(builder, font)));
  };
}

export function preloadSystemFonts({ byFamily }: { byFamily?: string[] }): BeforeBuildFn {
  return async (_, { builder }: InitContext) => {
    const t = performance.now();

    if ('queryLocalFonts' in window) {
      const fonts = await (window as any).queryLocalFonts();
      console.log('local fonts count:', fonts.length);

      byFamily = byFamily ?? [];

      for (const font of fonts) {
        if (!byFamily.includes(font.family)) {
          continue;
        }

        console.log(font.family);

        const data: ArrayBuffer = await (await font.blob()).arrayBuffer();
        await builder.add_raw_font(new Uint8Array(data));
      }
    }

    const t2 = performance.now();
    console.log('font loading', t2 - t);
  };
}

// todo: search browser
// searcher.search_browser().await?;

interface InitContext {
  ref: {
    loadFont(builder: typst.TypstRendererBuilder, fontPath: string): Promise<void>;
  };
  builder: typst.TypstRendererBuilder;
}
