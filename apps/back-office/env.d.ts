/// <reference types="vite/client" />

interface ImportMetaEnv {
  /** Backend gRPC-web base URL, e.g. https://api.cymbra.app */
  readonly VITE_GRPC_WEB_URL: string;
  /** Google OIDC web client id used for back-office sign-in (targets the `music` audience). */
  readonly VITE_GOOGLE_CLIENT_ID?: string;
}
interface ImportMeta {
  readonly env: ImportMetaEnv;
}

declare module "*.vue" {
  import type { DefineComponent } from "vue";
  const component: DefineComponent<Record<string, unknown>, Record<string, unknown>, unknown>;
  export default component;
}
