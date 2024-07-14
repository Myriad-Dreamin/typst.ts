import { NodeCompiler } from '@myriaddreamin/typst-ts-node-compiler';
import { resolve } from 'path';

async function main() {
  const compileArgs = {
    // Set a workspace to resolve the relative paths in the main file.
    // This is same as `--root` option in the CLI.
    workspace: resolve(__dirname, '../../'),
  };

  const compiler = NodeCompiler.create(compileArgs);

  {
    // In node.js, we don't have to heavily optimize the bundle size, so the renderer is integrated into the compiler.
    const pdf = compiler.pdf({
      mainFilePath: 'examples/main.typ',
    });

    console.log('Renderer works exactly! The rendered PDF file:', pdf.length);

    // You can save the PDF file to the disk
    // const fs = require('fs');
    // fs.writeFileSync('output.pdf', pdf);
  }
}

main();
