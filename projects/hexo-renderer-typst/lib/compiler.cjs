const { NodeCompiler, DynLayoutCompiler } = require('@myriaddreamin/typst-ts-node-compiler');

class Compiler {
  constructor(hexo) {
    this.hexo = hexo;
    this.baseDir = this.hexo.base_dir;
    const compileArgs = {
      ...NodeCompiler.defaultCompileArgs(),
      workspace: this.baseDir,
    };
    this.base = NodeCompiler.create(compileArgs);
    this.dyn = DynLayoutCompiler.fromBoxed(NodeCompiler.create(compileArgs).intoBoxed());
  }

  title(path) {
    return this.base.compile({
        mainFilePath: path
    }).title;
  }

  vector(path) {
    return this.dyn.vector({
        mainFilePath: path
    });
  }
}

module.exports = Compiler;
