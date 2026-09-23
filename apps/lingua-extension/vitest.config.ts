import { fileURLToPath, URL } from "node:url";
import { defineConfig } from "vitest/config";

export default defineConfig({
  // Mirror the build-time defines so any module referencing them resolves under test.
  define: {
    __TARGET__: JSON.stringify("chromium"),
    __ENGINE_IN_EVENT_PAGE__: "false",
    __REVIEW_IN_PAGE__: "false",
    __STATIC_READER__: "false",
    __NATIVE_PROVIDERS__: "false",
    __TRANSLATION_HOST__: JSON.stringify("none"),
    __GRPC_WEB_URL__: JSON.stringify("http://localhost:50051"),
    __GOOGLE_CLIENT_ID__: JSON.stringify(""),
    __APPLE_CLIENT_ID__: JSON.stringify(""),
  },
  resolve: {
    alias: { "@": fileURLToPath(new URL("./src", import.meta.url)) },
  },
  test: {
    environment: "jsdom",
    globals: true,
    include: ["test/**/*.spec.ts"],
    coverage: {
      provider: "v8",
      reporter: ["text", "lcov"],
      reportsDirectory: "coverage",
      include: ["src/**/*.ts"],
      // The repo's gate, on the metric CLAUDE.md names: line coverage ≥ 80%, enforced
      // by `yarn test` so it is the same number locally and in CI. It sat at 80.55% when
      // it was set, a thin margin on purpose — the modules that keep it there are the
      // untested ones listed below, not slack in what is measured.
      thresholds: { lines: 80 },
      // Excluded from coverage: the wasm-bindgen glue and the thin MV3 entry points
      // (service worker, content-script bootstrap, and the tabs' DOM wiring) — they need a
      // real browser + the WASM module and are exercised by manual/e2e testing, not
      // vitest. The pure reading/state logic under src/reading and src/state stays measured.
      // What every excluded entry point has in common: it reads the DOM, hydrates an
      // engine and mounts a view that IS measured (`mountStats`, `mountReview`).
      exclude: [
        "src/wasm/**",
        "src/background.ts",
        "src/content.ts",
        "src/popup/popup.ts",
        "src/account/account.ts",
        "src/sidepanel/sidepanel.ts",
        "src/stats/stats.ts",
        "src/onboarding/onboarding.ts",
        "src/reading/drawer.ts",
        "src/analyzer/engine.ts",
        "src/analyzer/create-port.ts",
        "src/analyzer/rpc.ts",
        // The translation engine's two entry points: the worker that loads Mozilla's glue and
        // the model (it needs the real wasm, like analyzer/engine.ts), and the offscreen
        // document that only owns that worker. Their logic lives in translate/host/channel.ts,
        // relay.ts and offscreen-engine.ts, which are measured.
        "src/translate/host/engine-worker.ts",
        "src/translate/host/offscreen.ts",
        "**/*.d.ts",
      ],
    },
  },
});
