// The engine's own thread — the only place in the extension it ever runs.
//
// It is a CLASSIC worker on purpose. Mozilla's generated glue assumes sloppy mode:
// `exportAsmFunctions` does `var global_object = this`, which is undefined in a module and
// takes the init down. `importScripts` evaluates the glue as a classic script, where `this` is
// the global object, so the artefact built from the pinned upstream commit runs as it is,
// unpatched.
//
// Everything below mirrors mozilla/translations
// inference/wasm/tests/engine/translations-engine.worker.mjs — the file alignments, the marian
// configuration, the model bytes handed over as `wasmBinary` — rather than rediscovering them.
//
// The engine is part of the package; the model is not (add-lingua-translation-delivery D1). It is
// read from the `lingua-model` database the download filled, by the sha256 the package's catalogue
// pins — so only verified bytes ever reach the engine. Without them it does not start, and every
// surface answers as it does without it. The languages it translates between are the model's
// (generalise-lingua-translation-catalogue D3), and a sentence goes through the route of the pair
// it is asked for, each model built once (generalise-lingua-translation-model-state D5,
// generalise-lingua-translation-routes-by-pair D2). A route of two models — es-fr, through
// English — translates through both in one request (add-lingua-spanish-translation-pivot D2).

import { isTrap, LONG_ROUTE, NO_MODEL, type WorkerRequest, type WorkerResponse } from "./engine.ts";
import { modelDb } from "./model-db.ts";
import { loadBundledCatalogue, type ModelManifest, routeOf } from "./model-manifest.ts";

/** What the glue exposes, as much of it as this worker calls. */
interface BergamotModule {
  AlignedMemory: new (size: number, alignment: number) => { getByteArrayView(): Uint8Array };
  AlignedMemoryList: new () => { push_back(memory: unknown): void };
  TranslationModel: new (
    from: string,
    to: string,
    config: string,
    model: unknown,
    shortlist: unknown,
    vocabs: unknown,
    qualityModel: null,
  ) => unknown;
  BlockingService: new (config: { cacheSize: number }) => {
    translate(model: unknown, messages: unknown, options: unknown): VectorResponse;
    translateViaPivoting(first: unknown, second: unknown, messages: unknown, options: unknown): VectorResponse;
  };
  VectorString: new () => Deletable & { push_back(text: string): void };
  VectorResponseOptions: new () => Deletable & {
    push_back(options: { qualityScores: boolean; alignment: boolean; html: boolean }): void;
  };
}
interface Deletable {
  delete(): void;
}
interface VectorResponse extends Deletable {
  get(i: number): { getTranslatedText(): string };
}

declare function loadBergamot(module: Record<string, unknown>): BergamotModule;

/** Where the package carries the engine (build.mjs). */
const ENGINE_DIR = "engine";
const FILES = {
  glue: `${ENGINE_DIR}/bergamot-translator.js`,
  wasm: `${ENGINE_DIR}/bergamot-translator.wasm`,
};

/** Mozilla's alignment for each file the model is made of. */
const ALIGNMENT = { model: 256, lex: 64, vocab: 64 };

/**
 * Mozilla's pre-allocation. Measured at 64, 128 and 223 MiB, the working set settles at
 * 195.4 MiB either way: less only buys a heap copy during the first translation.
 */
const INITIAL_MEMORY = 234_291_200;

/** The marian configuration Mozilla's own engine uses, key for key. */
const MARIAN_CONFIG: Readonly<Record<string, string>> = {
  "beam-size": "1",
  normalize: "1.0",
  "word-penalty": "0",
  "max-length-break": "128",
  "mini-batch-words": "1024",
  workspace: "128",
  "max-length-factor": "2.0",
  "skip-cost": "true",
  "cpu-threads": "0",
  quiet: "true",
  "quiet-translation": "true",
  "gemm-precision": "int8shiftAlphaAll",
  alignment: "soft",
};

interface Engine {
  bergamot: BergamotModule;
  service: InstanceType<BergamotModule["BlockingService"]>;
}

let engine: Engine | null = null;
/** The translation models built in the engine, by catalogue id. */
const models = new Map<string, unknown>();
/** Each pair's route, once loaded: the ids of its models, in order. */
const routes = new Map<string, string[]>();

const scope = self as unknown as {
  onmessage: ((event: MessageEvent<WorkerRequest>) => void) | null;
  postMessage(message: WorkerResponse): void;
  close(): void;
};

function textConfig(config: Readonly<Record<string, string>>): string {
  const indent = "            ";
  let out = "\n";
  for (const [key, value] of Object.entries(config)) out += `${indent}${key}: ${value}\n`;
  return out + indent;
}

async function bytes(path: string): Promise<Uint8Array> {
  const response = await fetch(path);
  if (!response.ok) throw new Error(`${path}: ${response.status}`);
  return new Uint8Array(await response.arrayBuffer());
}

