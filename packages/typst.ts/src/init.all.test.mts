import { afterEach, describe, expect, it, vi } from 'vitest';
import compilerUrl from '../../compiler/pkg/typst_ts_web_compiler_bg.wasm?url';
import { createTypstCompiler } from './compiler.mjs';
import { buildComponent, type TypstCommonBuilder } from './init.mjs';
import { disableDefaultFontAssets } from './options.init.mjs';
import { LazyWasmModule } from './wasm.mjs';

const isNode =
  typeof process !== 'undefined' && process.versions != null && process.versions.node != null;

const getCompilerModule = () => {
  if (!isNode) {
    return compilerUrl;
  }
  const fs = require('fs');
  const path = require('path');
  return fs.readFileSync(
    path.join(import.meta.dirname, '../../compiler/pkg/typst_ts_web_compiler_bg.wasm'),
  );
};

class TestBuilder implements TypstCommonBuilder<number> {
  readonly fonts: Uint8Array[] = [];

  free(): void {}

  async add_raw_font(font: Uint8Array): Promise<void> {
    this.fonts.push(font);
  }

  async add_lazy_font(): Promise<void> {}

  async build(): Promise<number> {
    return this.fonts.length;
  }
}

afterEach(() => {
  vi.unstubAllGlobals();
});

describe('component initialization', () => {
  it('initializes the compiler without dynamic Function construction', async () => {
    const constructFunction = vi.fn(() => {
      throw new Error('dynamic Function construction is blocked by CSP');
    });
    vi.stubGlobal('Function', constructFunction);
    const compiler = createTypstCompiler();

    await compiler.init({
      beforeBuild: [disableDefaultFontAssets()],
      getModule: getCompilerModule,
    });

    expect(constructFunction).not.toHaveBeenCalled();
  });

  it('does not construct the Node import fallback when a fetcher is available', async () => {
    const constructFunction = vi.fn(() => {
      throw new Error('dynamic Function construction is blocked by CSP');
    });
    vi.stubGlobal('Function', constructFunction);
    const fontBuffer = new Uint8Array([1, 2, 3]).buffer;
    const fetcher = vi.fn(async () => ({
      arrayBuffer: async () => fontBuffer,
    })) as unknown as typeof fetch;
    const wasm = new LazyWasmModule(async () => undefined);

    const fontCount = await buildComponent(
      {
        getModule: () => new Uint8Array(),
        beforeBuild: [async (_mark, context) => {
          const { ref, builder } = context as {
            ref: {
              setFetcher(fetcher: typeof fetch): void;
              loadFonts(builder: TestBuilder, fonts: string[]): Promise<void>;
            };
            builder: TestBuilder;
          };
          ref.setFetcher(fetcher);
          await ref.loadFonts(builder, ['https://example.com/font.ttf']);
        }],
      },
      wasm,
      TestBuilder,
      {},
    );

    vi.unstubAllGlobals();
    expect(fontCount).toBe(1);
    expect(fetcher).toHaveBeenCalledOnce();
    expect(constructFunction).not.toHaveBeenCalled();
  });
});
