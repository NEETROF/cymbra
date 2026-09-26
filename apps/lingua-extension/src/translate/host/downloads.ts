// The model download, seen from the engine's host that owns it — the offscreen document on
// Chromium, the event page on Firefox (add-lingua-translation-delivery D4). One download at a time,
// in a worker of its own that lives only while it runs: it reads no engine and holds no memory
// once the model is stored. Progress, completion and failure are reported to whoever asked; a
// cancellation is not, because the one who cancels already knows.

import type { DownloadFailure } from "./model-download.ts";

/** What the download worker is told, and what it says back. */
export type ModelWorkerRequest = { op: "download" };
export type ModelWorkerEvent =
  | { kind: "progress"; received: number; total: number }
  | { kind: "done" }
  | { kind: "failed"; reason: DownloadFailure; detail: string };

/** What the host reports: a failure the reader may be told about never carries the raw detail. */
export type DownloadEvent =
  | { kind: "progress"; received: number; total: number }
  | { kind: "done" }
  | { kind: "failed"; reason: Exclude<DownloadFailure, "cancelled"> };

export interface DownloadWorkerLike {
  postMessage(message: ModelWorkerRequest): void;
  terminate(): void;
  onmessage: ((event: { data: ModelWorkerEvent }) => void) | null;
  onerror: ((event: unknown) => void) | null;
}

export type DownloadLog = (message: string, detail?: unknown) => void;

const LOG: DownloadLog = (message, detail) => console.warn(`[Cymbra Lingua] ${message}`, detail ?? "");

export class DownloadHost {
  private worker: DownloadWorkerLike | null = null;

  constructor(
    private readonly spawn: () => DownloadWorkerLike,
    private readonly report: (event: DownloadEvent) => void,
    private readonly opts: { onEnded?: () => void; log?: DownloadLog } = {},
  ) {}

  running(): boolean {
    return this.worker !== null;
  }

  /** Start a download unless one is running. Whatever happens next is reported. */
  start(): void {
    if (this.worker) return;
    let worker: DownloadWorkerLike;
    try {
      worker = this.spawn();
    } catch (e) {
      this.fail("unknown", e);
      this.opts.onEnded?.();
      return;
    }
    this.worker = worker;
    worker.onmessage = ({ data }) => {
      if (this.worker !== worker) return; // cancelled: nothing it says counts any more
      if (data.kind === "progress") {
        this.report(data);
        return;
      }
      this.stop();
      if (data.kind === "done") this.report({ kind: "done" });
      else if (data.reason !== "cancelled") this.fail(data.reason, data.detail);
      this.opts.onEnded?.();
    };
    worker.onerror = (e) => {
      if (this.worker !== worker) return;
      this.stop();
      this.fail("unknown", e);
      this.opts.onEnded?.();
    };
    worker.postMessage({ op: "download" });
  }

  /** Stop the download in progress, if any. Nothing verified is lost; nothing is reported. */
  cancel(): void {
    if (this.stop()) this.opts.onEnded?.();
  }

  private fail(reason: Exclude<DownloadFailure, "cancelled">, detail: unknown): void {
    (this.opts.log ?? LOG)(`model download failed (${reason}):`, detail);
    this.report({ kind: "failed", reason });
  }

  /** Terminate the worker; whether there was one. */
  private stop(): boolean {
    const worker = this.worker;
    if (!worker) return false;
    this.worker = null;
    worker.terminate();
    return true;
  }
}
