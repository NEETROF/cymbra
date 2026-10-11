// The soak (harden-lingua-translation-engine D4): the real engine, in Node, through a pair's route
// over the committed corpus of the pair's studied language — every selection's sentence tagged as
// the extension tags it (relay.ts), then the fragment alone — and a report of what trapped, by
// corpus id, how long a sentence took and how much memory the run reached. It is how a model is
// tried before it ships (en-es, change 35; fr-en and fr-es, change 52), and it never runs in CI:
// the programme's M25 says a manual tool, and a run costs ≈ 100 MB of models and about a minute.
//
// Usage: node --experimental-strip-types tool/soak_engine.mjs --pair <pair> --models <dir> [--limit N] [--isolate]
//   --pair    one the catalogue routes: en-fr, es-fr, es-en, en-es, fr-en or fr-es
//             (add-lingua-translation-matrix-models pins en-es and routes es-en and en-es;
//             add-lingua-french-translation pins fr-en and routes fr-en, and fr-es through English);
//             the corpus is its studied language's
//   --models  an assembled site directory (tool/assemble_model_site.mjs); every file is checked
//             against the catalogue's sha256 before use
//   --limit   the first N selections only, N a positive whole number
//   --isolate each sentence in a child process of its own. In Node a trap throws a
//             WebAssembly.RuntimeError this tool catches, but the instance is poisoned from then
//             on — as it is in the worker, which closes itself — so without --isolate the run stops
//             at the first trap and says so; with it, the run goes on to the end and names every
//             input that trapped. A child that ends without its report — a trap while the route is
//             built, a crash, a signal — counts as trapped for its id; one past CHILD_TIMEOUT_MS is
//             counted apart, as timed out. Slower: each child loads the engine and the route again.
//
// The memory figure is process.resourceUsage().maxRSS: the process's resident high-water mark,
// the parent's or, with --isolate, the highest a child reported. It is Node's, not the browser
// worker's — the ≈ 322 MiB of es-fr's two models was measured in the worker, and Node's RSS is not
// like for like. Nor does the soak say anything of the worker's two-model bound: a run loads one
// route and deletes nothing, and a wasm instance's linear memory never shrinks — the bound caps
// growth, it does not lower what is held.
//
// The summary names a trapped input by its corpus id, never by its text; the sentence is in
// tool/marks/corpus.json and the PUD file for whoever needs it. Exit status 1 when anything
// trapped or timed out, so a script can tell.

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

/** How long one isolated sentence may take, the engine and the route loaded included: ≈ 1 s on a Mac, 3–4 s on the measured tablet. */
const CHILD_TIMEOUT_MS = 120_000;

/** The value after `--name`, or null. */
function arg(name) {
  const at = process.argv.indexOf(name);
  return at > 0 ? (process.argv[at + 1] ?? null) : null;
}
const flag = (name) => process.argv.includes(name);

const MiB = (bytes) => `${(bytes / 1_048_576).toFixed(1)} MiB`;

/** The process's resident high-water mark, in bytes: Node reports maxRSS in KiB on every platform (libuv normalises macOS's bytes). */
const highWater = () => process.resourceUsage().maxRSS * 1024;

/** Node's own flags for a child, the types stripped as they are for the parent. */
function childExecArgv() {
  const argv = [...process.execArgv];
  if (!argv.includes("--experimental-strip-types")) argv.push("--experimental-strip-types");
  return argv;
}

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
    const message = e instanceof Error ? e.message : String(e);
    return { ms: performance.now() - t0, trapped: true, error: message.split("\n")[0] };
  }
}

