import { defineConfig } from 'vite';
import { TypstPlugin, checkExecResult } from '@myriaddreamin/vite-plugin-typst';

export default defineConfig({
  plugins: [
    TypstPlugin({
      onResolveParts: (mainFilePath, project, ctx) => {
        const res = checkExecResult(mainFilePath, project.compileHtml({ mainFilePath }), ctx);
        return {
          frontmatter: res && project.query(res, { selector: '<frontmatter>', field: 'value' })[0],
        };
      },
    }),
  ],
});
