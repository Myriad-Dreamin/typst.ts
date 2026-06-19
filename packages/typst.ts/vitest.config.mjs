import { defineConfig } from 'vitest/config';
import { playwright } from '@vitest/browser-playwright';
import {imageHash} from 'image-hash'
import fs from 'fs'
import path from 'path'
import { browserSkippedTests } from './scripts/browser-skipped-tests.mjs';

const isUpdate = (process.argv.includes('--update') || process.argv.includes('-u'));
const includeSkippedBrowserTests = process.env.TYPST_TS_INCLUDE_SKIPPED_BROWSER_TESTS === '1';

const saveRef = (refPath, screenshotPath, screenshotHash) => {
  fs.mkdirSync(path.dirname(refPath), { recursive: true });
  fs.writeFileSync(`${refPath}.hash.txt`, screenshotHash);
  // copy screenshot to refs/renderer
  fs.copyFileSync(screenshotPath, refPath);
  const newName = `${refPath}.failure.png`;
  if (fs.existsSync(newName)) {
    fs.unlinkSync(newName);
  }
}

const saveFailure = (refPath, screenshotPath) => {
  fs.mkdirSync(path.dirname(refPath), { recursive: true });
  const newName = `${refPath}.failure.png`;
  // copy screenshot to refs/renderer
  fs.copyFileSync(screenshotPath, newName);
}

const ensureRefImage = (refPath, screenshotPath) => {
  if (!fs.existsSync(refPath)) {
    fs.mkdirSync(path.dirname(refPath), { recursive: true });
    fs.copyFileSync(screenshotPath, refPath);
  }
}

const parseRefHashes = (refHash) => {
  return refHash.split(/\r?\n/).map(hash => hash.trim()).filter(Boolean);
}

const createSnapshot = async (_ctx, screenshotPath, name, extras) => {
  const refPath= `${import.meta.dirname}/refs/renderer/${name}`;
  // console.log(import.meta.dirname,screenshotPath, name, refPath);
  const refHashPath= `${refPath}.hash.txt`;
  // const screenshotHash =  imageHash(screenshotPath);
  const screenshotHash = await  new Promise((resolve, reject) => {
    imageHash(screenshotPath, 16, true, (err, hash) => {
      if (err) {
        reject(err);
      }
      // resolve(`ihash16:${hash}${extras ? `?${extras}` : ''}`);
      extras ? resolve(`ihash16:${hash}?${extras}`) : resolve(`ihash16:${hash}`);
    });
  });
  console.log(screenshotHash, refHashPath, isUpdate);
  if (fs.existsSync(refHashPath)) {
    const refHash = fs.readFileSync(refHashPath, 'utf-8').trimEnd();
    const refHashes = parseRefHashes(refHash);
    if (!refHashes.includes(screenshotHash)) {
      if ( isUpdate) {
        saveRef(refPath, screenshotPath, screenshotHash);
         return {  screenshotHash, refHash: screenshotHash };
      }  else {
        ensureRefImage(refPath, screenshotPath);
        saveFailure(refPath, screenshotPath);
        return {  screenshotHash, refHash };
      }
    }
    ensureRefImage(refPath, screenshotPath);
    return {  screenshotHash, refHash: screenshotHash };
  }

   saveRef(refPath, screenshotPath, screenshotHash);
  // copy screenshot to refs/renderer
  return {  screenshotHash, refHash: screenshotHash };
}

const skipTests = (tests) => {
  return tests.map(test => `{tests,src}/${test}.browser.test.mts`);
}

export default defineConfig({
  test: {
    projects: [
      {
        extends: false,
        test: {
          include: ['{tests,src}/**/*.all.{test,spec}.mts', '{tests,src}/**/*.node.{test,spec}.mts'],
          environment: 'node',
        },
      },
      {
        extends: false,
        test: {
          name: 'browser',
          environment: 'happy-dom',
          include: ['{tests,src}/**/*.all.{test,spec}.mts', '{tests,src}/**/*.browser.{test,spec}.mts'],
          // todo: these tests are broken across platform.
          exclude: includeSkippedBrowserTests ? [] : skipTests(browserSkippedTests),
          testTimeout: 120_000,
          browser: {
            // By default this is 63315, but windows doesn't permit high ports.
            api: 9528,
            headless: true,
            provider: playwright(), // or 'webdriverio'
            enabled: true,
            commands: {
              createSnapshot: createSnapshot,
            },
            // at least one instance is required
            instances: [
              {
                browser: 'chromium',
                isolate: true,
                screenshotFailures: false,
              },
            ],
          },
        },
      },
    ],
  },
});
