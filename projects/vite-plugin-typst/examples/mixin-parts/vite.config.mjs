import { defineConfig } from 'vite';
import { TypstPlugin, checkExecResult } from '@myriaddreamin/vite-plugin-typst';

export default defineConfig({
  plugins: [
    TypstPlugin({
      onResolveParts: (input, project, ctx) => {
        const res = checkExecResult(input, project.compileHtml(input), ctx);
        return {
          frontmatter: res && project.query(res, { selector: '<frontmatter>', field: 'value' })[0],
        };
      },
    }),
  ],
});
