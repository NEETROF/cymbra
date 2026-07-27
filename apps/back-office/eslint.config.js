import pluginVue from "eslint-plugin-vue";
import { defineConfigWithVueTs, vueTsConfigs } from "@vue/eslint-config-typescript";
import skipFormatting from "@vue/eslint-config-prettier/skip-formatting";

// Flat config. Lints TS + Vue SFCs for code-quality and Vue best practices;
// `skipFormatting` hands all whitespace/quote decisions to Prettier so the two
// tools never fight. Type-checking itself is `vue-tsc` (see the `typecheck`
// script), not ESLint, so no type-aware (project) parsing is needed here.
export default defineConfigWithVueTs(
  {
    name: "app/files-to-lint",
    files: ["**/*.{ts,mts,vue}"],
  },
  {
    name: "app/files-to-ignore",
    ignores: ["dist/**", "src/gen/**", "coverage/**", "playwright-report/**", "test-results/**"],
  },
  pluginVue.configs["flat/recommended"],
  vueTsConfigs.recommended,
  {
    // The single-word root component is a Vue convention.
    name: "app/overrides",
    files: ["src/App.vue"],
    rules: { "vue/multi-word-component-names": "off" },
  },
  skipFormatting,
);
