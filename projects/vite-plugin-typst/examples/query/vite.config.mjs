import { defineConfig } from 'vite';
import { TypstPlugin, checkExecResult } from '@myriaddreamin/vite-plugin-typst';

export default defineConfig({
  plugins: [
    TypstPlugin({
      documents: ['content/**/*.typ'],
      onCompile: (mainFilePath, project, ctx) => {
        const htmlResult = checkExecResult(mainFilePath, project.tryHtml({ mainFilePath }), ctx);
        if (!htmlResult) {
          return;
        }

        const htmlContent = htmlResult.result.html();
        ctx.compiled.set(ctx.resolveRel(mainFilePath), htmlContent);

        const htmlDoc = project.compileHtml({ mainFilePath }).result;
        if (htmlDoc) {
          console.log(project.query(htmlDoc, { selector: '<frontmatter>', field: 'value' }));
        }

        return htmlResult;
      },
    }),
  ],
});
