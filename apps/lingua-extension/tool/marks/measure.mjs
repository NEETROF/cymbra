// The per-selection loop of the marks measurement (../measure_marks.mjs): every selection marked
// exactly as relay.ts marks it — the sentence with the selection tagged, the fragment alone,
// readMarked then reconcileMarks, the extension's own modules — and, beside it, the experiment of
// release-lingua-spanish-translation D5: a mark located from the pack's gloss of the word. Here,
// apart from the engine and the files, so that a fake engine can trap
// (measure-lingua-translation-matrix-marks D2) and a result line be read without a model.
//
// A trap is answered as the extension answers it (harden-lingua-translation-engine D2), per
// request: the instance that trapped is poisoned, so it is put down, and the request is asked once
// more on a fresh engine — `engine()` called again. A sentence that traps twice is recorded
// `trapped: true`, with no translation and no mark: the reader would get no translation. A
// fragment that traps twice leaves the sentence's own tagged marks unreconciled, as relay.ts shows
// them — so does an empty fragment, which is not asked (relay.ts asks none either): `alone` is ""
// for it, null for one that trapped twice. Either way the next request gets a clean engine, built
// when it is asked. The native language a pair's route translates into is read here too
// (`nativeOfRoute`), so the harness's reading of the catalogue is tested, and so are the
// experiment's tables (`readGlossTables`): read only when the pair's gloss table holds a gloss.

import { existsSync, readFileSync } from "node:fs";
import { join } from "node:path";
import { isTrap } from "../../src/translate/host/engine.ts";
import { escapeText, markSelection, readMarked, selectedText } from "../../src/translate/markup.ts";
import { reconcileMarks } from "../../src/translate/reconcile.ts";
import { nativeOf } from "../packs.mjs";
import { fold } from "./stop-words.mjs";

/**
 * The native language `pair`'s route translates into: the `to` of the route's last model
 * (measure-lingua-translation-matrix-marks D1) — es-fr's is French, through the pivot's en-fr. The
 * gloss, the stop words and the judge all speak it, so a route that does not end in the pair's own
 * native language (tool/packs.mjs's reading of its name) is a catalogue error, refused here rather
 * than measured in the wrong language.
 */
export function nativeOfRoute(catalogue, pair) {
  const route = catalogue.routes[pair];
  if (!route?.length) throw new Error(`model-manifest.json does not route ${pair}`);
  const last = route.at(-1);
  const model = catalogue.models[last];
  if (!model) throw new Error(`model-manifest.json routes ${pair} through "${last}", a model it does not hold`);
  if (model.to !== nativeOf(pair)) {
    throw new Error(
      `model-manifest.json routes ${pair} into "${model.to}", not the pair's native language "${nativeOf(pair)}"`,
    );
  }
  return model.to;
}

/** A pack table, `<key>\t<value>` per line: the first value of every key. */
function readTable(path) {
  const rows = new Map();
  for (const line of readFileSync(path, "utf8").split("\n")) {
    const tab = line.indexOf("\t");
    if (tab > 0 && !rows.has(line.slice(0, tab))) rows.set(line.slice(0, tab), line.slice(tab + 1));
  }
  return rows;
}

/**
 * The experiment's tables for `pair`, under `dir` (scripts/lingua-data/tables): the studied
 * language's forms, kept once in `<studied>/forms.tsv` (split-lingua-pack-tables-by-language), and
 * the pair's glosses, `<pair>/gloss.tsv` — or null, the experiment's columns left empty, when the
 * pair has no gloss table (measure-lingua-translation-matrix-marks D3) or one that holds no gloss
 * (add-lingua-french-translation D6). An empty table is committed for a pair whose glosses come later
 * (fr-en's, by add-lingua-french-forms-tables); read, it would make every line « no gloss », a
 * column that looks measured and is not.
 */
export function readGlossTables(dir, pair, studied) {
  const glossTable = join(dir, pair, "gloss.tsv");
  if (!existsSync(glossTable)) return null;
  const gloss = readTable(glossTable);
  if (![...gloss.values()].some((value) => value.trim() !== "")) return null;
  return { forms: readTable(join(dir, studied, "forms.tsv")), gloss };
}

/** reconcile.ts's stem rule: a shared prefix of 5, covering 70 % of the shorter word. */
function sameWord(a, b) {
  if (a === b) return true;
  let n = 0;
  while (n < a.length && n < b.length && a[n] === b[n]) n++;
  return n >= Math.min(5, a.length, b.length) && n >= 0.7 * Math.min(a.length, b.length);
}

