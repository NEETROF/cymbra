// The model download's own thread (add-lingua-translation-delivery D4): spawned by the engine's
// host when a download starts, terminated when it ends or is cancelled. It reads the package's
// manifest, fetches what the device lacks and stores it verified. The logic is model-download.ts;
// this only wires it to the browser.

import type { ModelWorkerEvent, ModelWorkerRequest } from "./downloads.ts";
import { modelDb } from "./model-db.ts";
import { downloadModel, sha256Hex } from "./model-download.ts";
import { loadBundledManifest } from "./model-manifest.ts";

const scope = self as unknown as {
  onmessage: ((event: MessageEvent<ModelWorkerRequest>) => void) | null;
  postMessage(message: ModelWorkerEvent): void;
};

scope.onmessage = (event) => {
  if (event.data?.op !== "download") return;
  void (async () => {
    try {
      const manifest = await loadBundledManifest();
      const outcome = await downloadModel(manifest, {
        fetch: (input, init) => fetch(input, init),
        db: modelDb(),
        digest: sha256Hex,
        gunzip: () => new DecompressionStream("gzip") as unknown as TransformStream<Uint8Array, Uint8Array>,
        onProgress: (received, total) => scope.postMessage({ kind: "progress", received, total }),
        online: () => navigator.onLine,
      });
      scope.postMessage(
        outcome.ok ? { kind: "done" } : { kind: "failed", reason: outcome.reason, detail: outcome.detail },
      );
    } catch (e: unknown) {
      scope.postMessage({ kind: "failed", reason: "unknown", detail: e instanceof Error ? e.message : String(e) });
    }
  })();
};
