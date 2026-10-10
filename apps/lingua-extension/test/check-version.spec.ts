import { join } from "node:path";
import { describe, expect, it } from "vitest";
import {
  LANGUAGE_NAMES,
  localeProblems,
  manifestProblems,
  namedLanguages,
  summaryProblems,
  versionProblem,
} from "../tool/check_version.mjs";
import { readLocales } from "../tool/locales.mjs";
import { shippedPairs } from "../tool/packs.mjs";
import { languages as enLanguages } from "../src/i18n/en/languages";
import { languages as esLanguages } from "../src/i18n/es/languages";
import { languages as frLanguages } from "../src/i18n/fr/languages";

const app = join(__dirname, "..");
const manifest = (await import("../manifest.json")) as unknown as Record<string, unknown>;
const locales = readLocales(app);

describe("the release version guard", () => {
  it("passes a version the stores accept", () => {
    expect(versionProblem("1.4.0")).toBeNull();
  });

  it("refuses a pre-release, which a store rejects at upload", () => {
    // The failure this exists to move earlier: the tag is already pushed and the build
    // already done by the time a store says no.
    expect(versionProblem("1.4.0-rc.1")).toContain("three plain integers");
  });

  it("refuses a part above what a manifest version may hold", () => {
    expect(versionProblem("1.70000.0")).toContain("65535");
  });

  it("refuses a leading zero", () => {
    expect(versionProblem("1.04.0")).toContain("leading zero");
  });

  it("refuses a version put back into the source manifest", () => {
    // build.mjs stamps package.json's version onto every variant. A copy here is a second
    // source that drifts — and it is what release-please used to rewrite, reformatting the
    // whole file and failing the Prettier gate.
    const problems = manifestProblems({ description: "ok", version: "0.1.0" });

    expect(problems.join(" ")).toContain("second source");
  });

  it("refuses a description Apple would reject, which only the upload used to reveal", () => {
    // 113 characters. Apple's limit is 112 and it is checked when the signed archive is
    // uploaded — lingua-apple-v1.1.0 died there, after a full build. Chrome allows 132, so
    // calibrating on Chrome is exactly how you get an archive Apple refuses.
    const problems = manifestProblems({ description: "x".repeat(113) });

    expect(problems).toHaveLength(1);
    expect(problems[0]).toContain("113 characters");
    expect(problems[0]).toContain("112");
  });

  it("accepts a description at the limit", () => {
    expect(manifestProblems({ description: "x".repeat(112) })).toEqual([]);
  });

  it("refuses a manifest with no description at all", () => {
    expect(manifestProblems({}).join(" ")).toContain("no description");
  });

  it("accepts a manifest that defers to package.json", () => {
    expect(manifestProblems({ description: "Lisez l'anglais sur le web." })).toEqual([]);
  });

  it("holds the files the repository actually ships", async () => {
    // The guard is only worth having if it runs against the real files; this is the same
    // set `yarn check:version` reads in CI.
    const pkg = (await import("../package.json")) as unknown as { version: string };

    expect(versionProblem(pkg.version)).toBeNull();
    expect(manifestProblems(manifest)).toEqual([]);
    expect(Object.keys(locales)).toEqual(["en", "es", "fr"]);
    expect(localeProblems(manifest, locales)).toEqual([]);
  });
});

