import test from 'ava';

import { NodeCompiler } from '../index';

function defaultCompiler() {
  return NodeCompiler.create(NodeCompiler.defaultCompileArgs());
}

test('it creates', t => {
  const compiler = defaultCompiler();
  t.truthy(
    compiler.svg({
      mainFileContent: 'Hello, Typst!',
    }),
  );
});

test('it query document title', t => {
  const compiler = defaultCompiler();
  const doc = compiler.compile({
    mainFileContent: `
#set document(title: "My Typst Document for Node testing")

Hello, Typst!
`,
  });
  t.is(doc.title, 'My Typst Document for Node testing');
});
