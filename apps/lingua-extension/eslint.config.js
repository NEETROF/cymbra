import js from "@eslint/js";
import tseslint from "typescript-eslint";
import globals from "globals";

// Flat config, framework-free TypeScript (no Vue). typescript-eslint's recommended
// (non type-aware) rules cover the TS; real type-checking is `tsc --noEmit` (the
// `typecheck` script). Prettier owns whitespace, so no stylistic rules here.
export default tseslint.config(
  {
    name: "ext/ignores",
    ignores: ["dist/**", "coverage/**", "src/wasm/pkg/**", "assets/**"],
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
    files: ["build.mjs", "*.config.ts", "vitest.config.ts"],
    languageOptions: {
      globals: { ...globals.node },
    },
  },
);
