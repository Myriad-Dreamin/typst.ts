import { createTypstCompiler, createTypstRenderer } from '@myriaddreamin/typst.ts';
import { MemoryAccessModel } from '@myriaddreamin/typst.ts/fs/memory';
import { NodeFetchPackageRegistry } from '@myriaddreamin/typst.ts/fs/package.node';
import { withAccessModel, withPackageRegistry } from '@myriaddreamin/typst.ts/options.init';
import { cachedFontInitOptions } from './cached-font-middleware.mjs';
import { writeFileSync } from 'fs';
import request, { HttpVerb, Options } from 'sync-request';
// import request, { HttpVerb, Options } from 'sync-request-curl';

async function main(coordinate: { x: number; y: number }) {
  let typstCode: string = `#import "@preview/cetz:0.1.2"
#set page(margin: (top: 0pt, bottom: 0pt, left: 0pt, right: 0pt))
#cetz.canvas({
  import cetz.draw: *

  grid((0, 0), (${coordinate.x}, ${coordinate.y}), step: 1, stroke: gray + 2pt)
})`;

  const compiler = createTypstCompiler();
  const accessModel = new MemoryAccessModel();
  // use faster sync-request-curl but insecure
  await compiler.init({
    beforeBuild: [
      ...(await cachedFontInitOptions()).beforeBuild,
      withAccessModel(accessModel),
      withPackageRegistry(
        new NodeFetchPackageRegistry(
          accessModel,
          (method: HttpVerb, url: string, options?: Options) => {
            return request(method, url, {
              // insecure: true,
              ...(options ?? {}),
            });
          },
        ),
      ),
    ],
  });

  {
    compiler.addSource(
      '/main.typ',
      `
#import "@preview/example:0.1.0"
#example.add(1, left)`,
    );
    let artifact = await compiler.compile({
      mainFilePath: '/main.typ',
      format: 'vector',
      diagnostics: 'unix',
    });

    if (!artifact.diagnostics?.length) {
      throw new Error("Renderer doesn't produce diags exactly..");
    }

    console.log(
      'Renderer produces diags exactly! The diagnostics is in unix',
      artifact.diagnostics,
    );

    let artifact2 = await compiler.compile({
      mainFilePath: '/main.typ',
      format: 'vector',
      diagnostics: 'full',
    });

    if (!artifact2.diagnostics?.length) {
      throw new Error("Renderer doesn't produce diags exactly..");
    }

    console.log(
      'Renderer produces diags exactly! The diagnostics is in full',
      artifact2.diagnostics,
    );
  }

  {
    compiler.addSource('/main.typ', typstCode);
    let artifact: Uint8Array = (
      await compiler.compile({
        mainFilePath: '/main.typ',
        format: 'vector',
        diagnostics: 'unix',
      })
    ).result!;

    const renderer = createTypstRenderer();
    await renderer.init();

    const svg = await renderer.runWithSession(async session => {
      renderer.manipulateData({
        renderSession: session,
        action: 'reset',
        data: artifact,
      });
      return renderer.renderSvg({
        renderSession: session,
      });
    });

    return svg;
  }
}

main({ x: 15, y: 15 }).then(svg => {
  //   console.log(svg);
  writeFileSync('test.artifact.svg', svg);
});
