const fs = require('fs');
const path = require('path');
const { execSync } = require('child_process');

class Processor {
  constructor(hexo) {
    this.hexo = hexo;
    this.Post = hexo.model('Post');
    this.renderCli = 'typst-ts-cli';

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

    const title = JSON.parse(execSync([
      this.renderCli,
      'query',
      '--workspace',
      base_dir,
      '--entry',
      `"source/${data.source}"`,
      '--selector',
      'document_title',
    ].join(' '), {
      encoding: 'utf-8',
    }));

    if ((!title) || title === null) {
      console.log('[typst]', `title not found in ${data.source}`);
    } else {
      data.title = title;
      data.published = true;
    }

    return data;
  }
}

module.exports = Processor;
