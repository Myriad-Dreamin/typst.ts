#!/usr/bin/env node

import { spawnSync } from 'node:child_process';
import { readFileSync } from 'node:fs';
import { join, resolve } from 'node:path';

const alreadyPublishedPatterns = [
  'cannot publish over the previously published versions',
  'previously published versions',
];

const notFoundPatterns = [
  'e404',
  '404 not found',
  'is not in this registry',
];

const args = process.argv.slice(2);
const packageDirs = [];
let access = 'public';
let dryRun = false;

for (let i = 0; i < args.length; i++) {
  const arg = args[i];

  if (arg === '--dry-run') {
    dryRun = true;
    continue;
  }

  if (arg === '--access') {
    access = args[++i];
    if (!access) {
      throw new Error('--access requires a value');
    }
    continue;
  }

  if (arg.startsWith('--')) {
    throw new Error(`Unknown option: ${arg}`);
  }

  packageDirs.push(arg);
}

if (packageDirs.length === 0) {
  throw new Error('Usage: node scripts/npm-publish-if-needed.mjs [--dry-run] <package-dir>...');
}

const publishArgs = ['publish', '--access', access];
if (dryRun) {
  publishArgs.push('--dry-run');
}

function runNpm(args, cwd) {
  return spawnSync('npm', args, {
    cwd,
    encoding: 'utf8',
    maxBuffer: 10 * 1024 * 1024,
  });
}

function printResult(result) {
  if (result.stdout) {
    process.stdout.write(result.stdout);
  }
  if (result.stderr) {
    process.stderr.write(result.stderr);
  }
}

function outputOf(result) {
  return `${result.stdout ?? ''}\n${result.stderr ?? ''}`.toLowerCase();
}

function hasAny(output, patterns) {
  return patterns.some(pattern => output.includes(pattern));
}

function packageExists(cwd, name, version) {
  const result = runNpm(['view', `${name}@${version}`, 'version', '--silent'], cwd);

  if (result.status === 0 && result.stdout.trim()) {
    return true;
  }

  const output = outputOf(result);
  if (hasAny(output, notFoundPatterns)) {
    return false;
  }

  console.warn(`Could not confirm whether ${name}@${version} exists before publish.`);
  printResult(result);
  return false;
}

function publishPackage(packageDir) {
  const cwd = resolve(packageDir);
  const manifest = JSON.parse(readFileSync(join(cwd, 'package.json'), 'utf8'));
  const { name, version, private: isPrivate } = manifest;

  if (!name || !version) {
    throw new Error(`${packageDir}/package.json must define name and version`);
  }

  if (isPrivate) {
    console.log(`Skipping private package ${name}@${version}`);
    return;
  }

  if (packageExists(cwd, name, version)) {
    console.log(`Skipping already published package ${name}@${version}`);
    return;
  }

  console.log(`Publishing ${name}@${version} from ${packageDir}`);
  const result = runNpm(publishArgs, cwd);
  printResult(result);

  if (result.status === 0) {
    return;
  }

  const output = outputOf(result);
  if (hasAny(output, alreadyPublishedPatterns)) {
    console.warn(`Package ${name}@${version} was already published, continuing.`);
    return;
  }

  if (result.error) {
    throw result.error;
  }

  throw new Error(`npm publish failed for ${name}@${version} with status ${result.status}`);
}

for (const packageDir of packageDirs) {
  publishPackage(packageDir);
}
