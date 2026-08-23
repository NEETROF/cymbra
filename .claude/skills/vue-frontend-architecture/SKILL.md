---
name: vue-frontend-architecture
description: Architecture rules for Vue 3 + TypeScript front-ends in this repo (e.g. apps/back-office). Use when creating or editing any Vue screen/component, Pinia store, composable, gRPC-web/API client call, async/loading/error state, or Playwright e2e tests — and whenever adding or editing user-facing strings or locale files (en/fr must stay aligned). Enforces two hard rules — components never call an API directly (only stores/composables do), and every async resource is one ts-pattern discriminated union (Async<T>), never scattered loading/error/data refs — plus the e2e pattern (a gated fake-client seam driven by Playwright, no backend) and the no-translation-drift rule.
metadata:
  author: cymbra
  version: "1.1"
---

# Vue front-end architecture (Cymbra)

Two **hard rules** for Vue 3 + TS apps in this repo. Apply them proactively — the
maintainer challenges architecture in review, so ship code that already follows
them. Reference implementation: `apps/back-office/` (`src/lib/async.ts`,
`src/lib/api.ts`, `src/stores/catalog.ts`, and the views).

## Rule 1 — a screen/component NEVER calls an API directly

Only **Pinia stores** (or composables) may call the API / gRPC-web clients.
Views and components depend on stores; they never import the client or `api()`.

- Keep the client behind an **injectable seam** so stores are unit-testable with
  fakes — e.g. `lib/api.ts` exposing `api()` + `setClientsForTest(fake)`.
- Quick check before committing: `grep -rn "from \"@/lib/api\"" src/views src/components`
  must return nothing.

## Rule 2 — async state is a discriminated union matched with `ts-pattern`

Never model a remote resource as separate `loading` / `error` / `data` refs (that
allows impossible states: loading + error + data at once). Model it as ONE
`Async<T>` value and `match(...).exhaustive()` it, so a forgotten state is a
**compile error** and failures live in the union rather than being thrown.

```ts
// src/lib/async.ts
export type Async<T, E = string> =
  | { readonly status: "idle" }
  | { readonly status: "loading" }
  | { readonly status: "success"; readonly data: T }
  | { readonly status: "error"; readonly error: E };

export const idle: Async<never> = { status: "idle" };
export const loading: Async<never> = { status: "loading" };
export const success = <T>(data: T): Async<T, never> => ({ status: "success", data });
export const failure = <E = string>(error: E): Async<never, E> => ({ status: "error", error });

/** Fold a promise into an Async ref — never throws; outcome lives in the state. */
export async function run<T>(ref: Ref<Async<T>>, fn: () => Promise<T>): Promise<Async<T>> {
  ref.value = loading;
  try { ref.value = success(await fn()); }
  catch (e) { ref.value = failure(e instanceof Error ? e.message : String(e)); }
  return ref.value;
}
```

**Store** holds the union and does all API work:

```ts
export const useCatalogStore = defineStore("catalog", () => {
  const result = ref<Async<CatalogResult>>(idle);
  async function search(p: SearchParams) {
    await run(result, async () => {
      const resp = await api().score.searchCatalog({ /* ... */ });
      return { hits: resp.hits, total: resp.total };
    });
  }
  return { result, search };
});
```

**View** folds the union into a flat, template-safe view-model with an exhaustive
match (do the `match` in `<script setup>`, NOT the template — `vue-tsc` does not
narrow unions reliably inside templates):

```ts
const vm = computed(() =>
  match(store.result)
    .with({ status: "idle" },    () => ({ loading: false, error: null as string | null, items: [] }))
    .with({ status: "loading" }, () => ({ loading: true,  error: null, items: [] }))
    .with({ status: "error" },   ({ error }) => ({ loading: false, error, items: [] }))
    .with({ status: "success" }, ({ data }) => ({ loading: false, error: null, items: data.hits }))
    .exhaustive(),
);
// template uses vm.loading / vm.error / vm.items — all always defined.
```

Model **every** async thing this way: list fetches, a submit/action outcome
(`Async<void>`), sign-in, byte fetches. A denied action is `{ status: "error" }`,
asserted in tests — not a thrown exception.

## Lint & format

ESLint (flat config) + Prettier gate the app. ESLint uses
`eslint-plugin-vue` + `@vue/eslint-config-typescript` for code-quality/Vue rules and
`@vue/eslint-config-prettier` **skip-formatting** so it never fights Prettier;
Prettier owns all whitespace/quotes. Ignore generated stubs (`src/gen/**`) in both.
`yarn lint` / `yarn format:check` are CI steps — fix with `yarn lint:fix` /
`yarn format`. Type-checking stays `vue-tsc` (not ESLint), so no type-aware parsing.

## Unit tests

Stores are the unit under test: `setClientsForTest(fake)` then assert on the
union (`store.result.status === "success"` / `"error"`), not on booleans. Vitest +
`@vue/test-utils`; it's the front-end's own gate, outside the Flutter/Rust CI.

## End-to-end tests (Playwright)

