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
// read from the `lingua-model` database the download filled, by the sha256 the package's manifest
// pins — so only verified bytes ever reach the engine. Without them it does not start, and every
// surface answers as it does without it.

import { NO_MODEL, type WorkerRequest, type WorkerResponse } from "./engine.ts";
import { modelDb } from "./model-db.ts";
import { loadBundledManifest } from "./model-manifest.ts";

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
  model: unknown;
  service: InstanceType<BergamotModule["BlockingService"]>;
}

let engine: Engine | null = null;

const scope = self as unknown as {
  onmessage: ((event: MessageEvent<WorkerRequest>) => void) | null;
  postMessage(message: WorkerResponse): void;
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

/** The model's three files, as the download stored them — or null when any is missing. */
async function storedModel(): Promise<{ model: Uint8Array; lex: Uint8Array; vocab: Uint8Array } | null> {
  const { files } = await loadBundledManifest();
  const db = modelDb();
  const [model, lex, vocab] = await Promise.all([
    db.get(files.model.sha256),
    db.get(files.lex.sha256),
    db.get(files.vocab.sha256),
  ]);
  return model && lex && vocab ? { model, lex, vocab } : null;
}

async function load(): Promise<void> {
  if (engine) return;
  // The model first: without it, nothing of the engine is worth loading.
  const stored = await storedModel();
  if (!stored) throw new Error(NO_MODEL);
  const { model, lex, vocab } = stored;
  importScripts(FILES.glue);
  const wasm = await bytes(FILES.wasm);
  const bergamot = await new Promise<BergamotModule>((resolve, reject) => {
    const instance: BergamotModule = loadBergamot({
      INITIAL_MEMORY,
      print: () => {},
      onAbort: (what: unknown) => reject(new Error(`the engine aborted: ${String(what)}`)),
      onRuntimeInitialized: () => resolve(instance),
      wasmBinary: wasm,
    });
  });
  const vocabs = new bergamot.AlignedMemoryList();
  vocabs.push_back(aligned(bergamot, vocab, ALIGNMENT.vocab));
  engine = {
    bergamot,
    model: new bergamot.TranslationModel(
      "en",
      "fr",
      textConfig(MARIAN_CONFIG),
      aligned(bergamot, model, ALIGNMENT.model),
      aligned(bergamot, lex, ALIGNMENT.lex),
      vocabs,
      null,
    ),
    service: new bergamot.BlockingService({ cacheSize: 0 }),
  };
}

function translate(markup: string): string {
  if (!engine) throw new Error("the engine is not loaded");
  const { bergamot, model, service } = engine;
  const messages = new bergamot.VectorString();
  const options = new bergamot.VectorResponseOptions();
  messages.push_back(markup);
  // html: the engine repositions our tag onto the target span it corresponds to.
  options.push_back({ qualityScores: false, alignment: true, html: true });
  try {
    const responses = service.translate(model, messages, options);
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
        await load();
        scope.postMessage({ id: request.id, ok: true });
      } else {
        scope.postMessage({ id: request.id, ok: true, html: translate(request.markup) });
      }
    } catch (e: unknown) {
      scope.postMessage({ id: request.id, ok: false, error: e instanceof Error ? e.message : String(e) });
    }
  })();
};
