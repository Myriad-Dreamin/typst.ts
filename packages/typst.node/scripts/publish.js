
const { execSync } = require('child_process');

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

  if (process.env.GITHUB_REF_NAME) {
    const GITHUB_REF_NAME = process.env.GITHUB_REF_NAME.replace('refs/tags/', '');
    console.log(`Upload typst-ts-node-compiler.${dir}.node to release ${GITHUB_REF_NAME}`);
    try {
      execSync(`gh release upload ${GITHUB_REF_NAME} typst-ts-node-compiler.${dir}.node`, {
        stdio: 'inherit',
        cwd: `./npm/${dir}`,
      })
    } catch (error) {
      console.error(`Could not upload to release: ${dir}`, error);
    }
  }
}
