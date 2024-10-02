// build.js

import { defineConfig } from 'vite';
// import { viteSingleFile } from 'vite-plugin-singlefile';

const component = process.argv.find((arg) => arg.startsWith('--component='));
const componentName = component ? component.split('=')[1] : 'main';

const libs = {
  'main': {
    entry: {
      'main': './src/main.mts',
    },
    name: 'main',
  },
  'main2': {
    entry: {
      'main2': './src/main2.mts',
    },
    name: 'main2',
  },
  'canvas-worker': {
    entry: {
      'canvas-worker': './src/contrib/canvas-worker.mts',
    },
    name: 'canvas-worker',
  }
};

const lib = libs[componentName];
if (!lib) {
  console.error(`[typst.ts] Unknown component: ${componentName}`);
  process.exit(1);
}

// build
export default defineConfig({
  plugins: [],
  configFile: false,
  build: {
    lib: {
      ...lib,
      formats: ['es', 'cjs'],
    },
    emptyOutDir: false,
    rollupOptions: {
      output: {
        inlineDynamicImports: true,
      }
      // other options
    },
  },
});
