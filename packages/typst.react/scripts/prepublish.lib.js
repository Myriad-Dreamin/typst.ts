const path = require('path');
const fs = require('fs');

function copy(src, dest) {
  src = path.resolve(__dirname, '..', src);
  dest = path.resolve(__dirname, '..', dest);

  fs.copyFileSync(src, dest);
}

copy('./package.json', './dist/lib/package.json');
copy('./README.md', './dist/lib/README.md');
copy('./LICENSE', './dist/lib/LICENSE');
copy('./.npmignore', './dist/lib/.npmignore');
