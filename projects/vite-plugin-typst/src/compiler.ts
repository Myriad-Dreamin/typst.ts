import * as path from 'path';
import { CompileArgs } from '@myriaddreamin/typst-ts-node-compiler';
import type { NodeTypstProject } from '@myriaddreamin/typst-ts-node-compiler';
import type { TypstPluginOptions } from '.';
import { ResolvedTypstInput } from './input.js';
import { NodeCompileProvider } from './compiler/node.js';
import { CliCompileProvider } from './compiler/cli.js';

/**
 * The callback to be called when the document is compiled.
 *
 * @param mainFilePath The source file path
 * @param project The compiling project (document)
 * @param ctx The compile provider
 */
export type OnCompileCallback<T = void> = (
  mainFilePath: ResolvedTypstInput,
  project: NodeTypstProject,
  ctx: CompileProvider,
) => T;
export type CompileProvider = NodeCompileProvider | CliCompileProvider;

/**
 * Creates a new provider for the plugin.
 *
 * @param options The plugin options
 * @param onCompile The callback to be called when the document is compiled
 * @returns
 */
export const makeProvider = (options: TypstPluginOptions, onCompile: OnCompileCallback) => {
  const compileArgs: CompileArgs = {
    workspace: path.resolve(options.root || '.'),
    ...{ inputs: options.inputs, fontArgs: options.fontArgs },
  };

  const compilerProvider = options?.compiler || '@myriaddreamin/typst-ts-node-compiler';
  if (compilerProvider === '@myriaddreamin/typst-ts-node-compiler') {
    return new NodeCompileProvider(compileArgs, false, onCompile);
  } else if (compilerProvider === 'typst-cli') {
    return new CliCompileProvider(compileArgs, false, onCompile);
  }
  throw new Error(`Unsupported compiler provider: ${compilerProvider}`);
};
