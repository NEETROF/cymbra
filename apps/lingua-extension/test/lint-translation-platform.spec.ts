import { readdirSync, readFileSync, statSync } from "node:fs";
import { dirname, join, relative } from "node:path";
import { fileURLToPath } from "node:url";
import { describe, expect, it } from "vitest";

// « Traduction étendue » is offered wherever the engine is packaged, Firefox for Android included
// (add-lingua-translation-android D1): the setting, its download, its deletion and its states are
// the desktop ones. It was once hidden on Android by a run-time platform check; this keeps any such
// check from coming back into the translation code, where it would quietly split the platforms.

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
const PLATFORM = /getPlatformInfo|["']android["']/;

function files(dir: string): string[] {
  return readdirSync(dir).flatMap((name) => {
    const path = join(dir, name);
    return statSync(path).isDirectory() ? files(path) : path.endsWith(".ts") ? [path] : [];
  });
}

/** The background's translation block: from its opening guard to the matching brace. */
function translationBlock(source: string): string {
  const start = source.indexOf('if (__TRANSLATION_HOST__ !== "none") {');
  expect(start).toBeGreaterThan(-1);
  let depth = 0;
  for (let i = source.indexOf("{", start); i < source.length; i++) {
    if (source[i] === "{") depth++;
    if (source[i] === "}" && --depth === 0) return source.slice(start, i + 1);
  }
  throw new Error("unterminated translation block");
}

describe("extended translation does not depend on the platform", () => {
  it("no module under src/translate/ reads the platform", () => {
    const offenders = files(join(root, "src", "translate"))
      .filter((file) => PLATFORM.test(readFileSync(file, "utf8")))
      .map((file) => relative(root, file));
    expect(offenders).toEqual([]);
  });

  it("the background's translation block reads no platform", () => {
    const block = translationBlock(readFileSync(join(root, "src", "background.ts"), "utf8"));
    expect(block).toContain("new ModelController");
    expect(block).not.toMatch(PLATFORM);
  });
});
