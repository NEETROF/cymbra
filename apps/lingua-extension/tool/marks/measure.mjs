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
// them. Either way the next request gets a clean engine, built when it is asked.

import { isTrap } from "../../src/translate/host/engine.ts";
import { escapeText, markSelection, readMarked, selectedText } from "../../src/translate/markup.ts";
import { reconcileMarks } from "../../src/translate/reconcile.ts";
import { fold } from "./stop-words.mjs";

/** reconcile.ts's stem rule: a shared prefix of 5, covering 70 % of the shorter word. */
export function sameWord(a, b) {
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
 * out, as tool/marks/engine.mjs builds one. `gloss`, when the pair's table exists, is
 * `(word, translation) => { lemma, gloss, marks }` (glossMark over the pair's tables); null leaves
 * the experiment empty. `trap` tells a trap from any other error, which is thrown. One result line
 * per selection, in order.
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
    const aloneHtml = fragment ? await ask(escapeText(fragment)) : null;
    const alone = aloneHtml === null ? null : readMarked(aloneHtml).sentence;
    const shown = alone === null ? tagged : reconcileMarks(tagged, alone);
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
    log(
      `${i + 1}/${selections.length} ${id}${alone === null ? ": the fragment trapped twice, marks unreconciled" : ""}`,
    );
  }
  return lines;
}
