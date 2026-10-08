import { readFileSync } from "node:fs";
import { join } from "node:path";
import { describe, expect, it } from "vitest";
import type { GrammarTag, StudiedLanguage, WordGrammar } from "@/analyzer/types.ts";
import {
  describeForm,
  FEATURES,
  finiteKey,
  formKind,
  isDictionaryForm,
  PARTS_OF_SPEECH,
  readingOf,
  tagsOf,
  tenseOrder,
} from "@/reading/grammar-description.ts";
import type { GrammarRenderer, StudiedLanguageCode } from "@/i18n/index.ts";
import { grammar as en } from "@/i18n/en/grammar.ts";
import { grammar as es } from "@/i18n/es/grammar.ts";
import { grammar as fr } from "@/i18n/fr/grammar.ts";
import { createCard } from "@/reading/wordpopup.ts";

// generalise-lingua-card-wording D1: a form described once, in no language — the readings as the
// engine's tags, merged and deduplicated by tag, the dictionary form left out on its own card — and
// "What each renderer names": the closed vocabulary enumerated, every renderer naming what the
// French card names, no more.

const grammar = (over: Partial<WordGrammar> = {}): WordGrammar => ({
  gloss: null,
  senses: [],
  readings: [],
  others: [],
  pieces: [],
  ...over,
});
const fin = (Mood: string, Person: string, Number: string, Tense?: string): GrammarTag => ({
  pos: "VERB",
  features: { Mood, Number, Person, ...(Tense ? { Tense } : {}), VerbForm: "Fin" },
});
const PAST: GrammarTag = { pos: "VERB", features: { Mood: "Ind", Tense: "Past", VerbForm: "Fin" } };

