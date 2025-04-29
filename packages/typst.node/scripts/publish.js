
const { execSync } = require('child_process');

const { readFileSync } = require('fs');

const tag = `v${JSON.parse(readFileSync('./package.json', 'utf8')).version}`;

const dirs = [
  'android-arm-eabi',
  'android-arm64',
  'darwin-arm64',
  'darwin-x64',
  'linux-arm-gnueabihf',
  'linux-arm64-gnu',
  'linux-arm64-musl',
  'linux-x64-gnu',
  'linux-x64-musl',
  'win32-arm64-msvc',
  'win32-x64-msvc',
];

for (const dir of dirs) {
  console.log(`Publish ${dir}`);
  try {
    execSync('npm publish --verbose --provenance --access public', {
      stdio: 'inherit',
      cwd: `./npm/${dir}`,
    })
  } catch (error) {
    console.error(`Could not publish: ${dir}`, error);
  }

    console.log(`Upload typst-ts-node-compiler.${dir}.node to release ${tag}`);
    try {
      execSync(`gh release upload ${tag} typst-ts-node-compiler.${dir}.node`, {
        stdio: 'inherit',
        cwd: `./npm/${dir}`,
      })
    } catch (error) {
      console.error(`Could not upload to release: ${dir}`, error);
    }
}
