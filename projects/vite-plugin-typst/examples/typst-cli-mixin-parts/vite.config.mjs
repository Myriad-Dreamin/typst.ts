import { defineConfig } from 'vite';
import { TypstPlugin, checkExecResult } from '@myriaddreamin/vite-plugin-typst';

export default defineConfig({
  plugins: [
    TypstPlugin({
      compiler: 'typst-cli',
      onResolveParts: (input, project, ctx) => {
        const res = checkExecResult(input, project.tryHtml(input), ctx);
        return {
          frontmatter: res && project.query(input, { selector: '<frontmatter>', field: 'value' })[0],
        };
      },
    }),
  ],
});
