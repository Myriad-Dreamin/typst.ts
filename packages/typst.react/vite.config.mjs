import { defineConfig } from 'vite';
import { resolve } from 'path';
import react from '@vitejs/plugin-react';

export default defineConfig({
  plugins: [react()],
  build: {
    emptyOutDir: false,
    lib: {
      entry: resolve(__dirname, 'src/lib/index.tsx'),
      formats: ['es', 'cjs'],
    },
    rollupOptions: {
      output: {
        inlineDynamicImports: true,
      },
      external: ['react', 'react/jsx-runtime'],
    },
  },
});
