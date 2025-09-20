import { defineConfig } from 'vite';
import type { RollupOptions } from 'rollup';
import vue from '@vitejs/plugin-vue';
import { resolve } from 'path';

export const rollupOptions: RollupOptions = {
  // make sure to externalize deps that shouldn't be bundled
  // into your library
  external: ['vue', '@myriaddreamin/typst.ts'],
  output: {
    // Provide global variables to use in the UMD build
    // for externalized deps
    globals: {
      vue: 'Vue',
      '@myriaddreamin/typst.ts': 'TypstTs',
    },
    compact: true,
    inlineDynamicImports: true,
  },
};

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [vue()],
  build: {
    lib: {
      entry: resolve(__dirname, 'src/index.ts'),
      name: 'Typst',
      fileName: (format) => `index.${format}.js`,
      formats: ['es', 'umd'],
    },
    rollupOptions,
  },
});
