// @vitest-environment node
// Node's own streams, DecompressionStream and WebCrypto: the download runs in a worker, not a page.
import { createHash } from "node:crypto";
import { gzipSync } from "node:zlib";
import { describe, expect, it, vi } from "vitest";
import type { ModelDb } from "@/translate/host/model-db.ts";
import { downloadModel, PROGRESS_EVERY_MS, sha256Hex } from "@/translate/host/model-download.ts";
import type { ModelManifest } from "@/translate/host/model-manifest.ts";

// The model download (add-lingua-translation-delivery D3, D4), against fixtures: real gzip, real
// sha256. A file is kept only when its DECOMPRESSED bytes have the pinned hash — which is also
// what catches the static host's trap, an unknown path answering 200 with the home page.

const bytes = (text: string) => new TextEncoder().encode(text);
const hex = (b: Uint8Array) => createHash("sha256").update(b).digest("hex");

const FILES = {
  model: bytes("model weights ".repeat(200)),
  lex: bytes("shortlist ".repeat(50)),
  vocab: bytes("vocab"),
};
const GZ = { model: gzipSync(FILES.model), lex: gzipSync(FILES.lex), vocab: gzipSync(FILES.vocab) };

const manifest: ModelManifest = {
  version: "en-fr/base-memory/2.0",
  base: "https://models.example/",
  files: {
    model: { path: `m/${hex(FILES.model)}/model.bin.gz`, size: GZ.model.byteLength, sha256: hex(FILES.model) },
    lex: { path: `m/${hex(FILES.lex)}/lex.bin.gz`, size: GZ.lex.byteLength, sha256: hex(FILES.lex) },
    vocab: { path: `m/${hex(FILES.vocab)}/vocab.spm.gz`, size: GZ.vocab.byteLength, sha256: hex(FILES.vocab) },
  },
};
const TOTAL = GZ.model.byteLength + GZ.lex.byteLength + GZ.vocab.byteLength;

function memoryDb(
  seed: Record<string, Uint8Array> = {},
): ModelDb & { files: Map<string, Uint8Array>; completed: string[] } {
  const files = new Map(Object.entries(seed));
  const completed: string[] = [];
  return {
    files,
    completed,
    has: async (sha) => files.has(sha),
    get: async (sha) => files.get(sha) ?? null,
    put: async (sha, b) => void files.set(sha, b),
    markComplete: async (m) => void completed.push(m.version),
    complete: async () => completed.length > 0,
    erase: async () => files.clear(),
  };
}

/** A host serving `routes` (path → body), each body in `chunks` pieces. */
function host(routes: Record<string, Uint8Array | string | number>, chunks = 1) {
  const calls: Array<{ url: string; init: RequestInit | undefined }> = [];
  const fetch = vi.fn(async (input: RequestInfo | URL, init?: RequestInit) => {
    const url = String(input);
    calls.push({ url, init });
    const path = url.slice(manifest.base.length);
    const route = routes[path];
    if (route === undefined || typeof route === "number")
      return new Response("nope", { status: (route as number) ?? 404 });
    const body = typeof route === "string" ? bytes(route) : route;
    const size = Math.ceil(body.byteLength / chunks);
    const stream = new ReadableStream<Uint8Array>({
      start(c) {
        for (let at = 0; at < body.byteLength; at += size) c.enqueue(body.slice(at, at + size));
        c.close();
      },
    });
    return new Response(stream, { status: 200 });
  });
  return { fetch: fetch as unknown as typeof globalThis.fetch, calls };
}

const serving = (over: Partial<Record<keyof typeof GZ, Uint8Array | string | number>> = {}) => ({
  [manifest.files.model.path]: over.model ?? GZ.model,
  [manifest.files.lex.path]: over.lex ?? GZ.lex,
  [manifest.files.vocab.path]: over.vocab ?? GZ.vocab,
});

