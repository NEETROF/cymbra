import { EngineChannel, type WorkerLike } from "../translate/host/channel.ts";
import { DownloadHost, type DownloadEvent, type DownloadWorkerLike } from "../translate/host/downloads.ts";

// The Cymbra Lingua app's two workers (add-lingua-review-translation-fallback D7): the extension's own
// engine worker and model worker, unchanged — the same files the Safari extension runs, served from
// the extension the app embeds (AppPageResource) under the page's origin, so the IndexedDB they read
// and fill is the page's. The engine never runs on the page's thread: this is the one module of the
// page that constructs a worker (test/lint-translator-placement.spec.ts). Excluded from coverage:
// browser glue over the channel and the download host, which are tested on their own.

/** The engine's channel: started on the first translation, put down when reading stops. */
export function appEngine(): EngineChannel {
  return new EngineChannel(() => new Worker("engine-worker.js") as unknown as WorkerLike);
}

/** Download `models` in a worker of its own, which lives only while it runs. */
export function appDownload(models: string[], report: (event: DownloadEvent) => void): void {
  new DownloadHost(() => new Worker("model-worker.js") as unknown as DownloadWorkerLike, report).start(models);
}
