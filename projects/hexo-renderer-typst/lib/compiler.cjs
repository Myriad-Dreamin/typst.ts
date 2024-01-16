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
    }).result.title;
  }

  vector(path) {
    try {
      return this.dyn.vector({
        mainFilePath: path
    });
    } catch (e) {
      console.log(e);
      throw e;
    }
  }

  watch(mainFilePath, callback) {
    console.log('[typst] compiler start to notify', mainFilePath, callback);
  }

  unwatch(mainFilePath, callback) {
    console.log('[typst] compiler stop to notify', mainFilePath, callback);
  }
}

module.exports = Compiler;
