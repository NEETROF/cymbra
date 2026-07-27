import { fileURLToPath, URL } from "node:url";
import { defineConfig } from "vitest/config";
import { loadEnv, type Plugin } from "vite";
import vue from "@vitejs/plugin-vue";

// Strict Content-Security-Policy injected as a <meta> tag at BUILD time only (dev
// stays permissive so Vite's inline scripts / HMR keep working). It's the main
// defense-in-depth against XSS: `script-src 'self'` blocks injected inline scripts,
// and `connect-src` is pinned to this app + the gRPC-web API origin so a successful
// XSS still can't exfiltrate tokens to an attacker host.
//
// NOTE: `frame-ancestors`, `report-uri` and `sandbox` are ignored in a <meta> CSP —
// the reverse proxy should ALSO send the header form in prod (with those + HSTS). If
// Google Identity Services is wired for sign-in later, extend script-src/connect-src
// /frame-src for `accounts.google.com`.
function cspMetaPlugin(apiOrigin: string): Plugin {
  const csp = [
    "default-src 'self'",
    "script-src 'self'",
    "style-src 'self' 'unsafe-inline'",
    "img-src 'self' data:",
    "font-src 'self'",
    `connect-src 'self'${apiOrigin ? ` ${apiOrigin}` : ""}`,
    "object-src 'none'",
    "base-uri 'none'",
    "form-action 'self'",
  ].join("; ");
  return {
    name: "inject-csp-meta",
    apply: "build",
    transformIndexHtml() {
      return [
        {
          tag: "meta",
          attrs: { "http-equiv": "Content-Security-Policy", content: csp },
          injectTo: "head-prepend",
        },
      ];
    },
  };
}

// Client-rendered SPA (no SSR). The gRPC-web endpoint + OIDC client id come from
// runtime env (VITE_*), so the same build ships to any environment.
export default defineConfig(({ mode }) => {
  const env = loadEnv(mode, process.cwd(), "VITE_");
  let apiOrigin = "";
  try {
    apiOrigin = env.VITE_GRPC_WEB_URL ? new URL(env.VITE_GRPC_WEB_URL).origin : "";
  } catch {
    apiOrigin = "";
  }

  return {
    plugins: [vue(), cspMetaPlugin(apiOrigin)],
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
  };
});