describe("the description of a form names no language", () => {
  it("merges the persons of one tag, and keeps the pack's order of first occurrence", () => {
    const form = describeForm(
      grammar({
        readings: [fin("Imp", "3", "Sing"), fin("Sub", "1", "Sing", "Pres"), fin("Sub", "3", "Sing", "Pres")],
      }),
      "hablar",
      "hable",
      "hable",
    );
    expect(form.own).toEqual([
      { pos: "VERB", mood: "Imp", number: "Sing", persons: ["3"], verbForm: "Fin" },
      { pos: "VERB", mood: "Sub", number: "Sing", persons: ["1", "3"], tense: "Pres", verbForm: "Fin" },
    ]);
    expect(form.sameAsHeadword).toBe(false);
    expect(JSON.stringify(form)).not.toMatch(/subjonctif|subjunctive|subjuntivo|personne|person\b/);
  });

  it("drops a tag seen twice, and a person seen twice", () => {
    const form = describeForm(
      grammar({ readings: [PAST, PAST, fin("Ind", "3", "Sing", "Pres"), fin("Ind", "3", "Sing", "Pres")] }),
      "go",
      "went",
      "went",
    );
    expect(form.own).toHaveLength(2);
    expect(form.own[1]!.persons).toEqual(["3"]);
  });

  it("keeps apart readings that differ by more than a person", () => {
    const form = describeForm(
      grammar({ readings: [fin("Ind", "1", "Sing", "Imp"), fin("Ind", "1", "Plur", "Imp")] }),
      "hablar",
      "x",
      "x",
    );
    expect(form.own.map((r) => r.number)).toEqual(["Sing", "Plur"]);
    // A person outside the vocabulary merges with nothing.
    const odd = describeForm(
      grammar({ readings: [fin("Ind", "3", "Sing", "Imp"), fin("Ind", "9", "Sing", "Imp")] }),
      "h",
      "x",
      "x",
    );
    expect(odd.own.map((r) => r.persons)).toEqual([["3"], ["9"]]);
  });

  it("leaves the dictionary form out on its own card, and only there", () => {
    const inf = { pos: "VERB", features: { VerbForm: "Inf" } };
    expect(describeForm(grammar({ readings: [inf] }), "hablar", " Hablar ", "Hablar").own).toEqual([]);
    expect(describeForm(grammar({ readings: [inf] }), "hablar", "hablarlo", "hablarlo").own).toHaveLength(1);
    const same = describeForm(grammar({ readings: [inf], others: [{ lemma: "x", readings: [inf] }] }), "a", "a", "a");
    expect(same.sameAsHeadword).toBe(true);
    expect(same.others[0]!.readings).toHaveLength(1);
  });

  it("knows the dictionary forms: an infinitive, a noun's singular, an adjective's masculine singular", () => {
    expect(isDictionaryForm({ pos: "VERB", features: { VerbForm: "Inf" } })).toBe(true);
    expect(isDictionaryForm({ pos: "NOUN", features: { Gender: "Fem", Number: "Sing" } })).toBe(true);
    expect(isDictionaryForm({ pos: "ADJ", features: { Gender: "Masc", Number: "Sing" } })).toBe(true);
    expect(isDictionaryForm({ pos: "ADJ", features: { Gender: "Fem", Number: "Sing" } })).toBe(false);
    expect(isDictionaryForm({ pos: "ADJ", features: { Number: "Sing", Degree: "Cmp" } })).toBe(false);
    expect(isDictionaryForm({ pos: "VERB", features: { Number: "Sing" } })).toBe(false);
  });

  it("keeps the pieces of a split word only, and the senses' headings as tags", () => {
    const split = describeForm(
      grammar({
        pieces: ["do", "not"],
        senses: [{ tag: { pos: "NOUN", features: { Gender: "Fem" } }, text: "x" }, { text: "y" }],
      }),
      "do",
      "do",
      "don't",
    );
    expect(split.pieces).toEqual(["do", "not"]);
    expect(split.written).toBe("don't");
    expect(split.senses).toEqual([{ pos: "NOUN", gender: "Fem" }, {}]);
    expect(describeForm(grammar({ pieces: ["go"] }), "go", "go", "go").pieces).toEqual([]);
  });

  it("carries every feature of the engine's vocabulary, and gives the tags back", () => {
    for (const [name, values] of Object.entries(FEATURES)) {
      for (const value of values) {
        const tag = { pos: "PRON", features: { [name]: value } };
        expect(Object.values(readingOf(tag)).flat(), `${name}=${value}`).toContain(value);
        expect(tagsOf(readingOf(tag))).toEqual([tag]);
      }
    }
    expect(readingOf({ pos: "X", features: { Aspect: "Perf" } })).toEqual({ pos: "X", persons: [] });
    expect(tagsOf(readingOf({ pos: "X" }))).toEqual([{ pos: "X" }]);
    // An empty value is no feature.
    expect(readingOf({ pos: "VERB", features: { Tense: "", Person: "", VerbForm: "Fin" } })).toEqual({
      pos: "VERB",
      persons: [],
      verbForm: "Fin",
    });
  });

  it("decides whether the infinitive is named: Spanish's is, English's never was", () => {
    const inf = readingOf({ pos: "VERB", features: { VerbForm: "Inf" } });
    expect(formKind(inf, "es")).toEqual({ kind: "infinitive" });
    expect(formKind(inf, "en")).toBeNull();
    for (const renderer of [fr, en, es]) {
      expect(renderer.readingName({ pos: "VERB", features: { VerbForm: "Inf" } }, "en")).toBeNull();
      expect(renderer.readingName({ pos: "VERB", features: { VerbForm: "Inf" } }, "es")).not.toBeNull();
    }
  });

  it("orders a studied language's tenses once, from the renderer's table: Spanish's as listed, English's as the pack has them", () => {
    const tenses = { en: { "Ind/Past": "a", "Ind/Pres": "b" }, es: { "Sub/Pres": "c", "Ind/Pres": "d", "Imp/": "e" } };
    expect(tenseOrder(tenses, "es")).toEqual(["Sub/Pres", "Ind/Pres", "Imp/"]);
    expect(tenseOrder(tenses, "en")).toEqual([]);
  });

  it("keys a finite form by the studied language's moods", () => {
    const r = (features: Record<string, string>) =>
      readingOf({ pos: "VERB", features: { VerbForm: "Fin", ...features } });
    expect(finiteKey(r({ Tense: "Past" }), "en")).toBe("Ind/Past");
    expect(finiteKey(r({ Mood: "Sub", Tense: "Past" }), "en")).toBeUndefined();
    expect(finiteKey(r({}), "en")).toBeUndefined();
    expect(finiteKey(r({ Mood: "Cnd", Tense: "Pres" }), "es")).toBe("Cnd/");
    expect(finiteKey(r({ Mood: "Sub", Tense: "Imp" }), "es")).toBe("Sub/Imp");
    expect(formKind(r({ Mood: "Sub" }), "en")).toBeNull();
  });
});

