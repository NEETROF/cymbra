/// <reference types="astro/client" />

interface ImportMetaEnv {
  /** Backend HTTP origin (web-auth + /web/plans/*), e.g. https://api.cymbra.app */
  readonly PUBLIC_API_URL?: string;
  /** Google Web OAuth client id (same as the back office); unset hides the button. */
  readonly PUBLIC_GOOGLE_CLIENT_ID?: string;
  /** Sign in with Apple Services ID (same as the back office); unset hides the button. */
  readonly PUBLIC_APPLE_CLIENT_ID?: string;
  /** Paddle.js environment: "sandbox" | "production" (default production). */
  readonly PUBLIC_PADDLE_ENV?: string;
  /** Paddle client-side token (public, per environment). */
  readonly PUBLIC_PADDLE_CLIENT_TOKEN?: string;
}

interface ImportMeta {
  readonly env: ImportMetaEnv;
}
