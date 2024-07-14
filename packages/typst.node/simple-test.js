// @ts-ignore
const { NodeCompiler } = require('./index')

const cc = NodeCompiler.create();

console.assert(!!cc.svg({
  mainFileContent: 'Hello, Typst!',
}), 'Simple test failed')

console.info('Simple test passed')