/**
 * The experiment's mark for `word` in `translation`: the first word of the translated sentence that
 * matches a word of the pack's gloss — `tables.forms` the studied language's form → lemma,
 * `tables.gloss` the pair's lemma → gloss — the native language's `stop` words and anything
 * shorter than three letters left out of the gloss's candidates.
 */
export function glossMark(word, translation, tables, stop) {
  const lemma = tables.forms.get(word.toLowerCase()) ?? word.toLowerCase();
  const gloss = tables.gloss.get(lemma);
  if (!gloss) return { lemma, gloss: null, marks: [] };
  const candidates = [
    ...new Set(
      fold(gloss)
        .split(/[^\p{L}]+/u)
        .filter((w) => w.length >= 3 && !stop.has(w)),
    ),
  ];
  for (const m of translation.matchAll(/\p{L}+/gu)) {
    const token = fold(m[0]);
    if (token.length >= 3 && candidates.some((c) => sameWord(token, c)))
      return { lemma, gloss, marks: [{ start: m.index, end: m.index + m[0].length }] };
  }
  return { lemma, gloss, marks: [] };
}

/** `sentence` with `marks` bracketed ⟦…⟧, so a judge sees which occurrence is marked. */
export function bracketed(sentence, marks) {
  let out = sentence;
  for (const m of [...marks].sort((a, b) => b.start - a.start))
    out = `${out.slice(0, m.start)}⟦${out.slice(m.start, m.end)}⟧${out.slice(m.end)}`;
  return out;
}

/**
 * Every selection of `selections` — `{ k, id, word, upos, sentence, selection }` — measured through
 * `engine`, a factory whose every call builds a fresh engine: one markup string in, its translation
 * out, as tool/marks/engine.mjs builds one. `gloss`, when the pair's table holds a gloss, is
 * `(word, translation) => { lemma, gloss, marks }` (glossMark over the pair's tables); null leaves
 * the experiment empty. `trap` tells a trap from any other error, which is thrown. The engine is
 * built when the first request asks for it: no selection, no engine. One result line per
 * selection, in order.
 */
export async function measureSelections(selections, { engine, gloss = null, trap = isTrap, log = () => {} }) {
  let translate = null;
  /** `markup` asked once, and once more on a fresh engine after a trap; null when it trapped twice. */
  const ask = async (markup) => {
    for (let attempt = 0; attempt < 2; attempt++) {
      translate ??= await engine();
      try {
        return await translate(markup);
      } catch (e) {
        if (!trap(e)) throw e;
        translate = null; // poisoned: put down, and the next request — this one's retry first — gets a fresh engine
        log(`trapped${attempt ? " twice" : ", asked again"}: markup of ${markup.length} characters`);
      }
    }
    return null;
  };
  const lines = [];
  for (const [i, item] of selections.entries()) {
    const { k, id, word, upos, sentence, selection } = item;
    const fragment = selectedText(sentence, selection);
    const taggedHtml = await ask(markSelection(sentence, selection));
    if (taggedHtml === null) {
      lines.push({
        k,
        id,
        word,
        upos,
        sentence,
        trapped: true,
        translation: null,
        marks: [],
        shown: null,
        engineMarks: [],
        alone: null,
        gloss: null,
      });
      log(`${i + 1}/${selections.length} ${id}: trapped twice`);
      continue;
    }
    const tagged = readMarked(taggedHtml);
    // An empty fragment is not asked — relay.ts asks none either — and its `alone` is "": the
    // engine's marks stand, as they do when the fragment trapped twice (`alone` null).
    const aloneHtml = fragment ? await ask(escapeText(fragment)) : null;
    const alone = aloneHtml === null ? (fragment ? null : "") : readMarked(aloneHtml).sentence;
    const shown = aloneHtml === null ? tagged : reconcileMarks(tagged, alone);
    const slices = (marks) => marks.map((m) => shown.sentence.slice(m.start, m.end));
    const experiment = gloss ? gloss(word, shown.sentence) : null;
    lines.push({
      k,
      id,
      word,
      upos,
      sentence,
      trapped: false,
      translation: shown.sentence,
      marks: slices(shown.marks),
      shown: bracketed(shown.sentence, shown.marks),
      engineMarks: tagged.marks.map((m) => tagged.sentence.slice(m.start, m.end)),
      alone,
      gloss: experiment && {
        lemma: experiment.lemma,
        text: experiment.gloss,
        marks: slices(experiment.marks),
        shown: bracketed(shown.sentence, experiment.marks),
      },
    });
    const note =
      aloneHtml !== null
        ? ""
        : fragment
          ? ": the fragment trapped twice, marks unreconciled"
          : ": an empty fragment, not asked, marks unreconciled";
    log(`${i + 1}/${selections.length} ${id}${note}`);
  }
  return lines;
}
