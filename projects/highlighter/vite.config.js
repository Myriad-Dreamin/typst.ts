import { defineConfig } from 'vite';

export default defineConfig({
  plugins: [],
  build: {
    rollupOptions: {
      external: /.*[\\/]contrib[\\/].*/g,
    },
    outDir: 'out',
    minify: false,
  },
});
