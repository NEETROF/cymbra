import { existsSync, readdirSync, readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { describe, expect, it } from "vitest";
import { bracketed, glossMark, measureSelections, type Selection, type Translate } from "../tool/marks/measure.mjs";
import { FRENCH, STOP_WORDS, fold, stopWords, wordsOf } from "../tool/marks/stop-words.mjs";

// The harness's loop and its stop words (measure-lingua-translation-matrix-marks D1, D2, D3),
// apart from the engine and the files: a fake engine that traps, a gloss table or none.

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
const tables = join(root, "../../scripts/lingua-data/tables");

describe("the stop words speak the pair's native language (D1)", () => {
  /**
   * The readings the committed tables give a word of `language`: every UD tag a senses table of a
   * pair studying it lists for the word itself and for each lemma forms.tsv maps it to.
   */
  function readings(language: string) {
    const senses = new Map<string, string[]>();
    for (const dir of readdirSync(tables).filter((d) => d.startsWith(`${language}-`))) {
      for (const line of readFileSync(join(tables, dir, "senses.tsv"), "utf8").split("\n")) {
        const [lemma, ...runs] = line.split("\t");
        if (!lemma) continue;
        const tags = runs.map((run) => run.split(":")[0].split("|")[0]);
        senses.set(lemma, [...(senses.get(lemma) ?? []), ...tags]);
      }
    }
    const forms = new Map<string, string[]>();
    for (const line of readFileSync(join(tables, language, "forms.tsv"), "utf8").split("\n")) {
      const [form, lemma] = line.split("\t");
      if (form) forms.set(form, [...(forms.get(form) ?? []), lemma]);
    }
    return (word: string) => [...new Set([word, ...(forms.get(word) ?? [])])].flatMap((l) => senses.get(l) ?? []);
  }
  /** UD's function classes; X is a contraction (« del », « al ») in the Spanish tables. */
  const FUNCTION = new Set(["DET", "ADP", "PRON", "CCONJ", "SCONJ", "PART", "AUX", "ADV", "NUM", "SYM", "INTJ", "X"]);

  it.each(["en", "es"])(
    "%s: no content word of its own language from the committed tables — every function word has a function-class reading, every auxiliary a verb's",
    (language) => {
      const read = readings(language);
      const words = wordsOf(STOP_WORDS[language]);
      const unknown: string[] = [];
      const content: string[] = [];
      for (const { category, word } of words) {
        const tags = read(word);
        if (tags.length === 0) {
          unknown.push(word);
          continue;
        }
        const ok =
          category === "auxiliaries"
            ? tags.some((t) => t === "VERB" || t === "AUX")
            : tags.some((t) => FUNCTION.has(t));
        if (!ok) content.push(`${word} (${category}: ${[...new Set(tags)].join(", ")})`);
      }
      expect(content).toEqual([]);
      // The check is not vacuous: the tables know nearly every word (« an » is the one English word they do not list).
      expect(unknown.length, `unknown to the tables: ${unknown.join(", ")}`).toBeLessThanOrEqual(1);
      expect(words.length).toBeGreaterThan(80);
    },
  );

  it("fr: the French set is the one en-fr and es-fr were measured with, word for word (French is native only: no committed table of its own)", () => {
    expect(existsSync(join(tables, "fr"))).toBe(false);
    const measuredWith =
      "le la les un une des de du d l et ou en au aux à a pour par sur dans avec sans qui que se sa son ses leur leurs ce cet cette ces ne pas plus est être avoir être faire";
    expect(new Set(wordsOf(FRENCH).map((w) => w.word))).toEqual(new Set(measuredWith.split(" ")));
  });

  it("is picked by native language, folded as the gloss's tokens are, and refuses a language without a set", () => {
    expect(stopWords("fr")).toEqual(new Set(wordsOf(FRENCH).map((w) => fold(w.word))));
    const spanish = stopWords("es");
    expect(spanish.has("esta")).toBe(true); // « está » and « esta » alike, once folded
    expect(spanish.has("está")).toBe(false);
    expect(spanish.has("el")).toBe(true);
    expect(stopWords("en").has("the")).toBe(true);
    expect(stopWords("es").has("the")).toBe(false);
    expect(() => stopWords("de")).toThrow(/no stop words for the native language "de"/);
  });
});

describe("the gloss experiment (D3, D5 of release-lingua-spanish-translation)", () => {
  const glossTables = {
    forms: new Map([["houses", "house"]]),
    gloss: new Map([
      ["house", "to; the house; a home"],
      ["casa", "the; house"],
    ]),
  };

  it("marks the first word of the translation that matches the gloss, the native language's stop words and short words left out", () => {
    const english = stopWords("en");
    const mark = glossMark("houses", "The houses by the sea.", glossTables, english);
    expect(mark.lemma).toBe("house");
    expect(mark.marks.map((m) => "The houses by the sea.".slice(m.start, m.end))).toEqual(["houses"]); // not « The » (a stop word) nor « to »
    expect(bracketed("The houses by the sea.", mark.marks)).toBe("The ⟦houses⟧ by the sea.");
  });

  it("marks nothing for a word the pack has no gloss for, or whose gloss is not in the sentence", () => {
    expect(glossMark("garden", "A garden.", glossTables, stopWords("en"))).toEqual({
      lemma: "garden",
      gloss: null,
      marks: [],
    });
    expect(glossMark("casa", "A garden.", glossTables, stopWords("en")).marks).toEqual([]);
  });
});

describe("the loop answers a trap as the extension answers it (D2)", () => {
  const sentence = "She gave up after the third attempt.";
  const selections: Selection[] = [
    { k: 0, id: "s1", word: "gave", upos: "VERB", sentence, selection: { start: 4, end: 11 } },
    { k: 1, id: "s2", word: "attempt", upos: "NOUN", sentence, selection: { start: 28, end: 35 } },
  ];
  const answers: Record<string, string> = {
    "She <b>gave up</b> after the third attempt.": "Elle <b>a abandonné</b> après la troisième tentative.",
    "gave up": "abandonné",
    "She gave up after the third <b>attempt</b>.": "Elle a abandonné après la troisième <b>tentative</b>.",
    attempt: "tentative",
  };
  const trap = () => new WebAssembly.RuntimeError("memory access out of bounds");

  /**
   * A factory of fake engines, numbered from 1: `traps(engine, markup)` says whether that engine
   * traps on that request. Each request is recorded with the engine that answered — or trapped on — it.
   */
  function fakeEngine(traps: (engine: number, markup: string) => boolean) {
    const asked: { engine: number; markup: string }[] = [];
    let built = 0;
    const engine = (): Translate => {
      const n = ++built;
      return (markup) => {
        asked.push({ engine: n, markup });
        if (traps(n, markup)) throw trap();
        return answers[markup] ?? markup;
      };
    };
    return { engine, asked, built: () => built };
  }

  it("measures a selection as relay.ts marks it: tagged sentence, fragment alone, reconciled", async () => {
    const fake = fakeEngine(() => false);
    const lines = await measureSelections(selections, { engine: fake.engine });
    expect(fake.built()).toBe(1);
    expect(fake.asked.map((a) => a.markup)).toEqual(Object.keys(answers));
    expect(lines[0]).toEqual({
      k: 0,
      id: "s1",
      word: "gave",
      upos: "VERB",
      sentence,
      trapped: false,
      translation: "Elle a abandonné après la troisième tentative.",
      marks: ["a abandonné"],
      shown: "Elle ⟦a abandonné⟧ après la troisième tentative.",
      engineMarks: ["a abandonné"],
      alone: "abandonné",
      gloss: null, // no gloss table for the pair: the experiment's columns stay empty (D3)
    });
    expect(lines[1].marks).toEqual(["tentative"]);
  });

  it("A selection the engine traps on: the sentence is asked once more on a fresh engine, and the poisoned one is never asked again", async () => {
    const fake = fakeEngine((engine, markup) => engine === 1 && markup.startsWith("She <b>gave"));
    const lines = await measureSelections(selections, { engine: fake.engine });
    expect(lines[0].trapped).toBe(false);
    expect(lines[0].marks).toEqual(["a abandonné"]);
    expect(fake.built()).toBe(2);
    // The first engine saw the sentence and trapped; everything after went to the second.
    expect(fake.asked.map((a) => a.engine)).toEqual([1, 2, 2, 2, 2]);
  });

  it("A selection the engine traps on twice: recorded trapped, no translation and no mark, and the measurement goes on", async () => {
    const fake = fakeEngine((engine, markup) => engine <= 2 && markup.startsWith("She <b>gave"));
    const log: string[] = [];
    const lines = await measureSelections(selections, { engine: fake.engine, log: (m) => log.push(m) });
    expect(lines[0]).toEqual({
      k: 0,
      id: "s1",
      word: "gave",
      upos: "VERB",
      sentence,
      trapped: true,
      translation: null,
      marks: [],
      shown: null,
      engineMarks: [],
      alone: null,
      gloss: null,
    });
    // The fragment of a trapped sentence is not asked; the next selection gets a third, clean engine.
    expect(fake.asked.map((a) => [a.engine, a.markup.slice(0, 12)])).toEqual([
      [1, "She <b>gave "],
      [2, "She <b>gave "],
      [3, "She gave up "],
      [3, "attempt"],
    ]);
    expect(lines[1].trapped).toBe(false);
    expect(lines[1].marks).toEqual(["tentative"]);
    expect(log.some((m) => m.includes("s1: trapped twice"))).toBe(true);
    expect(log.join("\n")).not.toContain("gave up"); // the markup's length, never its text
  });

  it("a fragment that traps twice leaves the sentence's own marks unreconciled, as the extension shows them", async () => {
    // An engine whose tag lands on the neighbour: reconciled, the mark would move to « rarement ».
    const friday = "They seldom ship on Friday.";
    const items: Selection[] = [
      { k: 0, id: "f1", word: "seldom", upos: "ADV", sentence: friday, selection: { start: 5, end: 11 } },
    ];
    const asked: number[] = [];
    let built = 0;
    const engine = (): Translate => {
      const n = ++built;
      return (markup) => {
        asked.push(n);
        if (markup === "seldom") throw trap();
        return "Ils <b>expédient</b> rarement le vendredi.";
      };
    };
    const [line] = await measureSelections(items, { engine });
    expect(line.trapped).toBe(false);
    expect(line.alone).toBeNull();
    expect(line.marks).toEqual(["expédient"]); // the engine's own, unreconciled
    expect(line.engineMarks).toEqual(["expédient"]);
    expect(asked).toEqual([1, 1, 2]); // the fragment once more on a fresh engine, then given up
    expect(built).toBe(2);
  });

  it("fills the experiment's columns when the pair's gloss is given", async () => {
    const fake = fakeEngine(() => false);
    const gloss = (word: string, translation: string) => ({
      lemma: word,
      gloss: "abandonner",
      marks: [{ start: translation.indexOf("abandonné"), end: translation.indexOf("abandonné") + 9 }],
    });
    const [line] = await measureSelections(selections.slice(0, 1), { engine: fake.engine, gloss });
    expect(line.gloss).toEqual({
      lemma: "gave",
      text: "abandonner",
      marks: ["abandonné"],
      shown: "Elle a ⟦abandonné⟧ après la troisième tentative.",
    });
  });

  it("throws anything that is not a trap", async () => {
    const engine = (): Translate => () => {
      throw new Error("the model is not on this device");
    };
    await expect(measureSelections(selections, { engine })).rejects.toThrow("not on this device");
  });
});
