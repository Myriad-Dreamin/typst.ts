const { NodeCompiler, DynLayoutCompiler } = require('@myriaddreamin/typst-ts-node-compiler');
const path = require('path');


/**
 * Hexo Compiler for Typst
 * 
 * @class
 * @constructor
 * @public
 */
class Compiler {
  constructor(hexo) {
    this.hexo = hexo;
    this.baseDir = this.hexo.base_dir;
    const fonts = path.resolve(this.baseDir, 'fonts');
    const assetsFonts = path.resolve(this.baseDir, 'assets/fonts');
    const assetFonts = path.resolve(this.baseDir, 'asset/fonts');
    console.log(
      '[typst] using fonts in',
      path.resolve(this.baseDir, '{fonts,assets/fonts,asset/fonts}'),
    );
    const compileArgs = {
      workspace: this.baseDir,
      fontArgs: [{ fontPaths: [fonts, assetsFonts, assetFonts] }],
      // todo: move this to session after we fixed the bug
      inputs: { 'x-target': 'web' },
    };
    this.base = NodeCompiler.create(compileArgs);
    this.dyn = DynLayoutCompiler.fromBoxed(NodeCompiler.create(compileArgs).intoBoxed());
  }

  title(path) {
    return this.base.compile({
      mainFilePath: path,
    }).result.title;
  }

  html(path) {
    try {
      return this.base.html({
        mainFilePath: path,
      });
    } catch (e) {
      console.log(e);
      throw e;
    }
  }

  vector(path) {
    try {
      return this.dyn.vector({
        mainFilePath: path,
      });
    } catch (e) {
      console.log(e);
      throw e;
    }
  }

  query(path, selector, field = undefined) {
    try {
      return this.base.query(
        {
          mainFilePath: path,
        },
        {
          selector,
          field,
        },
      );
    } catch (e) {
      console.log(e);
      throw e;
    }
  }

  watch(mainFilePath, callback) {
    // console.log('[typst] compiler start to notify', mainFilePath);
  }

  unwatch(mainFilePath, callback) {
    // console.log('[typst] compiler stop to notify', mainFilePath);
  }
}

module.exports = Compiler;
