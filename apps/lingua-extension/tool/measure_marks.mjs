// The marks measurement (release-lingua-spanish-translation D2): every selection of
// tool/marks/corpus.json in a pair's studied language marked exactly as relay.ts marks it — the
// sentence with the selection tagged, the selection alone, readMarked then reconcileMarks, the
// extension's own modules — by the engine the package ships, through the pair's route (es-fr:
// es-en then en-fr, pivoting). Beside it, the experiment of D5: a mark located from the pack's
// gloss of the word. A pair is measured on its own, and its results are filed by pair
// (generalise-lingua-translation-routes-by-pair D6): writes tool/marks/results-<pair>.jsonl.
//
// Usage: node --experimental-strip-types tool/measure_marks.mjs --pair <pair> [--models <assembled-site-dir>]
//   The pair is one the catalogue routes (en-fr, es-fr); the corpus is its studied language's. The
//   models come from that directory (tool/assemble_model_site.mjs) or, without it, from the
//   catalogue's host; every file is checked against the catalogue's sha256 before use. The engine
//   itself is loaded by tool/marks/engine.mjs, shared with the soak (tool/soak_engine.mjs).

import { readFileSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { escapeText, markSelection, readMarked, selectedText } from "../src/translate/markup.ts";
import { reconcileMarks } from "../src/translate/reconcile.ts";
import { catalogue, engine } from "./marks/engine.mjs";
import { parseConllu, pudText } from "./marks/pud.mjs";
import { studiedOf } from "./packs.mjs";

const here = dirname(fileURLToPath(import.meta.url));
const APP = join(here, "..");
const TABLES = join(APP, "../../scripts/lingua-data/tables");
const corpus = JSON.parse(readFileSync(join(here, "marks/corpus.json"), "utf8"));

// D5 — the experiment: the pack's own gloss of the selected word, found in the translated sentence.

/** A pack table under tables/<folder>/: the first value of every key. */
function table(folder, name) {
  const rows = new Map();
  for (const line of readFileSync(join(TABLES, folder, name), "utf8").split("\n")) {
    const tab = line.indexOf("\t");
    if (tab > 0 && !rows.has(line.slice(0, tab))) rows.set(line.slice(0, tab), line.slice(tab + 1));
  }
  return rows;
}
// The experiment's stop words, and the `french` key of a result line, are French-native: every pair
// measured so far is glossed in French. measure-lingua-translation-matrix-marks (change 26) generalises
// them with the first pair of another native language, es-en.
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
function glossMark(word, french, tables) {
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

/** The value after `--name`, or null. */
function arg(name) {
  const at = process.argv.indexOf(name);
  return at > 0 ? (process.argv[at + 1] ?? null) : null;
}

async function main() {
  const pair = arg("--pair");
  const route = pair ? catalogue.routes[pair] : undefined;
  if (!route) {
    throw new Error(
      `--pair <pair> is required: one of the catalogue's routes, ${Object.keys(catalogue.routes).join(" or ")}`,
    );
  }
  // The studied language, from the pair's name as the build reads it: the corpus and the PUD text are its.
  const studied = studiedOf(pair);
  const translate = await engine(arg("--models"), route);
  const sentences = new Map(parseConllu(await pudText(studied)).map((s) => [s.id, s.text]));
  // The forms are the studied language's, kept once in tables/<studied>/
  // (split-lingua-pack-tables-by-language); the glosses the pair's.
  const tables = { forms: table(studied, "forms.tsv"), gloss: table(pair, "gloss.tsv") };
  const lines = [];
  for (const item of corpus.items.filter((i) => i.lang === studied)) {
    const sentence = sentences.get(item.id);
    const selection = { start: item.start, end: item.end };
    const fragment = selectedText(sentence, selection);
    const tagged = readMarked(translate(markSelection(sentence, selection)));
    const alone = readMarked(translate(escapeText(fragment))).sentence;
    const shown = reconcileMarks(tagged, alone);
    const slices = (marks) => marks.map((m) => shown.sentence.slice(m.start, m.end));
    /** The translated sentence with `marks` bracketed ⟦…⟧, so a judge sees which occurrence is marked. */
    const bracketed = (marks) => {
      let out = shown.sentence;
      for (const m of [...marks].sort((a, b) => b.start - a.start))
        out = `${out.slice(0, m.start)}⟦${out.slice(m.start, m.end)}⟧${out.slice(m.end)}`;
      return out;
    };
    const experiment = glossMark(item.word, shown.sentence, tables);
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
  const out = join(here, `marks/results-${pair}.jsonl`);
  writeFileSync(out, lines.join("\n") + "\n");
  console.log(`${out}: ${lines.length} selections`);
}

main().catch((e) => {
  console.error(e);
  process.exit(1);
});
