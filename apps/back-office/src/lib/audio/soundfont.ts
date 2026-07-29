// Fetches the piano SoundFont from the backend SoundFont-delivery route
// (change: add-soundfont-delivery) — an authenticated `GET /soundfonts/{id}` on the
// API origin, NOT a static asset. It's ~57 MB, so it is fetched lazily (only when a
// moderator first hits Play), with the caller's access token, and **persisted in the
// Cache API** so it downloads at most once across sessions; in memory it is cached for
// the tab's lifetime and never unloaded. Injectable for unit tests.

const CACHE = "cymbra-soundfont-v1";

/** Absolute URL of the default SoundFont. `VITE_SOUNDFONT_URL` overrides; otherwise
 *  it is the delivery route on the web-auth/API origin (same host as sign-in). */
function soundfontUrl(): string {
  const explicit = import.meta.env.VITE_SOUNDFONT_URL as string | undefined;
  if (explicit) return explicit;
  const base = (import.meta.env.VITE_WEB_AUTH_URL as string | undefined) ?? "http://localhost:8081";
  return `${base.replace(/\/+$/, "")}/soundfonts/upright-piano-kw`;
}

let cached: Promise<Uint8Array> | null = null;

/** Load (once) the SoundFont bytes with the caller's access `token`, persisting them in
 *  the Cache API. Subsequent calls (and sessions) reuse the cached bytes without
 *  re-fetching or re-authenticating. A failed load resets so a later retry can succeed. */
export function loadSoundFont(token: string | null): Promise<Uint8Array> {
  if (cached) return cached;
  const url = soundfontUrl();
  const p = (async () => {
    // Cache API is keyed by URL (token-independent) — once fetched, no more auth. Its
    // `add()` can't set the Authorization header, so we fetch with the bearer ourselves
    // and `put()` the response.
    if (typeof caches !== "undefined") {
      try {
        const cache = await caches.open(CACHE);
        const hit = await cache.match(url);
        if (hit) return new Uint8Array(await hit.arrayBuffer());
        const resp = await authedFetch(url, token);
        if (!resp.ok) throw new Error(`soundfont ${resp.status}`);
        await cache.put(url, resp.clone());
        return new Uint8Array(await resp.arrayBuffer());
      } catch (e) {
        // A real HTTP failure propagates; a Cache-API fault (private mode, quota) falls
        // through to a plain fetch.
        if (e instanceof Error && e.message.startsWith("soundfont ")) throw e;
      }
    }
    const resp = await authedFetch(url, token);
    if (!resp.ok) throw new Error(`soundfont ${resp.status}`);
    return new Uint8Array(await resp.arrayBuffer());
  })();
  cached = p;
  // Don't cache a failure permanently — reset so a retry (after re-auth) can work. A
  // success stays cached for the tab's life (never unloaded).
  p.catch(() => {
    if (cached === p) cached = null;
  });
  return p;
}

function authedFetch(url: string, token: string | null): Promise<Response> {
  const headers: Record<string, string> = {};
  if (token) headers.Authorization = `Bearer ${token}`;
  return fetch(url, { headers });
}

/** Test seam: inject SoundFont bytes (or reset with `null`). */
export function setSoundFontForTest(bytes: Uint8Array | null): void {
  cached = bytes ? Promise.resolve(bytes) : null;
}
