const { execFileSync, spawnSync } = require('child_process');
const { readFileSync } = require('fs');
const { join } = require('path');

const { dirs } = require('./publish-targets');

const rootDir = join(__dirname, '..');
const rootPackage = JSON.parse(readFileSync(join(rootDir, 'package.json'), 'utf8'));
const versionTag = `v${rootPackage.version}`;
const releaseTag = process.argv[2] || process.env.RELEASE_TAG || versionTag;
const alreadyPublishedPatterns = [
  'cannot publish over the previously published versions',
  'previously published versions',
];

function uploadReleaseAsset(dir) {
  execFileSync(
    'gh',
    ['release', 'upload', releaseTag, `typst-ts-node-compiler.${dir}.node`, '--clobber'],
    {
      stdio: 'inherit',
      cwd: join(rootDir, 'npm', dir),
    },
  );
}

function publishPackage(cwd, label) {
  console.log(`Publish ${label}`);
  const result = spawnSync('npm', ['publish', '--verbose', '--provenance', '--access', 'public'], {
    cwd,
    encoding: 'utf8',
    maxBuffer: 10 * 1024 * 1024,
  });

  if (result.stdout) {
    process.stdout.write(result.stdout);
  }
  if (result.stderr) {
    process.stderr.write(result.stderr);
  }

  if (result.status === 0) {
    return;
  }

  const output = `${result.stdout ?? ''}\n${result.stderr ?? ''}`.toLowerCase();
  if (alreadyPublishedPatterns.some(pattern => output.includes(pattern))) {
    console.warn(`Package already published for ${label}, continuing.`);
    return;
  }

  if (result.error) {
    throw result.error;
  }

  throw new Error(`npm publish failed for ${label} with status ${result.status}`);
}

for (const dir of dirs) {
  publishPackage(join(rootDir, 'npm', dir), dir);
  console.log(`Upload typst-ts-node-compiler.${dir}.node to release ${releaseTag}`);
  uploadReleaseAsset(dir);
}

publishPackage(rootDir, rootPackage.name);
