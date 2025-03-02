import { defineConfig } from 'vite';
import { TypstPlugin } from '@myriad-dreamin/vite-plugin-typst';

export default defineConfig({
  plugins: [TypstPlugin({ documents: ['content/**/*.typ'] })],
});
