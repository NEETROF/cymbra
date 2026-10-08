/// <reference types="vitest/config" />
import { getViteConfig } from "astro/config";

// The `astro` project of `yarn test` (see `vitest.config.ts`): pages rendered with Astro's
// Container API (`experimental_AstroContainer`), which needs Astro's Vite config to compile
// `.astro` files. Node, not jsdom: the pages are rendered to strings, as the build does.
// `astro check` types these tests; `yarn typecheck` (vue-tsc) cannot read `.astro` imports.
export default getViteConfig({
  test: {
    name: "astro",
    environment: "node",
    globals: true,
    include: ["test/astro/**/*.spec.ts"],
  },
});
