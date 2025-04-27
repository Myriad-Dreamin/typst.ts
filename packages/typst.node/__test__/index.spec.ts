import test from 'ava';

import { NodeCompiler, PdfStandard } from '../index';

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
  t.truthy(doc);
  if (!doc) {
    return;
  }
  const date = Date.now();
  const pdf = compiler.pdf(doc, { creationTimestamp: date / 1000 });
  t.truthy(pdf);

  // latin1 performs string encoding per byte, so we are safe.
  const latin1Pdf = pdf.toString('latin1');
  const createDateMatched = latin1Pdf.match(/<xmp:CreateDate>([^<]+)<\/xmp:CreateDate>/);
  const createDateParsed = createDateMatched?.[1] as string;
  t.truthy(createDateParsed);
  const createDateInPdf = new Date(createDateParsed);
  const createDate = new Date(date);
  if (Math.abs(createDate.getTime() - createDateInPdf.getTime()) > 1000) {
    t.fail(`create date mismatch: expected ${createDate}, got ${createDateInPdf}`);
  }
});

test('it pdf by compiled artifact and Pdf Standard 1.7', t => {
  const compiler = NodeCompiler.create();
  const doc = compiler.compile({
    mainFileContent: `
Hello, Typst! <my-label>
`,
  }).result;
  t.truthy(doc);
  if (!doc) {
    return;
  }
  const pdf = compiler.pdf(doc, { pdfStandard: PdfStandard.V_1_7 });
  t.truthy(pdf);
});

test('it pdf by compiled artifact and Pdf Standard A2b', t => {
  const compiler = NodeCompiler.create();
  const doc = compiler.compile({
    mainFileContent: `
Hello, Typst! <my-label>
`,
  }).result;
  t.truthy(doc);
  if (!doc) {
    return;
  }
  const pdf = compiler.pdf(doc, { pdfStandard: PdfStandard.A_2b });
  t.truthy(pdf);

  // latin1 performs string encoding per byte, so we are safe.
  const latin1Pdf = pdf.toString('latin1');
  t.true(latin1Pdf.includes('<pdfaid:part>2</pdfaid:part>'));
  t.true(latin1Pdf.includes('<pdfaid:conformance>B</pdfaid:conformance>'));
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
  t.snapshot(diag.shortDiagnostics.length);
  t.snapshot(diag.shortDiagnostics[0].message);
  const fetched = compiler.fetchDiagnostics(diag);
  t.snapshot(fetched.length);
  t.snapshot(fetched[0].message);
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
