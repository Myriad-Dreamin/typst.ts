// @ts-check

import { defineConfig } from 'vite';
import { TypstPlugin, checkExecResult } from '@myriaddreamin/vite-plugin-typst';

export default defineConfig({
  plugins: [
    TypstPlugin({
      compiler: 'typst-cli',
      documents: ['content/**/*.typ'],
      onCompile: (input, project, ctx) => {
        const htmlResult = checkExecResult(input, project.tryHtml(input), ctx);
        if (!htmlResult) {
          return;
        }

        const htmlContent = htmlResult.html();
        ctx.compiled.set(ctx.resolveRel(input.mainFilePath), htmlContent);

        console.log(project.query(input, { selector: '<frontmatter>', field: 'value' }));

        /**
         * @type {import('@myriaddreamin/vite-plugin-typst/dist/compiler/cli').CliCompileProvider}
         */
        let _ctxType = ctx;

        return htmlResult;
      },
    }),
  ],
});
