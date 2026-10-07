// The soak (harden-lingua-translation-engine D4): the real engine, in Node, through a pair's route
// over the committed corpus of the pair's studied language — every selection's sentence tagged as
// the extension tags it (relay.ts), then the fragment alone — and a report of what trapped, by
// corpus id, how long a sentence took and how much memory the run reached. It is how a model is
// tried before it ships (en-es, change 35), and it never runs in CI: the programme's M25 says a
// manual tool, and a run costs ≈ 100 MB of models and about a minute.
//
// Usage: node --experimental-strip-types tool/soak_engine.mjs --pair <pair> --models <dir> [--limit N] [--isolate]
//   --pair    one the catalogue routes (en-fr, es-fr); the corpus is its studied language's
//   --models  an assembled site directory (tool/assemble_model_site.mjs); every file is checked
//             against the catalogue's sha256 before use
//   --limit   the first N selections only
//   --isolate each sentence in a child process of its own. In Node a trap throws a
//             WebAssembly.RuntimeError this tool catches, but the instance is poisoned from then
//             on — as it is in the worker, which closes itself — so without --isolate the run stops
//             at the first trap and says so; with it, the run goes on to the end and names every
//             input that trapped. Slower: each child loads the engine and the route again.
//
// The summary names a trapped input by its corpus id, never by its text; the sentence is in
// tool/marks/corpus.json and the PUD file for whoever needs it. Exit status 1 when anything
// trapped, so a script can tell.

import { spawnSync } from "node:child_process";
import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { isTrap } from "../src/translate/host/engine.ts";
import { escapeText, markSelection, selectedText } from "../src/translate/markup.ts";
import { catalogue, engine } from "./marks/engine.mjs";
import { parseConllu, pudText } from "./marks/pud.mjs";
import { studiedOf } from "./packs.mjs";

const here = dirname(fileURLToPath(import.meta.url));
const corpus = JSON.parse(readFileSync(join(here, "marks/corpus.json"), "utf8"));

/** The value after `--name`, or null. */
function arg(name) {
  const at = process.argv.indexOf(name);
  return at > 0 ? (process.argv[at + 1] ?? null) : null;
}
const flag = (name) => process.argv.includes(name);

const MiB = (bytes) => `${(bytes / 1_048_576).toFixed(1)} MiB`;

/** The pair's selections, each with its sentence from PUD: id, sentence, selection. */
async function selections(pair, limit) {
  const studied = studiedOf(pair);
  const sentences = new Map(parseConllu(await pudText(studied)).map((s) => [s.id, s.text]));
  const items = corpus.items.filter((i) => i.lang === studied).slice(0, limit ?? undefined);
  return items.map((item) => ({
    id: item.id,
    sentence: sentences.get(item.id),
    selection: { start: item.start, end: item.end },
  }));
}

/**
 * One selection through `translate`, as relay.ts asks it: the sentence with the selection tagged
 * (what the card waits for, and what is timed), then the fragment alone. A trap in either is the
 * selection's; nothing of the text is kept.
 */
function soakOne(translate, { sentence, selection }) {
  const fragment = selectedText(sentence, selection);
  const t0 = performance.now();
  try {
    translate(markSelection(sentence, selection));
    const ms = performance.now() - t0;
    if (fragment) translate(escapeText(fragment));
    return { ms, trapped: false };
  } catch (e) {
    if (!isTrap(e)) throw e;
    return { ms: performance.now() - t0, trapped: true, error: e.message.split("\n")[0] };
  }
}

/** The run's figures: how many translated, which trapped, the time per sentence, and the RSS high-water mark. */
function report(pair, results, rss, stopped) {
  const trapped = results.filter((r) => r.trapped).map((r) => r.id);
  const times = results
    .filter((r) => !r.trapped)
    .map((r) => r.ms)
    .sort((a, b) => a - b);
  const median = times.length ? times[Math.floor(times.length / 2)] : 0;
  const mean = times.length ? times.reduce((a, b) => a + b, 0) / times.length : 0;
  const lines = [
    `soak ${pair}: ${results.length} selections run, ${times.length} translated, ${trapped.length} trapped`,
    `time per sentence: median ${median.toFixed(0)} ms, mean ${mean.toFixed(0)} ms, max ${(times.at(-1) ?? 0).toFixed(0)} ms`,
    `RSS high-water mark: ${MiB(rss)}`,
  ];
  if (trapped.length) lines.push(`trapped (corpus ids): ${trapped.join(", ")}`);
  if (stopped) {
    lines.push(
      `stopped at the first trap (${stopped.id}, selection ${stopped.at} of ${stopped.of}): the instance is poisoned from here on — run with --isolate to go on to the end`,
    );
  }
  return lines.join("\n");
}

/** One selection in this process, for a child of `--isolate`: its figures on stdout as JSON. */
async function child(pair, modelsDir, id) {
  const [item] = (await selections(pair, null)).filter((s) => s.id === id);
  if (!item) throw new Error(`${id}: not a selection of ${pair}'s corpus`);
  const translate = await engine(modelsDir, catalogue.routes[pair]);
  const result = soakOne(translate, item);
  console.log(JSON.stringify({ id, ...result, rss: process.memoryUsage().rss }));
}

async function main() {
  const pair = arg("--pair");
  const route = pair ? catalogue.routes[pair] : undefined;
  if (!route) {
    throw new Error(
      `--pair <pair> is required: one of the catalogue's routes, ${Object.keys(catalogue.routes).join(" or ")}`,
    );
  }
  const modelsDir = arg("--models");
  if (!modelsDir)
    throw new Error("--models <dir> is required: an assembled site directory (tool/assemble_model_site.mjs)");
  if (arg("--one")) return child(pair, modelsDir, arg("--one"));

  const limit = arg("--limit") ? Number(arg("--limit")) : null;
  const items = await selections(pair, limit);
  const results = [];
  let rss = 0;
  let stopped = null;
  if (flag("--isolate")) {
    for (const [i, item] of items.entries()) {
      const run = spawnSync(
        process.execPath,
        [
          "--experimental-strip-types",
          fileURLToPath(import.meta.url),
          "--pair",
          pair,
          "--models",
          modelsDir,
          "--one",
          item.id,
        ],
        { encoding: "utf8", stdio: ["ignore", "pipe", "inherit"] },
      );
      if (run.status !== 0) throw new Error(`${item.id}: the child exited with ${run.status}`);
      const result = JSON.parse(run.stdout.trim().split("\n").at(-1));
      results.push(result);
      rss = Math.max(rss, result.rss);
      process.stderr.write(`\r${i + 1}/${items.length}${result.trapped ? ` — ${item.id} trapped` : ""}\n`);
    }
  } else {
    const translate = await engine(modelsDir, route);
    rss = process.memoryUsage().rss;
    for (const [i, item] of items.entries()) {
      const result = { id: item.id, ...soakOne(translate, item) };
      results.push(result);
      rss = Math.max(rss, process.memoryUsage().rss);
      if (result.trapped) {
        stopped = { id: item.id, at: i + 1, of: items.length };
        break;
      }
    }
  }
  console.log(report(pair, results, rss, stopped));
  if (results.some((r) => r.trapped)) process.exitCode = 1;
}

main().catch((e) => {
  console.error(e);
  process.exit(1);
});
