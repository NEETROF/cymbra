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
