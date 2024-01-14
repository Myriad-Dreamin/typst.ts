import test from 'ava';

import { NodeCompiler } from '../index';

function defaultCompiler() {
  return NodeCompiler.create(NodeCompiler.defaultCompileArgs());
}

test('it creates compiler', t => {
  const compiler = defaultCompiler();
  t.truthy(
    compiler.svg({
      mainFileContent: 'Hello, Typst!',
    }),
  );
});

test('it queries document title', t => {
  const compiler = defaultCompiler();
  const doc = compiler.compile({
    mainFileContent: `
#set document(title: "My Typst Document for Node testing")

Hello, Typst!
`,
  }).result;
  t.is(doc?.title, 'My Typst Document for Node testing');
});

test('it queries label in `Hello, Typst! <my-label>`', t => {
  const compiler = defaultCompiler();
  const doc = compiler.compile({
    mainFileContent: `
Hello, Typst! <my-label>
`,
  }).result;
  t.snapshot(doc && compiler.query(doc, `<my-label>`));
});

test('it throws error`', t => {
  const compiler = defaultCompiler();
  let doc = compiler.compile({
    mainFileContent: `
Hello, Typst! <my-
`,
  });
  try {
    doc.result!.title;
    t.fail('it should throw error');
  } catch {}

  const diag = doc.takeDiagnostics()!;
  t.is(diag.compilationStatus, 'error');
  t.snapshot(diag.shortDiagnostics);
  t.snapshot(compiler.fetchDiagnostics(diag));
});
