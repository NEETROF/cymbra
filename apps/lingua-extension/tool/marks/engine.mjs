// The pinned engine in Node, with a route's models built as engine-worker.ts builds them: one
// markup string in, its translation out (release-lingua-spanish-translation D2). Shared by the
// marks harness (../measure_marks.mjs) and the soak (../soak_engine.mjs,
// harden-lingua-translation-engine D4); the constants are engine-worker.ts's own, key for key.
//
// The glue is run as a classic script in this context — `self` is the global, as in the worker —
// and the models come from an assembled site directory (../assemble_model_site.mjs) or, without
// one, from the catalogue's host; every file is checked against the catalogue's sha256 before use.

import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import vm from "node:vm";
import { gunzipSync } from "node:zlib";

const here = dirname(fileURLToPath(import.meta.url));
const APP = join(here, "../..");
/** The package's catalogue: its models by id, and each pair's route (model-manifest.json). */
export const catalogue = JSON.parse(readFileSync(join(APP, "model-manifest.json"), "utf8"));

// engine-worker.ts's own constants, key for key.
const MARIAN_CONFIG = {
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
const ALIGNMENT = { model: 256, lex: 64, vocab: 64 };
const INITIAL_MEMORY = 234_291_200;

function textConfig(config) {
  const indent = "            ";
  let out = "\n";
  for (const [key, value] of Object.entries(config)) out += `${indent}${key}: ${value}\n`;
  return out + indent;
}

const sha256 = (bytes) => createHash("sha256").update(bytes).digest("hex");

async function modelFile(file, modelsDir) {
  const raw = modelsDir
    ? readFileSync(join(modelsDir, file.path))
    : Buffer.from(await (await fetch(new URL(file.path, catalogue.base))).arrayBuffer());
  const bytes = gunzipSync(raw);
  if (sha256(bytes) !== file.sha256) throw new Error(`${file.path}: not the pinned bytes`);
  return bytes;
}

/** The engine with `route`'s models built, as engine-worker.ts builds them: one markup string in, its translation out. */
export async function engine(modelsDir, route) {
  globalThis.self = globalThis;
  vm.runInThisContext(
    readFileSync(join(APP, "engine/bergamot-translator.js"), "utf8") + "\n;globalThis.__loadBergamot = loadBergamot;",
  );
  const wasmBinary = readFileSync(join(APP, "engine/bergamot-translator.wasm"));
  const bergamot = await new Promise((resolve, reject) => {
    const instance = globalThis.__loadBergamot({
      INITIAL_MEMORY,
      print: () => {},
      onAbort: (what) => reject(new Error(`the engine aborted: ${String(what)}`)),
      onRuntimeInitialized: () => resolve(instance),
      wasmBinary,
    });
  });
  const aligned = (data, alignment) => {
    const memory = new bergamot.AlignedMemory(data.byteLength, alignment);
    memory.getByteArrayView().set(data);
    return memory;
  };
  const build = async (id) => {
    const manifest = catalogue.models[id];
    const [model, lex, vocab] = await Promise.all(
      ["model", "lex", "vocab"].map((role) => modelFile(manifest.files[role], modelsDir)),
    );
    const vocabs = new bergamot.AlignedMemoryList();
    vocabs.push_back(aligned(vocab, ALIGNMENT.vocab));
    return new bergamot.TranslationModel(
      manifest.from,
      manifest.to,
      textConfig(MARIAN_CONFIG),
      aligned(model, ALIGNMENT.model),
      aligned(lex, ALIGNMENT.lex),
      vocabs,
      null,
    );
  };
  const service = new bergamot.BlockingService({ cacheSize: 0 });
  const [first, second] = await Promise.all(route.map(build));
  /** One markup string through the pair's route, as engine-worker.ts translates it. */
  return (markup) => {
    const messages = new bergamot.VectorString();
    const options = new bergamot.VectorResponseOptions();
    messages.push_back(markup);
    options.push_back({ qualityScores: false, alignment: true, html: true });
    try {
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
  };
}
