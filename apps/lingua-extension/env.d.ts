// Ambient module declarations for the extension build.

// Build-time target, injected by esbuild `define` per manifest variant. Prefer the
// capability defines below; __TARGET__ stays for what is genuinely browser-specific
// (internal settings URLs, whether a shortcut editor exists).
declare const __TARGET__: "chromium" | "firefox" | "safari";

// Build-time capabilities (build.mjs `capabilities`), true for firefox and safari:
// the analysis engine lives in the background event page, reached over messaging…
declare const __ENGINE_IN_EVENT_PAGE__: boolean;
// …review, stats and settings open in the in-page drawer (no native panel reachable)…
declare const __REVIEW_IN_PAGE__: boolean;
// …and the reader is injected by a static content script, not registered dynamically.
declare const __STATIC_READER__: boolean;
// Safari only: Apple and Google sign-in come from the host app over native messaging
// (add-lingua-connected-clients D6), since Safari has no identity.launchWebAuthFlow.
declare const __NATIVE_PROVIDERS__: boolean;

// Where the translation engine is hosted (add-lingua-translation-engine). "none" in every
// shipped build: the engine is built in only by a development build that side-loads a model
// (build.mjs, LINGUA_TRANSLATION_ENGINE). "offscreen" on Chromium, whose service worker cannot
// construct a Worker; "event-page" on Firefox, whose background page can. Never on a thread
// that paints — there is no value for that.
declare const __TRANSLATION_HOST__: "none" | "offscreen" | "event-page";

// Backend gRPC-web origin for the sync transport, injected by esbuild `define`
// (build.mjs, from LINGUA_GRPC_WEB_URL). Defaults to the local backend for dogfooding.
declare const __GRPC_WEB_URL__: string;

// Google OAuth Web client id (chromiumapp.org redirect) for "Continue with Google",
// injected by esbuild `define` (from LINGUA_GOOGLE_CLIENT_ID). Empty until the client is
// registered in Cloud console (task 1.2); an empty value disables Google sign-in with a
// clear error, leaving email/password working.
declare const __GOOGLE_CLIENT_ID__: string;

// Apple Services ID (the site's web client id) for "Continue with Apple", injected by
// esbuild `define` (from LINGUA_APPLE_CLIENT_ID). Empty by default: the Apple button is
// hidden (add-lingua-account-parity, design D5).
declare const __APPLE_CLIENT_ID__: string;

// tokens.css (and other .css) imported as a string, inlined into the injected
// closed shadow root so the single token sheet stays the only home for colors.
declare module "*.css" {
  const css: string;
  export default css;
}

// The wasm-pack `--target web` output (produced by `yarn gen:wasm` into
// src/wasm/pkg/). Typed minimally here; the real .d.ts is generated and gitignored,
// and the content script loads the glue at runtime via chrome.runtime.getURL.
declare module "@/wasm/pkg/lingua_wasm.js" {
  export class LinguaEngine {
    constructor(packBytes: Uint8Array);
    free(): void;
    setCalibration(threshold: number): void;
    setStatus(lemma: string, status: string): void;
    analyse(blocks: string[]): string;
    gloss(lemma: string): string | undefined;
  }
  export default function init(
    moduleOrPath?: { module_or_path: string | Request | Response | URL } | string | Request | Response | URL,
  ): Promise<unknown>;
}

// The vendored foliate-js (vendor/foliate-js, add-lingua-reader D2), resolved by build.mjs's
// `foliate-js` alias. Plain JavaScript with no types of its own: only what the adapter in
// src/reader/foliate.ts touches is declared here.
declare module "foliate-js/epub.js" {
  export class EPUB {
    constructor(loader: {
      loadText(name: string): Promise<string | null>;
      loadBlob(name: string, type?: string): Promise<Blob | null>;
      getSize(name: string): number;
    });
    init(): Promise<FoliateBook>;
  }
  export interface FoliateTocItem {
    label?: string;
    href?: string;
    subitems?: FoliateTocItem[] | null;
  }
  export interface FoliateBook {
    toc?: FoliateTocItem[];
    sections: unknown[];
  }
}

declare module "foliate-js/view.js" {
  import type { FoliateBook } from "foliate-js/epub.js";
  export interface FoliateRelocate {
    cfi: string;
    fraction: number;
    tocItem?: { label?: string } | null;
  }
  export interface FoliatePaginator extends HTMLElement {
    setStyles(styles: string | [string, string]): void;
  }
  export class View extends HTMLElement {
    renderer: FoliatePaginator;
    open(book: FoliateBook): Promise<void>;
    init(options: { lastLocation?: string | null; showTextStart?: boolean }): Promise<void>;
    goTo(target: string): Promise<unknown>;
    next(): Promise<void>;
    prev(): Promise<void>;
    close(): void;
  }
}
