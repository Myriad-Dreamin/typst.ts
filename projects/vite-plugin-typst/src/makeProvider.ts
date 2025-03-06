import { CompileArgs } from '@myriaddreamin/typst-ts-node-compiler';
import * as path from 'path';
import type { TypstPluginOptions } from '.';
import { OnCompileCallback, CompileProvider } from './compiler.js';
import { CliCompileProvider } from './compiler/cli.js';
import { NodeCompileProvider } from './compiler/node.js';

// export type CompileProvider = NodeCompileProvider | CliCompileProvider;
/**
 * Creates a new provider for the plugin.
 *
 * @param options The plugin options
 * @param onCompile The callback to be called when the document is compiled
 * @returns
 */
export const makeProvider = (
  options: TypstPluginOptions,
  onCompile: OnCompileCallback<CompileProvider<any>>
) => {
  const compileArgs: CompileArgs = {
    workspace: path.resolve(options.root || '.'),
    ...{ inputs: options.inputs, fontArgs: options.fontArgs },
  };

  const compilerProvider = options?.compiler || '@myriaddreamin/typst-ts-node-compiler';
  if (compilerProvider === '@myriaddreamin/typst-ts-node-compiler') {
    return new NodeCompileProvider(false, compileArgs, onCompile);
  } else if (compilerProvider === 'typst-cli') {
    return new CliCompileProvider(false, compileArgs, onCompile);
  }
  throw new Error(`Unsupported compiler provider: ${compilerProvider}`);
};
