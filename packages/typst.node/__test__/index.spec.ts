import test from 'ava';

import { NodeCompiler } from '../index';

// Switch to the current directory for the tests interacting with FS
process.chdir(__dirname);

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
  t.snapshot(doc && compiler.query(doc, { selector: `<my-label>` }));
});

test('it vec by arguments', t => {
  const compiler = defaultCompiler();
  t.truthy(
    compiler.vector({
      mainFileContent: `
Hello, Typst! <my-label>
`,
    }),
  );
});

test('it vec by compiled artifact', t => {
  const compiler = defaultCompiler();
  const doc = compiler.compile({
    mainFileContent: `
Hello, Typst! <my-label>
`,
  }).result;
  t.truthy(doc && compiler.vector(doc));
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

test('it takes in the workspace and entry file in compiler ctor', t => {
  const compiler = NodeCompiler.create({
    ...NodeCompiler.defaultCompileArgs(),
    workspace: '.',
    entry: 'inputs/post1.typ',
  });
  // No mutation taken place on the entry state
  const doc = compiler.compile({}).result;
  // Should be main.typ
  t.snapshot(doc?.title);
});

test('it modifys the entry file to another when call `compile`', t => {
  const compiler = NodeCompiler.create({
    ...NodeCompiler.defaultCompileArgs(),
    workspace: '.',
    entry: 'inputs/post1.typ',
  });
  // No mutation taken place on the entry state
  const doc = compiler.compile({
    mainFilePath: 'inputs/post2.typ'
  }).result;
  // Should be main.typ
  t.snapshot(doc?.title);
});

test('it throws error when entry file does not exist', t => {
  const compiler = NodeCompiler.create({
    ...NodeCompiler.defaultCompileArgs(),
    workspace: '.',
    entry: 'inputs/post1.typ',
  });
  try {
    compiler.compile({
      mainFilePath: 'inputs/fake.typ'
    });
    t.fail('it should throw error');
  } catch (err: any) {
    t.assert(
      err.message.startsWith("0: file not found"),
      `error message does not match the expectation: ${err.message}`
    );
  }
});

test('it throws error when entry file does not reside in the workspace', t => {
  try {
    NodeCompiler.create({
      ...NodeCompiler.defaultCompileArgs(),
      workspace: '.',
      entry: '../../fake.typ',
    });
    t.fail('it should throw error');
  } catch (err: any) {
    t.assert(err.message.startsWith(
      "entry file path must be in workspace directory"),
      `error message does not match the expectation: ${err.message}`
    );
  }
});
