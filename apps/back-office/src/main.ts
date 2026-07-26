import { createApp } from "vue";
import { createPinia } from "pinia";
import App from "./App.vue";
import { createAppRouter } from "./router";
import { initApi } from "./lib/api";
import { useAuthStore } from "./stores/auth";
import "./styles.css";

const app = createApp(App);
const pinia = createPinia();
app.use(pinia);

// Wire the gRPC-web clients with a token getter that reads the live auth store.
const auth = useAuthStore(pinia);
initApi(() => auth.accessToken);

app.use(createAppRouter());
app.mount("#app");
