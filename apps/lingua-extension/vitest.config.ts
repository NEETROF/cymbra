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
      // Excluded from coverage: the wasm-bindgen glue and the thin MV3 entry points
      // (service worker, content-script bootstrap, popup DOM wiring) — they need a
      // real browser + the WASM module and are exercised by manual/e2e testing, not
      // vitest. The pure reading/state logic under src/reading and src/state stays measured.
      exclude: [
        "src/wasm/**",
        "src/background.ts",
        "src/content.ts",
        "src/popup/popup.ts",
        "src/account/account.ts",
        "src/sidepanel/sidepanel.ts",
        "src/reading/drawer.ts",
        "src/analyzer/engine.ts",
        "src/analyzer/create-port.ts",
        "src/analyzer/rpc.ts",
        "**/*.d.ts",
      ],
    },
  },
});
