/// <reference types="vite/client" />
import { describe, expect, it } from "vitest";
import {
  DEFAULT_INTERFACE_LANGUAGE,
  formatDate,
  formatNumber,
  formatPercent,
  INTERFACE_LANGUAGE_KEY,
  type InterfaceLanguage,
  interfaceLanguage,
  isInterfaceLanguage,
  plural,
  type PluralForms,
  setDocumentLanguage,
  SURFACES,
} from "@/i18n/index.ts";
import { popup as frPopup } from "@/i18n/fr/popup.ts";
import { review as frReview } from "@/i18n/fr/review.ts";
import { review as enReview } from "@/i18n/en/review.ts";
import { review as esReview } from "@/i18n/es/review.ts";
import { stats as frStats } from "@/i18n/fr/stats.ts";
import { sync as frSync } from "@/i18n/fr/sync.ts";
import { translation as frTranslation } from "@/i18n/fr/translation.ts";
import { account as frAccount } from "@/i18n/fr/account.ts";

// The catalogue (add-lingua-interface-language): the key every surface reads before its copy, the
// helpers a count and a figure go through, and the parity of the English and Spanish drafts with the
// French source — what the compiler says about the key set, this says about the values.

const NNBSP = "\u202F";
const NBSP = "\u00A0";
/** The narrow no-break space `fr-FR` groups with (what the surfaces write today). */
const NARROW = (1000).toLocaleString("fr-FR").charAt(1);

/** A surface's preferences area, with no engine behind it. */
function fakeArea(seed: Record<string, unknown> = {}) {
  return {
    async get(key: string) {
      return key in seed ? { [key]: seed[key] } : {};
    },
  };
}

describe("the interface language, under its own key", () => {
  it("Every installed reader: an absent key reads French", async () => {
    expect(await interfaceLanguage(fakeArea())).toBe("fr");
    expect(DEFAULT_INTERFACE_LANGUAGE).toBe("fr");
    expect(INTERFACE_LANGUAGE_KEY).toBe("cymbra-lingua-interface-language");
  });

  it("A surface before any engine: the key is read from the area alone", async () => {
    expect(await interfaceLanguage(fakeArea({ [INTERFACE_LANGUAGE_KEY]: "es" }))).toBe("es");
    expect(await interfaceLanguage(fakeArea({ [INTERFACE_LANGUAGE_KEY]: "en" }))).toBe("en");
  });

  it("an unknown value reads French too", async () => {
    expect(await interfaceLanguage(fakeArea({ [INTERFACE_LANGUAGE_KEY]: "de" }))).toBe("fr");
    expect(await interfaceLanguage(fakeArea({ [INTERFACE_LANGUAGE_KEY]: 3 }))).toBe("fr");
    expect(isInterfaceLanguage("es")).toBe(true);
    expect(isInterfaceLanguage("pt")).toBe(false);
  });

  it("names the document's language", () => {
    setDocumentLanguage(document, "es");
    expect(document.documentElement.lang).toBe("es");
    setDocumentLanguage(document, "fr");
    expect(document.documentElement.lang).toBe("fr");
  });
});

