// jsdom, the test environment's DOM, which apple-activation-page.spec.ts also builds by hand to
// load the Safari host app's activation page as the app does (a window of its own per page,
// scripts evaluated in it). Typed minimally, beside the one test that imports it: only what that
// test touches.
declare module "jsdom" {
  export class JSDOM {
    constructor(html: string, options?: { runScripts?: "outside-only" | "dangerously" });
    readonly window: Window & typeof globalThis;
  }
}
