import { BeforeBuildMark, InitOptions } from './options.init';
import { LazyWasmModule } from './wasm';

/** @internal */
export interface TypstCommonBuilder<T> {
  free(): void;

  add_raw_font(font_buffer: Uint8Array): Promise<void>;

  add_web_fonts(font: any[]): Promise<void>;

  build(): Promise<T>;
}

/** @internal */
export interface ComponentBuildHooks {}

interface InitContext<T> {
  ref: {
    loadFont(builder: TypstCommonBuilder<T>, fontPath: string): Promise<void>;
  };
  builder: TypstCommonBuilder<T>;
  hooks: ComponentBuildHooks;
}

async function addPartialFonts<T>({ builder, hooks }: InitContext<T>): Promise<void> {
  const t = performance.now();

  if ('queryLocalFonts' in window) {
    const fonts: any[] = await (window as any).queryLocalFonts();
    console.log('local fonts count:', fonts.length);
    await builder.add_web_fonts(fonts);
  }

  const t2 = performance.now();
  console.log('addPartialFonts time used:', t2 - t);
}

/** @internal */
async function buildComponentInternal<T>(
  options: Partial<InitOptions> | undefined,
  gModule: LazyWasmModule,
  builder: TypstCommonBuilder<T>,
  hooks: ComponentBuildHooks,
): Promise<T> {
  /// init typst wasm module
  if (options?.getModule) {
    await gModule.init(options.getModule());
  }

  /// build typst component
  const buildCtx: InitContext<T> = { ref: this, builder, hooks };

  for (const fn of options?.beforeBuild ?? []) {
    await fn(undefined as unknown as BeforeBuildMark, buildCtx);
  }
  addPartialFonts(buildCtx);

  const component = await builder.build();

  return component;
}

/** @internal */
export async function buildComponent<T>(
  options: Partial<InitOptions> | undefined,
  gModule: LazyWasmModule,
  Builder: { new (): TypstCommonBuilder<T> },
  hooks: ComponentBuildHooks,
): Promise<T> {
  const builder = new Builder();
  try {
    return buildComponentInternal(options, gModule, builder, hooks);
  } finally {
    builder.free();
  }
}
