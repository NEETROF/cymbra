// @cymbra/web-auth — the one browser sign-in implementation for every Cymbra web
// front-end (back office, public site). Source-only TypeScript; Vite compiles it in
// place. Never persists a token in web storage.
export { type Async, failure, idle, loading, messageOf, run, success } from "./async";
export {
  createWebAuthClient,
  CSRF_HEADER,
  fetchJson,
  toError,
  WebAuthError,
  type FetchJsonInit,
  type WebAuthClient,
  type WebAuthTokens,
} from "./client";
export { useGoogleSignIn, type GsiButtonOptions, type SignInOptions } from "./google";
export { useAppleSignIn } from "./apple";
export { createWebSession, type WebSession, type WebSessionState } from "./session";
