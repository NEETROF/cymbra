// The marks measurement (release-lingua-spanish-translation D2): every selection of
// tool/marks/corpus.json marked exactly as relay.ts marks it — the sentence with the selection
// tagged, the selection alone, readMarked then reconcileMarks, the extension's own modules — by the
// engine the package ships, through the language's route (Spanish: es-en then en-fr, pivoting).
// Beside it, the experiment of D5: a mark located from the pack's French gloss of the word.
// Writes tool/marks/results-<lang>.jsonl.
//
// Usage: node --experimental-strip-types tool/measure_marks.mjs [--models <assembled-site-dir>]
//   The models come from that directory (tool/assemble_model_site.mjs) or, without it, from the
//   catalogue's host; every file is checked against the catalogue's sha256 before use.

import { createHash } from "node:crypto";
import { readFileSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import vm from "node:vm";
import { gunzipSync } from "node:zlib";
import { escapeText, markSelection, readMarked, selectedText } from "../src/translate/markup.ts";
import { reconcileMarks } from "../src/translate/reconcile.ts";
import { parseConllu, pudText } from "./marks/pud.mjs";

const here = dirname(fileURLToPath(import.meta.url));
const APP = join(here, "..");
const TABLES = join(APP, "../../scripts/lingua-data/tables");
const catalogue = JSON.parse(readFileSync(join(APP, "model-manifest.json"), "utf8"));
const corpus = JSON.parse(readFileSync(join(here, "marks/corpus.json"), "utf8"));
const PAIRS = { en: "en-fr", es: "es-fr" };

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

async function engine(modelsDir) {
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
  const built = new Map();
  const build = async (id) => {
    if (built.has(id)) return built.get(id);
    const manifest = catalogue.models[id];
    const [model, lex, vocab] = await Promise.all(
      ["model", "lex", "vocab"].map((role) => modelFile(manifest.files[role], modelsDir)),
    );
    const vocabs = new bergamot.AlignedMemoryList();
    vocabs.push_back(aligned(vocab, ALIGNMENT.vocab));
    const instance = new bergamot.TranslationModel(
      manifest.from,
      manifest.to,
      textConfig(MARIAN_CONFIG),
      aligned(model, ALIGNMENT.model),
      aligned(lex, ALIGNMENT.lex),
      vocabs,
      null,
    );
    built.set(id, instance);
    return instance;
  };
  const service = new bergamot.BlockingService({ cacheSize: 0 });
  const routes = {};
  for (const language of Object.keys(PAIRS))
    routes[language] = await Promise.all(catalogue.routes[language].map(build));
  /** One markup string through `language`'s route, as engine-worker.ts translates it. */
  return (markup, language) => {
    const [first, second] = routes[language];
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

// D5 — the experiment: the pack's own French gloss of the selected word, found in the sentence.
function table(pair, name) {
  const rows = new Map();
  for (const line of readFileSync(join(TABLES, pair, name), "utf8").split("\n")) {
    const tab = line.indexOf("\t");
    if (tab > 0 && !rows.has(line.slice(0, tab))) rows.set(line.slice(0, tab), line.slice(tab + 1));
  }
  return rows;
}
const STOP = new Set(
  "le la les un une des de du d l et ou en au aux à a pour par sur dans avec sans qui que se sa son ses leur leurs ce cet cette ces ne pas plus est être avoir être faire".split(
    " ",
  ),
);
const fold = (s) => s.normalize("NFD").replace(/\p{M}/gu, "").toLowerCase();
/** reconcile.ts's stem rule: a shared prefix of 5, covering 70 % of the shorter word. */
function sameWord(a, b) {
  if (a === b) return true;
  let n = 0;
  while (n < a.length && n < b.length && a[n] === b[n]) n++;
  return n >= Math.min(5, a.length, b.length) && n >= 0.7 * Math.min(a.length, b.length);
}
function glossMark(pair, word, french, tables) {
  const lemma = tables.forms.get(word.toLowerCase()) ?? word.toLowerCase();
  const gloss = tables.gloss.get(lemma);
  if (!gloss) return { lemma, gloss: null, marks: [] };
  const candidates = [
    ...new Set(
      fold(gloss)
        .split(/[^\p{L}]+/u)
        .filter((w) => w.length >= 3 && !STOP.has(w)),
    ),
  ];
  for (const m of french.matchAll(/\p{L}+/gu)) {
    const token = fold(m[0]);
    if (token.length >= 3 && candidates.some((c) => sameWord(token, c)))
      return { lemma, gloss, marks: [{ start: m.index, end: m.index + m[0].length }] };
  }
  return { lemma, gloss, marks: [] };
}

async function main() {
  const at = process.argv.indexOf("--models");
  const modelsDir = at > 0 ? process.argv[at + 1] : null;
  const translate = await engine(modelsDir);
  const sentences = {
    en: new Map(parseConllu(await pudText("en")).map((s) => [s.id, s.text])),
    es: new Map(parseConllu(await pudText("es")).map((s) => [s.id, s.text])),
  };
  const tables = Object.fromEntries(
    Object.entries(PAIRS).map(([language, pair]) => [
      language,
      { forms: table(pair, "forms.tsv"), gloss: table(pair, "gloss.tsv") },
    ]),
  );
  for (const language of Object.keys(PAIRS)) {
    const lines = [];
    for (const item of corpus.items.filter((i) => i.lang === language)) {
      const sentence = sentences[language].get(item.id);
      const selection = { start: item.start, end: item.end };
      const fragment = selectedText(sentence, selection);
      const tagged = readMarked(translate(markSelection(sentence, selection), language));
      const alone = readMarked(translate(escapeText(fragment), language)).sentence;
      const shown = reconcileMarks(tagged, alone);
      const slices = (marks) => marks.map((m) => shown.sentence.slice(m.start, m.end));
      /** The French sentence with `marks` bracketed ⟦…⟧, so a judge sees which occurrence is marked. */
      const bracketed = (marks) => {
        let out = shown.sentence;
        for (const m of [...marks].sort((a, b) => b.start - a.start))
          out = `${out.slice(0, m.start)}⟦${out.slice(m.start, m.end)}⟧${out.slice(m.end)}`;
        return out;
      };
      const experiment = glossMark(PAIRS[language], item.word, shown.sentence, tables[language]);
      lines.push(
        JSON.stringify({
          k: item.k,
          id: item.id,
          word: item.word,
          upos: item.upos,
          sentence,
          french: shown.sentence,
          marks: slices(shown.marks),
          shown: bracketed(shown.marks),
          engineMarks: tagged.marks.map((m) => tagged.sentence.slice(m.start, m.end)),
          alone,
          gloss: {
            lemma: experiment.lemma,
            text: experiment.gloss,
            marks: slices(experiment.marks),
            shown: bracketed(experiment.marks),
          },
        }),
      );
    }
    const out = join(here, `marks/results-${language}.jsonl`);
    writeFileSync(out, lines.join("\n") + "\n");
    console.log(`${out}: ${lines.length} selections`);
  }
}

main().catch((e) => {
  console.error(e);
  process.exit(1);
});
