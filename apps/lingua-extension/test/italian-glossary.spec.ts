import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { JSDOM } from "jsdom";
import { describe, expect, it } from "vitest";
import { SURFACES } from "@/i18n/index.ts";
import { readLocales } from "../tool/locales.mjs";
import { catalogue, slotsOf, texts } from "./support/catalogue.ts";

// One Italian vocabulary (review-lingua-italian-wording D3): the glossary (`italian-glossary.json`,
// presented by docs/lingua/italian-glossary.md) names, for each product concept, its French forms, the
// Italian forms accepted for it and the variants refused. Each Italian text the extension ships as
// keyed messages — the catalogue's text surfaces, `_locales/it` and the Safari app's activation page
// (`copy.js`) — is read beside its French under the same key:
//   1. where the French names a concept, the Italian names it with an accepted form;
//   2. no Italian text uses a refused variant, whether or not its French names the concept;
//   3. every concept is named by some French text, and an exception that no longer applies fails.
// The site, the listings and `SignInCopy` are read by the reviewer agent's pass, not here (D3).

const HERE = dirname(fileURLToPath(import.meta.url));
const EXTENSION = join(HERE, "..");
const COPY_JS = join(EXTENSION, "..", "lingua-apple", "Shared (App)", "Resources", "copy.js");

interface Exception {
  key: string;
  reason: string;
}

interface Concept {
  id: string;
  /** The concept in English, for the reader of a failure. */
  en: string;
  fr: string[];
  it: string[];
  itRejected: string[];
  /** Why these forms: the precedence the glossary followed (D1), or where Italian has a reason to differ. */
  why: string;
  /** Keys whose Italian answers the French otherwise, each with its reason; one that no longer applies fails. */
  exceptions: Exception[];
}

interface Glossary {
  /** Keys written for their audience, not translated (D2): out of rule 1 by design, still read by rule 2. */
  perAudience: Exception[];
  concepts: Concept[];
  /** Concepts of the surfaces the test does not read (the site, Music's words): the pass's reference. */
  readByThePass: Omit<Concept, "exceptions">[];
}

/** A text the test reads, by its key: the catalogue's path, `_locales.<key>` or `copy.<key>`. */
type Texts = ReadonlyMap<string, string>;

/**
 * As the test reads a text: NFC; the typographic apostrophe as `'`; no-break spaces and runs of
 * whitespace as one space; the brand « Cymbra Lingua » masked, so that « Lingua » the brand and a
 * concept's word never meet.
 */
function normalise(text: string): string {
  return text
    .normalize("NFC")
    .replace(/\u2019/g, "'")
    .replace(/[\s\u00A0\u202F]+/gu, " ")
    .replace(/Cymbra Lingua/g, "\u2022");
}

const escape = (form: string): string => form.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");

/** A form matched whole and case-free, bounded by letters — not by JavaScript's ASCII `\b`. */
function matcher(forms: readonly string[]): RegExp | null {
  if (forms.length === 0) return null;
  const alternatives = forms.map((form) => escape(normalise(form))).join("|");
  return new RegExp(`(?<![\\p{L}\\p{M}\\p{N}])(?:${alternatives})(?![\\p{L}\\p{M}\\p{N}])`, "iu");
}

/** Every failure of the three rules, each naming its key and its concept. */
function check(glossary: Glossary, french: Texts, italian: Texts): string[] {
  const failures: string[] = [];
  const perAudience = new Set(glossary.perAudience.map((entry) => entry.key));
  for (const { key } of glossary.perAudience) {
    if (!italian.has(key)) failures.push(`perAudience « ${key} »: no such key`);
  }
  for (const concept of glossary.concepts) {
    const fr = matcher(concept.fr);
    const accepted = matcher(concept.it);
    const rejected = matcher(concept.itRejected);
    const excepted = new Map(concept.exceptions.map((entry) => [entry.key, entry]));
    const breaks = (key: string): string[] => {
      const it = italian.get(key);
      if (it === undefined) return [];
      const found: string[] = [];
      const source = french.get(key);
      if (!perAudience.has(key) && source !== undefined && fr?.test(source) && !accepted?.test(it)) {
        found.push(
          `${key} [${concept.id}]: the French « ${source} » names ${concept.en}, the Italian « ${it} » uses none of « ${concept.it.join(" », « ")} »`,
        );
      }
      const variant = rejected?.exec(it);
      if (variant) found.push(`${key} [${concept.id}]: « ${variant[0]} » is refused for ${concept.en}, in « ${it} »`);
      return found;
    };
    for (const key of italian.keys()) {
      if (!excepted.has(key)) failures.push(...breaks(key));
    }
    for (const { key } of concept.exceptions) {
      if (!italian.has(key)) failures.push(`${key} [${concept.id}]: a stale exception, no such key`);
      else if (breaks(key).length === 0) failures.push(`${key} [${concept.id}]: a stale exception, nothing to excuse`);
    }
    if (fr && ![...french.values()].some((text) => fr.test(text))) {
      failures.push(`[${concept.id}]: no French text names ${concept.en} as « ${concept.fr.join(" », « ")} »`);
    }
  }
  return failures;
}

