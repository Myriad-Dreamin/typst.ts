import { execSync } from 'child_process';
import { readFileSync, writeFileSync, existsSync } from 'fs';
import { resolve, join } from 'path';

const gzBundleSize = path => {
  const filePath = resolve(import.meta.dirname, path);
  const command = `gzip -c ${filePath} | wc -c`;
  const size = execSync(command).toString().trim();

  return parseInt(size, 10);
};

const typstTsRendererWasmSize = gzBundleSize(
  '../node_modules/@myriaddreamin/typst-ts-renderer/pkg/typst_ts_renderer_bg.wasm',
);
const typstTsWebCompilerWasmSize = gzBundleSize(
  '../node_modules/@myriaddreamin/typst-ts-web-compiler/pkg/typst_ts_web_compiler_bg.wasm',
);

const defaultFonts = [
  'https___raw_githubusercontent_com_Myriad_Dreamin_typst_assets_fonts_DejaVuSansMono_BoldOblique_ttf',
  'https___raw_githubusercontent_com_Myriad_Dreamin_typst_assets_fonts_DejaVuSansMono_Bold_ttf',
  'https___raw_githubusercontent_com_Myriad_Dreamin_typst_assets_fonts_DejaVuSansMono_Oblique_ttf',
  'https___raw_githubusercontent_com_Myriad_Dreamin_typst_assets_fonts_DejaVuSansMono_ttf',
  'https___raw_githubusercontent_com_Myriad_Dreamin_typst_assets_fonts_LinLibertine_RBI_ttf',
  'https___raw_githubusercontent_com_Myriad_Dreamin_typst_assets_fonts_LinLibertine_RB_ttf',
  'https___raw_githubusercontent_com_Myriad_Dreamin_typst_assets_fonts_LinLibertine_RI_ttf',
  'https___raw_githubusercontent_com_Myriad_Dreamin_typst_assets_fonts_LinLibertine_R_ttf',
  'https___raw_githubusercontent_com_Myriad_Dreamin_typst_assets_fonts_NewCM10_BoldItalic_otf',
  'https___raw_githubusercontent_com_Myriad_Dreamin_typst_assets_fonts_NewCM10_Bold_otf',
  'https___raw_githubusercontent_com_Myriad_Dreamin_typst_assets_fonts_NewCM10_Italic_otf',
  'https___raw_githubusercontent_com_Myriad_Dreamin_typst_assets_fonts_NewCM10_Regular_otf',
  'https___raw_githubusercontent_com_Myriad_Dreamin_typst_assets_fonts_NewCMMath_Book_otf',
  'https___raw_githubusercontent_com_Myriad_Dreamin_typst_assets_fonts_NewCMMath_Regular_otf',
];

const cjkFonts = [
  'https___raw_githubusercontent_com_Myriad_Dreamin_typst_assets_fonts_InriaSerif_BoldItalic_ttf',
  'https___raw_githubusercontent_com_Myriad_Dreamin_typst_assets_fonts_InriaSerif_Bold_ttf',
  'https___raw_githubusercontent_com_Myriad_Dreamin_typst_assets_fonts_InriaSerif_Italic_ttf',
  'https___raw_githubusercontent_com_Myriad_Dreamin_typst_assets_fonts_InriaSerif_Regular_ttf',
  'https___raw_githubusercontent_com_Myriad_Dreamin_typst_assets_fonts_NotoSerifCJKsc_Regular_otf',
  'https___raw_githubusercontent_com_Myriad_Dreamin_typst_assets_fonts_Roboto_Regular_ttf',
];

const emojiFonts = [
  'https___raw_githubusercontent_com_Myriad_Dreamin_typst_assets_fonts_NotoColorEmoji_ttf',
  'https___raw_githubusercontent_com_Myriad_Dreamin_typst_assets_fonts_TwitterColorEmoji_ttf',
];

const fontSize = fonts => {
  let fontSize = 0;
  for (const font of fonts) {
    // ~/.local/share/typst/fonts
    const fontPath = join(process.env.HOME, '.local/share/typst/fonts', font);
    if (!existsSync(fontPath)) {
      console.warn(`Font not found: ${fontPath}`);
      continue;
    }
    const command = `gzip -c ${fontPath} | wc -c`;
    const size = execSync(command).toString().trim();
    fontSize += parseInt(size, 10);
  }
  return fontSize;
};

const defaultFontSize = fontSize(defaultFonts);
const cjkFontSize = fontSize(cjkFonts);
const emojiFontSize = fontSize(emojiFonts);

const version = JSON.parse(
  readFileSync(
    resolve(
      import.meta.dirname,
      '../node_modules/@myriaddreamin/typst-ts-web-compiler/package.json',
    ),
  ),
).version;

const result = {
  version,
  sizes: {
    'typst-ts-renderer': typstTsRendererWasmSize,
    'typst-ts-web-compiler': typstTsWebCompilerWasmSize,
    'text-math-fonts': defaultFontSize,
    'cjk-fonts': cjkFontSize,
    'emoji-fonts': emojiFontSize,
  },
};

writeFileSync(
  resolve(import.meta.dirname, '../assets/data/bundle-size.json'),
  JSON.stringify(result, null, 2),
);