describe("the vocabulary is lingua-core's", () => {
  // The engine's closed vocabulary, read from its source (as test/packs.spec.ts reads the analyser's
  // version): a part of speech or a feature added there is added here, or this fails.
  const grammarRs = readFileSync(join(__dirname, "../../..", "crates/lingua-core/src/packs/grammar.rs"), "utf8");
  const strings = (rust: string): string[] => [...rust.matchAll(/"([^"]*)"/g)].map((m) => m[1]!);

  it("PARTS_OF_SPEECH equals the Rust list, in its order", () => {
    const block = /pub const PARTS_OF_SPEECH[^=]*=\s*\[([\s\S]*?)\];/.exec(grammarRs)?.[1];
    expect(block).toBeDefined();
    expect(PARTS_OF_SPEECH).toEqual(strings(block!));
    expect(PARTS_OF_SPEECH).toHaveLength(17);
  });

  it("FEATURES equals the Rust table, names, values and order", () => {
    const block = /pub const FEATURES[^=]*=\s*&\[([\s\S]*?)\n\];/.exec(grammarRs)?.[1];
    expect(block).toBeDefined();
    const rust = Object.fromEntries(
      [...block!.matchAll(/\("(\w+)",\s*&\[([^\]]*)\]\)/g)].map((m) => [m[1]!, strings(m[2]!)]),
    );
    expect(Object.keys(rust)).toHaveLength(11);
    expect(FEATURES).toEqual(rust);
    expect(Object.keys(FEATURES)).toEqual(Object.keys(rust));
  });
});

// — What each renderer names —

/** The tags the vocabulary can make that a card might name: every part of speech, alone and with each feature, and the verbal and nominal combinations. */
function vocabularyTags(): GrammarTag[] {
  const tags: GrammarTag[] = [];
  for (const pos of [...PARTS_OF_SPEECH, "WORD"]) {
    tags.push({ pos });
    for (const [name, values] of Object.entries(FEATURES)) {
      for (const value of values) tags.push({ pos, features: { [name]: value } });
    }
    for (const Gender of [undefined, ...FEATURES.Gender!, "Other"]) {
      for (const Number of [undefined, ...FEATURES.Number!]) {
        for (const Degree of [undefined, "Cmp"]) {
          const features = Object.fromEntries(
            Object.entries({ Gender, Number, Degree }).filter(([, v]) => v !== undefined),
          ) as Record<string, string>;
          tags.push({ pos, features });
          tags.push({ pos, features: { ...features, Tense: "Past", VerbForm: "Part" } });
        }
      }
    }
  }
  for (const pos of ["VERB", "AUX"]) {
    for (const VerbForm of FEATURES.VerbForm!) {
      for (const Mood of [undefined, ...FEATURES.Mood!]) {
        for (const Tense of [undefined, ...FEATURES.Tense!]) {
          for (const Person of [undefined, ...FEATURES.Person!]) {
            for (const Number of [undefined, ...FEATURES.Number!]) {
              const features = Object.fromEntries(
                Object.entries({ Mood, Tense, Person, Number, VerbForm }).filter(([, v]) => v !== undefined),
              ) as Record<string, string>;
              tags.push({ pos, features });
            }
          }
        }
      }
    }
  }
  return tags;
}