/** The catalogue's text surfaces, keyed by path — the Italian `many` beside the French `other` (D3). */
function catalogueTexts(): { french: Map<string, string>; italian: Map<string, string> } {
  const french = new Map<string, string>();
  const italian = new Map<string, string>();
  // `grammar` renders tags, not text: change 34's snapshot and the pass read it.
  for (const surface of SURFACES.filter((surface) => surface !== "grammar")) {
    const fr = catalogue("fr", surface);
    const slots = slotsOf(fr, surface);
    for (const { path, text } of texts(fr, surface, slots)) french.set(path, text);
    for (const { path, text } of texts(catalogue("it", surface), surface, slots)) {
      italian.set(path, text);
      const many = path.replace(/\.many\(\)$/, ".other()");
      if (many !== path && french.has(many)) french.set(path, french.get(many)!);
    }
  }
  return { french, italian };
}

/** `_locales/<language>`'s messages by key, never their descriptions. */
function localeTexts(language: string): Map<string, string> {
  const messages = readLocales(EXTENSION)[language] ?? {};
  return new Map(Object.entries(messages).map(([key, entry]) => [`_locales.${key}`, entry.message]));
}

/** The activation page's table in `language`, loaded as the app loads it, its fragments' markup stripped. */
function copyTexts(language: string): Map<string, string> {
  const dom = new JSDOM("<!doctype html><html lang='fr'><body></body></html>", { runScripts: "outside-only" });
  const window = dom.window as typeof dom.window & { linguaCopy: { COPY: Record<string, Record<string, string>> } };
  window.eval(readFileSync(COPY_JS, "utf8"));
  const table = window.linguaCopy.COPY[language] ?? {};
  const plain = (html: string): string => {
    const holder = window.document.createElement("div");
    holder.innerHTML = html;
    return holder.textContent ?? "";
  };
  return new Map(Object.entries(table).map(([key, text]) => [`copy.${key}`, plain(text)]));
}

function normalised(...sources: Map<string, string>[]): Map<string, string> {
  return new Map(sources.flatMap((source) => [...source].map(([key, text]) => [key, normalise(text)] as const)));
}

const GLOSSARY: Glossary = JSON.parse(readFileSync(join(HERE, "italian-glossary.json"), "utf8"));

describe("the Italian wording is one vocabulary across its surfaces (review-lingua-italian-wording D3)", () => {
  const { french: frCatalogue, italian: itCatalogue } = catalogueTexts();
  const french = normalised(frCatalogue, localeTexts("fr"), copyTexts("fr"));
  const italian = normalised(itCatalogue, localeTexts("it"), copyTexts("it"));

  it("reads the three surfaces: the catalogue, _locales/it and the activation page", () => {
    expect([...italian.keys()].some((key) => key.startsWith("review."))).toBe(true);
    expect(italian.get("_locales.commandSidePanel")).toBeDefined();
    expect(italian.get("copy.step3")).toBeDefined();
    expect(italian.get("copy.step1")).not.toContain("<");
  });

  it("holds a well-formed glossary: unique ids, every concept with its French, Italian and reason", () => {
    const ids = [...GLOSSARY.concepts, ...GLOSSARY.readByThePass].map((concept) => concept.id);
    expect(new Set(ids).size).toBe(ids.length);
    for (const concept of [...GLOSSARY.concepts, ...GLOSSARY.readByThePass]) {
      expect(concept.fr.length, concept.id).toBeGreaterThan(0);
      expect(concept.it.length, concept.id).toBeGreaterThan(0);
      expect(concept.why.trim(), concept.id).not.toBe("");
      const accepted = new Set(concept.it.map((form) => normalise(form).toLowerCase()));
      for (const form of concept.itRejected) {
        expect(
          accepted.has(normalise(form).toLowerCase()),
          `${concept.id}: « ${form} » is both accepted and refused`,
        ).toBe(false);
      }
    }
    for (const concept of GLOSSARY.concepts) {
      for (const { key, reason } of concept.exceptions) expect(reason.trim(), `${concept.id} ${key}`).not.toBe("");
    }
  });

  it("names each concept as the glossary does, uses no refused variant, keeps no stale exception", () => {
    expect(check(GLOSSARY, french, italian)).toEqual([]);
  });

  it("is presented, concept by concept, by docs/lingua/italian-glossary.md", () => {
    const page = readFileSync(join(EXTENSION, "..", "..", "docs", "lingua", "italian-glossary.md"), "utf8");
    for (const concept of [...GLOSSARY.concepts, ...GLOSSARY.readByThePass]) {
      expect(page, concept.id).toContain(`\`${concept.id}\``);
    }
  });
});

