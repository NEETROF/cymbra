import { defineConfig } from "vitest/config";
import vue from "@vitejs/plugin-vue";

// Vitest over the islands' pure logic (lib/*) and the components, on jsdom. Plain
// Vite config (not Astro's) so `astro check` stays clean; `PUBLIC_*` env is stubbed
// per test where needed. `@vitejs/plugin-vue` comes with `@astrojs/vue`.
export default defineConfig({
  plugins: [vue()],
  resolve: { dedupe: ["vue"] },
  test: {
    environment: "jsdom",
    globals: true,
    include: ["test/**/*.spec.ts"],
  },
});
