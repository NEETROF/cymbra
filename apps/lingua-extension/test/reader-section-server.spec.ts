import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import {
  clearSections,
  type LoadableSection,
  routeSections,
  SECTION_CACHE,
  SECTION_CSP,
  SECTION_PROBE,
  sectionKey,
  type SectionFetchEvent,
  type SectionHost,
  serveSection,
  workerServesSections,
} from "@/reader/section-server.ts";
import { useNodeBlob } from "./epub-fixtures.ts";

// On Chromium the reader's sections come from the service worker, not from blob: documents a V8
// policy experiment may isolate in another process (add-lingua-reader, D3 as amended).

/** A Cache Storage in memory, keyed like the real one. */
function fakeCaches(): CacheStorage & { stores: Map<string, Map<string, Response>> } {
  const stores = new Map<string, Map<string, Response>>();
  const open = async (name: string) => {
    if (!stores.has(name)) stores.set(name, new Map());
    const store = stores.get(name)!;
    return {
      put: async (key: string, res: Response) => void store.set(key, res),
      match: async (key: string) => store.get(key)?.clone(),
      delete: async (key: string) => store.delete(key),
    } as unknown as Cache;
  };
  return {
    stores,
    open,
    delete: async (name: string) => stores.delete(name),
  } as unknown as CacheStorage & { stores: Map<string, Map<string, Response>> };
}

function event(url: string): SectionFetchEvent & { answer: Promise<Response> | null } {
  const e = {
    request: { url },
    answer: null as Promise<Response> | null,
    respondWith(r: Promise<Response>) {
      e.answer = r;
    },
  };
  return e;
}

const XHTML = `<?xml version="1.0"?><html xmlns="http://www.w3.org/1999/xhtml"><body><p>Hi</p></body></html>`;

let caches: ReturnType<typeof fakeCaches>;
let blobs: Map<string, Blob>;
let host: SectionHost;

beforeEach(() => {
  useNodeBlob();
  caches = fakeCaches();
  blobs = new Map([["blob:ext/1", new Blob([XHTML], { type: "application/xhtml+xml" })]]);
  host = {
    caches,
    fetch: async (url) => new Response(blobs.get(url)),
    pageUrl: (path) => `chrome-extension://id/${path}`,
    served: async () => true,
    token: "t0k",
  };
});

afterEach(() => vi.unstubAllGlobals());

describe("the worker's side", () => {
  it("answers a section from the cache, and leaves every other request alone", async () => {
    const cache = await caches.open(SECTION_CACHE);
    await cache.put(sectionKey("reader-section/t0k/3"), new Response("section 3"));
    const hit = event("chrome-extension://id/reader-section/t0k/3");
    expect(serveSection(hit, caches)).toBe(true);
    expect(await (await hit.answer!).text()).toBe("section 3");
    const other = event("chrome-extension://id/reader.js");
    expect(serveSection(other, caches)).toBe(false);
    expect(other.answer).toBeNull();
  });

  it("says a section it does not hold is not found", async () => {
    const miss = event("chrome-extension://id/reader-section/t0k/9");
    serveSection(miss, caches);
    expect((await miss.answer!).status).toBe(404);
  });
});

describe("asking whether the worker serves sections", () => {
  it("answers the probe itself, marked", async () => {
    const probe = event(`chrome-extension://id/${SECTION_PROBE}`);
    expect(serveSection(probe, caches)).toBe(true);
    const answer = await probe.answer!;
    expect(answer.status).toBe(204);
    const fetchUrl = async () => answer;
    expect(await workerServesSections(fetchUrl, (p) => p)).toBe(true);
  });

  it("says no to an unmarked answer, and to no answer at all (an older worker, no worker)", async () => {
    expect(
      await workerServesSections(
        async () => new Response("file"),
        (p) => p,
      ),
    ).toBe(false);
    expect(
      await workerServesSections(
        async () => Promise.reject(new Error("ERR_FILE_NOT_FOUND")),
        (p) => p,
      ),
    ).toBe(false);
  });
});

describe("the reader page's side", () => {
  it("serves each section from the worker, as foliate made it, with no script of the book's", async () => {
    const section: LoadableSection = { load: async () => "blob:ext/1", unload: vi.fn() };
    routeSections([section], host);
    expect(await section.load!()).toBe("chrome-extension://id/reader-section/t0k/0");
    const kept = await (await caches.open(SECTION_CACHE)).match(sectionKey("reader-section/t0k/0"));
    expect(kept?.headers.get("content-type")).toBe("application/xhtml+xml");
    expect(kept?.headers.get("content-security-policy")).toBe(SECTION_CSP);
    expect(await kept?.text()).toBe(XHTML);
  });

  it("drops a section from the cache when foliate unloads it", async () => {
    const unload = vi.fn();
    const section: LoadableSection = { load: async () => "blob:ext/1", unload };
    routeSections([section], host);
    await section.load!();
    section.unload!();
    expect(unload).toHaveBeenCalledOnce();
    await vi.waitFor(async () =>
      expect(await (await caches.open(SECTION_CACHE)).match(sectionKey("reader-section/t0k/0"))).toBeUndefined(),
    );
  });

  it("keeps foliate's blob URL where no worker serves sections, or the route fails", async () => {
    const a: LoadableSection = { load: async () => "blob:ext/1" };
    routeSections([a], { ...host, served: async () => false });
    expect(await a.load!()).toBe("blob:ext/1");
    const b: LoadableSection = { load: async () => "blob:ext/1" };
    routeSections([b], { ...host, fetch: async () => Promise.reject(new Error("revoked")) });
    expect(await b.load!()).toBe("blob:ext/1");
  });

  it("names each opening's sections apart, and skips a section with nothing to load", async () => {
    const first: LoadableSection = { load: () => "blob:ext/1" };
    const second: LoadableSection = { load: () => "blob:ext/1" };
    routeSections([{}, first], host);
    routeSections([second], { ...host, token: "other" });
    expect(await first.load!()).toBe("chrome-extension://id/reader-section/t0k/1");
    expect(await second.load!()).toBe("chrome-extension://id/reader-section/other/0");
  });

  it("clears what an earlier reading left, and never throws doing it", async () => {
    await (await caches.open(SECTION_CACHE)).put(sectionKey("reader-section/old/0"), new Response("x"));
    await clearSections(caches);
    expect(caches.stores.has(SECTION_CACHE)).toBe(false);
    await expect(
      clearSections({ delete: async () => Promise.reject(new Error("no")) } as unknown as CacheStorage),
    ).resolves.toBeUndefined();
  });

  it("keys a section under an origin Cache Storage accepts", () => {
    expect(sectionKey("/reader-section/a/1")).toBe("https://lingua.invalid/reader-section/a/1");
  });
});
