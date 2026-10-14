// THROWAWAY spike (add-lingua-review-translation-fallback, task 1.4). Never merged.
//
// Runs, under the app's lingua-app://app/ origin: (1) IndexedDB fill + relaunch survival, (2) the
// real en-fr model download into `lingua-model` with the extension's own model worker, (3) the
// extension's own engine worker (classic, importScripts, Bergamot WASM) on that model, cold and
// warm, (4) cross-origin calls to see the Origin WebKit sends and whether CORS passes.
// Every line is shown on screen and posted to the native `spike` handler (NSLog "[spike] …").

import { createClient } from "@connectrpc/connect";
import { createGrpcWebTransport } from "@connectrpc/connect-web";
import { OnlineTranslationService } from "@/gen/online_translation_pb";
import { modelDb } from "../../translate/host/model-db.ts";
import { loadBundledCatalogue, routeOf } from "../../translate/host/model-manifest.ts";

const out = document.getElementById("out")!;
const params = new URLSearchParams(location.search);
const ECHO = params.get("echo") ?? "http://127.0.0.1:8765";
const FILL_MIB = Number(params.get("fill") ?? "64");
const PROD = params.get("api") ?? "https://api.cymbra.app";
const t0 = performance.now();

type Native = { webkit?: { messageHandlers?: Record<string, { postMessage(v: unknown): void }> } };

function log(cls: "ok" | "ko" | "info", key: string, value: unknown): void {
  const text = typeof value === "string" ? value : JSON.stringify(value);
  const line = `[${((performance.now() - t0) / 1000).toFixed(2)}s] ${key}: ${text}`;
  const span = document.createElement("span");
  span.className = cls;
  span.textContent = line + "\n";
  out.appendChild(span);
  try {
    (window as unknown as Native).webkit?.messageHandlers?.spike?.postMessage(`${cls.toUpperCase()} ${line}`);
  } catch {
    /* no native handler: a browser */
  }
}
const err = (e: unknown): string => (e instanceof Error ? `${e.name}: ${e.message}` : String(e));
const MiB = (n: number): string => `${(n / 1048576).toFixed(1)} MiB`;

// ---------------------------------------------------------------------------------------------
// 0. Environment
async function env(): Promise<void> {
  log("info", "origin", {
    href: location.href,
    origin: location.origin,
    isSecureContext,
    crossOriginIsolated: (self as { crossOriginIsolated?: boolean }).crossOriginIsolated,
    ua: navigator.userAgent,
    wasm: typeof WebAssembly,
    worker: typeof Worker,
    indexedDB: typeof indexedDB,
    storageApi: typeof navigator.storage,
    cpus: navigator.hardwareConcurrency,
  });
  await estimate("estimate.start");
  try {
    log("info", "storage.persisted", await navigator.storage?.persisted?.());
  } catch (e) {
    log("ko", "storage.persisted", err(e));
  }
  try {
    const granted = await navigator.storage?.persist?.();
    log(granted ? "ok" : "info", "storage.persist()", granted);
    log("info", "storage.persisted.after", await navigator.storage?.persisted?.());
  } catch (e) {
    log("ko", "storage.persist()", err(e));
  }
}

async function estimate(key: string): Promise<void> {
  try {
    const e = await navigator.storage?.estimate?.();
    log("info", key, e ? { usage: MiB(e.usage ?? 0), quota: MiB(e.quota ?? 0) } : "no navigator.storage.estimate");
  } catch (e) {
    log("ko", key, err(e));
  }
}

// ---------------------------------------------------------------------------------------------
// 1. IndexedDB fill and relaunch survival
const FILL_DB = "spike-fill";
const CHUNK = 8 * 1048576;

function openFill(): Promise<IDBDatabase> {
  return new Promise((resolve, reject) => {
    const req = indexedDB.open(FILL_DB, 1);
    req.onupgradeneeded = () => {
      req.result.createObjectStore("chunks");
      req.result.createObjectStore("meta");
    };
    req.onsuccess = () => resolve(req.result);
    req.onerror = () => reject(req.error);
  });
}
function idb<T>(r: IDBRequest<T>): Promise<T> {
  return new Promise((res, rej) => {
    r.onsuccess = () => res(r.result);
    r.onerror = () => rej(r.error);
  });
}
function txDone(tx: IDBTransaction): Promise<void> {
  return new Promise((res, rej) => {
    tx.oncomplete = () => res();
    tx.onabort = tx.onerror = () => rej(tx.error ?? new Error("tx aborted"));
  });
}
function chunk(index: number): Uint8Array {
  const a = new Uint8Array(CHUNK);
  for (let i = 0; i < CHUNK; i += 4096) a[i] = (index * 31 + i / 4096) & 0xff;
  a[CHUNK - 1] = index & 0xff;
  return a;
}

