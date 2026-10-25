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
  // The descriptions changes 34 and 35 shipped, before change 52 named French.
  const EN_STAGE_2 = "Read Spanish on the web: unknown words highlighted, an honest percentage. Offline and private.";
  const ES_STAGE_2 =
    "Lee inglés en la web: palabras desconocidas resaltadas, porcentaje honesto. Sin conexión y privado.";
  const TODAY = ["en-fr", "es-fr"];
  // Stage 2 as changes 34 and 35 list it: es-en, then en-es.
  const STAGE_2 = [...TODAY, "es-en", "en-es"];
  // Change 52 lists French last: fr-en, then fr-es if it ships.
  const WITH_FR_EN = [...STAGE_2, "fr-en"];
  const WITH_FR_ES = [...WITH_FR_EN, "fr-es"];
  const WITH_IT_FR = [...WITH_FR_ES, "it-fr"];
  // enable-lingua-italian-for-english-speakers lists it-en after it-fr.
  const WITH_IT_EN = [...WITH_IT_FR, "it-en"];
  // The English description that names Italian (enable-lingua-italian-for-english-speakers D6, the
  // owner's version A): the earlier lists are read with change 52's, EN_DRAFT.
  const EN_ITALIAN =
    "The web in Spanish, French or Italian: unknown words highlighted, an honest percentage. Offline and private.";
  // enable-lingua-italian-for-spanish-speakers lists it-es after it-en.
  const WITH_IT_ES = [...WITH_IT_EN, "it-es"];
  // The Spanish description that names Italian (enable-lingua-italian-for-spanish-speakers D6, the
  // owner's words, 2026-10-25): the earlier lists are read with change 52's, ES_DRAFT.
  const ES_ITALIAN =
    "La web en inglés, francés o italiano: palabras desconocidas resaltadas, porcentaje honesto. Sin conexión.";
  // The French description change 52 shipped, before enable-lingua-italian named Italian, and the
  // draft that names it (enable-lingua-italian D7): the earlier lists are read with the former.
  const FR_FRENCH =
    "Lisez l'anglais et l'espagnol sur le web : mots inconnus surlignés, pourcentage honnête. Hors ligne et privé.";
  const FR_DRAFT =
    "Le web en anglais, espagnol ou italien : mots inconnus surlignés, pourcentage honnête. Hors ligne et privé.";
  // Change 52 lists French last; enable-lingua-italian lists it-fr after it.
  const reading = (language: string, message: string, from = locales) => ({
    ...from,
    [language]: { ...from[language], extensionDescription: { message } },
  });
  /** manifest.json and the locales as change 52 shipped them: the French description naming no Italian. */
  const manifest52 = { ...manifest, description: FR_FRENCH };
  const locales52 = reading("fr", FR_FRENCH);
  /** The locales as change 35 shipped them: English and Spanish naming no French. */
  const stage2 = reading("es", ES_STAGE_2, reading("en", EN_STAGE_2, locales52));

  it("commits the drafts with the French pairs (enable-lingua-french D7), each since replaced by the one naming Italian", () => {
    expect(locales.en?.extensionDescription?.message).toBe(EN_ITALIAN);
    expect(locales.es?.extensionDescription?.message).toBe(ES_ITALIAN);
  });

  it("holds the drafts within Apple's limit, as the committed descriptions are", () => {
    expect(EN_DRAFT).toHaveLength(105);
    expect(ES_DRAFT).toHaveLength(109);
    expect(localeProblems(manifest, reading("es", ES_DRAFT))).toEqual([]);
    expect(localeProblems(manifest, reading("en", EN_DRAFT))).toEqual([]);
  });

  it("the pairs before change 34 pass with change 52's French description, in both homes", () => {
    expect(summaryProblems(manifest52, locales52, TODAY)).toEqual([]);
    // English and Spanish are not read while no pair is glossed in them.
    expect(summaryProblems(manifest52, reading("en", "Read French on the web.", locales52), TODAY)).toEqual([]);
  });

  it("es-en and en-es listed pass with change 35's descriptions", () => {
    expect(summaryProblems(manifest52, stage2, [...TODAY, "es-en"])).toEqual([]);
    expect(summaryProblems(manifest52, stage2, STAGE_2)).toEqual([]);
  });

  it("fr-en listed with change 35's English description fails for English, naming Spanish and the pairs' Spanish and French", () => {
    const problems = summaryProblems(manifest52, stage2, WITH_FR_EN);
    expect(problems).toHaveLength(1);
    expect(problems[0]).toContain("the en description (_locales/en/messages.json) names Spanish while");
    expect(problems[0]).toContain("(es-en, fr-en) study Spanish and French");
    expect(problems[0]).toContain("it does not name French");
  });

  it("fr-en listed with the English draft passes", () => {
    expect(summaryProblems(manifest52, reading("en", EN_DRAFT, stage2), WITH_FR_EN)).toEqual([]);
  });

  it("the English draft before fr-en is listed fails for English: it names French, which no English-glossed pair studies", () => {
    const problems = summaryProblems(manifest52, reading("en", EN_DRAFT, stage2), STAGE_2);
    expect(problems).toHaveLength(1);
    expect(problems[0]).toContain("the en description (_locales/en/messages.json) names Spanish and French while");
    expect(problems[0]).toContain("(es-en) study Spanish");
    expect(problems[0]).toContain("it names French, which no shipped pair glossed in en studies");
  });

  it("fr-es listed with change 35's Spanish description fails for Spanish; with the Spanish draft it passes", () => {
    const committedSpanish = summaryProblems(manifest52, reading("en", EN_DRAFT, stage2), WITH_FR_ES);
    expect(committedSpanish).toHaveLength(1);
    expect(committedSpanish[0]).toContain("the es description (_locales/es/messages.json) names Inglés while");
    expect(committedSpanish[0]).toContain("(en-es, fr-es) study Inglés and Francés");
    expect(committedSpanish[0]).toContain("it does not name Francés");
    const both = {
      ...reading("en", EN_DRAFT, locales52),
      es: { ...locales.es, extensionDescription: { message: ES_DRAFT } },
    };
    expect(summaryProblems(manifest52, both, WITH_FR_ES)).toEqual([]);
  });

  it("the Spanish draft with fr-es unlisted fails for Spanish: it names French, which no Spanish-glossed pair studies", () => {
    const both = {
      ...reading("en", EN_DRAFT, locales52),
      es: { ...locales.es, extensionDescription: { message: ES_DRAFT } },
    };
    const problems = summaryProblems(manifest52, both, WITH_FR_EN);
    expect(problems).toHaveLength(1);
    expect(problems[0]).toContain("the es description (_locales/es/messages.json) names Inglés and Francés while");
    expect(problems[0]).toContain("it names Francés, which no shipped pair glossed in es studies");
  });

  it("reads the French in both homes: manifest.json's literal is held too", () => {
    const problems = summaryProblems({ ...manifest, description: "Lisez l'anglais sur le web." }, locales52, TODAY);
    expect(problems).toHaveLength(1);
    expect(problems[0]).toContain("the fr description (manifest.json) names Anglais while");
    expect(problems[0]).toContain("it does not name Espagnol");
  });

  it("refuses a shipped native with no description, and one whose names it does not hold", () => {
    const { en: _english, ...withoutEnglish } = locales;
    void _english;
    expect(summaryProblems(manifest52, { ...withoutEnglish, fr: locales52.fr }, STAGE_2).join(" ")).toContain(
      "the en description (_locales/en/messages.json) is missing, yet es-en ship glossed in en",
    );
    expect(summaryProblems(manifest52, locales52, [...TODAY, "en-de"]).join(" ")).toContain(
      'en-de ship glossed in "de", whose names for the studied languages tool/check_version.mjs does not hold',
    );
  });

  it("commits the French draft with it-fr, within Apple's limit, in both homes (enable-lingua-italian D7)", () => {
    expect(locales.fr?.extensionDescription?.message).toBe(FR_DRAFT);
    expect(manifest.description).toBe(FR_DRAFT);
    expect(FR_DRAFT).toHaveLength(107);
    expect(localeProblems(manifest, locales)).toEqual([]);
    // The direct wording would not fit: 120 characters (design, Measured).
    expect(
      "Lisez l'anglais, l'espagnol ou l'italien sur le web : mots inconnus surlignés, pourcentage honnête. Hors ligne et privé.",
    ).toHaveLength(120);
  });

  it("it-fr listed with change 52's French description fails for French, in both homes; with the draft it passes", () => {
    const problems = summaryProblems(
      manifest52,
      reading("es", ES_DRAFT, reading("en", EN_DRAFT, locales52)),
      WITH_IT_FR,
    );
    expect(problems).toHaveLength(2);
    expect(problems[0]).toContain("the fr description (_locales/fr/messages.json) names Anglais and Espagnol while");
    expect(problems[0]).toContain("(en-fr, es-fr, it-fr) study Anglais, Espagnol and Italien");
    expect(problems[0]).toContain("it does not name Italien");
    expect(problems[1]).toContain("the fr description (manifest.json) names Anglais and Espagnol while");
    expect(summaryProblems(manifest, reading("es", ES_DRAFT, reading("en", EN_DRAFT)), WITH_IT_FR)).toEqual([]);
  });

  it("the French draft before it-fr is listed fails for French: it names Italian, which no French-glossed pair studies", () => {
    const problems = summaryProblems(manifest, reading("es", ES_DRAFT, reading("en", EN_DRAFT)), WITH_FR_ES);
    expect(problems).toHaveLength(2);
    expect(problems[0]).toContain(
      "the fr description (_locales/fr/messages.json) names Anglais, Espagnol and Italien while",
    );
    expect(problems[0]).toContain("it names Italien, which no shipped pair glossed in fr studies");
  });

  it("commits the English description naming Italian with it-en, within Apple's limit (enable-lingua-italian-for-english-speakers D6)", () => {
    expect(EN_ITALIAN).toHaveLength(108);
    expect(localeProblems(manifest, locales)).toEqual([]);
    // The direct wording would not fit: 114 characters (design D6).
    expect(
      "Read Spanish, French and Italian on the web: unknown words highlighted, an honest percentage. Offline and private.",
    ).toHaveLength(114);
  });

  it("it-en listed with change 52's English description fails for English; with the one naming Italian it passes", () => {
    const problems = summaryProblems(manifest, reading("es", ES_DRAFT, reading("en", EN_DRAFT)), WITH_IT_EN);
    expect(problems).toHaveLength(1);
    expect(problems[0]).toContain("the en description (_locales/en/messages.json) names Spanish and French while");
    expect(problems[0]).toContain("(es-en, fr-en, it-en) study Spanish, French and Italian");
    expect(problems[0]).toContain("it does not name Italian");
    expect(summaryProblems(manifest, reading("es", ES_DRAFT), WITH_IT_EN)).toEqual([]);
  });

  it("the English description naming Italian before it-en is listed fails for English: it names Italian, which no English-glossed pair studies", () => {
    const problems = summaryProblems(manifest, reading("es", ES_DRAFT), WITH_IT_FR);
    expect(problems).toHaveLength(1);
    expect(problems[0]).toContain(
      "the en description (_locales/en/messages.json) names Spanish, French and Italian while",
    );
    expect(problems[0]).toContain("it names Italian, which no shipped pair glossed in en studies");
  });

  it("commits the Spanish description naming Italian with it-es, within Apple's limit (enable-lingua-italian-for-spanish-speakers D6)", () => {
    expect(ES_ITALIAN).toHaveLength(105);
    expect(localeProblems(manifest, locales)).toEqual([]);
    // The direct wording would not fit: 119 characters (design D6).
    expect(
      "Lee inglés, francés e italiano en la web: palabras desconocidas resaltadas, porcentaje honesto. Sin conexión y privado.",
    ).toHaveLength(119);
  });

  it("it-es listed with change 52's Spanish description fails for Spanish; with the one naming Italian it passes", () => {
    const problems = summaryProblems(manifest, reading("es", ES_DRAFT), WITH_IT_ES);
    expect(problems).toHaveLength(1);
    expect(problems[0]).toContain("the es description (_locales/es/messages.json) names Inglés and Francés while");
    expect(problems[0]).toContain("(en-es, fr-es, it-es) study Inglés, Francés and Italiano");
    expect(problems[0]).toContain("it does not name Italiano");
    expect(summaryProblems(manifest, locales, WITH_IT_ES)).toEqual([]);
  });

  it("the Spanish description naming Italian before it-es is listed fails for Spanish: it names Italian, which no Spanish-glossed pair studies", () => {
    const problems = summaryProblems(manifest, locales, WITH_IT_EN);
    expect(problems).toHaveLength(1);
    expect(problems[0]).toContain(
      "the es description (_locales/es/messages.json) names Inglés, Francés and Italiano while",
    );
    expect(problems[0]).toContain("it names Italiano, which no shipped pair glossed in es studies");
  });

  it("matches a name whole and whatever its case", () => {
    expect(namedLanguages("fr", "Lisez l'anglais et l'ESPAGNOL")).toEqual(["en", "es"]);
    expect(namedLanguages("fr", "Le web en anglais, espagnol ou ITALIEN")).toEqual(["en", "es", "it"]);
    expect(namedLanguages("fr", "italienne")).toEqual([]);
    expect(namedLanguages("en", "READ SPANISH; french too")).toEqual(["es", "fr"]);
    expect(namedLanguages("en", "Spanishness, Frenchified, Englishes")).toEqual([]);
    expect(namedLanguages("en", "The web in Spanish, French or ITALIAN")).toEqual(["es", "fr", "it"]);
    expect(namedLanguages("es", "Lee INGLÉS y francés")).toEqual(["en", "fr"]);
    expect(namedLanguages("es", "inglesa")).toEqual([]);
    expect(namedLanguages("es", "La web en inglés, francés o ITALIANO")).toEqual(["en", "fr", "it"]);
    expect(namedLanguages("es", "italianos")).toEqual([]);
  });

  it("names each language as the interface's catalogue does", () => {
    const catalogue = (words: typeof frLanguages) => ({
      en: words.english.name,
      es: words.spanish.name,
      fr: words.french.name,
      it: words.italian.name,
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
