const { execSync } = require('child_process');
const { readFileSync } = require('fs');

const versionTag = `v${JSON.parse(readFileSync('./package.json', 'utf8')).version}`;
const releaseTag = process.argv[2] || process.env.RELEASE_TAG || versionTag;
const alreadyPublishedPatterns = [
  'cannot publish over the previously published versions',
  'previously published versions',
];

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

function run(command, cwd) {
  execSync(command, {
    stdio: 'inherit',
    cwd,
  });
}

for (const dir of dirs) {
  console.log(`Publish ${dir}`);
  try {
    const stdout = execSync('npm publish --verbose --provenance --access public', {
      cwd: `./npm/${dir}`,
      encoding: 'utf8',
      stdio: 'pipe',
    });
    if (stdout) {
      process.stdout.write(stdout);
    }
  } catch (error) {
    if (error.stdout) {
      process.stdout.write(error.stdout);
    }
    if (error.stderr) {
      process.stderr.write(error.stderr);
    }
    const output = `${error.stdout ?? ''}\n${error.stderr ?? ''}`.toLowerCase();
    if (alreadyPublishedPatterns.some(pattern => output.includes(pattern))) {
      console.warn(`Package already published for ${dir}, continuing.`);
    } else {
      throw error;
    }
  }

  console.log(`Upload typst-ts-node-compiler.${dir}.node to release ${releaseTag}`);
  run(`gh release upload ${releaseTag} typst-ts-node-compiler.${dir}.node --clobber`, `./npm/${dir}`);
}
