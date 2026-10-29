import test from 'ava';
import { resolve } from 'path';

import { NodeCompiler, PdfStandard } from '../index';

// Switch to the current directory for the tests interacting with FS
process.chdir(__dirname);

for (const target of ['compile', 'compileHtml', 'tryHtml'] as const) {
  test(`${target} reports source and data dependencies`, t => {
    const compiler = NodeCompiler.create({ workspace: '.' });
    const main = resolve('dependencies-main.typ');
    const part = resolve('dependencies-part.typ');
    const nested = resolve('dependencies-nested.typ');
    const data = resolve('dependencies-data.csv');
    compiler.addSource(main, '#include "dependencies-part.typ"\n#include "dependencies-part.typ"');
    compiler.addSource(part, '#include "dependencies-nested.typ"\n#read("dependencies-data.csv")');
    compiler.addSource(nested, 'Nested content');
    compiler.mapShadow(data, Buffer.from('data'));

    const result = compiler[target]({ mainFilePath: main });
    t.false(result.hasError());
    const expected = [main, part, nested, data].sort();
    t.deepEqual(result.dependencies(), expected);
    t.deepEqual(result.dependencies(), expected);

    if (target === 'compileHtml') {
      const compiled = compiler.compileHtml({ mainFilePath: main });
      t.truthy(compiled.result);
      const exported = compiler.tryHtml(compiled.result!);
      t.false(exported.hasError());
      t.deepEqual(exported.dependencies(), expected);
    }

    // A later compilation must not change the dependencies of this result.
    compiler.addSource(main, 'No imports');
    const next = compiler[target]({ mainFilePath: main });
    t.false(next.hasError());
    t.deepEqual(next.dependencies(), [main]);
    t.deepEqual(result.dependencies(), expected);
  });

  test(`${target} reports dependencies after a compilation error`, t => {
    const compiler = NodeCompiler.create({ workspace: '.' });
    const main = resolve('dependencies-error.typ');
    const part = resolve('dependencies-invalid.typ');
    compiler.addSource(main, '#include "dependencies-invalid.typ"');
    compiler.addSource(part, '#panic("dependency error")');

    const result = compiler[target]({ mainFilePath: main });
    t.true(result.hasError());
    result.takeDiagnostics();
    t.deepEqual(result.dependencies(), [main, part].sort());
  });
}

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

test("it doesn't pdf by wrong standard", t => {
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
  t.throws(() => {
    const pdf = compiler.pdf(doc, { pdfStandard: '0.0' });
    t.falsy(pdf);
  });
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

  // latin1 performs string encoding per byte, so we are safe.
  const latin1Pdf = pdf.toString('latin1');
  t.true(!latin1Pdf.includes('<pdfaid:part>2</pdfaid:part>'));
  t.true(!latin1Pdf.includes('<pdfaid:conformance>B</pdfaid:conformance>'));
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
  require('fs').writeFileSync('test2b.pdf', pdf);

  // latin1 performs string encoding per byte, so we are safe.
  const latin1Pdf = pdf.toString('latin1');
  t.true(latin1Pdf.includes('<pdfaid:part>2</pdfaid:part>'));
  t.true(latin1Pdf.includes('<pdfaid:conformance>B</pdfaid:conformance>'));
});

test('it pdf by compiled artifact and Pdf Standard A3b', t => {
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
  const pdf = compiler.pdf(doc, { pdfStandard: PdfStandard.A_3b });
  t.truthy(pdf);
  require('fs').writeFileSync('test.pdf', pdf);

  // latin1 performs string encoding per byte, so we are safe.
  const latin1Pdf = pdf.toString('latin1');
  t.true(latin1Pdf.includes('<pdfaid:part>3</pdfaid:part>'));
  t.true(latin1Pdf.includes('<pdfaid:conformance>B</pdfaid:conformance>'));
});

test('it hast', t => {
  const compiler = NodeCompiler.create();

  const h = compiler.tryHtml({
    mainFileContent: `
Hello, Typst!
`,
  });
  t.snapshot(h.result?.hast());
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
