const path = require('path');

class Processor {
  constructor(hexo, compiler) {
    this.hexo = hexo;
    this.compiler = compiler;
  }

  process(data) {
    if (!(data.source.endsWith('.typ') || data.source.endsWith('.typst'))) {
      return;
    }

    const base_dir = this.hexo.base_dir;
    let title = this.compiler.title(path.resolve(base_dir, `source/${data.source}`));

    if ((!title) || title === null) {
      console.error('[typst]', `title not found in ${data.source}`);
      title = 'Untitled Typst';
    }

    data.title = title;
    data.published = true;
    return data;
  }
}

module.exports = Processor;
