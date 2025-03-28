import { defineConfig } from 'vite';
import { TypstPlugin } from '@myriaddreamin/vite-plugin-typst';

export default defineConfig({
  plugins: [TypstPlugin({compiler: 'typst-cli'})],
});