function aligned(bergamot: BergamotModule, data: Uint8Array, alignment: number): unknown {
  const memory = new bergamot.AlignedMemory(data.byteLength, alignment);
  memory.getByteArrayView().set(data);
  return memory;
}

/** A model's three files, as the download stored them — or null when any is missing. */
async function storedFiles(
  manifest: ModelManifest,
): Promise<{ model: Uint8Array; lex: Uint8Array; vocab: Uint8Array } | null> {
  const db = modelDb();
  const [model, lex, vocab] = await Promise.all([
    db.get(manifest.files.model.sha256),
    db.get(manifest.files.lex.sha256),
    db.get(manifest.files.vocab.sha256),
  ]);
  return model && lex && vocab ? { model, lex, vocab } : null;
}

/** The engine itself, instantiated once: the glue and the wasm, from the package. */
async function instance(): Promise<Engine> {
  if (engine) return engine;
  importScripts(FILES.glue);
  const wasm = await bytes(FILES.wasm);
  const bergamot = await new Promise<BergamotModule>((resolve, reject) => {
    const instance: BergamotModule = loadBergamot({
      INITIAL_MEMORY,
      print: () => {},
      // For the initialisation only: an abort during a translation throws after this callback,
      // and the catch below reads the throw (D1).
      onAbort: (what: unknown) => reject(new Error(`the engine aborted: ${String(what)}`)),
      onRuntimeInitialized: () => resolve(instance),
      wasmBinary: wasm,
    });
  });
  engine = { bergamot, service: new bergamot.BlockingService({ cacheSize: 0 }) };
  return engine;
}

/**
 * Load `pair`'s route: each of its models built once. The models first — without them, nothing
 * of the engine is worth loading. A route chains two models at most.
 */
async function load(pair: string): Promise<void> {
  if (routes.has(pair)) return;
  const route = routeOf(await loadBundledCatalogue(), pair);
  if (route.length === 0) throw new Error(NO_MODEL);
  if (route.length > 2) throw new Error(LONG_ROUTE);
  for (const manifest of route) {
    if (models.has(manifest.version)) continue;
    const stored = await storedFiles(manifest);
    if (!stored) throw new Error(NO_MODEL);
    const { bergamot } = await instance();
    const vocabs = new bergamot.AlignedMemoryList();
    vocabs.push_back(aligned(bergamot, stored.vocab, ALIGNMENT.vocab));
    models.set(
      manifest.version,
      new bergamot.TranslationModel(
        manifest.from,
        manifest.to,
        textConfig(MARIAN_CONFIG),
        aligned(bergamot, stored.model, ALIGNMENT.model),
        aligned(bergamot, stored.lex, ALIGNMENT.lex),
        vocabs,
        null,
      ),
    );
  }
  routes.set(
    pair,
    route.map((manifest) => manifest.version),
  );
}

function translate(markup: string, pair: string): string {
  const route = routes.get(pair);
  if (!engine || !route) throw new Error("the engine is not loaded");
  const { bergamot, service } = engine;
  const [first, second] = route.map((version) => models.get(version));
  const messages = new bergamot.VectorString();
  const options = new bergamot.VectorResponseOptions();
  messages.push_back(markup);
  // html: the engine repositions our tag onto the target span it corresponds to.
  options.push_back({ qualityScores: false, alignment: true, html: true });
  try {
    // Through the pivot, the engine carries the alignment from the first model to the second.
    const responses = second
      ? service.translateViaPivoting(first, second, messages, options)
      : service.translate(first, messages, options);
    try {
      return responses.get(0).getTranslatedText();
    } finally {
      responses.delete();
    }
  } finally {
    messages.delete();
    options.delete();
  }
}

scope.onmessage = (event) => {
  const request = event.data;
  void (async () => {
    try {
      if (request.op === "load") {
        await load(request.pair);
        scope.postMessage({ id: request.id, ok: true });
      } else {
        await load(request.pair);
        scope.postMessage({ id: request.id, ok: true, html: translate(request.markup, request.pair) });
      }
    } catch (e: unknown) {
      const error = e instanceof Error ? e.message : String(e);
      if (!isTrap(e)) {
        scope.postMessage({ id: request.id, ok: false, error });
        return;
      }
      // The engine trapped (harden-lingua-translation-engine D1): its linear memory is not to be
      // trusted for a next request, so this instance answers nothing more. Reported once, under
      // the request it trapped — an abort raised mid-translation calls `onAbort` (a no-op once the
      // init has resolved) and then throws, and that throw is what lands here — and the worker
      // closes itself, so that it cannot be asked by mistake. Its owner starts a fresh one and
      // asks again (channel.ts).
      scope.postMessage({ id: request.id, ok: false, error, trap: true });
      scope.close();
    }
  })();
};