describe("the glossary's check, on texts made for it", () => {
  const panel: Concept = {
    id: "review-panel",
    en: "the review panel",
    fr: ["panneau de révision"],
    it: ["pannello di ripasso"],
    itRejected: ["pannello di revisione"],
    why: "change 39's word",
    exceptions: [],
  };
  const card: Concept = {
    id: "card",
    en: "a card",
    fr: ["carte"],
    it: ["carta"],
    itRejected: ["scheda"],
    why: "the interface's word",
    exceptions: [],
  };
  const count: Concept = { ...card, id: "count", en: "a count", fr: ["compte"], it: ["conto"], itRejected: [] };
  const glossary = (...concepts: Concept[]): Glossary => ({ perAudience: [], concepts, readByThePass: [] });
  const map = (entries: Record<string, string>): Map<string, string> => normalised(new Map(Object.entries(entries)));

  it("A drifted term: the second key fails, naming the key and the concept", () => {
    const french = map({ "a.one": "Ouvre le panneau de révision", "a.two": "Ferme le panneau de révision" });
    const italian = map({ "a.one": "Apri il pannello di ripasso", "a.two": "Chiudi il pannello di revisione" });
    const failures = check(glossary(panel), french, italian);
    expect(failures).toHaveLength(2); // no accepted form (rule 1), and a refused one (rule 2)
    for (const failure of failures) expect(failure).toMatch(/^a\.two \[review-panel\]/);
  });

  it("A rejected variant with no French anchor: fails, naming the key and the variant", () => {
    const failures = check(
      glossary(card),
      map({ "b.x": "Une carte", "b.y": "Ouvre un onglet" }),
      map({ "b.x": "Una carta", "b.y": "Apri una scheda" }),
    );
    expect(failures).toEqual(["b.y [card]: « scheda » is refused for a card, in « Apri una scheda »"]);
  });

  it("A word inside a longer word: « cartes » and « décompte » are not « carte » and « compte »", () => {
    const french = map({ "c.x": "Tes cartes", "c.y": "Le décompte", "c.z": "une carte, un compte" });
    const italian = map({ "c.x": "Le tue carte", "c.y": "Il conteggio", "c.z": "una carta, un conto" });
    expect(check(glossary(card, count), french, italian)).toEqual([]);
    expect(matcher(["carta"])!.test("cartaceo")).toBe(false);
    expect(matcher(["citt"])!.test("città")).toBe(false); // « à » is a letter, where ASCII's \b sees a boundary
  });

  it("An exception that no longer applies: fails, naming the stale exception", () => {
    const excepted = { ...card, exceptions: [{ key: "d.x", reason: "the Italian says « la »" }] };
    const failures = check(
      glossary(excepted),
      map({ "d.x": "Il pleut", "d.y": "Une carte" }),
      map({ "d.x": "Piove", "d.y": "Una carta" }),
    );
    expect(failures).toEqual(["d.x [card]: a stale exception, nothing to excuse"]);
  });

  it("an exception still needed holds; a key written for its audience is out of rule 1, not rule 2", () => {
    const excepted = { ...card, exceptions: [{ key: "e.x", reason: "the Italian says « la »" }] };
    expect(check(glossary(excepted), map({ "e.x": "Garde la carte" }), map({ "e.x": "Tienila" }))).toEqual([]);
    const audience: Glossary = { ...glossary(card), perAudience: [{ key: "e.y", reason: "written per audience" }] };
    expect(check(audience, map({ "e.y": "Une carte" }), map({ "e.y": "Un mazzo" }))).toEqual([]);
    expect(check(audience, map({ "e.y": "Une carte" }), map({ "e.y": "Una scheda" }))).toHaveLength(1);
  });

  it("normalises the apostrophe, the no-break spaces and the brand", () => {
    expect(normalise("l\u2019estensione\u00A0:\u202F«\u00A0x\u00A0»")).toBe("l'estensione : « x »");
    expect(matcher(["lingua"])!.test(normalise("Apri Cymbra Lingua"))).toBe(false);
    expect(matcher(["lingua"])!.test(normalise("Scheda Lingua"))).toBe(true);
  });

  it("A concept no French text names fails rule 3", () => {
    expect(check(glossary(card), map({ "f.x": "Rien" }), map({ "f.x": "Niente" }))).toEqual([
      "[card]: no French text names a card as « carte »",
    ]);
  });
});