async function fillStatus(db: IDBDatabase): Promise<{ count: number; bytes: number; written?: string } | null> {
  const tx = db.transaction(["meta"], "readonly");
  return ((await idb(tx.objectStore("meta").get("status"))) as never) ?? null;
}

async function verifyFill(db: IDBDatabase, count: number): Promise<boolean> {
  for (const i of [0, count - 1]) {
    const tx = db.transaction(["chunks"], "readonly");
    const v = (await idb(tx.objectStore("chunks").get(i))) as Uint8Array | undefined;
    if (!v || v.byteLength !== CHUNK || v[CHUNK - 1] !== (i & 0xff) || v[4096] !== ((i * 31 + 1) & 0xff)) return false;
  }
  return true;
}

async function fill(mib: number): Promise<void> {
  const db = await openFill();
  try {
    const before = await fillStatus(db);
    let count = before?.count ?? 0;
    const target = count + Math.ceil((mib * 1048576) / CHUNK);
    const start = performance.now();
    for (; count < target; count++) {
      const tx = db.transaction(["chunks", "meta"], "readwrite");
      const done = txDone(tx);
      tx.objectStore("chunks").put(chunk(count), count);
      tx.objectStore("meta").put({ count: count + 1, bytes: (count + 1) * CHUNK, written: new Date().toISOString() }, "status");
      try {
        await done;
      } catch (e) {
        log("ko", "idb.fill", `failed at chunk ${count} (${MiB(count * CHUNK)} stored): ${err(e)}`);
        await estimate("estimate.after-failure");
        return;
      }
    }
    const ms = performance.now() - start;
    log("ok", "idb.fill", `stored ${MiB(count * CHUNK)} total (+${mib} MiB in ${ms.toFixed(0)} ms)`);
    await estimate("estimate.after-fill");
  } finally {
    db.close();
  }
}

async function idbPhase(): Promise<void> {
  try {
    const db = await openFill();
    const status = await fillStatus(db);
    if (status) {
      const ok = await verifyFill(db, status.count);
      log(ok ? "ok" : "ko", "idb.relaunch", {
        survived: true,
        verified: ok,
        stored: MiB(status.bytes),
        writtenAt: status.written,
      });
      db.close();
    } else {
      db.close();
      log("info", "idb.relaunch", "no fill from a previous launch: writing now (relaunch the app to check survival)");
      await fill(FILL_MIB);
    }
  } catch (e) {
    log("ko", "idb", err(e));
  }
}

// ---------------------------------------------------------------------------------------------
// 2. The real model download (extension's model worker) into `lingua-model`
const PAIR = "en-fr";

async function modelPhase(): Promise<boolean> {
  try {
    const catalogue = await loadBundledCatalogue();
    const route = routeOf(catalogue, PAIR);
    const total = route.reduce(
      (s, m) => s + m.files.model.size + m.files.lex.size + m.files.vocab.size,
      0,
    );
    const unpacked = route.reduce(
      (s, m) =>
        s +
        ((m.files.model as { unpacked?: number }).unpacked ?? 0) +
        ((m.files.lex as { unpacked?: number }).unpacked ?? 0) +
        ((m.files.vocab as { unpacked?: number }).unpacked ?? 0),
      0,
    );
    const db = modelDb();
    const complete = (await Promise.all(route.map((m) => db.complete(m)))).every(Boolean);
    if (complete) {
      log("ok", "model.stored", `en-fr already in lingua-model (from a previous launch): download ${MiB(total)}, stored ~${MiB(unpacked)}`);
      return true;
    }
    log("info", "model.download", `fetching ${route.map((m) => m.version).join(",")} (${MiB(total)} gz, ~${MiB(unpacked)} unpacked) from ${catalogue.base}`);
    const start = performance.now();
    const worker = new Worker("model-worker.js");
    const result = await new Promise<{ kind: string; reason?: string; detail?: string }>((resolve) => {
      let last = 0;
      worker.onerror = (e) => resolve({ kind: "failed", reason: "worker-error", detail: e.message || "error event" });
      worker.onmessage = (e: MessageEvent<{ kind: string; received?: number; total?: number }>) => {
        if (e.data.kind === "progress") {
          const now = performance.now();
          if (now - last > 3000) {
            last = now;
            log("info", "model.progress", `${MiB(e.data.received ?? 0)} / ${MiB(e.data.total ?? 0)}`);
          }
          return;
        }
        resolve(e.data);
      };
      worker.postMessage({ op: "download", models: route.map((m) => m.version) });
    });
    worker.terminate();
    const ms = performance.now() - start;
    if (result.kind === "done") {
      log("ok", "model.download", `done in ${(ms / 1000).toFixed(1)} s (download + gunzip + sha256 + IDB put)`);
      await estimate("estimate.after-model");
      return true;
    }
    log("ko", "model.download", result);
    return false;
  } catch (e) {
    log("ko", "model", err(e));
    return false;
  }
}