describe("a count in each language", () => {
  it("A count in each language: the review's remaining cards for 1 and 3", () => {
    expect(plural("fr", 1, frReview.remaining)).toBe("1 carte(s) à revoir");
    expect(plural("fr", 3, frReview.remaining)).toBe("3 carte(s) à revoir");
    expect(plural("en", 1, enReview.remaining)).toBe("1 card to review");
    expect(plural("en", 3, enReview.remaining)).toBe("3 cards to review");
    expect(plural("es", 1, esReview.remaining)).toBe("1 tarjeta por repasar");
    expect(plural("es", 3, esReview.remaining)).toBe("3 tarjetas por repasar");
  });

  it("A French format Intl would change: the raw count, the megabytes, the minutes", () => {
    expect(plural("fr", 1234, frReview.remaining)).toBe("1234 carte(s) à revoir");
    expect(plural("en", 1234, enReview.remaining)).toBe("1,234 cards to review");
    expect(plural("es", 1234, esReview.remaining)).toBe("1234 tarjetas por repasar");
    const oneDecimal = { minimumFractionDigits: 1, maximumFractionDigits: 1 };
    expect(frTranslation.megabytes(formatNumber("fr", 25.8, oneDecimal))).toBe("25,8 Mo");
    expect(frSync.syncedMinutesAgo("3")).toBe("Synchronisé il y a 3 min.");
    expect(frSync.lastSyncOn(formatDate("fr", new Date(2026, 9, 4)))).toBe("Dernière synchronisation le 04/10/2026.");
  });

  it("many, for French and Spanish at a million; English falls back to other", () => {
    const forms: PluralForms = { one: (n) => `one ${n}`, many: (n) => `many ${n}`, other: (n) => `other ${n}` };
    expect(plural("fr", 1_000_000, forms)).toBe("many 1000000");
    expect(plural("es", 1_000_000, forms)).toBe(`many 1${NNBSP}000${NNBSP}000`);
    expect(plural("en", 1_000_000, forms)).toBe("other 1,000,000");
    const twoForms: PluralForms = { one: (n) => `one ${n}`, other: (n) => `other ${n}` };
    expect(plural("fr", 1_000_000, twoForms)).toBe("other 1000000");
    expect(plural("fr", 0, twoForms)).toBe("one 0");
  });

  it("formats a figure as each language writes it, the Spanish as the RAE does", () => {
    expect(formatNumber("fr", 20_000)).toBe(`20${NARROW}000`);
    expect(formatNumber("en", 20_000)).toBe("20,000");
    expect(formatNumber("es", 5_000)).toBe("5000");
    expect(formatNumber("es", 20_000)).toBe(`20${NNBSP}000`);
    expect(formatNumber("es", 1_234_567)).toBe(`1${NNBSP}234${NNBSP}567`);
    expect(formatNumber("es", -12_345)).toBe(`-12${NNBSP}345`);
    const oneDecimal = { minimumFractionDigits: 1, maximumFractionDigits: 1 };
    expect(formatNumber("es", 25.8, oneDecimal)).toBe("25,8");
    expect(formatNumber("es", 12345.6, oneDecimal)).toBe(`12${NNBSP}345,6`);
    expect(formatNumber("en", 25.8, oneDecimal)).toBe("25.8");
  });

  it("writes a percentage in each surface's French form, the Spanish with its space", () => {
    expect(formatPercent("fr", 45, "tight")).toBe("45%");
    expect(formatPercent("fr", 110, "spaced")).toBe("110 %");
    expect(formatPercent("en", 96, "spaced")).toBe("96%");
    expect(formatPercent("es", 96, "tight")).toBe(`96${NNBSP}%`);
  });

  it("writes a date through each language's locale", () => {
    const date = new Date(2026, 9, 4);
    const long = { day: "numeric", month: "long", year: "numeric" } as const;
    expect(frAccount.linkedOn(formatDate("fr", date, long))).toBe("Lié le 4 octobre 2026");
    expect(formatDate("en", date)).toBe("10/4/2026");
    expect(formatDate("en", date, long)).toBe("October 4, 2026");
    expect(formatDate("es", date)).toBe("4/10/2026");
    expect(formatDate("es", date, long)).toBe("4 de octubre de 2026");
  });

  it("keeps the ladder's escapes as the surface wrote them", () => {
    expect(frStats.scopeCommon).toBe(
      `«${NNBSP}courants${NNBSP}»${NBSP}: les mots les plus fréquents jusqu'à ce niveau. `,
    );
    expect(frStats.approxWords("16" + NARROW + "000")).toBe(`≈${NBSP}16${NARROW}000 mots`);
    expect(plural("fr", 20_000, frStats.words, formatNumber("fr", 20_000))).toBe(`20${NARROW}000 mots`);
  });
});

// — Parity of the drafts with the French (D6) —

type Catalogue = Record<string, unknown>;
const modules = import.meta.glob<Catalogue>("../src/i18n/{fr,en,es}/*.ts", { eager: true });

function catalogue(language: InterfaceLanguage, surface: string): Catalogue {
  const mod = modules[`../src/i18n/${language}/${surface}.ts`];
  if (!mod) throw new Error(`no ${language} module for ${surface}`);
  const names = Object.keys(mod);
  expect(names, `${language}/${surface} exports one object`).toHaveLength(1);
  return mod[names[0]] as Catalogue;
}

/** Texts that are the same in every language: names, symbols, separators, spoken previews. */
const SAME_EVERYWHERE = new Set([
  "Cymbra Lingua",
  "Cymbra",
  "Google",
  "Apple",
  "—",
  "–",
  "…",
  "⚙",
  "⌄",
  "×",
  "‹",
  "›",
  "✕",
  "ⓘ",
  "Aa",
  "A−",
  "A+",
  "Alt",
  "S",
  "D",
  "L",
  "+",
  "/",
  "; ",
  " · ",
  ", ",
  " + ",
  "This is how your pages will sound when Lingua reads them aloud.",
  "Así sonarán tus páginas cuando Lingua las lea en voz alta.",
]);

/** Texts a language happens to share with the French, and no other. */
const SAME_AS_FRENCH: Record<"en" | "es", Set<string>> = {
  en: new Set(["Email", "Stats", "Option", "Double", "+ Deck", "interjection"]),
  es: new Set(["la"]),
};

/** Slot messages whose text is only their parts and a symbol — the same shape in every language. */
const SAME_SHAPE_EVERYWHERE = new Set(["@α", "▶ α", "α → β", "α / β", `≈${NBSP}α`, "α…", "α β", "α β γ"]);
const SAME_SHAPE_AS_FRENCH: Record<"en" | "es", Set<string>> = {
  en: new Set(),
  es: new Set(["de α"]),
};

const ARGS = ["α", "β", "γ", "δ"];