describe("What each renderer names", () => {
  const renderers: [string, GrammarRenderer][] = [
    ["en", en],
    ["es", es],
  ];
  const tags = vocabularyTags();

  it("enumerates the vocabulary", () => {
    expect(tags.length).toBeGreaterThan(2000);
  });

  for (const studied of ["en", "es"] as StudiedLanguageCode[]) {
    for (const [language, renderer] of renderers) {
      it(`${language}, of ${studied}: names what the French names, and leaves unnamed what it leaves unnamed`, () => {
        let named = 0;
        for (const tag of tags) {
          const french = fr.readingName(tag, studied);
          const draft = renderer.readingName(tag, studied);
          expect(draft === null, `${JSON.stringify(tag)}`).toBe(french === null);
          if (french) named++;
          const lines = (r: GrammarRenderer) =>
            r.grammarLines(grammar({ readings: [tag] }), "lemma", "form", "form", studied);
          expect(lines(renderer).length, `${JSON.stringify(tag)}`).toBe(lines(fr).length);
          if (draft) expect(draft.name).not.toBe(french!.name);
        }
        expect(named).toBeGreaterThan(50);
      });
    }
  }

  for (const [language, renderer] of renderers) {
    it(`${language}: heads the same parts of speech as the French`, () => {
      for (const pos of [...PARTS_OF_SPEECH, "WORD"]) {
        for (const Gender of [undefined, ...FEATURES.Gender!]) {
          const tag = Gender ? { pos, features: { Gender } } : { pos };
          expect(renderer.senseHeading(tag) === null, pos).toBe(fr.senseHeading(tag) === null);
        }
      }
      expect(renderer.senseHeading(undefined)).toBeNull();
    });
  }

  it("a form the French card says nothing about gets no line in any language", () => {
    const silent = grammar({
      readings: [
        { pos: "PRON", features: { Case: "Dat", PronType: "Prs", Reflex: "Yes" } },
        { pos: "VERB", features: { Tense: "Pqp", VerbForm: "Fin" } },
      ],
    });
    for (const renderer of [fr, en, es]) {
      expect(renderer.grammarLines(silent, "se", "le", "le", "es")).toEqual([]);
    }
  });
});

describe("Every reader today", () => {
  const show = (language: StudiedLanguage | undefined, headword: string, surface: string, readings: GrammarTag[]) => {
    const card = createCard();
    card.show(
      {
        headword,
        surface,
        gloss: "x",
        rarity: "",
        sentence: "",
        rect: { left: 0, top: 0, bottom: 0 },
        grammar: grammar({ readings }),
        ...(language ? { language } : {}),
      },
      () => {},
    );
    const line = card.el.querySelector(".grammar-line")!;
    return {
      text: line.textContent,
      words: [...line.querySelectorAll("em")].map((em) => [em.textContent, em.getAttribute("lang")]),
      headword: card.el.querySelector(".headword")!.getAttribute("lang"),
    };
  };

  it("a reader of French reads the lines byte for byte as before, each studied word in its language", () => {
    expect(show(undefined, "go", "went", [PAST])).toEqual({
      text: "prétérit de go",
      words: [["go", "en"]],
      headword: "en",
    });
    expect(show("es", "venir", "vino", [fin("Ind", "3", "Sing", "Past")])).toEqual({
      text: "3e personne du singulier du passé simple de venir",
      words: [["venir", "es"]],
      headword: "es",
    });
    expect(show("es", "él", "ella", [{ pos: "PRON", features: { Gender: "Fem", Number: "Sing" } }]).text).toBe(
      "féminin singulier d’él",
    );
  });
});
