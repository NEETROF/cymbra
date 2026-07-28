import { createApp } from "vue";
import { createPinia } from "pinia";
import App from "./App.vue";
import { createAppRouter } from "./router";
import { initApi } from "./lib/api";
import { initWebAuth } from "./lib/web-auth";
import { setTokenRefresher, setUnauthenticatedHandler } from "./lib/transport";
import { useAuthStore } from "./stores/auth";
import { currentLocale, i18n } from "./i18n";
import "./styles.css";

// Wrapped in an async bootstrap so the cookie-refresh await stays inside a function —
// a top-level `await` would force a build target the app doesn't ship (es2020/Safari 14).
async function bootstrap() {
  const app = createApp(App);
  const pinia = createPinia();
  app.use(pinia);
  app.use(i18n);
  if (globalThis.document) document.documentElement.lang = currentLocale();

  // The access token lives in memory only; the transport's token getter reads it live,
  // and `auth.bootstrap()` (below) re-mints it from the HttpOnly refresh cookie on load.
  const auth = useAuthStore(pinia);

  // E2E seam: when served with VITE_E2E=1, Playwright seeds `window.__CYMBRA_E2E__`
  // (via addInitScript) before boot and this installs fake gRPC-web + web-auth clients,
  // so the real app runs with no backend. The dynamic import keeps the seam out of
  // normal builds — the whole branch is dead-code-eliminated when VITE_E2E is unset.
  if (import.meta.env.VITE_E2E) {
    const { installE2EClients } = await import("./lib/e2e-seam");
    installE2EClients();
  } else {
    // gRPC data plane sends the in-memory access token; the web-auth surface handles
    // the cookie-carried refresh token.
    initApi(() => auth.accessToken);
    initWebAuth();
  }

  // Silent refresh: on an expired access token, mint a new one from the refresh cookie
  // and retry rather than signing the user out. Returns false (→ sign out) only when the
  // cookie refresh itself fails (no/invalid session).
  setTokenRefresher(async () => {
    try {
      await auth.refresh();
      return true;
    } catch {
      return false;
    }
  });

  // Boot: re-mint an access token from the refresh cookie (silent) BEFORE the router
  // is installed. Vue Router 4 kicks off its initial navigation during `app.use(router)`
  // (in `install()`), and that guard runs on the next microtask — so if we bootstrapped
  // AFTER installing the router, the guard would evaluate the (still-empty) session
  // during this await and wrongly bounce a returning user to sign-in. A failed refresh
  // just means "no session" — the router then legitimately routes to sign-in.
  await auth.bootstrap();

  const router = createAppRouter();
  app.use(router);

  // Session expiry: if any call returns UNAUTHENTICATED while a session exists, the
  // token is expired/rejected AND the refresh above already failed — sign out and send
  // the user to sign-in. Guarded on an existing session so a failed sign-in (bad
  // credentials, also UNAUTHENTICATED) does NOT redirect.
  setUnauthenticatedHandler(() => {
    if (auth.isAuthenticated) {
      void auth.signOut();
      router.push({ name: "signin" });
    }
  });

  app.mount("#app");
}

void bootstrap();
