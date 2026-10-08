// The marks measurement (release-lingua-spanish-translation D2): every selection of
// tool/marks/corpus.json in a pair's studied language marked exactly as relay.ts marks it — the
// sentence with the selection tagged, the selection alone, readMarked then reconcileMarks, the
// extension's own modules — by the engine the package ships, through the pair's route (es-fr:
// es-en then en-fr, pivoting). Beside it, the experiment of D5: a mark located from the pack's
// gloss of the word, when the pair's tables are committed. A pair is measured on its own, and its
// results are filed by pair (generalise-lingua-translation-routes-by-pair D6): writes
// tool/marks/results-<pair>.jsonl. The loop itself is tool/marks/measure.mjs; a trap is answered
// there as the extension answers it (measure-lingua-translation-matrix-marks D2).
//
// Usage: node --experimental-strip-types tool/measure_marks.mjs --pair <pair> [--models <assembled-site-dir>]
//   The pair is one the catalogue routes (en-fr, es-fr, es-en, en-es); the corpus is its studied
//   language's, and the gloss experiment's stop words are its native language's, read from the
//   route's last model (tool/marks/stop-words.mjs). The models come from that directory
//   (tool/assemble_model_site.mjs) or, without it, from the catalogue's host; every file is checked
//   against the catalogue's sha256 before use. The engine itself is loaded by tool/marks/engine.mjs,
//   shared with the soak (tool/soak_engine.mjs); a request that traps it is asked once more on a
//   fresh one.

import { existsSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { catalogue, engine } from "./marks/engine.mjs";
import { glossMark, measureSelections, nativeOfRoute } from "./marks/measure.mjs";
import { parseConllu, pudText } from "./marks/pud.mjs";
import { stopWords } from "./marks/stop-words.mjs";
import { studiedOf } from "./packs.mjs";

const here = dirname(fileURLToPath(import.meta.url));
const APP = join(here, "..");
const TABLES = join(APP, "../../scripts/lingua-data/tables");
const corpus = JSON.parse(readFileSync(join(here, "marks/corpus.json"), "utf8"));

/** A pack table under tables/<folder>/: the first value of every key. */
function table(folder, name) {
  const rows = new Map();
  for (const line of readFileSync(join(TABLES, folder, name), "utf8").split("\n")) {
    const tab = line.indexOf("\t");
    if (tab > 0 && !rows.has(line.slice(0, tab))) rows.set(line.slice(0, tab), line.slice(tab + 1));
  }
  return rows;
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
  // The studied language, from the pair's name as the build reads it: the corpus and the PUD text
  // are its. The native language is the one the route translates into — its last model's `to`
  // (measure-lingua-translation-matrix-marks D1), held to the pair's name by nativeOfRoute, which
  // refuses a catalogue that routes a pair into another language; the gloss and the translated
  // sentence are both in it.
  const studied = studiedOf(pair);
  const native = nativeOfRoute(catalogue, pair);
  const stop = stopWords(native);
  const modelsDir = arg("--models");
  const sentences = new Map(parseConllu(await pudText(studied)).map((s) => [s.id, s.text]));
  // The experiment reads the pair's glosses, kept in tables/<pair>/ once the pair's pack is built
  // (D3): without them its columns stay empty, and the engine's are measured all the same. The
  // forms are the studied language's, kept once in tables/<studied>/ (split-lingua-pack-tables-by-language).
  const glossTable = join(TABLES, pair, "gloss.tsv");
  const tables = existsSync(glossTable)
    ? { forms: table(studied, "forms.tsv"), gloss: table(pair, "gloss.tsv") }
    : null;
  const gloss = tables ? (word, translation) => glossMark(word, translation, tables, stop) : null;
  console.log(
    `${pair}: ${studied} selections, ${native} stop words, the gloss experiment ${tables ? "from " + glossTable : "left empty (no " + glossTable + ")"}`,
  );
  const selections = corpus.items
    .filter((i) => i.lang === studied)
    .map((item) => ({ ...item, sentence: sentences.get(item.id), selection: { start: item.start, end: item.end } }));
  let built = 0;
  const lines = await measureSelections(selections, {
    engine: async () => {
      built++;
      return engine(modelsDir, route);
    },
    gloss,
    log: (message) => process.stderr.write(`${message}\n`),
  });
  const out = join(here, `marks/results-${pair}.jsonl`);
  writeFileSync(out, lines.map((line) => JSON.stringify(line)).join("\n") + "\n");
  // A fragment that trapped twice (`alone` null) and an empty one, never asked (`alone` ""), both
  // leave the engine's marks unreconciled; they are counted apart.
  const ids = (label, picked) =>
    `${picked.length} ${label}${picked.length ? ` (${picked.map((l) => l.id).join(", ")})` : ""}`;
  const trapped = ids(
    "trapped twice",
    lines.filter((l) => l.trapped),
  );
  const fragmentTrapped = ids(
    "fragments trapped twice (marks unreconciled)",
    lines.filter((l) => !l.trapped && l.alone === null),
  );
  const empty = ids(
    "empty fragments (not asked, marks unreconciled)",
    lines.filter((l) => !l.trapped && l.alone === ""),
  );
  console.log(
    `${out}: ${lines.length} selections, ${built} engine${built === 1 ? "" : "s"} built, ${trapped}, ${fragmentTrapped}, ${empty}`,
  );
}

main().catch((e) => {
  console.error(e);
  process.exit(1);
});
