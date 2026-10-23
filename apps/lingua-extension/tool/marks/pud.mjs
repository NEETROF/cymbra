// Parallel Universal Dependencies (PUD), English, Spanish, French and Italian, as the marks
// measurement reads it (release-lingua-spanish-translation D1, add-lingua-french-translation D4,
// measure-lingua-it-fr-translation-marks D1): fetched at pinned commits, checked by sha256, kept in
// scripts/lingua-data/work/marks/ (git-ignored) and never committed — CC BY-SA, as es-pud.sh keeps it.

import { createHash } from "node:crypto";
import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const here = dirname(fileURLToPath(import.meta.url));
export const WORK = join(here, "../../../../scripts/lingua-data/work/marks");

/**
 * The four treebanks, pinned. Spanish is the file es-pud.sh pins; French is the one
 * add-lingua-french-forms-tables pins (its D9 hands the pin over), the same commit and bytes.
 * Italian is `master`'s head on 2026-10-22 (measure-lingua-it-fr-translation-marks D1), the corpus's
 * own pin: were the Italian forms tables' measurement (it-ud.sh) to pin another commit, this one
 * stays, since moving it would move the selections the corpus holds.
 */
export const PUD = {
  en: {
    repo: "UD_English-PUD",
    commit: "f16eba4ae7f3d161870ed320676c5088b8fa476c",
    file: "en_pud-ud-test.conllu",
    sha256: "c80584f2bc2b31d5bada78a1136f9feec7ac49e5e18898db02dea434b5b8f0aa",
  },
  es: {
    repo: "UD_Spanish-PUD",
    commit: "818a82b8628c9cbec78750c7e83ccba34b9ce22b",
    file: "es_pud-ud-test.conllu",
    sha256: "48a7b5c7f409100b24eba90c7397e01f6fecaaf3207315a9a1bda029f77e98d3",
  },
  fr: {
    repo: "UD_French-PUD",
    commit: "db260db10fe728853c549760801229ef4e7b16e1",
    file: "fr_pud-ud-test.conllu",
    sha256: "4dfed37b83d76e77fd2e9963d0be00d723e9a010e7e2a746f8b7640c48063c10",
  },
  it: {
    repo: "UD_Italian-PUD",
    commit: "71d932aa096368331ba2fd9bc906ec8dd853da8e",
    file: "it_pud-ud-test.conllu",
    sha256: "ad5d302bdfd05194155d9ad43dc54c3cb915076d66b3eb6ef33eca9a11f1372a",
  },
};

const sha256 = (bytes) => createHash("sha256").update(bytes).digest("hex");

/** A treebank's text, fetched once and checked against its pin. */
export async function pudText(language) {
  const pin = PUD[language];
  const path = join(WORK, pin.file);
  if (!existsSync(path)) {
    const url = `https://raw.githubusercontent.com/UniversalDependencies/${pin.repo}/${pin.commit}/${pin.file}`;
    const response = await fetch(url);
    if (!response.ok) throw new Error(`${url}: ${response.status}`);
    mkdirSync(WORK, { recursive: true });
    writeFileSync(path, Buffer.from(await response.arrayBuffer()));
  }
  const bytes = readFileSync(path);
  const got = sha256(bytes);
  if (got !== pin.sha256) throw new Error(`${pin.file}: sha256 ${got}, pinned ${pin.sha256}`);
  return bytes.toString("utf8");
}

/**
 * The sentences in file order: id, text, and the words with their part of speech, whether they sit
 * inside a multiword token, and their offsets in the text (the tokens' surface forms aligned in order).
 */
export function parseConllu(text) {
  const sentences = [];
  let current = null;
  let mwtEnd = 0;
  let surfaces = [];
  const finish = () => {
    if (!current) return;
    let cursor = 0;
    for (const surface of surfaces) {
      const at = current.text.indexOf(surface.form, cursor);
      if (at < 0) throw new Error(`${current.id}: « ${surface.form} » not found in the text`);
      cursor = at + surface.form.length;
      for (const word of surface.words) Object.assign(word, surface.mwt ? {} : { start: at, end: cursor });
    }
    sentences.push(current);
    current = null;
  };
  for (const line of text.split("\n")) {
    if (line.startsWith("# sent_id = ")) {
      finish();
      current = { id: line.slice("# sent_id = ".length).trim(), text: "", words: [] };
      surfaces = [];
      mwtEnd = 0;
    } else if (line.startsWith("# text = ") && current) {
      current.text = line.slice("# text = ".length);
    } else if (line && !line.startsWith("#") && current) {
      const [id, form, , upos] = line.split("\t");
      if (id.includes(".")) continue; // an empty node
      if (id.includes("-")) {
        mwtEnd = Number(id.split("-")[1]);
        surfaces.push({ form, mwt: true, words: [] });
        continue;
      }
      const index = Number(id);
      const word = { index, form, upos, mwt: index <= mwtEnd };
      current.words.push(word);
      if (word.mwt) surfaces.at(-1).words.push(word);
      else surfaces.push({ form, mwt: false, words: [word] });
    }
  }
  finish();
  return sentences;
}
