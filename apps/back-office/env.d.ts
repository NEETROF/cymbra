/// <reference types="vite/client" />

interface ImportMetaEnv {
  /** Backend gRPC-web base URL, e.g. https://api.cymbra.app */
  readonly VITE_GRPC_WEB_URL: string;
  /** Backend web-auth HTTP base URL (cookie sign-in/refresh/logout), e.g.
   * https://api.cymbra.app — must be same-site with the SPA in production. */
  readonly VITE_WEB_AUTH_URL?: string;
  /** Google OIDC web client id used for back-office sign-in (targets the `music` audience). */
  readonly VITE_GOOGLE_CLIENT_ID?: string;
  /** Apple "Sign in with Apple" web client id — a Services ID (NOT the app bundle id),
   * registered with the SPA domain + Return URL. Unset → the Apple button is hidden. */
  readonly VITE_APPLE_CLIENT_ID?: string;
  /** Apple Return URL that must exactly match one registered on the Services ID.
   * Defaults to the SPA origin when unset. */
  readonly VITE_APPLE_REDIRECT_URI?: string;
  /** Set to "1" only by the Playwright dev server: installs fake gRPC-web clients
   * (see lib/e2e-seam.ts) so the app runs end-to-end with no backend. Never set in
   * production builds. */
  readonly VITE_E2E?: string;
  /** Store aggregator project id (change: swap-store-billing-to-revenuecat):
   * builds the "open in RevenueCat" customer link on the plan console. Unset →
   * no link. */
}
interface ImportMeta {
  readonly env: ImportMetaEnv;
}

declare module "*.vue" {
  import type { DefineComponent } from "vue";
  const component: DefineComponent<Record<string, unknown>, Record<string, unknown>, unknown>;
  export default component;
}
