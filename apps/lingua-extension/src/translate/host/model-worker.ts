// The model download's own thread (add-lingua-translation-delivery D4): spawned by the engine's
// host when a download starts, terminated when it ends or is cancelled. It reads the models it is
// told from the package's catalogue (generalise-lingua-translation-model-state D4), fetches what the
// device lacks and stores it verified. The logic is model-download.ts; this only wires it to the
// browser.

import type { ModelWorkerEvent, ModelWorkerRequest } from "./downloads.ts";
import { modelDb } from "./model-db.ts";
import { downloadModels, sha256Hex } from "./model-download.ts";
import { loadBundledCatalogue, modelsById } from "./model-manifest.ts";

const scope = self as unknown as {
  onmessage: ((event: MessageEvent<ModelWorkerRequest>) => void) | null;
  postMessage(message: ModelWorkerEvent): void;
};

scope.onmessage = (event) => {
  if (event.data?.op !== "download") return;
  const ids = Array.isArray(event.data.models) ? event.data.models : [];
  void (async () => {
    try {
      const models = modelsById(await loadBundledCatalogue(), ids);
      const outcome = await downloadModels(models, {
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
