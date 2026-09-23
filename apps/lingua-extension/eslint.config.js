import js from "@eslint/js";
import tseslint from "typescript-eslint";
import globals from "globals";

// Flat config, framework-free TypeScript (no Vue). typescript-eslint's recommended
// (non type-aware) rules cover the TS; real type-checking is `tsc --noEmit` (the
// `typecheck` script). Prettier owns whitespace, so no stylistic rules here.
export default tseslint.config(
  {
    name: "ext/ignores",
    // vendor/: third-party code kept byte-for-byte at a pinned commit (vendor/VENDOR.md).
    // engine/: Mozilla's translation engine, fetched pinned (tool/fetch_engine.sh), never ours to lint.
    ignores: ["dist/**", "dist-*/**", "coverage/**", "src/wasm/pkg/**", "src/gen/**", "assets/**", "vendor/**", "engine/**"],
  },
  js.configs.recommended,
  tseslint.configs.recommended,
  {
    name: "ext/browser-and-worker",
    files: ["src/**/*.ts"],
    languageOptions: {
      globals: { ...globals.browser, ...globals.webextensions, ...globals.serviceworker },
    },
  },
  {
    name: "ext/node-tooling",
    files: ["build.mjs", "tool/**/*.mjs", "*.config.ts", "vitest.config.ts"],
    languageOptions: {
      globals: { ...globals.node },
    },
  },
);
