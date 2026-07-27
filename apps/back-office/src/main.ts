import { createApp } from "vue";
import { createPinia } from "pinia";
import App from "./App.vue";
import { createAppRouter } from "./router";
import { initApi } from "./lib/api";
import { setUnauthenticatedHandler } from "./lib/transport";
import { useAuthStore } from "./stores/auth";
import { currentLocale, i18n } from "./i18n";
import "./styles.css";

const app = createApp(App);
const pinia = createPinia();
app.use(pinia);
app.use(i18n);
if (globalThis.document) document.documentElement.lang = currentLocale();

// Auth store reads persisted tokens; the transport's token getter reads it live.
const auth = useAuthStore(pinia);

async function boot() {
  // E2E seam: when served with VITE_E2E=1, Playwright seeds `window.__CYMBRA_E2E__`
  // (via addInitScript) before boot and this installs fake gRPC-web clients, so the
  // real app runs with no backend. The dynamic import keeps the seam out of normal
  // builds — the whole branch is dead-code-eliminated when VITE_E2E is unset.
  if (import.meta.env.VITE_E2E) {
    const { installE2EClients } = await import("./lib/e2e-seam");
    installE2EClients();
  } else {
    initApi(() => auth.accessToken);
  }

  const router = createAppRouter();
  app.use(router);

  // Session expiry: if any call returns UNAUTHENTICATED while a session exists, the
  // token is expired/rejected — sign out and send the user to sign-in. Guarded on an
  // existing session so a failed sign-in (bad credentials, also UNAUTHENTICATED) does
  // NOT redirect.
  setUnauthenticatedHandler(() => {
    if (auth.isAuthenticated) {
      auth.signOut();
      router.push({ name: "signin" });
    }
  });

  app.mount("#app");
}

void boot();
