import { defineWorkspace } from 'vitest/config';

export default defineWorkspace([
  {
    test: {
      include: ['{tests,src}/**/*.all.{test,spec}.mts', '{tests,src}/**/*.node.{test,spec}.mts'],
    },
  },
  {
    test: {
      include: ['{tests,src}/**/*.all.{test,spec}.mts', '{tests,src}/**/*.browser.{test,spec}.mts'],
      testTimeout: 120_000,
      browser: {
        headless: true,
        provider: 'playwright', // or 'webdriverio'
        enabled: true,
        // at least one instance is required
        instances: [
          {
            browser: 'chromium',
            isolate: false,
            screenshotFailures: false,
          },
        ],
      },
    },
  },
]);
