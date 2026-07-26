# Cymbra moderation back office (`bo.cymbra.app`)

A client-rendered Vue 3 + Vite SPA where moderators/admins review catalog scores
and accept/reject them, and admins grant/revoke roles. It talks to the backend over
**gRPC-web** (tonic-web) using generated Connect clients — no REST. Part of the
`add-moderation-back-office` change (frontend slice).

## Develop

```bash
cd apps/back-office
pnpm install
pnpm gen        # generate TS gRPC stubs from the backend protos (needs protoc)
cp .env.example .env   # set VITE_GRPC_WEB_URL to your backend
pnpm dev
```

The backend must run with `CYMBRA_BACK_OFFICE_ORIGINS` including this app's origin
(e.g. `http://localhost:5173`) so CORS allows the browser calls, and a moderator/
admin account must exist (see `backend/scripts/seed_admin.sh`).

## Scripts

- `pnpm gen` — regenerate `src/gen/*` from `backend/*/proto/*.proto` (gitignored).
- `pnpm test` — Vitest component/store tests (its own gate, outside the Flutter/Rust CI).
- `pnpm build` — type-check + production build.

## Auth & access

Sign-in targets the `music` audience so `music`-scoped roles (`moderator`/`admin`)
ride in the access token. The router gates the console: unauthenticated → sign-in;
signed-in non-moderator → access-denied; `/roles` requires `admin`. This is UX only —
**every RPC is independently role-guarded server-side**, so the gate can't be bypassed.

## Preview (notation)

`ScorePreview.vue` is the isolated preview seam. Today it shows metadata and confirms
the fetched bytes; the notation renderer (compile the app's Rust `layout_systems` to
wasm + a JS/SVG SMuFL painter, so it matches the app exactly) is a deferred module that
drops in behind this component — or is swapped for a JS fallback if the wasm cost is
too high.

## Not yet wired (deferred)

- The wasm notation renderer.
- Cloudflare Pages deploy config for `bo.cymbra.app`.
- Full Google OIDC button (the token-exchange path is wired; the GIS button needs a
  configured client id).
