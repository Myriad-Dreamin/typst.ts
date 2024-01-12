// @ts-ignore
const { NodeCompiler } = require('./index')

const cc = NodeCompiler.create(NodeCompiler.defaultCompileArgs());

console.assert(!!cc.svg({
  mainFileContent: 'Hello, Typst!',
}), 'Simple test failed')

console.info('Simple test passed')
