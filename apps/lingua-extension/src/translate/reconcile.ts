// Where the tag landed is the engine's alignment, and the alignment can be wrong. Measured on a
// real page: "They <b>seldom</b> ship on Friday, even when the customer asks nicely." comes back
// "Ils <b>expédient</b> rarement…" — a right translation with the mark on the neighbour. And a
// tag can only ever mark ONE run of words, where the reader's words may land apart: "They
// seldom" is "Ils … rarement", with the untouched "expédient" between.
//
// So the mark is checked against a second, independent signal: the fragment translated on its
// own. That translation loses the context's grammar — which is why it is never what the reader
// is shown — but it says which French words belong to the fragment, and that is all it is used
// for here. Everything below is text arithmetic on the two answers; no engine is called.

import type { MarkedTranslation, Span } from "./markup.ts";

/**
 * Below this length a word is never dropped from a mark: articles, auxiliaries and pronouns take
 * their form from the sentence (gender, tense, elision), not from the fragment, so the fragment's
 * own translation is no evidence against them — "gave up" alone is "abandonné", the sentence
 * says "a abandonné", and the "a" belongs.
 */
const SHORT = 4;

/** Two forms of a word count as the same when they share this long a prefix… */
const STEM = 5;
/** …covering at least this share of the shorter one — so "expédient" / "expédiés" match. */
const STEM_SHARE = 0.7;

