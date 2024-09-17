import { defineConfig } from 'vite'
import solid from 'vite-plugin-solid'

import Package from './package.json';

export default defineConfig({
  plugins: [solid()],
  build: {
    lib: {
      name: Package.name,
      entry: 'src/index.tsx',
      fileName: 'index',
      formats: ['es', 'cjs'],
    },
    rollupOptions: {
      external: ['solid-js', 'solid-js/web', '@myriaddreamin/typst.ts',
        '@myriaddreamin/typst-ts-web-compiler', '@myriaddreamin/typst-ts-renderer'],
    },
  },
})
