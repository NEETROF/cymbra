import { fileURLToPath, URL } from "node:url";
import { defineConfig } from "vitest/config";
import vue from "@vitejs/plugin-vue";

// Client-rendered SPA (no SSR). The gRPC-web endpoint + OIDC client id come from
// runtime env (VITE_*), so the same build ships to any environment.
export default defineConfig({
  plugins: [vue()],
  resolve: {
    alias: { "@": fileURLToPath(new URL("./src", import.meta.url)) },
  },
  test: {
    environment: "jsdom",
    globals: true,
    include: ["test/**/*.spec.ts"],
    // Unit/widget coverage of the testable logic layer (stores, lib helpers, i18n,
    // and the widget-tested components). Excluded from coverage — and mirrored in
    // sonar-project.properties `sonar.coverage.exclusions` — are the seams a unit
    // test can't reach and that the Playwright e2e suite (e2e/) is the gate for:
    // the app shell + router + screens, the presentation-only StatBar/ScorePreview,
    // the gRPC-web transport/devtools IO glue, the bootstrap, generated stubs and
    // the test seam. Sonar still ANALYSES these files for bugs/smells (they stay in
    // sonar.sources) — only their coverage is delegated to e2e + `vue-tsc`.
    coverage: {
      provider: "v8",
      reporter: ["text", "lcov"],
      reportsDirectory: "coverage",
      include: ["src/**/*.{ts,vue}"],
      exclude: [
        "src/gen/**",
        "src/main.ts",
        "src/router.ts",
        "src/App.vue",
        "src/views/**",
        "src/components/StatBar.vue",
        "src/components/ScorePreview.vue",
        "src/lib/transport.ts",
        "src/lib/grpc-devtools.ts",
        "src/lib/e2e-seam.ts",
        "**/*.d.ts",
      ],
    },
  },
});