const WORD = /[\p{L}\p{N}'’-]+/gu;

interface Word extends Span {
  /** The comparable form: lower-cased, one apostrophe, no elided clitic. */
  form: string;
}

/** Case and apostrophes folded; the same length as the input, so offsets carry over. */
function fold(text: string): string {
  return text.toLowerCase().replace(/[’ʼ]/g, "'");
}

/** A word as compared: folded, with an elided clitic ("l'", "s'", "qu'") taken off. */
function formOf(word: string): string {
  return fold(word).replace(/^\p{L}{1,2}'(?=\p{L})/u, "");
}

function wordsOf(text: string): Word[] {
  return [...text.matchAll(WORD)].map((m) => ({ start: m.index, end: m.index + m[0].length, form: formOf(m[0]) }));
}

function sameWord(a: string, b: string): boolean {
  if (a === b) return true;
  let common = 0;
  while (common < a.length && common < b.length && a[common] === b[common]) common++;
  return common >= STEM && common >= STEM_SHARE * Math.min(a.length, b.length);
}

function overlaps(a: Span, b: Span): boolean {
  return a.start < b.end && b.start < a.end;
}

/**
 * Where the fragment's own translation stands, word for word, in the sentence's — only when it
 * stands there exactly once; twice would be a guess.
 */
function locate(sentence: string, alone: string): Span | null {
  const needle = fold(alone.trim().replace(/^[\s"«»“”.,;:!?]+|[\s"«»“”.,;:!?]+$/gu, ""));
  if (!needle) return null;
  const hay = fold(sentence);
  const isWordChar = (c: string | undefined) => c !== undefined && /[\p{L}\p{N}'-]/u.test(c);
  let found: Span | null = null;
  for (let i = hay.indexOf(needle); i >= 0; i = hay.indexOf(needle, i + 1)) {
    if (isWordChar(hay[i - 1]) || isWordChar(hay[i + needle.length])) continue;
    if (found) return null;
    found = { start: i, end: i + needle.length };
  }
  return found;
}

/** Consecutive kept words become one mark; a word left out between them splits it. */
function runs(words: Word[], keep: boolean[]): Span[] {
  const out: Span[] = [];
  for (let i = 0; i < words.length; i++) {
    if (!keep[i]) continue;
    const last = out.at(-1);
    if (last && keep[i - 1]) last.end = words[i].end;
    else out.push({ start: words[i].start, end: words[i].end });
  }
  return out;
}

/**
 * The engine's marks, checked against `alone` — the fragment translated on its own.
 *
 * 1. `alone` found exactly once in the sentence, clear of the engine's marks: the tag landed on
 *    a neighbour, and `alone` replaces it ("seldom" → "rarement", not "expédient").
 * 2. Otherwise the region is the words the engine marked, grown over the full neighbouring words
 *    `alone` also contains ("[Ils expédient] rarement" + "Ils rarement" reaches "rarement").
 *    Inside it, a word the engine marked stays when `alone` contains it, when it is short, or
 *    when `alone` is not wholly found in the region (it may have used a synonym); a word the
 *    engine did not mark joins on `alone`'s evidence, or — if short — as the glue between two
 *    kept words. A stray run of short words `alone` lacks is dropped. "They seldom" thus
 *    becomes "[Ils] expédient [rarement]".
 * 3. `alone` shares no word with the region: it says nothing about this mark, which stands.
 *
 * With no mark from the engine there is nothing to check, and nothing is invented; and a check
 * that would leave no mark at all leaves the engine's.
 */
export function reconcileMarks(marked: MarkedTranslation, alone: string): MarkedTranslation {
  const { sentence, marks } = marked;
  if (marks.length === 0) return marked;

  const exact = locate(sentence, alone);
  if (exact && !marks.some((m) => overlaps(m, exact))) return { sentence, marks: [exact] };

  const words = wordsOf(sentence);
  const own = wordsOf(alone).map((w) => w.form);
  const inOwn = (form: string) => own.some((o) => sameWord(form, o));

  const tagged = words.map((w) => marks.some((m) => overlaps(m, w)));
  const full = (i: number) => words[i].form.length >= SHORT;
  let first = tagged.indexOf(true);
  let last = tagged.lastIndexOf(true);
  if (first < 0) return marked;
  // Grow over full words only — a short one ("en", "à", "le") is in too many translations to be
  // evidence — and by no more words than `alone` has, so the mark can never walk away.
  let grown = 0;
  while (grown < own.length && last + 1 < words.length && full(last + 1) && inOwn(words[last + 1].form)) {
    last++;
    grown++;
  }
  while (grown < own.length && first > 0 && full(first - 1) && inOwn(words[first - 1].form)) {
    first--;
    grown++;
  }
  const inRegion = (i: number) => i >= first && i <= last;
  if (!words.some((w, i) => inRegion(i) && inOwn(w.form))) return marked;

  const accounted = own
    .filter((o) => o.length >= SHORT)
    .every((o) => words.some((w, i) => inRegion(i) && sameWord(w.form, o)));
  // A word the engine marked stays unless `alone` is wholly found and lacks it; a full word it
  // did not mark joins only on `alone`'s evidence. Never on the grounds that `alone` fell short:
  // "[Nous] rendons [souvent]" must not become one mark over "rendons".
  const decided = words.map((w, i) => {
    if (!inRegion(i)) return false;
    if (tagged[i]) return inOwn(w.form) || !full(i) || !accounted;
    return full(i) && inOwn(w.form);
  });
  const keep = decided.map((k, i) => k || (inRegion(i) && !tagged[i] && !full(i) && glues(i)));
  dropStrayShortRuns(keep);
  // The check may move, split or trim a mark — never take it away entirely.
  if (!keep.some(Boolean)) return marked;
  // Confirmed word for word: the engine's own spans stand, with the punctuation they hold.
  if (keep.every((k, i) => k === tagged[i])) return marked;
  return { sentence, marks: runs(words, keep) };

  /** The engine sometimes tags a lone article far from the rest — "avant tout [le] monde" for
   *  "She always". A run of short words only, none of them `alone`'s, is dropped. */
  function dropStrayShortRuns(kept: boolean[]): void {
    const groups: number[][] = [];
    kept.forEach((k, i) => {
      if (!k) return;
      if (kept[i - 1]) groups.at(-1)?.push(i);
      else groups.push([i]);
    });
    for (const g of groups) if (g.every((i) => !full(i) && !inOwn(words[i].form))) for (const i of g) kept[i] = false;
  }

  /** A short word the engine left unmarked joins only as the glue between two kept words —
   *  "clés [de] voiture" — never at a mark's edge. */
  function glues(i: number): boolean {
    const gap = (j: number) => inRegion(j) && !tagged[j] && !full(j);
    let left = i;
    while (gap(left - 1)) left--;
    let right = i;
    while (gap(right + 1)) right++;
    return decided[left - 1] === true && decided[right + 1] === true;
  }
}