/** Every text a value can produce, slot messages called with `ARGS`, plural forms with "7". */
function texts(value: unknown, path: string): { path: string; text: string; slots: number }[] {
  if (typeof value === "string") return [{ path, text: value, slots: 0 }];
  if (typeof value === "function") {
    const out = (value as (...args: string[]) => unknown)(...ARGS.slice(0, Math.max(1, value.length)));
    return texts(out, `${path}()`).map((t) => ({ ...t, slots: Math.max(1, value.length) }));
  }
  if (value && typeof value === "object") {
    return Object.entries(value as Catalogue).flatMap(([key, v]) => texts(v, `${path}.${key}`));
  }
  throw new Error(`${path}: a catalogue value is a string, a function or an object of them`);
}

/** The key set of a value, nested; a plural form's `many` is optional. */
function shape(value: unknown): unknown {
  if (typeof value === "function") return shape((value as (...args: string[]) => unknown)(...ARGS));
  if (value && typeof value === "object") {
    const entries = Object.entries(value as Catalogue).filter(([key]) => key !== "many");
    return Object.fromEntries(entries.map(([key, v]) => [key, shape(v)]));
  }
  return typeof value;
}

describe("the drafts are whole", () => {
  it("has a module for every surface in every language, and no other", () => {
    for (const language of ["fr", "en", "es"] as const) {
      const found = Object.keys(modules)
        .filter((p) => p.includes(`/${language}/`))
        .map((p) => p.replace(/.*\//, "").replace(/\.ts$/, ""))
        .sort();
      expect(found, language).toEqual([...SURFACES].sort());
    }
  });

  for (const surface of SURFACES) {
    describe(surface, () => {
      const fr = catalogue("fr", surface);

      it("has the French key set in English and in Spanish, plural forms and slots included", () => {
        expect(shape(catalogue("en", surface))).toEqual(shape(fr));
        expect(shape(catalogue("es", surface))).toEqual(shape(fr));
      });

      for (const language of ["en", "es"] as const) {
        it(`The drafts are whole: ${language} is full, takes every slot, and is not the French`, () => {
          const draft = texts(catalogue(language, surface), surface);
          const source = new Map(texts(fr, surface).map((t) => [t.path, t]));
          for (const { path, text, slots } of draft) {
            // The `many` a translation adds answers the French `other` (the French has no `many`).
            const original = source.get(path) ?? source.get(path.replace(/\.many\(\)$/, ".other()"));
            expect(original, `${language}: ${path} has no French source`).toBeDefined();
            expect(text.trim(), `${language}: ${path} is empty`).not.toBe("");
            for (const arg of ARGS.slice(0, slots)) {
              expect(text, `${language}: ${path} drops the slot ${arg}`).toContain(arg);
            }
            if (text !== original!.text) continue;
            const allowed =
              SAME_EVERYWHERE.has(text) ||
              SAME_AS_FRENCH[language].has(text) ||
              SAME_SHAPE_EVERYWHERE.has(text) ||
              SAME_SHAPE_AS_FRENCH[language].has(text);
            expect(allowed, `${language}: ${path} is still the French « ${text} »`).toBe(true);
          }
        });
      }
    });
  }

  it("the lists of texts that are the same everywhere name only texts that are", () => {
    const equal = { en: new Set<string>(), es: new Set<string>() };
    for (const surface of SURFACES) {
      const source = new Map(texts(catalogue("fr", surface), surface).map((t) => [t.path, t.text]));
      for (const language of ["en", "es"] as const) {
        for (const { path, text } of texts(catalogue(language, surface), surface)) {
          if (source.get(path) === text) equal[language].add(text);
        }
      }
    }
    for (const text of [...SAME_EVERYWHERE, ...SAME_SHAPE_EVERYWHERE]) {
      expect(equal.en.has(text) && equal.es.has(text), `« ${text} » is not the same everywhere`).toBe(true);
    }
    for (const language of ["en", "es"] as const) {
      for (const text of [...SAME_AS_FRENCH[language], ...SAME_SHAPE_AS_FRENCH[language]]) {
        expect(equal[language].has(text), `${language}: « ${text} » is not the French`).toBe(true);
        const other = language === "en" ? "es" : "en";
        expect(equal[other].has(text), `« ${text} » is the same in both: list it as same everywhere`).toBe(false);
      }
    }
  });

  it("A message in three languages: the quoted « Réviser » is the French, the drafts differ", () => {
    expect(frReview.start).toBe("Réviser");
    expect(enReview.start).not.toBe("Réviser");
    expect(esReview.start).not.toBe("Réviser");
    expect(enReview.start).toBe("Review");
    expect(esReview.start).toBe("Repasar");
  });

  it("A key missing in Spanish does not compile", () => {
    // What `yarn typecheck` enforces: a draft typed after the French cannot leave a key out.
    const incomplete = Object.fromEntries(Object.entries(frPopup).filter(([key]) => key !== "settings")) as Omit<
      typeof frPopup,
      "settings"
    >;
    // @ts-expect-error — `settings` is missing from the draft.
    const draft: typeof frPopup = incomplete;
    expect(draft).toBeDefined();
  });
});
