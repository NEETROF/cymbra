// Book sections served by the extension's own service worker, on Chromium (add-lingua-reader,
// D3 as amended). foliate-js loads each section into an iframe from a `blob:` URL, and the reader
// page reads that iframe's document to mount the reading module on it. Chrome's migration to
// "block the V8 optimizer on unfamiliar sites" (`MigrateToBlockV8OptimizerOnUnfamiliarSites`)
// puts such a blob document in a process of its own: its origin is still the extension's, but
// the page can no longer reach it (`contentDocument` is null) and foliate stops before painting.
//
// A section loaded from a plain extension URL stays in the page's process. So on Chromium the
// section's content — exactly what foliate made, resources already rewritten — is put in Cache
// Storage, and the frame is pointed at `chrome-extension://<id>/reader-section/…`, which the
// background service worker answers from that cache. Anything else the worker is asked is not
// its business. Where no worker controls the page, foliate's blob URL is used as before.

/** Where a section is served under the extension's origin. */
export const SECTION_PATH = "reader-section/";

/** The Cache Storage the sections are kept in while a book is open. */
export const SECTION_CACHE = "cymbra-lingua-sections";

/**
 * The policy a section is served with: none of the book's own scripts ever runs, whatever the
 * extension pages' policy — a book is content, and its frame shares the extension's origin.
 */
export const SECTION_CSP = "script-src 'none'; object-src 'none'";

/** The address the page asks to learn whether the worker serves sections at all. */
export const SECTION_PROBE = "reader-section/probe";

/** The header the worker marks its answer to the probe with. */
const SERVED_HEADER = "x-cymbra-lingua-sections";

/** Cache Storage refuses `chrome-extension:` URLs as keys: a section is kept under this origin. */
const KEY_ORIGIN = "https://lingua.invalid/";

/** The cache key of a section path (`reader-section/<token>/<index>`). */
export function sectionKey(path: string): string {
  return `${KEY_ORIGIN}${path.replace(/^\/+/, "")}`;
}

/** The part of a worker's fetch event this module uses. */
export interface SectionFetchEvent {
  request: { url: string };
  respondWith(response: Promise<Response>): void;
}

/**
 * The worker's side: answer a request for a section from the cache. Returns whether the request
 * was a section's; any other request is left to the browser, untouched.
 */
export function serveSection(event: SectionFetchEvent, caches: CacheStorage): boolean {
  const url = new URL(event.request.url);
  if (!url.pathname.startsWith(`/${SECTION_PATH}`)) return false;
  if (url.pathname === `/${SECTION_PROBE}`) {
    event.respondWith(Promise.resolve(new Response(null, { status: 204, headers: { [SERVED_HEADER]: "1" } })));
    return true;
  }
  event.respondWith(
    caches
      .open(SECTION_CACHE)
      .then((cache) => cache.match(sectionKey(url.pathname)))
      .then((found) => found ?? new Response("", { status: 404 })),
  );
  return true;
}

/** A foliate-js section, as far as loading goes. */
export interface LoadableSection {
  load?: () => Promise<string> | string;
  unload?: () => void;
}

/** What routing a book's sections through the worker needs of the reader page. */
export interface SectionHost {
  caches: CacheStorage;
  /** Read a URL the page made (foliate's blob URL). */
  fetch: (url: string) => Promise<Response>;
  /** The page's URL for an extension path (`chrome.runtime.getURL`). */
  pageUrl: (path: string) => string;
  /** Whether the worker answers for sections (`workerServesSections`), asked once per book. */
  served: () => Promise<boolean>;
  /** Unique to this opening of a book, so two reader tabs never share a section. */
  token: string;
}

/**
 * The reader page's side: from now on, each section of `sections` loads from the worker. Its
 * content is what foliate's own `load` produced; `unload` also drops it from the cache.
 */
export function routeSections(sections: LoadableSection[], host: SectionHost): void {
  sections.forEach((section, index) => {
    const load = section.load?.bind(section);
    if (!load) return;
    const unload = section.unload?.bind(section);
    const path = `${SECTION_PATH}${host.token}/${index}`;
    section.load = async () => {
      const blobUrl = await load();
      if (!(await host.served())) return blobUrl;
      try {
        const blob = await (await host.fetch(blobUrl)).blob();
        const cache = await host.caches.open(SECTION_CACHE);
        const headers = {
          "content-type": blob.type || "application/xhtml+xml",
          "content-security-policy": SECTION_CSP,
        };
        await cache.put(sectionKey(path), new Response(blob, { headers }));
        return host.pageUrl(path);
      } catch {
        return blobUrl; // the worker's route failed: foliate's own URL, which works where no V8 policy splits it
      }
    };
    section.unload = () => {
      unload?.();
      void host.caches
        .open(SECTION_CACHE)
        .then((cache) => cache.delete(sectionKey(path)))
        .catch(() => {});
    };
  });
}

/**
 * Whether a worker answers for sections, asked of the worker itself: a page can be controlled by
 * an older worker without the route — Chrome keeps an unpacked extension's worker across a
 * restart when its version did not change — and a frame pointed at an address nobody serves
 * shows an error page. Without the answer, sections keep foliate's blob URL. Never throws.
 */
export async function workerServesSections(
  fetchUrl: (url: string) => Promise<Response>,
  pageUrl: (path: string) => string,
): Promise<boolean> {
  try {
    return (await fetchUrl(pageUrl(SECTION_PROBE))).headers.get(SERVED_HEADER) === "1";
  } catch {
    return false;
  }
}

/** Forget every section an earlier reading left behind (a closed tab, a crash). Never throws. */
export async function clearSections(caches: CacheStorage): Promise<void> {
  await caches.delete(SECTION_CACHE).catch(() => false);
}
