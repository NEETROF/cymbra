import { configDefaults, defineConfig } from "vitest/config";
import vue from "@vitejs/plugin-vue";

// Two projects, one `yarn test`:
// - `unit`: the islands' pure logic (lib/*) and the components, on jsdom. Plain Vite
//   config (not Astro's) so `astro check` stays clean; `PUBLIC_*` env is stubbed per test
//   where needed. `@vitejs/plugin-vue` comes with `@astrojs/vue`.
// - `astro` (`vitest.astro.config.ts`): `.astro` pages rendered through Astro's Container
//   API, under Astro's own Vite config (change: add-site-lingua-matrix-pages).
export default defineConfig({
  plugins: [vue()],
  resolve: { dedupe: ["vue"] },
  test: {
    projects: [
      {
        extends: true,
        test: {
          name: "unit",
          environment: "jsdom",
          globals: true,
          include: ["test/**/*.spec.ts"],
          // `test/post-build/` asserts against `dist/` and belongs to `yarn check:routes`,
          // which runs AFTER the build. Left in, it fails this suite on a clean checkout —
          // and in CI, where `yarn test` runs before `yarn build`. `test/astro/` is the
          // other project's.
          exclude: [...configDefaults.exclude, "test/post-build/**", "test/astro/**"],
        },
      },
      "./vitest.astro.config.ts",
    ],
  },
});
