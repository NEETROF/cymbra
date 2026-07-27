---
name: vue-frontend-architecture
description: Architecture rules for Vue 3 + TypeScript front-ends in this repo (e.g. apps/back-office). Use when creating or editing any Vue screen/component, Pinia store, composable, gRPC-web/API client call, or async/loading/error state. Enforces two hard rules — components never call an API directly (only stores/composables do), and every async resource is one ts-pattern discriminated union (Async<T>), never scattered loading/error/data refs.
metadata:
  author: cymbra
  version: "1.0"
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

## Testing

Stores are the unit under test: `setClientsForTest(fake)` then assert on the
union (`store.result.status === "success"` / `"error"`), not on booleans. Vitest +
`@vue/test-utils`; it's the front-end's own gate, outside the Flutter/Rust CI.

## Checklist

- [ ] No `api()` / client import in `src/views` or `src/components`.
- [ ] Each remote resource is an `Async<T>` in a store/composable (no `loading`+`error`+`data` trio).
- [ ] Views use `match(...).exhaustive()` in `<script setup>`.
- [ ] Errors captured in the union; actions don't throw for expected failures.
- [ ] Store tests inject fakes via `setClientsForTest` and assert on `status`.