describe("the committed `_locales` guard (localise-lingua-manifest D3)", () => {
  it("refuses a Spanish description Apple would reject, naming the language", () => {
    // The scenario *A description too long for Apple*: 113 characters in es alone.
    const problems = localeProblems(manifest, {
      ...locales,
      es: { ...locales.es, extensionDescription: { message: "x".repeat(113) } },
    });

    expect(problems).toHaveLength(1);
    expect(problems[0]).toContain("the es description");
    expect(problems[0]).toContain("_locales/es/messages.json");
    expect(problems[0]).toContain("113 characters");
    expect(problems[0]).toContain("112");
  });

  it("accepts every language's description at the limit", () => {
    const atLimit = Object.fromEntries(
      Object.entries(locales).map(([language, messages]) => [
        language,
        { ...messages, extensionDescription: { message: "x".repeat(112) } },
      ]),
    );
    expect(localeProblems({ ...manifest, description: "x".repeat(112) }, atLimit)).toEqual([]);
  });

  it("refuses a French message that differs from the manifest's literal, naming the key", () => {
    // The French has two homes: the literal every package carries today, and the message a
    // localised package reads. A drift between them would change the text at the first localised build.
    const problems = localeProblems(manifest, {
      ...locales,
      fr: { ...locales.fr, commandSidePanel: { message: "Ouvrir le panneau latéral" } },
    });

    expect(problems).toHaveLength(1);
    expect(problems[0]).toContain('"commandSidePanel"');
    expect(problems[0]).toContain('"Ouvrir le panneau latéral"');
    expect(problems[0]).toContain("held equal");
    expect(localeProblems({ ...manifest, description: "Autre" }, locales).join(" ")).toContain(
      '"extensionDescription"',
    );
  });

  it("refuses a language with no description, and a missing French folder", () => {
    const { extensionDescription: _dropped, ...noDescription } = locales.en!;
    void _dropped;
    expect(localeProblems(manifest, { ...locales, en: noDescription }).join(" ")).toContain(
      '_locales/en/messages.json has no "extensionDescription" message',
    );
    const { fr: _french, ...withoutFrench } = locales;
    void _french;
    expect(localeProblems(manifest, withoutFrench).join(" ")).toContain("_locales/fr/messages.json is missing");
  });
});

