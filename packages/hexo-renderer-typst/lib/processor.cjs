const fs = require('fs');
const path = require('path');

class Processor {
  constructor(hexo) {
    this.hexo = hexo;
    this.Post = hexo.model('Post');

    const postProcessor = require(path.resolve(
      hexo.base_dir,
      `node_modules/hexo/lib/plugins/processor/post`,
    ));
    this.pp = postProcessor(hexo);
  }

  process(data) {
    if (!(data.source.endsWith('.typ') || data.source.endsWith('.typst'))) {
      return;
    }

    const base_dir = this.hexo.base_dir;

    const rawDataPath = data.source.replace(/\.[^/.]+$/, '');
    const dataPath = path.resolve(base_dir, 'public/artifacts/typst/source', rawDataPath);

    const artifactPath = path.join(dataPath + '.artifact.json');
    const artifactContent = fs.readFileSync(artifactPath);
    const artifact = JSON.parse(artifactContent);
    const title = artifact.title;

    data.title = title;
    data.published = true;

    return data;
  }
}

module.exports = Processor;
