import * as _1 from '@myriaddreamin/typst-ts-renderer';

import {
  createTypstCompiler,
  createTypstRenderer,
  FetchPackageRegistry,
} from '@myriaddreamin/typst.ts';
import { MemoryAccessModel } from '@myriaddreamin/typst.ts/dist/cjs/fs/memory.cjs';
import { NodeFetchPackageRegistry } from '@myriaddreamin/typst.ts/dist/cjs/fs/package.node.cjs';
import {
  withAccessModel,
  withPackageRegistry,
} from '@myriaddreamin/typst.ts/dist/cjs/options.init.cjs';
import { cachedFontInitOptions } from './cached-font-middleware';
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

  compiler.addSource('/main.typ', typstCode);
  let artifact: Uint8Array = await compiler.compile({
    mainFilePath: '/main.typ',
    format: 'vector',
  });

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

main({ x: 15, y: 15 }).then(svg => {
  //   console.log(svg);
  writeFileSync('test.artifact.svg', svg);
});
