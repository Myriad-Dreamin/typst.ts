import { defineConfig } from 'vite';
import { TypstPlugin } from '@myriaddreamin/vite-plugin-typst';

export default defineConfig({
  plugins: [TypstPlugin({
    compiler: 'typst-cli',
    documents: ['**/*.typ'],
    onCompile: (input, project, ctx) => {
      const res = project.compileHtml(input);
      if (res.hasError()) {
        res.printErrors();
        process.exit(1);
      }
    }
  })],
});
