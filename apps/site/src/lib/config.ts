// Public build-time configuration of the islands (Astro `PUBLIC_*` env, inlined at
// build). A missing provider client id hides that button; a missing API URL falls
// back to the local backend so `yarn dev` works against a dev server.

const DEFAULT_API_URL = "http://localhost:8081";

export interface SiteConfig {
  apiUrl: string;
  googleClientId: string | null;
  appleClientId: string | null;
  paddleEnv: "sandbox" | "production";
  paddleClientToken: string | null;
}

function nonEmpty(v: string | undefined): string | null {
  const s = (v ?? "").trim();
  return s.length ? s : null;
}

export function readConfig(env: ImportMetaEnv = import.meta.env): SiteConfig {
  return {
    apiUrl: nonEmpty(env.PUBLIC_API_URL) ?? DEFAULT_API_URL,
    googleClientId: nonEmpty(env.PUBLIC_GOOGLE_CLIENT_ID),
    appleClientId: nonEmpty(env.PUBLIC_APPLE_CLIENT_ID),
    paddleEnv: nonEmpty(env.PUBLIC_PADDLE_ENV) === "sandbox" ? "sandbox" : "production",
    paddleClientToken: nonEmpty(env.PUBLIC_PADDLE_CLIENT_TOKEN),
  };
}

export const config: SiteConfig = readConfig();
