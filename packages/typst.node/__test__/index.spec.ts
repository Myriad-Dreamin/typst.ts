import test from 'ava';

import { NodeCompiler } from '../index';

// Switch to the current directory for the tests interacting with FS
process.chdir(__dirname);

test('it creates compiler', t => {
  const compiler = NodeCompiler.create();
  t.truthy(
    compiler.svg({
      mainFileContent: 'Hello, Typst!',
    }),
  );
});

test('it queries document title', t => {
  const compiler = NodeCompiler.create();
  const doc = compiler.compile({
    mainFileContent: `
#set document(title: "My Typst Document for Node testing")

Hello, Typst!
`,
  }).result;
  t.is(doc?.title, 'My Typst Document for Node testing');
});

test('it queries label in `Hello, Typst! <my-label>`', t => {
  const compiler = NodeCompiler.create();
  const doc = compiler.compile({
    mainFileContent: `
Hello, Typst! <my-label>
`,
  }).result;
  t.snapshot(doc && compiler.query(doc, { selector: `<my-label>` }));
});

test('it vec by arguments', t => {
  const compiler = NodeCompiler.create();
  t.truthy(
    compiler.vector({
      mainFileContent: `
Hello, Typst! <my-label>
`,
    }),
  );
});

test('it pdf by compiled artifact', t => {
  const compiler = NodeCompiler.create();
  const doc = compiler.compile({
    mainFileContent: `
Hello, Typst! <my-label>
`,
  }).result;
  t.truthy(doc && compiler.pdf(doc));
});

test('it pdf by compiled artifact and timestamp', t => {
  const compiler = NodeCompiler.create();
  const doc = compiler.compile({
    mainFileContent: `
Hello, Typst! <my-label>
`,
  }).result;
  t.truthy(doc && compiler.pdf(doc, { creationTimestamp: Date.now() }));
});

test('it throws error`', t => {
  const compiler = NodeCompiler.create();
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
    workspace: '.',
  });
  // No mutation taken place on the entry state
  const doc = compiler.compile({
    mainFilePath: 'inputs/post1.typ',
  }).result;
  // Should be main.typ
  t.snapshot(doc?.title);
});

test('it modifys the entry file to another when call `compile`', t => {
  const compiler = NodeCompiler.create({
    workspace: '.',
  });
  // No mutation taken place on the entry state
  const doc = compiler.compile({
    mainFilePath: 'inputs/post2.typ',
  }).result;
  // Should be main.typ
  t.snapshot(doc?.title);
});

test('it throws error when entry file does not exist', t => {
  const compiler = NodeCompiler.create({
    workspace: '.',
  });
  const res = compiler.compile({
    mainFilePath: 'inputs/fake.typ',
  });
  const diags = res.takeDiagnostics()?.shortDiagnostics;
  t.assert(diags && diags.length > 0);
  const fileNotFound = diags!.find(d => d.message.includes('file not found'));
  t.truthy(fileNotFound);
});
