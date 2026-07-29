// Fetches the piano SoundFont (the app's exact UprightPianoKW, CC0) served
// same-origin. It's ~57 MB, so it is fetched lazily (only when a moderator hits Play)
// and persisted in the Cache API so it downloads at most once across sessions. In
// memory it is also cached for the tab's lifetime. Injectable for unit tests.

const SF2_URL = "/soundfonts/UprightPianoKW-20220221.sf2";
const CACHE = "cymbra-soundfont-v1";

let cached: Promise<Uint8Array> | null = null;

/** Load (once) the SoundFont bytes, persisting in the Cache API when available. */
export function loadSoundFont(): Promise<Uint8Array> {
  if (cached) return cached;
  cached = (async () => {
    if (typeof caches !== "undefined") {
      try {
        const cache = await caches.open(CACHE);
        let resp = await cache.match(SF2_URL);
        if (!resp) {
          await cache.add(SF2_URL);
          resp = await cache.match(SF2_URL);
        }
        if (resp) return new Uint8Array(await resp.arrayBuffer());
      } catch {
        // Cache API unavailable/failed (private mode, quota) — fall back to fetch.
      }
    }
    const resp = await fetch(SF2_URL);
    if (!resp.ok) throw new Error(`soundfont ${resp.status}`);
    return new Uint8Array(await resp.arrayBuffer());
  })();
  return cached;
}

/** Test seam: inject SoundFont bytes (or reset with `null`). */
export function setSoundFontForTest(bytes: Uint8Array | null): void {
  cached = bytes ? Promise.resolve(bytes) : null;
}
