// Downloading the model (add-lingua-translation-delivery D3, D4). It runs in a worker of the
// engine's host — the offscreen document on Chromium, the event page on Firefox — because that is
// where the bytes will be read, and because a Chromium service worker can be stopped in the middle
// of a 25 MB transfer.
//
// Each file is fetched from its fixed address, decompressed, and kept only if the sha256 of its
// decompressed bytes is the one the package pins. That check is the whole trust story — the host
// only serves bytes — and it also catches a known trap of Cymbra's static hosting: an unknown path
// answers 200 with the French home page, not 404. Files already stored are verified by
// construction (model-db.ts), so a resumed download fetches only what is missing.
//
// The request carries nothing of the reader's: no cookie, no referrer, no identifier, no page text
// (lingua-privacy: The model download carries nothing of the reader's).

import type { ModelFailure } from "../setting.ts";
import type { ModelDb } from "./model-db.ts";
import { fileUrl, MODEL_ROLES, type ModelManifest, totalSize } from "./model-manifest.ts";

/** Why a download stopped: the reader-facing reasons, or a cancellation nobody reports. */
export type DownloadFailure = ModelFailure | "cancelled";

export type DownloadOutcome = { ok: true } | { ok: false; reason: DownloadFailure; detail: string };

export interface DownloadDeps {
  fetch: typeof fetch;
  db: ModelDb;
  /** sha256 of `bytes`, as lowercase hex. */
  digest(bytes: Uint8Array): Promise<string>;
  /** A gzip decoder: DecompressionStream("gzip") in the browser. */
  gunzip(): TransformStream<Uint8Array, Uint8Array>;
  signal?: AbortSignal;
  /** Bytes received so far, of the total the manifest states. At most every PROGRESS_EVERY_MS. */
  onProgress(received: number, total: number): void;
  now?: () => number;
  /**
   * Whether the device has a network (navigator.onLine). A request that fails while it does is the
   * HOST's failure — an unknown name, a host down, a refused CORS — not the reader's connection.
   */
  online?: () => boolean;
}

/** How often progress is reported at most: every report crosses a thread and lands in storage. */
export const PROGRESS_EVERY_MS = 500;

const GZIP_MAGIC = [0x1f, 0x8b];

class Failure extends Error {
  constructor(
    readonly reason: DownloadFailure,
    detail: string,
  ) {
    super(detail);
  }
}

export async function sha256Hex(bytes: Uint8Array): Promise<string> {
  const digest = await crypto.subtle.digest("SHA-256", bytes);
  return [...new Uint8Array(digest)].map((b) => b.toString(16).padStart(2, "0")).join("");
}

function concat(chunks: Uint8Array[], length: number): Uint8Array {
  const out = new Uint8Array(length);
  let at = 0;
  for (const chunk of chunks) {
    out.set(chunk, at);
    at += chunk.byteLength;
  }
  return out;
}

async function readAll(stream: ReadableStream<Uint8Array>, onChunk?: (n: number) => void): Promise<Uint8Array> {
  const reader = stream.getReader();
  const chunks: Uint8Array[] = [];
  let length = 0;
  for (;;) {
    const { done, value } = await reader.read();
    if (done) break;
    chunks.push(value);
    length += value.byteLength;
    onChunk?.(value.byteLength);
  }
  return concat(chunks, length);
}

function isGzip(bytes: Uint8Array): boolean {
  return bytes[0] === GZIP_MAGIC[0] && bytes[1] === GZIP_MAGIC[1];
}

/**
 * The file's decompressed bytes. A body that is not gzip is taken as it came: a host that
 * decompresses on the way (Content-Encoding) hands over the model itself, and anything else —
 * an HTML page answering 200 — fails the hash check that follows, which is where it belongs.
 */
async function decompressed(body: Uint8Array, deps: DownloadDeps): Promise<Uint8Array> {
  if (!isGzip(body)) return body;
  try {
    return await readAll(new Blob([body]).stream().pipeThrough(deps.gunzip()));
  } catch (e) {
    throw new Failure("not-the-model", `not a readable gzip file: ${String(e)}`);
  }
}

function classify(e: unknown, deps: DownloadDeps): Failure {
  if (e instanceof Failure) return e;
  if (deps.signal?.aborted || (e as { name?: string } | null)?.name === "AbortError") {
    return new Failure("cancelled", "cancelled");
  }
  return new Failure(deps.online?.() === true ? "unavailable" : "network", String(e));
}

/**
 * Fetch every file of `manifest` that the database lacks, verify it, store it, then record the
 * model complete. Nothing unverified is ever stored, so a failure leaves only whole, checked files.
 */
export async function downloadModel(manifest: ModelManifest, deps: DownloadDeps): Promise<DownloadOutcome> {
  const now = deps.now ?? Date.now;
  const total = totalSize(manifest);
  let received = 0;
  let reportedAt = -Infinity;
  const report = (force: boolean): void => {
    if (!force && now() - reportedAt < PROGRESS_EVERY_MS) return;
    reportedAt = now();
    deps.onProgress(Math.min(received, total), total);
  };

  try {
    report(true);
    for (const role of MODEL_ROLES) {
      const file = manifest.files[role];
      if (await deps.db.has(file.sha256)) {
        received += file.size;
        report(true);
        continue;
      }
      const startedAt = received;
      let response: Response;
      try {
        response = await deps.fetch(fileUrl(manifest, file), {
          credentials: "omit",
          referrerPolicy: "no-referrer",
          cache: "no-store",
          signal: deps.signal,
        });
      } catch (e) {
        throw classify(e, deps);
      }
      if (!response.ok || !response.body) {
        throw new Failure("unavailable", `${role}: HTTP ${response.status}`);
      }
      let body: Uint8Array;
      try {
        body = await readAll(response.body, (n) => {
          // A host that decompresses on the way sends more bytes than the manifest says; the
          // bar must still end at this file's share.
          received = Math.min(received + n, startedAt + file.size);
          report(false);
        });
      } catch (e) {
        throw classify(e, deps);
      }
      const bytes = await decompressed(body, deps);
      if ((await deps.digest(bytes)) !== file.sha256) {
        throw new Failure("not-the-model", `${role}: sha256 differs from the pinned model`);
      }
      try {
        await deps.db.put(file.sha256, bytes);
      } catch (e) {
        throw new Failure("storage", `${role}: ${String(e)}`);
      }
      received = startedAt + file.size;
      report(true);
    }
    try {
      await deps.db.markComplete(manifest);
    } catch (e) {
      throw new Failure("storage", `complete: ${String(e)}`);
    }
    return { ok: true };
  } catch (e) {
    const failure = e instanceof Failure ? e : new Failure("unknown", String(e));
    return { ok: false, reason: failure.reason, detail: failure.message };
  }
}
