import { defineConfig } from "vitest/config";

// The post-build gate (`yarn check:routes`), kept apart from `yarn test` on purpose:
// it asserts against `dist/`, so it is only meaningful AFTER `yarn build`, while the
// unit suite runs on sources in jsdom and must stay runnable at any time.
export default defineConfig({
  test: {
    environment: "node",
    globals: true,
    include: ["test/post-build/**/*.spec.ts"],
  },
});
