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
// the reverse proxy should ALSO send the header form in prod (with those + HSTS).
//
// Social sign-in widens the policy PER PROVIDER, and only when that provider's client
// id is set at build time — so a build with neither keeps the tight `'self'` policy.
// Google Identity Services and Sign in with Apple each need their script/style/frame
// /connect origins allow-listed (Google's are documented at
// developers.google.com/identity/gsi/web/guides/csp).
function cspMetaPlugin(opts: { apiOrigin: string; google: boolean; apple: boolean }): Plugin {
  const { apiOrigin, google, apple } = opts;
  // 'wasm-unsafe-eval' is the narrow, wasm-only allowance browsers require to
  // instantiate a WebAssembly module (the notation renderer) under a restrictive
  // CSP — it does NOT permit JS eval(). Scripts otherwise stay same-origin.
  const scriptSrc = ["'self'", "'wasm-unsafe-eval'"];
  const styleSrc = ["'self'", "'unsafe-inline'"];
  const connectSrc = ["'self'"];
  const frameSrc: string[] = [];
  const formAction = ["'self'"];
  if (apiOrigin) connectSrc.push(apiOrigin);
  if (google) {
    scriptSrc.push("https://accounts.google.com/gsi/client");
    styleSrc.push("https://accounts.google.com/gsi/style");
    connectSrc.push("https://accounts.google.com/gsi/");
    frameSrc.push("https://accounts.google.com/gsi/");
  }
  if (apple) {
    scriptSrc.push("https://appleid.cdn-apple.com");
    connectSrc.push("https://appleid.apple.com");
    frameSrc.push("https://appleid.apple.com");
    formAction.push("https://appleid.apple.com");
  }
  const csp = [
    "default-src 'self'",
    `script-src ${scriptSrc.join(" ")}`,
    `style-src ${styleSrc.join(" ")}`,
    "img-src 'self' data:",
    "font-src 'self'",
    `connect-src ${connectSrc.join(" ")}`,
    "object-src 'none'",
    "base-uri 'none'",
    `form-action ${formAction.join(" ")}`,
    ...(frameSrc.length ? [`frame-src ${frameSrc.join(" ")}`] : []),
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
  const csp = {
    apiOrigin,
    google: !!env.VITE_GOOGLE_CLIENT_ID,
    apple: !!env.VITE_APPLE_CLIENT_ID,
  };

  return {
    plugins: [vue(), cspMetaPlugin(csp)],
    resolve: {
      alias: { "@": fileURLToPath(new URL("./src", import.meta.url)) },
      // `@cymbra/web-auth` is a source-only portal package (packages/web-auth) whose
      // composables import `vue`: force ONE Vue instance from this app, never the
      // package's own node_modules (a second copy would break reactivity).
      dedupe: ["vue"],
    },
    // The score engine worker (lib/worker/) lazy-imports the wasm modules, so its
    // bundle is code-split — which requires ES-module worker output (the default
    // "iife" can't code-split). Matches the `{ type: "module" }` worker we create.
    worker: { format: "es" },
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
          "src/wasm/**",
          "src/main.ts",
          "src/router.ts",
          "src/App.vue",
          "src/views/**",
          "src/components/StatBar.vue",
          "src/components/ScorePreview.vue",
          "src/lib/transport.ts",
          "src/lib/grpc-devtools.ts",
          "src/lib/e2e-seam.ts",
          // Untestable-in-jsdom seams (dynamic wasm import, Web Audio, SVG DOM) — the
          // pure logic behind them (painter, schedule, review session, soundfont fetch)
          // stays measured. Mirrored in sonar-project.properties.
          "src/lib/notation/wasm.ts",
          "src/lib/audio/synth.ts",
          // The score engine Web Worker + its main-thread client: jsdom can't spawn
          // the worker, and unit tests inject stubs at the seam above it. The pure
          // logic it runs (painter, schedule) stays measured on the main-thread path.
          "src/lib/worker/**",
          "src/composables/useScorePlayer.ts",
          "src/composables/usePlayhead.ts",
          "**/*.d.ts",
        ],
      },
    },
  };
});
