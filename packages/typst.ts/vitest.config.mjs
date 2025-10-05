import { defineConfig } from 'vitest/config';
import { playwright } from '@vitest/browser-playwright';
import {imageHash} from 'image-hash'
import fs from 'fs'

const isUpdate = (process.argv.includes('--update') || process.argv.includes('-u'));

const saveRef = (refPath, screenshotPath, screenshotHash) => {
  fs.mkdirSync(`${refPath}/..`, { recursive: true });
  fs.writeFileSync(`${refPath}.hash.txt`, screenshotHash);
  // copy screenshot to refs/renderer
  fs.copyFileSync(screenshotPath, refPath);
  const newName = `${refPath}.failure.png`;
  if (fs.existsSync(newName)) {
    fs.unlinkSync(newName);
  }
}

const saveFailure = (refPath, screenshotPath) => {
  fs.mkdirSync(`${refPath}/..`, { recursive: true });
  const newName = `${refPath}.failure.png`;
  // copy screenshot to refs/renderer
  fs.copyFileSync(screenshotPath, newName);
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
    const refHash = fs.readFileSync(refHashPath, 'utf-8');
    if ((refHash!==screenshotHash) ) {
      if ( isUpdate) {
        saveRef(refPath, screenshotPath, screenshotHash);
         return {  screenshotHash, refHash: screenshotHash };
      }  else {
        saveFailure(refPath, screenshotPath);
      }
    }
    return {  screenshotHash, refHash };
  }

   saveRef(refPath, screenshotPath, screenshotHash);
  // copy screenshot to refs/renderer
  return {  screenshotHash, refHash: screenshotHash };
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
