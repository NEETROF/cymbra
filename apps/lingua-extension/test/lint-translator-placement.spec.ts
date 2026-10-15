import { existsSync, readFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { describe, expect, it } from "vitest";

// The translation engine blocks for as long as a sentence takes, so on any thread that paints
// it is a freeze — measured: a page blocking 200 ms shows a 202 ms frame gap, and the same
// translation in a worker leaves the page at its idle 18 ms (add-lingua-translation-engine).
//
// The obvious model to copy is the analyser, which on Chromium runs IN the content script.
// Copying it here would be exactly that freeze. So the rule is not a comment: this walks the
// real import graph from every entry point that paints, and fails if any of them can reach the
// engine's host — or construct a worker, or load the engine's glue, anywhere it can reach.

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
const src = join(root, "src");

/** Every surface that paints: the content script in the visited page, and the extension pages. */
const PAINTING = [
  "src/content.ts",
  "src/popup/popup.ts",
  "src/sidepanel/sidepanel.ts",
  "src/stats/stats.ts",
  "src/onboarding/onboarding.ts",
  "src/account/account.ts",
];

const HOST = join(src, "translate", "host");

/** The files a module imports, resolved (relative paths and the `@/` alias; packages ignored). */
function importsOf(file: string): string[] {
  const text = readFileSync(file, "utf8");
  const specs = [
    ...text.matchAll(/\bimport\s+(?:type\s+)?(?:[^'"]*?\sfrom\s+)?["']([^"']+)["']/g),
    ...text.matchAll(/\bexport\s+(?:type\s+)?[^'"]*?\sfrom\s+["']([^"']+)["']/g),
    ...text.matchAll(/\bimport\(\s*["']([^"']+)["']\s*\)/g),
  ].map((m) => m[1]!);
  const out: string[] = [];
  for (const spec of specs) {
    let path: string | null = null;
    if (spec.startsWith("@/")) path = join(src, spec.slice(2));
    else if (spec.startsWith(".")) path = resolve(dirname(file), spec);
    if (!path) continue;
    for (const candidate of [path, `${path}.ts`, join(path, "index.ts")]) {
      if (candidate.endsWith(".ts") && existsSync(candidate)) {
        out.push(candidate);
        break;
      }
    }
  }
  return out;
}

/** Everything reachable from `entry` through imports, `entry` included. */
function reachable(entry: string): Set<string> {
  const seen = new Set<string>();
  const stack = [join(root, entry)];
  while (stack.length > 0) {
    const file = stack.pop()!;
    if (seen.has(file)) continue;
    seen.add(file);
    stack.push(...importsOf(file));
  }
  return seen;
}

const rel = (f: string) => f.slice(root.length + 1);

/** A line that is code, not a comment explaining the rule. */
const codeLines = (text: string) =>
  text.split("\n").filter((line) => {
    const t = line.trimStart();
    return !t.startsWith("//") && !t.startsWith("*") && !t.startsWith("/*");
  });

/**
 * The Cymbra Lingua app's page (add-lingua-review-translation-fallback D7) owns its engine as the
 * background does: it may reach the host's channel and relay, but it constructs the engine's workers in
 * one module only, and never pulls in a worker's own code or the engine's glue — which run in the worker.
 */
const APP = "src/app/app.ts";
const APP_WORKERS = join(src, "app", "workers.ts");
const WORKER_ENTRIES = [join(HOST, "engine-worker.ts"), join(HOST, "model-worker.ts")];

describe("the Cymbra Lingua app's page keeps the engine in a worker", () => {
  it("reaches its workers' module, and through it the engine's channel", () => {
    const fromApp = reachable(APP);
    expect(fromApp.has(APP_WORKERS)).toBe(true);
    expect(fromApp.has(join(HOST, "channel.ts"))).toBe(true);
  });

  it("never imports a worker's own code", () => {
    const offending = [...reachable(APP)].filter((f) => WORKER_ENTRIES.includes(f)).map(rel);
    expect(offending).toEqual([]);
  });

  it("constructs workers in app/workers.ts alone, and loads no engine glue", () => {
    const offending: string[] = [];
    for (const file of reachable(APP)) {
      for (const line of codeLines(readFileSync(file, "utf8"))) {
        if (/\bloadBergamot\b/.test(line) || (/\bnew\s+Worker\s*\(/.test(line) && file !== APP_WORKERS))
          offending.push(`${rel(file)}: ${line.trim()}`);
      }
    }
    expect(offending).toEqual([]);
  });
});

describe("the translation engine never runs on a thread that paints", () => {
  it("walks the import graph for real — the background does reach the engine's host", () => {
    // Without this, a walker that followed nothing would pass every test below.
    const fromBackground = [...reachable("src/background.ts")];
    expect(fromBackground.some((f) => f.startsWith(HOST))).toBe(true);
  });

  for (const entry of PAINTING) {
    it(`${entry} cannot reach the engine's host`, () => {
      const offending = [...reachable(entry)].filter((f) => f.startsWith(HOST)).map(rel);
      expect(offending, `${entry} reaches the engine host — ask through createTranslatorPort()`).toEqual([]);
    });

    it(`${entry} constructs no worker and loads no engine glue`, () => {
      const offending: string[] = [];
      for (const file of reachable(entry)) {
        for (const line of codeLines(readFileSync(file, "utf8"))) {
          if (/\bnew\s+Worker\s*\(/.test(line) || /\bloadBergamot\b/.test(line))
            offending.push(`${rel(file)}: ${line.trim()}`);
        }
      }
      expect(offending, `${entry} would put the engine on its own thread`).toEqual([]);
    });
  }
});
