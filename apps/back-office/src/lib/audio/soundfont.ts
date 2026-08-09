// Fetches SoundFonts from the backend delivery route (change: add-soundfont-delivery) —
// an authenticated `GET /soundfonts/{id}` on the API origin, NOT a static asset. A font
// is tens (up to hundreds) of MB, so every one of them — the app's default piano *and*
// any catalog font a moderator auditions — is fetched lazily, with the caller's access
// token, and **persisted in the Cache API** so it downloads at most once across
// sessions; in memory it is cached per id for the tab's lifetime and never unloaded.
// Injectable for unit tests.

// Bumped v1 → v2 to evict entries poisoned before the delivery-route fix: a misrouted
// backend (Caddy forwarding /soundfonts/* to the gRPC upstream) answered `200
// application/grpc` with a 0-byte body, which the old code happily cached — serving 0
// bytes forever after. v2 misses those, and `isPlausibleSoundFont` below stops it
// recurring.
const CACHE = "cymbra-soundfont-v2";

/** Guard against caching a non-SoundFont `200`. A misrouted backend answers `200
 *  application/grpc` with an empty body; caching it poisons every later load (a `200`
 *  passes `resp.ok`). We don't parse the SF2 here — that's the synth's job — we only
 *  reject the empty / wrong-framing payload that must never reach the cache. */
function isPlausibleSoundFont(resp: Response, bytes: Uint8Array): boolean {
  const contentType = resp.headers.get("content-type") ?? "";
  if (contentType.startsWith("application/grpc")) return false;
  return bytes.length > 0;
}

/** Catalog id of the app's default piano — the one `loadSoundFont` serves. The preview
 *  font picker pre-selects this row (and keeps using this lazy loader for it, rather than
 *  eagerly fetching its bytes). Single source of truth for "the default sound". */
export const DEFAULT_SOUNDFONT_ID = "upright-piano-kw";

/** Absolute URL of a font on the delivery route. `VITE_SOUNDFONT_URL` overrides the
 *  **default** font's URL (a deployment may serve it from elsewhere); every other id
 *  resolves against the web-auth/API origin (same host as sign-in). */
function soundfontUrl(id: string): string {
  const explicit = import.meta.env.VITE_SOUNDFONT_URL as string | undefined;
  if (explicit && id === DEFAULT_SOUNDFONT_ID) return explicit;
  let base = (import.meta.env.VITE_WEB_AUTH_URL as string | undefined) ?? "http://localhost:8081";
  while (base.endsWith("/")) base = base.slice(0, -1); // trim trailing slashes (no regex)
  return `${base}/soundfonts/${encodeURIComponent(id)}`;
}

/** In-flight / resolved bytes per catalog id, for the tab's lifetime. */
const cached = new Map<string, Promise<Uint8Array>>();

/** Ask the browser to make this origin's storage **persistent**, once per tab. Without
 *  it a Cache API holding a 57 MB font is "best-effort" and Chrome may evict it under
 *  disk pressure — which is exactly the "it re-downloads again today" symptom. Purely
 *  advisory: the browser may refuse (and does, silently, in a non-secure context), and
 *  the caching path works either way. */
let persistenceRequested = false;
async function requestPersistence(): Promise<void> {
  if (persistenceRequested) return;
  persistenceRequested = true;
  try {
    await navigator.storage?.persist?.();
  } catch {
    // Storage API absent or refused — the Cache API still works, just evictable.
  }
}

/** Load (once) the **default** piano's bytes. Thin alias over [`loadSoundFontById`]. */
export function loadSoundFont(token: string | null): Promise<Uint8Array> {
  return loadSoundFontById(DEFAULT_SOUNDFONT_ID, token);
}

/** Load (once) the bytes of catalog font `id` with the caller's access `token`,
 *  persisting them in the Cache API. Subsequent calls (and sessions) reuse the cached
 *  bytes without re-fetching or re-authenticating. A failed load resets so a later retry
 *  can succeed. */
export function loadSoundFontById(id: string, token: string | null): Promise<Uint8Array> {
  const hit = cached.get(id);
  if (hit) return hit;
  const url = soundfontUrl(id);
  const p = (async () => {
    // Cache API is keyed by URL (token-independent) — once fetched, no more auth. Its
    // `add()` can't set the Authorization header, so we fetch with the bearer ourselves
    // and `put()` the response.
    if (typeof caches !== "undefined") {
      try {
        const cache = await caches.open(CACHE);
        const stored = await cache.match(url);
        if (stored) return new Uint8Array(await stored.arrayBuffer());
        const resp = await authedFetch(url, token);
        if (!resp.ok) throw new Error(`soundfont ${resp.status}`);
        const bytes = new Uint8Array(await resp.arrayBuffer());
        if (!isPlausibleSoundFont(resp, bytes)) throw new Error("soundfont empty-or-misframed");
        await requestPersistence();
        // Body already consumed — re-wrap the validated bytes to cache them.
        await cache.put(url, new Response(bytes));
        return bytes;
      } catch (e) {
        // A real HTTP failure or a rejected payload propagates (message starts with
        // "soundfont "); a Cache-API fault (private mode, quota) falls through to a
        // plain fetch.
        if (e instanceof Error && e.message.startsWith("soundfont ")) throw e;
      }
    }
    const resp = await authedFetch(url, token);
    if (!resp.ok) throw new Error(`soundfont ${resp.status}`);
    const bytes = new Uint8Array(await resp.arrayBuffer());
    if (!isPlausibleSoundFont(resp, bytes)) throw new Error("soundfont empty-or-misframed");
    return bytes;
  })();
  cached.set(id, p);
  // Don't cache a failure permanently — reset so a retry (after re-auth) can work. A
  // success stays cached for the tab's life (never unloaded).
  p.catch(() => {
    if (cached.get(id) === p) cached.delete(id);
  });
  return p;
}

/** Drop font `id` from both caches. The stored bytes are treated as immutable per id
 *  (the upload route refuses a duplicate id, and dedups identical content), so the only
 *  way they can change is delete-then-re-upload under the same id — which is what this
 *  evicts, from the console that performed the deletion. */
export async function evictSoundFont(id: string): Promise<void> {
  cached.delete(id);
  if (typeof caches === "undefined") return;
  try {
    const cache = await caches.open(CACHE);
    await cache.delete(soundfontUrl(id));
  } catch {
    // Cache API unavailable (private mode) — nothing was persisted to evict.
  }
}

function authedFetch(url: string, token: string | null): Promise<Response> {
  const headers: Record<string, string> = {};
  if (token) headers.Authorization = `Bearer ${token}`;
  return fetch(url, { headers });
}

/** Test seam: inject the default font's bytes (or reset **every** id with `null`). */
export function setSoundFontForTest(bytes: Uint8Array | null): void {
  cached.clear();
  if (bytes) cached.set(DEFAULT_SOUNDFONT_ID, Promise.resolve(bytes));
}
