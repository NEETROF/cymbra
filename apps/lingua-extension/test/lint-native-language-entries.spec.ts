import { readdirSync, readFileSync, statSync } from "node:fs";
import { dirname, join, relative } from "node:path";
import { fileURLToPath } from "node:url";
import { describe, expect, it } from "vitest";

// Every surface follows a change of native language (add-lingua-native-language-choice D3, D4), and
// the wiring lives in the entry points — excluded from coverage, which no unit test imports. The
// first implementation passed every test with all of it removed together: nothing failed. So the
// calls are read here, in the entries' code, comments left out:
// - every extension page reloads on the announced change (`reloadOnNativeLanguageChange`, which the
//   onboarding's `mountNativeStep` holds), and the book reader takes its session down first;
// - the content script builds its reading session anew (`followNativeLanguage`);
// - the popup presets a new install's choice before it paints, and asks it (`mountNativeCta`);
// - the onboarding presets before it starts, and asks its first question (`mountNativeStep`).

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
const src = join(root, "src");

/** `code` without its comments: a call written in a comment wires nothing. */
function code(path: string): string {
  return readFileSync(join(root, path), "utf8")
    .replace(/\/\*[\s\S]*?\*\//g, "")
    .replace(/(^|[^:"'`])\/\/.*$/gm, "$1");
}

/** Every `<name>(…)` call in `text`, from its name to its closing parenthesis. */
function callsOf(text: string, name: string): string[] {
  return [...text.matchAll(new RegExp(`\\b${name}\\(`, "g"))].map((m) => {
    let depth = 0;
    for (let i = m.index + m[0].length - 1; i < text.length; i++) {
      if (text[i] === "(") depth++;
      else if (text[i] === ")" && --depth === 0) return text.slice(m.index, i + 1);
    }
    return text.slice(m.index);
  });
}

function pages(dir: string): string[] {
  return readdirSync(dir).flatMap((name) => {
    const path = join(dir, name);
    if (statSync(path).isDirectory()) return pages(path);
    return name.endsWith(".html") ? [relative(root, path)] : [];
  });
}

/**
 * The extension pages a reader sees, by their entry. The offscreen document is no surface: it owns
 * the translation engine's workers, shows nothing, and holds no copy. A new page belongs here.
 */
const ENTRIES: Record<string, string> = {
  "src/popup/popup.html": "src/popup/popup.ts",
  "src/sidepanel/sidepanel.html": "src/sidepanel/sidepanel.ts",
  "src/stats/stats.html": "src/stats/stats.ts",
  "src/onboarding/onboarding.html": "src/onboarding/onboarding.ts",
  "src/account/account.html": "src/account/account.ts",
  "src/reader/reader.html": "src/reader/reader.ts",
};
const NOT_A_SURFACE = ["src/translate/host/offscreen.html"];

describe("the entries follow a change of native language (add-lingua-native-language-choice)", () => {
  it("knows every extension page", () => {
    expect(pages(src).sort()).toEqual([...Object.keys(ENTRIES), ...NOT_A_SURFACE].sort());
  });

  it.each(Object.entries(ENTRIES))("%s reloads on the announced change (D3)", (_page, entry) => {
    const text = code(entry);
    const hooks = [...callsOf(text, "reloadOnNativeLanguageChange"), ...callsOf(text, "mountNativeStep")];
    expect(hooks, `${entry}: no reload hook`).not.toEqual([]);
  });

  it("the book reader takes its session down before it reloads", () => {
    const [hook] = callsOf(code("src/reader/reader.ts"), "reloadOnNativeLanguageChange");
    expect(hook).toMatch(/session\?\.stop\(\);\s*location\.reload\(\)/);
  });

  it("the content script builds its reading session anew (D3)", () => {
    expect(callsOf(code("src/content.ts"), "followNativeLanguage")).toEqual(["followNativeLanguage(startSession)"]);
  });

  it("the popup presets before it paints, then asks (D4)", () => {
    const text = code("src/popup/popup.ts");
    const preset = text.search(/\bpresetNativeLanguage\(/);
    const paint = text.search(/\bfillPageInLanguage\(/);
    expect(preset).toBeGreaterThan(-1);
    expect(preset).toBeLessThan(paint);
    expect(callsOf(text, "mountNativeCta")).toHaveLength(1);
    // Asked only when the preset says the choice is still to be made.
    expect(text).toMatch(/if \(firstRun\)[^;]*mountNativeCta\(/);
  });

  it("the choice is built only while two native languages ship (__NATIVE_CHOICE__, D1)", () => {
    // The define folds the views, the preset and the copy out of a bundle whose pairs share one native
    // language, and builds them in from the second (check_variants).
    expect(code("src/popup/popup.ts")).toMatch(/__NATIVE_CHOICE__\s*\?\s*await presetNativeLanguage\(/);
    expect(code("src/popup/popup.ts")).toMatch(/if \(__NATIVE_CHOICE__\) \{[^}]*mountNativeCta\(/);
    expect(code("src/onboarding/onboarding.ts")).toMatch(/if \(__NATIVE_CHOICE__\) mountNativeStep\(/);
    expect(code("src/onboarding/onboarding.ts")).toMatch(/__NATIVE_CHOICE__ \? presetThenStart\(main\) : main\(\)/);
    expect(code("src/reading/settings-view.ts")).toMatch(
      /__NATIVE_CHOICE__ && nativeBlock\s*\?\s*mountNativeLanguage\(/,
    );
  });

  it("the onboarding presets before it starts, and asks its first question (D4)", () => {
    const text = code("src/onboarding/onboarding.ts");
    expect(callsOf(text, "presetThenStart")).toEqual(["presetThenStart(main)"]);
    expect(text).not.toMatch(/void main\(\)/); // started by the preset, never before it
    expect(callsOf(text, "mountNativeStep")).toHaveLength(1);
  });
});
