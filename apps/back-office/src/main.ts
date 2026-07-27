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

// Wire the gRPC-web clients with a token getter that reads the live auth store.
const auth = useAuthStore(pinia);
initApi(() => auth.accessToken);

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
