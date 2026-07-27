/// <reference types="vite/client" />

interface ImportMetaEnv {
  /** Backend gRPC-web base URL, e.g. https://api.cymbra.app */
  readonly VITE_GRPC_WEB_URL: string;
  /** Backend web-auth HTTP base URL (cookie sign-in/refresh/logout), e.g.
   * https://api.cymbra.app — must be same-site with the SPA in production. */
  readonly VITE_WEB_AUTH_URL?: string;
  /** Google OIDC web client id used for back-office sign-in (targets the `music` audience). */
  readonly VITE_GOOGLE_CLIENT_ID?: string;
  /** Set to "1" only by the Playwright dev server: installs fake gRPC-web clients
   * (see lib/e2e-seam.ts) so the app runs end-to-end with no backend. Never set in
   * production builds. */
  readonly VITE_E2E?: string;
}
interface ImportMeta {
  readonly env: ImportMetaEnv;
}

declare module "*.vue" {
  import type { DefineComponent } from "vue";
  const component: DefineComponent<Record<string, unknown>, Record<string, unknown>, unknown>;
  export default component;
}