// ---------------------------------------------------------------------------------------------
// 3. A probe classic worker, then the engine worker
async function probeWorker(): Promise<void> {
  try {
    const w = new Worker("probe-worker.js");
    const r = await new Promise<unknown>((resolve) => {
      const timer = setTimeout(() => resolve({ timeout: true }), 15000);
      w.onerror = (e) => {
        clearTimeout(timer);
        resolve({ error: e.message || "error event (no message)", filename: e.filename });
      };
      w.onmessage = (e) => {
        clearTimeout(timer);
        resolve(e.data);
      };
    });
    w.terminate();
    const ok = typeof r === "object" && r !== null && (r as { importScripts?: boolean }).importScripts === true;
    log(ok ? "ok" : "ko", "worker.probe", r);
  } catch (e) {
    log("ko", "worker.probe", `new Worker threw: ${err(e)}`);
  }
}

type Resp = { id: number; ok: boolean; html?: string; error?: string };
let engine: Worker | null = null;
let nextId = 1;
function ask(w: Worker, req: Record<string, unknown>, timeoutMs = 120000): Promise<Resp> {
  const id = nextId++;
  return new Promise((resolve) => {
    const timer = setTimeout(() => resolve({ id, ok: false, error: "timeout" }), timeoutMs);
    const onMsg = (e: MessageEvent<Resp>) => {
      if (e.data.id !== id) return;
      clearTimeout(timer);
      w.removeEventListener("message", onMsg);
      resolve(e.data);
    };
    w.addEventListener("message", onMsg);
    w.onerror = (e) => {
      clearTimeout(timer);
      resolve({ id, ok: false, error: `worker error: ${e.message || "(no message)"} ${e.filename ?? ""}:${e.lineno ?? ""}` });
    };
    w.postMessage({ id, ...req });
  });
}

const SENTENCES = [
  "He finally decided to <b>give up</b> smoking after the doctor's warning.",
  "The rope began to <b>slacken</b> as the boat drifted towards the shore.",
  "She would <b>rather</b> stay at home than go to the party tonight.",
  "This phenomenon is <b>colloquially referred to as</b> a sugar rush.",
];

function blobEngine(src: string): Worker {
  const base = new URL(".", location.href).href;
  const prelude =
    `const __BASE=${JSON.stringify(base)};` +
    "const __is=self.importScripts.bind(self);self.importScripts=(...u)=>__is(...u.map(x=>new URL(x,__BASE).href));" +
    "const __f=self.fetch.bind(self);self.fetch=(i,o)=>__f(typeof i==='string'?new URL(i,__BASE).href:i,o);\n";
  return new Worker(URL.createObjectURL(new Blob([prelude, src], { type: "text/javascript" })));
}

async function enginePhase(kind: "classic" | "blob"): Promise<boolean> {
  try {
    const start = performance.now();
    engine = kind === "classic" ? new Worker("engine-worker.js") : blobEngine(await (await fetch("engine-worker.js")).text());
    const loaded = await ask(engine, { op: "load", pair: PAIR });
    const loadMs = performance.now() - start;
    if (!loaded.ok) {
      log("ko", `engine.${kind}.load`, loaded);
      engine.terminate();
      engine = null;
      return false;
    }
    log("ok", `engine.${kind}.load`, `cold: worker start + importScripts + wasm instantiate + model build from IDB = ${loadMs.toFixed(0)} ms`);
    for (const [i, markup] of SENTENCES.entries()) {
      const s = performance.now();
      const r = await ask(engine, { op: "translate", pair: PAIR, markup });
      const ms = performance.now() - s;
      log(r.ok ? "ok" : "ko", `engine.${kind}.translate[${i === 0 ? "first" : "warm"}]`, `${ms.toFixed(0)} ms  ${markup}  →  ${r.ok ? r.html : r.error}`);
    }
    return true;
  } catch (e) {
    log("ko", `engine.${kind}`, err(e));
    return false;
  }
}