function deps(
  fetch: typeof globalThis.fetch,
  db = memoryDb(),
  over: { signal?: AbortSignal; now?: () => number } = {},
) {
  const progress: Array<[number, number]> = [];
  return {
    db,
    progress,
    deps: {
      fetch,
      db,
      digest: sha256Hex,
      gunzip: () => new DecompressionStream("gzip") as unknown as TransformStream<Uint8Array, Uint8Array>,
      onProgress: (r: number, t: number) => progress.push([r, t]),
      ...over,
    },
  };
}

describe("downloadModel", () => {
  it("fetches, decompresses, verifies and stores every file, then records the model complete", async () => {
    const { fetch } = host(serving());
    const { deps: d, db } = deps(fetch);
    await expect(downloadModel(manifest, d)).resolves.toEqual({ ok: true });
    expect([...db.files.keys()].sort()).toEqual(
      Object.values(manifest.files)
        .map((f) => f.sha256)
        .sort(),
    );
    expect(new TextDecoder().decode(db.files.get(manifest.files.vocab.sha256))).toBe("vocab");
    expect(db.completed).toEqual(["en-fr/base-memory/2.0"]);
  });

  it("carries nothing of the reader's: no cookie, no referrer, no cache copy", async () => {
    const { fetch, calls } = host(serving());
    await downloadModel(manifest, deps(fetch).deps);
    expect(calls.map((c) => c.url)).toEqual(Object.values(manifest.files).map((f) => `${manifest.base}${f.path}`));
    for (const { url, init } of calls) {
      expect(url).not.toMatch(/[?#]/); // a fixed address: no query, no identifier
      expect(init).toMatchObject({ credentials: "omit", referrerPolicy: "no-referrer", cache: "no-store" });
    }
  });

  it("discards a 200 answer that is the site's home page, not the model", async () => {
    // Cymbra's static site answers an unknown path with 200 and the French home page.
    const { fetch } = host(serving({ lex: "<!doctype html><html lang=fr><title>Cymbra</title></html>" }));
    const { deps: d, db } = deps(fetch);
    await expect(downloadModel(manifest, d)).resolves.toMatchObject({ ok: false, reason: "not-the-model" });
    expect(db.files.has(manifest.files.lex.sha256)).toBe(false);
    expect(db.completed).toEqual([]);
  });

  it("discards a gzip file whose bytes are not the pinned model", async () => {
    const { fetch } = host(serving({ model: gzipSync(bytes("some other model")) }));
    const { deps: d, db } = deps(fetch);
    await expect(downloadModel(manifest, d)).resolves.toMatchObject({ ok: false, reason: "not-the-model" });
    expect(db.files.size).toBe(0);
  });

  it("discards a file that claims to be gzip and is not readable", async () => {
    const broken = new Uint8Array([0x1f, 0x8b, 1, 2, 3, 4, 5, 6]);
    const { fetch } = host(serving({ vocab: broken }));
    await expect(downloadModel(manifest, deps(fetch).deps)).resolves.toMatchObject({
      ok: false,
      reason: "not-the-model",
    });
  });

  it("accepts a host that decompresses on the way: the hash decides, not the encoding", async () => {
    const { fetch } = host(serving({ model: FILES.model, lex: FILES.lex, vocab: FILES.vocab }));
    const { deps: d, progress } = deps(fetch);
    await expect(downloadModel(manifest, d)).resolves.toEqual({ ok: true });
    expect(progress.every(([r]) => r <= TOTAL)).toBe(true); // more bytes than stated, bar still ends
  });

  it("reports a missing file as the host being unavailable", async () => {
    const { fetch } = host(serving({ lex: 404 }));
    await expect(downloadModel(manifest, deps(fetch).deps)).resolves.toMatchObject({
      ok: false,
      reason: "unavailable",
    });
  });

  it("reports a dropped connection as the network", async () => {
    const fetch = vi.fn(async () => {
      throw new TypeError("Failed to fetch");
    }) as unknown as typeof globalThis.fetch;
    await expect(downloadModel(manifest, deps(fetch).deps)).resolves.toMatchObject({ ok: false, reason: "network" });
  });

  it("blames the host, not the connection, when the device is online — an unknown name, a host down", async () => {
    // models.cymbra.app not deployed yet answered ENOTFOUND: "pas de connexion" would have been false.
    const fetch = vi.fn(async () => {
      throw new TypeError("Failed to fetch");
    }) as unknown as typeof globalThis.fetch;
    const { deps: d } = deps(fetch);
    await expect(downloadModel(manifest, { ...d, online: () => true })).resolves.toMatchObject({
      ok: false,
      reason: "unavailable",
    });
    await expect(downloadModel(manifest, { ...d, online: () => false })).resolves.toMatchObject({
      ok: false,
      reason: "network",
    });
  });

  it("reports a body cut mid-way as the network", async () => {
    const fetch = vi.fn(
      async () =>
        new Response(
          new ReadableStream({
            pull: (c) => c.error(new TypeError("network error")),
          }),
        ),
    ) as unknown as typeof globalThis.fetch;
    await expect(downloadModel(manifest, deps(fetch).deps)).resolves.toMatchObject({ ok: false, reason: "network" });
  });

  it("reports a full disk as storage", async () => {
    const { fetch } = host(serving());
    const db = memoryDb();
    db.put = async () => {
      throw new DOMException("quota", "QuotaExceededError");
    };
    await expect(downloadModel(manifest, deps(fetch, db).deps)).resolves.toMatchObject({
      ok: false,
      reason: "storage",
    });
  });

  it("reports a failure to record completion as storage", async () => {
    const { fetch } = host(serving());
    const db = memoryDb();
    db.markComplete = async () => {
      throw new Error("aborted");
    };
    await expect(downloadModel(manifest, deps(fetch, db).deps)).resolves.toMatchObject({
      ok: false,
      reason: "storage",
    });
  });

  it("reports a cancellation as cancelled", async () => {
    const controller = new AbortController();
    const fetch = vi.fn(async () => {
      controller.abort();
      throw new DOMException("aborted", "AbortError");
    }) as unknown as typeof globalThis.fetch;
    await expect(
      downloadModel(manifest, deps(fetch, memoryDb(), { signal: controller.signal }).deps),
    ).resolves.toMatchObject({
      ok: false,
      reason: "cancelled",
    });
  });

  it("resumes: files already verified are not fetched again", async () => {
    const { fetch, calls } = host(serving());
    const db = memoryDb({ [manifest.files.model.sha256]: FILES.model });
    const { deps: d, progress } = deps(fetch, db);
    await expect(downloadModel(manifest, d)).resolves.toEqual({ ok: true });
    expect(calls.map((c) => c.url)).toEqual([
      `${manifest.base}${manifest.files.lex.path}`,
      `${manifest.base}${manifest.files.vocab.path}`,
    ]);
    // The stored file counts as received from the start.
    expect(progress[1]).toEqual([GZ.model.byteLength, TOTAL]);
  });

  it("reports progress from 0 to the total, at most every PROGRESS_EVERY_MS in between", async () => {
    let clock = 0;
    const { fetch } = host(serving(), 10);
    const { deps: d, progress } = deps(fetch, memoryDb(), { now: () => (clock += 100) });
    await downloadModel(manifest, d);
    expect(progress[0]).toEqual([0, TOTAL]);
    expect(progress.at(-1)).toEqual([TOTAL, TOTAL]);
    for (let i = 1; i < progress.length; i++) expect(progress[i]![0]).toBeGreaterThanOrEqual(progress[i - 1]![0]);
    // 30 chunks at 100 ms each would be 30 reports; throttled, far fewer.
    expect(progress.length).toBeLessThan(30);
    expect(PROGRESS_EVERY_MS).toBeGreaterThan(100);
  });
});

describe("sha256Hex", () => {
  it("is the lowercase hex sha256", async () => {
    expect(await sha256Hex(bytes("abc"))).toBe("ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad");
  });
});
