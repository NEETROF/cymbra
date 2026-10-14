// THROWAWAY spike probe (task 1.4): a classic worker under lingua-app://app/ — importScripts, a
// fetch of the engine's wasm from the custom scheme, WebAssembly.compile, IndexedDB from a worker.
const report: Record<string, unknown> = { location: self.location.href };
(async () => {
  try {
    importScripts("probe-import.js");
    report.importScripts = (self as unknown as { __probeImported?: boolean }).__probeImported === true;
  } catch (e) {
    report.importScripts = `threw: ${String(e)}`;
  }
  try {
    const t = performance.now();
    const r = await fetch("engine/bergamot-translator.wasm");
    report.wasmFetch = `${r.status} ${r.headers.get("content-type")}`;
    const bytes = await r.arrayBuffer();
    await WebAssembly.compile(bytes);
    report.wasmCompile = `ok ${(bytes.byteLength / 1048576).toFixed(1)} MiB in ${(performance.now() - t).toFixed(0)} ms`;
  } catch (e) {
    report.wasmCompile = `failed: ${String(e)}`;
  }
  try {
    const req = indexedDB.open("spike-worker-probe", 1);
    await new Promise((res, rej) => {
      req.onupgradeneeded = () => req.result.createObjectStore("s");
      req.onsuccess = res;
      req.onerror = () => rej(req.error);
    });
    req.result.close();
    report.indexedDB = "ok";
  } catch (e) {
    report.indexedDB = `failed: ${String(e)}`;
  }
  (self as unknown as { postMessage(v: unknown): void }).postMessage(report);
})();
