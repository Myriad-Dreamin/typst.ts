import * as driver from './driver';
export { createTypstRenderer } from './driver';

// Export module on window.
// todo: graceful way?
if (window) {
  (window as any).TypstRenderModule = {
    createTypstRenderer: driver.createTypstRenderer,
  };
}
