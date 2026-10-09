// The corpus of the marks measurement (release-lingua-spanish-translation D1): every tenth PUD
// sentence, the same in English, Spanish and French, one word each — NOUN, VERB, NOUN, ADJ in turn,
// the first such word that is not the sentence's first, is letters only (three or more) and is not
// inside a multiword token; otherwise the first NOUN, VERB or ADJ that is; otherwise the next
// sentence, for every language. One rule over every studied language at once
// (add-lingua-french-translation D4): a sentence is taken only where each language has a word, so
// each step's selections read the same text — and French, added, moved no English or Spanish
// selection. Writes tool/marks/corpus.json.
//
// Usage: node tool/marks/select_corpus.mjs

import { writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { parseConllu, PUD, pudText } from "./pud.mjs";

const here = dirname(fileURLToPath(import.meta.url));
const OUT = join(here, "corpus.json");
const ITEMS = 100;
const STEP = 10;
const CYCLE = ["NOUN", "VERB", "NOUN", "ADJ"];
/** The studied languages, in the order their items are written: English drives the sentence order. */
const LANGUAGES = ["en", "es", "fr"];
const FALLBACK = ["NOUN", "VERB", "ADJ"];

const eligible = (word, upos) =>
  word.upos === upos && word.index !== 1 && !word.mwt && /^\p{L}{3,}$/u.test(word.form) && word.start !== undefined;

/** The word the rule picks in `sentence` for `upos`, or null. */
export function pick(sentence, upos) {
  const first = sentence.words.find((word) => eligible(word, upos));
  if (first) return first;
  for (const other of FALLBACK) {
    const found = sentence.words.find((word) => eligible(word, other));
    if (found) return found;
  }
  return null;
}

async function main() {
  const texts = await Promise.all(LANGUAGES.map((language) => pudText(language).then(parseConllu)));
  const en = texts[0];
  const byId = Object.fromEntries(LANGUAGES.map((language, i) => [language, new Map(texts[i].map((s) => [s.id, s]))]));
  const items = [];
  for (let k = 0; k < ITEMS; k++) {
    const upos = CYCLE[k % CYCLE.length];
    for (let at = k * STEP; at < en.length; at++) {
      const id = en[at].id;
      const sentences = Object.fromEntries(LANGUAGES.map((language) => [language, byId[language].get(id)]));
      if (LANGUAGES.some((language) => !sentences[language])) continue;
      const words = Object.fromEntries(LANGUAGES.map((language) => [language, pick(sentences[language], upos)]));
      if (LANGUAGES.some((language) => !words[language])) continue; // the next sentence, for every language
      for (const language of LANGUAGES) {
        const word = words[language];
        items.push({
          k,
          id,
          lang: language,
          token: word.index,
          word: word.form,
          upos: word.upos,
          start: word.start,
          end: word.end,
        });
      }
      break;
    }
  }
  const corpus = {
    source: PUD,
    rule: "every tenth PUD sentence; NOUN, VERB, NOUN, ADJ in turn; not the first word, letters only (3+), not in a multiword token; else the first NOUN/VERB/ADJ; else the next sentence (every language)",
    items,
  };
  writeFileSync(OUT, JSON.stringify(corpus, null, 2) + "\n");
  const count = (language) => items.filter((item) => item.lang === language).length;
  console.log(`${OUT}: ${count("en")} English, ${count("es")} Spanish and ${count("fr")} French selections`);
}

main().catch((e) => {
  console.error(e);
  process.exit(1);
});