Run the **real app in a browser with no backend** by reusing the same client seam
(`setClientsForTest`) — but installed from inside the page. This exercises routing,
guards, stores, i18n and error mapping together. Reference: `apps/back-office/`
(`src/lib/e2e-seam.ts`, `playwright.config.ts`, `e2e/`).

**Gated seam (never ships to prod).** A tiny module reads canned data off `window`
and installs fake clients; `main.ts` imports it **dynamically, behind a build-time
env flag**, so it's dead-code-eliminated from normal builds (verify: `grep
__CYMBRA_E2E__ dist/assets` is empty).

```ts
// main.ts — the ONLY wiring change
if (import.meta.env.VITE_E2E) {
  const { installE2EClients } = await import("./lib/e2e-seam"); // tree-shaken when unset
  installE2EClients();
} else {
  initApi(() => auth.accessToken);
}

// lib/e2e-seam.ts — build fake Clients from window.__CYMBRA_E2E__, then
// setClientsForTest(fakes). Throw a real ConnectError to exercise error mapping.
```

**Playwright drives it.** `webServer` runs the dev server with the flag; tests seed
data (and optional auth) via `addInitScript` **before** boot:

```ts
// playwright.config.ts
webServer: { command: `yarn dev --port 5180 --strictPort`, port: 5180,
             reuseExistingServer: !process.env.CI, env: { VITE_E2E: "1" } },
use: { baseURL: "http://localhost:5180", locale: "en-US" },

// e2e/fixtures.ts — seed BEFORE goto()
export async function seed(page, { data = {}, loginAs } = {}) {
  await page.addInitScript(() => localStorage.setItem("cymbra.bo.locale", "en")); // force locale
  await page.addInitScript((d) => { window.__CYMBRA_E2E__ = d; }, data);
  if (loginAs) await page.addInitScript((t) => localStorage.setItem("cymbra.bo.tokens",
    JSON.stringify({ accessToken: t, refreshToken: "r" })), tokenFor(loginAs));
}
```

Rules:
- **Force the locale** in every test (seed `cymbra.bo.locale` + `use.locale`) so i18n
  assertions don't depend on the runner. Assert against the app's own strings.
- **Assert user-visible outcomes**: URL, headings, localized text — and that raw
  error codes/messages never leak (`await expect(page.locator("body")).not
  .toContainText("unauthenticated")`).
- Cover the flows a store test can't: route guards, redirects, deep-link/refresh
  self-sufficiency, sign-in → landing, action → navigation.
- Reporter: `[["list"], ["html", { open: "never" }]]` (always write the report so
  `yarn e2e:report` works locally and CI can upload it).
- CI must generate the gitignored gRPC-web stubs (`yarn gen`, needs `protoc`) before
  typecheck/test, then `yarn playwright install --with-deps chromium` + `yarn e2e`.

## i18n: locales never drift

**Hard rule — no translation drift.** A key added, renamed or removed in one
locale lands in every other **in the same change**, with a real translation.

- **Back office** (`apps/back-office/src/i18n/locales/en.json` + `fr.json`):
  the two files mirror each other key-for-key, nested keys included. vue-i18n
  silently falls back on a missing key, so drift ships unnoticed — diff the
  flattened key sets before pushing:

  ```bash
  python3 -c "
  import json
  def flat(d,p=''):
      return {k2:v2 for k,v in d.items() for k2,v2 in
              (flat(v,f'{p}.{k}' if p else k).items() if isinstance(v,dict)
               else {(f'{p}.{k}' if p else k):v}.items())}
  en=flat(json.load(open('src/i18n/locales/en.json')))
  fr=flat(json.load(open('src/i18n/locales/fr.json')))
  print('fr missing:',sorted(set(en)-set(fr)) or 'OK')
  print('fr extra:',sorted(set(fr)-set(en)) or 'OK')
  "
  ```

- **Site** (`apps/site/src/lib/i18n.ts`): `fr` is the source of truth and `en`
  is declared `const en: typeof fr`, so the **compiler** enforces alignment.
  Never loosen that annotation (no `Partial`, no `as`); add new locales the
  same way (`const xx: typeof fr`).

## Checklist

- [ ] No `api()` / client import in `src/views` or `src/components`.
- [ ] Each remote resource is an `Async<T>` in a store/composable (no `loading`+`error`+`data` trio).
- [ ] Views use `match(...).exhaustive()` in `<script setup>`.
- [ ] Errors captured in the union; actions don't throw for expected failures.
- [ ] Store tests inject fakes via `setClientsForTest` and assert on `status`.
- [ ] E2E seam is dynamically imported behind an env flag (absent from `dist/`).
- [ ] E2E tests force the locale and assert no raw error code leaks into the DOM.
- [ ] Locale files aligned: BO `en.json`/`fr.json` mirror key-for-key; site `en` stays typed `typeof fr`.
- [ ] `yarn lint` + `yarn format:check` pass (ESLint flat config with skip-formatting; Prettier owns formatting; `src/gen/**` ignored).
