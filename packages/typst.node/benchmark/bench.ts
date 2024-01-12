async function bench() {
  const b = await import('benny');
  const { NodeCompiler } = await import('../index.js');

  function defaultCompiler() {
    return NodeCompiler.create(NodeCompiler.defaultCompileArgs());
  }

  const compiler = defaultCompiler();
  await b.suite(
    'Export',

    b.add('Export to SVG', () => {
      compiler.svg({
        mainFileContent: 'Hello, Typst!',
      });
    }),

    b.add('Export to PDF', () => {
      compiler.pdf({
        mainFileContent: 'Hello, Typst!',
      });
    }),

    b.add('Export to vector IR', () => {
      compiler.vector({
        mainFileContent: 'Hello, Typst!',
      });
    }),

    b.cycle(),
    b.complete(),
  );
}

bench().catch(e => {
  console.error(e);
});