describe("the stores' summary names what its readers can study (add-lingua-french-listings D6)", () => {
  // The drafts change 52 (enable-lingua-french) commits with the pairs that make them true: the
  // English one with fr-en, the Spanish one with fr-es.
  const EN_DRAFT =
    "Read Spanish and French on the web: unknown words highlighted, an honest percentage. Offline and private.";
  const ES_DRAFT =
    "Lee inglés y francés en la web: palabras desconocidas resaltadas, porcentaje honesto. Sin conexión y privado.";
  const TODAY = ["en-fr", "es-fr"];
  // Stage 2 as changes 34 and 35 list it: es-en, then en-es.
  const STAGE_2 = [...TODAY, "es-en", "en-es"];
  // Change 52 lists French last: fr-en, then fr-es if it ships.
  const WITH_FR_EN = [...STAGE_2, "fr-en"];
  const WITH_FR_ES = [...WITH_FR_EN, "fr-es"];
  const reading = (language: string, message: string) => ({
    ...locales,
    [language]: { ...locales[language], extensionDescription: { message } },
  });

  it("holds the drafts within Apple's limit, as the committed descriptions are", () => {
    expect(EN_DRAFT).toHaveLength(105);
    expect(ES_DRAFT).toHaveLength(109);
    expect(localeProblems(manifest, reading("es", ES_DRAFT))).toEqual([]);
    expect(localeProblems(manifest, reading("en", EN_DRAFT))).toEqual([]);
  });

  it("today's pairs pass: the French description names English and Spanish, in both homes", () => {
    expect(summaryProblems(manifest, locales, TODAY)).toEqual([]);
    // English and Spanish are not read while no pair is glossed in them.
    expect(summaryProblems(manifest, reading("en", "Read French on the web."), TODAY)).toEqual([]);
  });

  it("es-en and en-es listed pass with the committed descriptions", () => {
    expect(summaryProblems(manifest, locales, [...TODAY, "es-en"])).toEqual([]);
    expect(summaryProblems(manifest, locales, STAGE_2)).toEqual([]);
  });

  it("fr-en listed with the committed English description fails for English, naming Spanish and the pairs' Spanish and French", () => {
    const problems = summaryProblems(manifest, locales, WITH_FR_EN);
    expect(problems).toHaveLength(1);
    expect(problems[0]).toContain("the en description (_locales/en/messages.json) names Spanish while");
    expect(problems[0]).toContain("(es-en, fr-en) study Spanish and French");
    expect(problems[0]).toContain("it does not name French");
  });

  it("fr-en listed with the English draft passes", () => {
    expect(summaryProblems(manifest, reading("en", EN_DRAFT), WITH_FR_EN)).toEqual([]);
  });

  it("the English draft before fr-en is listed fails for English: it names French, which no English-glossed pair studies", () => {
    const problems = summaryProblems(manifest, reading("en", EN_DRAFT), STAGE_2);
    expect(problems).toHaveLength(1);
    expect(problems[0]).toContain("the en description (_locales/en/messages.json) names Spanish and French while");
    expect(problems[0]).toContain("(es-en) study Spanish");
    expect(problems[0]).toContain("it names French, which no shipped pair glossed in en studies");
  });

  it("fr-es listed with the committed Spanish description fails for Spanish; with the Spanish draft it passes", () => {
    const committedSpanish = summaryProblems(manifest, reading("en", EN_DRAFT), WITH_FR_ES);
    expect(committedSpanish).toHaveLength(1);
    expect(committedSpanish[0]).toContain("the es description (_locales/es/messages.json) names Inglés while");
    expect(committedSpanish[0]).toContain("(en-es, fr-es) study Inglés and Francés");
    expect(committedSpanish[0]).toContain("it does not name Francés");
    const both = { ...reading("en", EN_DRAFT), es: { ...locales.es, extensionDescription: { message: ES_DRAFT } } };
    expect(summaryProblems(manifest, both, WITH_FR_ES)).toEqual([]);
  });

  it("the Spanish draft with fr-es unlisted fails for Spanish: it names French, which no Spanish-glossed pair studies", () => {
    const both = { ...reading("en", EN_DRAFT), es: { ...locales.es, extensionDescription: { message: ES_DRAFT } } };
    const problems = summaryProblems(manifest, both, WITH_FR_EN);
    expect(problems).toHaveLength(1);
    expect(problems[0]).toContain("the es description (_locales/es/messages.json) names Inglés and Francés while");
    expect(problems[0]).toContain("it names Francés, which no shipped pair glossed in es studies");
  });

  it("reads the French in both homes: manifest.json's literal is held too", () => {
    const problems = summaryProblems({ ...manifest, description: "Lisez l'anglais sur le web." }, locales, TODAY);
    expect(problems).toHaveLength(1);
    expect(problems[0]).toContain("the fr description (manifest.json) names Anglais while");
    expect(problems[0]).toContain("it does not name Espagnol");
  });

  it("refuses a shipped native with no description, and one whose names it does not hold", () => {
    const { en: _english, ...withoutEnglish } = locales;
    void _english;
    expect(summaryProblems(manifest, withoutEnglish, STAGE_2).join(" ")).toContain(
      "the en description (_locales/en/messages.json) is missing, yet es-en ship glossed in en",
    );
    expect(summaryProblems(manifest, locales, [...TODAY, "en-de"]).join(" ")).toContain(
      'en-de ship glossed in "de", whose names for the studied languages tool/check_version.mjs does not hold',
    );
  });

  it("matches a name whole and whatever its case", () => {
    expect(namedLanguages("fr", "Lisez l'anglais et l'ESPAGNOL")).toEqual(["en", "es"]);
    expect(namedLanguages("en", "READ SPANISH; french too")).toEqual(["es", "fr"]);
    expect(namedLanguages("en", "Spanishness, Frenchified, Englishes")).toEqual([]);
    expect(namedLanguages("es", "Lee INGLÉS y francés")).toEqual(["en", "fr"]);
    expect(namedLanguages("es", "inglesa")).toEqual([]);
  });

  it("names each language as the interface's catalogue does", () => {
    const catalogue = (words: typeof frLanguages) => ({
      en: words.english.name,
      es: words.spanish.name,
      fr: words.french.name,
    });
    expect(LANGUAGE_NAMES).toEqual({
      fr: catalogue(frLanguages),
      en: catalogue(enLanguages),
      es: catalogue(esLanguages),
    });
  });

  it("holds the files the repository actually ships", () => {
    expect(summaryProblems(manifest, locales, shippedPairs(app))).toEqual([]);
  });
});
