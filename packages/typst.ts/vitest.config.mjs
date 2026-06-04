import { defineConfig } from 'vitest/config';
import { playwright } from '@vitest/browser-playwright';
import {imageHash} from 'image-hash'
import fs from 'fs'
import path from 'path'

const isUpdate = (process.argv.includes('--update') || process.argv.includes('-u'));

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
          exclude: skipTests([
            "bugs/2105-linebreak-tofu_00",
            "bugs/bidi-tofus_00",
            "bugs/new-cm-svg_00",
            "introspection/counter-rtl",
            "meta/figure-localization_01",
            "math/spacing_04",
            "math/style_01",
            "math/style_04",
            "math/math-font-fallback",
            "math/math-lr-mid-size-nested-equation",
            "math/math-multiline-trailing-linebreak",
            "math/math-op-scripts-vs-limits",
            "math/math-spacing-decorated",
            "model/emph-syntax",
            "model/figure-localization-zh",
            "skyzh-cv/main",
            "symbols/symbol-constructor",
            "symbols/symbol-modifier-deprecated",
            "viewers/preview-incr_01",
            "layout/align_02",
            "layout/cjk-latin-spacing_01",
            "layout/flow/place-basic",
            "layout/grid/grid-footer-repeatable-unbreakable",
            "layout/grid/grid-exam",
            "layout/grid/grid-subheaders-alone-with-footer-no-orphan-prevention",
            "layout/inline/hyphenate-pt-no-repeat-hyphen",
            "layout/inline/hyphenate-pt-dash-emphasis",
            "layout/inline/hyphenate-pt-repeat-hyphen-hyphenate-true",
            "layout/inline/hyphenate-pt-repeat-hyphen-hyphenate-true-with-emphasis",
            "layout/inline/hyphenate-pt-repeat-hyphen-natural-word-breaking",
            "layout/inline/cjk-punctuation-adjustment-3",
            "layout/inline/hyphenate-es-repeat-hyphen",
            "layout/inline/issue-1373-bidi-tofus",
            "layout/inline/issue-2105-linebreak-tofu",
            "layout/inline/issue-2538-cjk-latin-spacing-before-linebreak",
            "layout/inline/issue-2650-cjk-latin-spacing-meta",
            "layout/inline/issue-3082-chinese-punctuation",
            "layout/inline/issue-5276-shaping-consecutive-ltr-with-lang",
            "layout/inline/issue-5490-bidi-invalid-range-2",
            "layout/inline/issue-6539-cjk-latin-spacing-at-manual-linebreak",
            "layout/inline/issue-7113-cjk-latin-spacing-shift",
            "layout/inline/justify-chinese",
            "layout/inline/justify-japanese",
            "layout/inline/justify-no-leading-spaces",
            "layout/inline/justify-punctuation-adjustment",
            "layout/inline/justify-variants",
            "layout/inline/linebreak-thai",
            "layout/inline/shaping-font-fallback",
            "layout/inline/text-chinese-basic",
            "layout/inline/text-cjk-latin-spacing",
            "layout/issue-7292-page-height-auto-margin-zero",
            "layout/issue-7292-page-size-auto-margin-zero",
            "layout/issue-7292-page-width-auto-margin-zero",
            "layout/page-numbering-pdf-label",
            "layout/par-bidi_07",
            "layout/par-indent_04",
            "layout/place-float-figure_00",
            "layout/repeat_01",
            "layout/repeat_04",
            "layout/place_01",
            "text/emoji_00",
            "text/emphasis_00",
            "text/fallback_00",
            "text/linebreak_09",
            "text/numbers_00",
            "text/quote_00",
            "text/quote_01",
            "text/issue-5760-disable-cjk-latin-spacing-in-raw",
            "text/raw-highlight-html",
            "text/raw-highlight-typc",
            "text/raw-line_01",
            "text/shaping_00",
            "text/shaping_01",
            "text/shaping_02",
            "text/space-ideographic-kept",
            "text/stroke_00",
            "text/text-font-covers-chinese",
            "text/text-lang-region",
            "text/text-lang-shaping",
            "text/tracking-spacing_03",
            "visualize/gradient-text-cjk",
            "visualize/gradient-text-emoji",
            "visualize/image-svg-text-font",
            "visualize/issue-2051-new-cm-svg",
            "visualize/stroke-text",
            "visualize/svg-text_00",
            "visualize/svg-text_01"
          ]),
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
