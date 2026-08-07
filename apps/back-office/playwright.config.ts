import { defineConfig, devices } from "@playwright/test";

// E2E runs the real Vite app with the fake-client seam enabled (VITE_E2E=1), so the
// whole browser stack — router, guards, Pinia stores, i18n, error mapping — is
// exercised with no backend. Deterministic and CI-friendly.
// Overridable because `reuseExistingServer` would otherwise silently run the suite
// against ANOTHER checkout's dev server when several worktrees are open at once.
const PORT = Number(process.env.BO_E2E_PORT ?? 5180);

export default defineConfig({
  testDir: "./e2e",
  fullyParallel: true,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 1 : 0,
  // `list` for readable console output + an HTML report written every run (never
  // auto-opened) so `yarn e2e:report` works locally and CI can upload it.
  reporter: [["list"], ["html", { open: "never" }]],
  use: {
    baseURL: `http://localhost:${PORT}`,
    // Pin the browser locale; the app is forced to English per test via seed() too.
    locale: "en-US",
    trace: "on-first-retry",
  },
  projects: [{ name: "chromium", use: { ...devices["Desktop Chrome"] } }],
  webServer: {
    command: `yarn dev --port ${PORT} --strictPort`,
    port: PORT,
    reuseExistingServer: !process.env.CI,
    env: { VITE_E2E: "1" },
    timeout: 60_000,
  },
});
