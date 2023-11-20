// npx esbuild --watch ./src/index.ts --minify --outfile=dist/index.min.js

import esbuild from 'esbuild';
import {updateDebugScript} from './scripts/debug.mjs';

const plugins = [{
    name: 'my-plugin',
    setup(build) {
      build.onEnd(result => {
        updateDebugScript();
        console.log('update finished');
      });
    },
  }];

const ctx = await esbuild.context({
  entryPoints: [
    'src/index.ts',
  ],
  outfile: 'dist/index.min.js',
  bundle: true,
  format: 'cjs',
  platform: 'browser',
  logLevel: 'info',
  sourcemap: 'inline',
  treeShaking: true,
  plugins,
});

await ctx.watch();
// await ctx.dispose(); // To free resources

