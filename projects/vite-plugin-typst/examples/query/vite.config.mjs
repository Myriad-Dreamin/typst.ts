// @ts-check

import { defineConfig } from 'vite';
import { TypstPlugin, checkExecResult } from '@myriaddreamin/vite-plugin-typst';

export default defineConfig({
  plugins: [
    TypstPlugin({
      documents: ['content/**/*.typ'],
      onCompile: (input, project, ctx) => {
        const htmlResult = checkExecResult(input, project.tryHtml(input), ctx);
        if (!htmlResult) {
          return;
        }

        const htmlContent = htmlResult.html();
        ctx.compiled.set(ctx.resolveRel(input.mainFilePath), htmlContent);

        const htmlRes = project.compileHtml(input);
        if (htmlRes.result) {
          console.log(project.query(htmlRes.result, { selector: '<frontmatter>', field: 'value' }));
        } else {
          htmlRes.printErrors();
          process.exit(1);
        }

        /**
         * @type {import('@myriaddreamin/vite-plugin-typst/dist/compiler/node').NodeCompileProvider}
         */
        let _ctxType = ctx;

        return htmlResult;
      },
    }),
  ],
});