/** The run's figures: how many translated, which trapped and which timed out, the time per sentence, and the memory high-water mark. */
function report(pair, results, rss, stopped, isolated) {
  const trapped = results.filter((r) => r.trapped).map((r) => r.id);
  const timedOut = results.filter((r) => r.timedOut).map((r) => r.id);
  const times = results
    .filter((r) => !r.trapped && !r.timedOut)
    .map((r) => r.ms)
    .sort((a, b) => a - b);
  const median = times.length ? times[Math.floor(times.length / 2)] : 0;
  const mean = times.length ? times.reduce((a, b) => a + b, 0) / times.length : 0;
  const whose = isolated ? "the highest child's" : "this process's";
  const lines = [
    `soak ${pair}: ${results.length} selections run, ${times.length} translated, ${trapped.length} trapped, ${timedOut.length} timed out`,
    `time per sentence: median ${median.toFixed(0)} ms, mean ${mean.toFixed(0)} ms, max ${(times.at(-1) ?? 0).toFixed(0)} ms`,
    `memory high-water mark (maxRSS, ${whose}; Node's, not the worker's): ${MiB(rss)}`,
  ];
  if (trapped.length) lines.push(`trapped (corpus ids): ${trapped.join(", ")}`);
  if (timedOut.length) lines.push(`timed out past ${CHILD_TIMEOUT_MS / 1000} s (corpus ids): ${timedOut.join(", ")}`);
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
  console.log(JSON.stringify({ id, ...result, rss: highWater() }));
}

/**
 * One selection in a child of its own: its figures as it reported them — or what ended it before
 * it could. A trap while the route was built (in Node the engine throws it before any sentence),
 * a crash or a signal counts as trapped for this id; the timeout is counted apart. The run goes on.
 */
function isolated(pair, modelsDir, item) {
  const run = spawnSync(
    process.execPath,
    [...childExecArgv(), fileURLToPath(import.meta.url), "--pair", pair, "--models", modelsDir, "--one", item.id],
    { encoding: "utf8", stdio: ["ignore", "pipe", "inherit"], timeout: CHILD_TIMEOUT_MS },
  );
  if (run.error?.code === "ETIMEDOUT")
    return { id: item.id, ms: CHILD_TIMEOUT_MS, trapped: false, timedOut: true, rss: 0 };
  const line = run.status === 0 ? (run.stdout ?? "").trim().split("\n").at(-1) : "";
  if (!line) {
    const how = run.error ? run.error.message : run.signal ? `signal ${run.signal}` : `exit status ${run.status}`;
    return { id: item.id, ms: 0, trapped: true, error: `the child ended with ${how}, before its report`, rss: 0 };
  }
  return JSON.parse(line);
}

async function main() {
  const pair = arg("--pair");
  const route = pair ? catalogue.routes[pair] : undefined;
  if (!route) {
    throw new Error(
      `--pair <pair> is required: one of the catalogue's routes, ${Object.keys(catalogue.routes).join(", ")}`,
    );
  }
  const modelsDir = arg("--models");
  if (!modelsDir)
    throw new Error("--models <dir> is required: an assembled site directory (tool/assemble_model_site.mjs)");
  if (arg("--one")) return child(pair, modelsDir, arg("--one"));

  const limitArg = arg("--limit");
  const limit = limitArg === null ? null : Number(limitArg);
  if (flag("--limit") && (limit === null || !Number.isInteger(limit) || limit < 1)) {
    throw new Error(`--limit ${limitArg ?? ""}: a positive whole number is expected`);
  }
  const items = await selections(pair, limit);
  const results = [];
  let rss = 0;
  let stopped = null;
  const isolate = flag("--isolate");
  if (isolate) {
    for (const [i, item] of items.entries()) {
      const result = isolated(pair, modelsDir, item);
      results.push(result);
      rss = Math.max(rss, result.rss);
      const ended = result.timedOut ? ` — ${item.id} timed out` : result.trapped ? ` — ${item.id} trapped` : "";
      process.stderr.write(`\r${i + 1}/${items.length}${ended}\n`);
    }
  } else {
    const translate = await engine(modelsDir, route);
    for (const [i, item] of items.entries()) {
      const result = { id: item.id, ...soakOne(translate, item) };
      results.push(result);
      if (result.trapped) {
        stopped = { id: item.id, at: i + 1, of: items.length };
        break;
      }
    }
    rss = highWater();
  }
  console.log(report(pair, results, rss, stopped, isolate));
  if (results.some((r) => r.trapped || r.timedOut)) process.exitCode = 1;
}

main().catch((e) => {
  console.error(e);
  process.exit(1);
});
