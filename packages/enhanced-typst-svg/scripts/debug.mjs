import { projectRoot } from './common.mjs';
import { existsSync, writeFileSync, readFileSync } from 'fs';

export function updateDebugScript() {
  const svg_util = projectRoot + '/packages/enhanced-typst-svg/dist/index.min.js';
  const svg_html = projectRoot + '/fuzzers/corpora/perf/long.artifact.svg.html';
  const svg_html_debug =
    projectRoot + '/fuzzers/corpora/perf/long.debug.artifact.svg.html';

  const svg_util_data = readFileSync(svg_util, 'utf8');
  const svg_html_data = readFileSync(svg_html, 'utf8');

  const svg_util_data_proc = svg_util_data.replace(/</g, '&lt;').replace(/>/g, '&gt;');

  const replaced = svg_html_data.replace(
    /<script type="text\/javascript">[\s\S]*?<\/script>/mg,
    `<script type="text/javascript">${svg_util_data_proc}</script>`,
  );

  console.log('index.min.js', '->', svg_html_debug);

  writeFileSync(svg_html_debug, replaced);
}
