//! This works on any browser that supports service worker https://jasonformat.com/javascript-sleep/
//!   See https://developer.mozilla.org/en-US/docs/Web/API/ServiceWorkerGlobalScope/fetch_event
//! It is quite important to note that the service worker can only intercept requests that has same origin.
//! For example, if you have registered this worker with url:
//!   register('typst.ts/article/sync-worker.js')
//! This worker will be able to intercept requests that has same origin as the above url:
//!   typst.ts/article/...
//! But it will not be able to intercept requests that has different origin:
//!   typst.ts/book/...
//!   typst.ts/...
//! To intercept requests that has different origin, you need to register the worker with url:
//!   register('typst.ts/sync-worker.js')
//! or all urls that you want to intercept:
//!   register('typst.ts/article/sync-worker.js')
//!   register('typst.ts/book/sync-worker.js')

export function registerSyncWorker(_workerArgs: unknown) {}
