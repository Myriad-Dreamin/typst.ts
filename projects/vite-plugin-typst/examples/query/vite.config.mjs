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

        const htmlDoc = project.compileHtml(input).result;
        if (htmlDoc) {
          console.log(project.query(htmlDoc, { selector: '<frontmatter>', field: 'value' }));
        }

        return htmlResult;
      },
    }),
  ],
});