async function memoryLoop(n: number): Promise<void> {
  if (!engine) {
    log("ko", "memory", "no engine worker running");
    return;
  }
  log("info", "memory.begin", `translating ${n} sentences — sample the WebContent process now`);
  const s = performance.now();
  for (let i = 0; i < n; i++) await ask(engine, { op: "translate", pair: PAIR, markup: SENTENCES[i % SENTENCES.length] });
  log("info", "memory.end", `${n} translations in ${(performance.now() - s).toFixed(0)} ms`);
}

// ---------------------------------------------------------------------------------------------
// 4. Origin and CORS
async function networkPhase(): Promise<void> {
  // a. A local echo server (tool: spike/echo_server.py) that prints the Origin it receives and
  //    answers CORS with that origin echoed — what a server listing the origin would do.
  for (const [label, init] of [
    ["simple GET", { method: "GET" }],
    [
      "gRPC-web-shaped POST (preflighted)",
      {
        method: "POST",
        headers: { "content-type": "application/grpc-web+proto", "x-grpc-web": "1", authorization: "Bearer spike" },
        body: new Uint8Array([0, 0, 0, 0, 0]),
      },
    ],
  ] as const) {
    try {
      const r = await fetch(`${ECHO}/echo`, init as RequestInit);
      log("ok", `net.echo ${label}`, { status: r.status, seenByServer: await r.json() });
    } catch (e) {
      log("ko", `net.echo ${label}`, `${err(e)} (echo at ${ECHO})`);
    }
  }
  // b. A public echo (httpbin), reachable from a real device.
  try {
    const r = await fetch("https://httpbin.org/anything", { method: "POST", headers: { "content-type": "application/grpc-web+proto" }, body: "x" });
    const j = (await r.json()) as { headers?: Record<string, string> };
    log("ok", "net.httpbin", { status: r.status, Origin: j.headers?.Origin ?? "(none)", acao: r.headers.get("access-control-allow-origin") });
  } catch (e) {
    log("ko", "net.httpbin", err(e));
  }
  // c. The real Lingua server, unauthenticated gRPC-web call.
  try {
    const client = createClient(OnlineTranslationService, createGrpcWebTransport({ baseUrl: PROD }));
    const res = await client.translate({} as never);
    log("ok", "net.prod", { unexpectedSuccess: JSON.stringify(res) });
  } catch (e) {
    const code = (e as { code?: number; rawMessage?: string }).code;
    log(code === 16 ? "ok" : "ko", "net.prod", `${err(e)} (connect code ${code}) — 16 = UNAUTHENTICATED reached the server; a network/CORS failure is code 14 / "Load failed"`);
  }
}

// ---------------------------------------------------------------------------------------------
async function all(): Promise<void> {
  out.textContent = "";
  await env();
  await idbPhase();
  const model = await modelPhase();
  await probeWorker();
  if (model && !(await enginePhase("classic"))) {
    log("info", "engine", "classic worker failed: trying a blob worker");
    await enginePhase("blob");
  }
  if (params.get("blob") === "1" && model) await enginePhase("blob");
  await networkPhase();
  await estimate("estimate.end");
  if (params.get("mem")) await memoryLoop(Number(params.get("mem")));
  log("info", "done", "all phases finished");
}

document.getElementById("rerun")!.onclick = () => void all();
document.getElementById("fill100")!.onclick = () => void fill(100);
document.getElementById("mem")!.onclick = () => void memoryLoop(20);
document.getElementById("wipe")!.onclick = () => {
  engine?.terminate();
  engine = null;
  indexedDB.deleteDatabase(FILL_DB);
  void modelDb()
    .erase()
    .then(() => log("info", "wipe", "spike-fill and lingua-model deleted"));
};
if (params.get("wipe") === "1") {
  indexedDB.deleteDatabase(FILL_DB);
  void modelDb()
    .erase()
    .then(() => log("info", "wipe", "spike-fill and lingua-model deleted"));
} else {
  void all();
}
