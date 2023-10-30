import * as _1 from '@myriaddreamin/typst-ts-renderer';

import {
  createTypstCompiler,
  createTypstRenderer,
  FetchPackageRegistry,
} from '@myriaddreamin/typst.ts';
import { MemoryAccessModel } from '@myriaddreamin/typst.ts/dist/cjs/fs/memory.cjs';
import {
  withAccessModel,
  withPackageRegistry,
} from '@myriaddreamin/typst.ts/dist/cjs/options.init.cjs';
import { PackageSpec } from '@myriaddreamin/typst.ts/dist/cjs/internal.types.cjs';
import { cachedFontInitOptions } from './cached-font-middleware';
import request from 'sync-request-curl';
import { writeFileSync } from 'fs';

class NodeFetchPackageRegistry extends FetchPackageRegistry {
  pullPackageData(path: PackageSpec): Uint8Array | undefined {
    const response = request('GET', this.resolvePath(path), {
      insecure: true,
    });

    if (response.statusCode === 200) {
      return response.getBody(undefined);
    }
    return undefined;
  }
}

async function main(coordinate: { x: number; y: number }) {
  let typstCode: string = `#import "@preview/cetz:0.1.2"
#set page(margin: (top: 0pt, bottom: 0pt, left: 0pt, right: 0pt))
#cetz.canvas({
  import cetz.draw: *

  grid((0, 0), (${coordinate.x}, ${coordinate.y}), step: 1, stroke: gray + 2pt)
})`;

  const compiler = createTypstCompiler();
  const accessModel = new MemoryAccessModel();
  await compiler.init({
    beforeBuild: [
      ...(await cachedFontInitOptions()).beforeBuild,
      withAccessModel(accessModel),
      withPackageRegistry(new NodeFetchPackageRegistry(accessModel)),
    ],
  });

  console.log('/main.typ');
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
    return renderer.renderSvgDiff({
      renderSession: session,
    });
  });

  return svg;
}

main({ x: 15, y: 15 }).then(svg => {
  //   console.log(svg);
  writeFileSync('test.svg', svg);
});
