import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { describe, expect, it, vi } from "vitest";
import { nativeOf, readerPairs, SHIPPED_PAIRS, studiedOf } from "@/analyzer/pairs.ts";
import {
  fileUrl,
  loadBundledCatalogue,
  MANIFEST_PATH,
  type ModelCatalogue,
  type ModelManifest,
  modelsById,
  modelsFor,
  parseCatalogue,
  routeOf,
  totalSize,
  unpackedSize,
} from "@/translate/host/model-manifest.ts";

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
const committed = JSON.parse(readFileSync(join(root, "model-manifest.json"), "utf8"));

// The sha256 of each file's DECOMPRESSED bytes, as TRANSLATION.md pinned them when the engine was
// measured (add-lingua-translation-engine). The download is checked against these, nothing else.
const PINNED = {
  model: "6322e296d4fecfe395a8d5723da4ec37ecbe6d7613bb1dfcf4b28e2a47498b68",
  lex: "2585ed98d3af0bc949865aedeb390493d591f56870814376e73e4144c41ed059",
  vocab: "783abf3abe075afdf8d85d233994bef2c3a064e935ab1bed946820aff6ac002a",
};

const EN_FR = "en-fr/base-memory/2.0";
const ES_EN = "es-en/base-memory/2.0";
const EN_ES = "en-es/base-memory/2.1";
const FR_EN = "fr-en/base-memory/2.0";
const IT_EN = "it-en/base-memory/2.0";
const EN_IT = "en-it/base-memory/2.1";

// en-es 2.1 as Mozilla publishes it (add-lingua-translation-matrix-models D1): the sha256 of each
// file's decompressed bytes, read from the files downloaded from the registry and equal to Firefox's
// Remote Settings for en→es 2.1; the model's is the registry's `uncompressedHash`.
const EN_ES_PINNED = {
  model: "3b1c399511c01c84c36fae5c0524df44096288efdc8236e182b5c97d7ad2244c",
  lex: "7d51237c0a07027dcd61643cfbbb0f8c48597d19907ef53d2cae9d6bec2cf25c",
  vocab: "5ae254fa9b15aa182e70fd2a6186b1333c63a29a48043a9224c6aa4fcac058ad",
};

// fr-en 2.0 as Mozilla publishes it (add-lingua-french-translation D1): the sha256 of each file's
// decompressed bytes, read from the files downloaded from the registry and equal to Firefox's Remote
// Settings for fr→en 2.0 (`translations-models-v2` calls the same bytes 3.0); the model's is the
// registry's `uncompressedHash`, and the vocabulary's is en-fr 2.0's.
const FR_EN_PINNED = {
  model: "15f997bc0d13808b0b0fbd0786e684a3c8a52adcd8071844b76123fdacbf2b90",
  lex: "87c6752ea908f5f0347c10ac0cf7d80d9c2f4f20c81c90168f3e8230b56d4440",
  vocab: "783abf3abe075afdf8d85d233994bef2c3a064e935ab1bed946820aff6ac002a",
};

// it-en 2.0 and en-it 2.1 as Mozilla publishes them (add-lingua-italian-translation-models D1): the
// sha256 of each file's decompressed bytes, read from the files downloaded from the registry and equal
// to Firefox's Remote Settings for it→en 2.0 and en→it 2.1; each model's is the registry's
// `uncompressedHash`, and the two vocabularies are one file once decompressed (D3).
const IT_EN_PINNED = {
  model: "21b70978ce2f3b4da7a06b5de86a09abe3acd30b9eee1b2ebb3582b9bad790bf",
  lex: "0700d6d70b30490e68ff9deca0b45b80310745b491855bde3a13e1692d0cbce1",
  vocab: "3ef0211d4ae6db21440892f180f2019fe2bfc110a330ffa9d2eca9665e4f2bc5",
};
const EN_IT_PINNED = {
  model: "248f47568788ecc351da7e5e07064d4153b4f71e011364ae2c931ffeec4d1cc2",
  lex: "8b21914804625b2777dae7fdb636eb78f02a3eb8b7bceaa50f29ac740961da93",
  vocab: "3ef0211d4ae6db21440892f180f2019fe2bfc110a330ffa9d2eca9665e4f2bc5",
};

/** Each file's decompressed sha256, by role. */
const digests = (model: ModelManifest) =>
  Object.fromEntries(Object.entries(model.files).map(([role, f]) => [role, f.sha256]));
/** Each file's size as served and decompressed, by role. */
const sizes = (model: ModelManifest) =>
  Object.fromEntries(Object.entries(model.files).map(([role, f]) => [role, [f.size, f.unpacked]]));
/** The catalogue as committed: each file's source in Mozilla's registry, path and gzip sha256, by role. */
const sources = (id: string) =>
  Object.values(committed.models[id].files).map((f) => {
    const { source } = f as { source: { path: string; sha256: string } };
    return [source.path, source.sha256];
  });

describe("the committed catalogue", () => {
  it("Every reader today: six models, en-fr's and es-fr's routes as before, es-en, en-es, fr-en, it-en and en-it one model each, fr-es, it-fr, it-es, fr-it and es-it through English, and nothing else is listed (matrix-models D2, french-translation D2, italian-translation-models D2)", () => {
    const c = parseCatalogue(committed);
    // In the order they were pinned.
    expect(Object.keys(c.models)).toEqual([EN_FR, ES_EN, EN_ES, FR_EN, IT_EN, EN_IT]);
    expect(c.routes).toEqual({
      "en-fr": [EN_FR],
      "es-fr": [ES_EN, EN_FR],
      "es-en": [ES_EN],
      "en-es": [EN_ES],
      "fr-en": [FR_EN],
      "fr-es": [FR_EN, EN_ES],
      "it-en": [IT_EN],
      "en-it": [EN_IT],
      "it-fr": [IT_EN, EN_FR],
      "it-es": [IT_EN, EN_ES],
      "fr-it": [FR_EN, EN_IT],
      "es-it": [ES_EN, EN_IT],
    });
    // Italian's six appended after the matrix's six (italian-translation-models D2).
    expect(Object.keys(c.routes)).toEqual([
      "en-fr",
      "es-fr",
      "es-en",
      "en-es",
      "fr-en",
      "fr-es",
      "it-en",
      "en-it",
      "it-fr",
      "it-es",
      "fr-it",
      "es-it",
    ]);
    // Two models at most per route, as the engine's hardening allows (harden-lingua-translation-engine D3).
    for (const [pair, route] of Object.entries(c.routes)) expect(route.length, pair).toBeLessThanOrEqual(2);
    // The same models in the same order as the routes keyed by studied language gave.
    expect(routeOf(c, "en-fr").map((m) => m.version)).toEqual([EN_FR]);
    expect(routeOf(c, "es-fr").map((m) => m.version)).toEqual([ES_EN, EN_FR]);
  });

  it("routes English through the en→fr base-memory model the engine was measured on", () => {
    const [model, ...rest] = routeOf(parseCatalogue(committed), "en-fr");
    expect(rest).toEqual([]);
    // The id is the version a device recorded before the catalogue: a stored model stays complete.
    expect(model.version).toBe(EN_FR);
    expect([model.from, model.to]).toEqual(["en", "fr"]);
    expect(Object.fromEntries(Object.entries(model.files).map(([role, f]) => [role, f.sha256]))).toEqual(PINNED);
  });

  it("downloads 25 752 472 bytes — Mozilla's own gzip files — and keeps 36 749 127 on the device", () => {
    const [model] = routeOf(parseCatalogue(committed), "en-fr");
    expect(totalSize(model)).toBe(25_752_472);
    expect(unpackedSize(model)).toBe(36_749_127);
  });

  it("serves each file under a content-addressed path, from Cymbra over https", () => {
    const [model] = routeOf(parseCatalogue(committed), "en-fr");
    expect(model.base).toMatch(/^https:\/\/[a-z.]+cymbra\.app\/$/);
    for (const file of Object.values(model.files)) {
      expect(file.path).toContain(`/${file.sha256}/`);
      expect(file.path).toMatch(/\.gz$/);
      expect(fileUrl(model, file)).toBe(`${model.base}${file.path}`);
    }
  });

  it("routes es-fr through English: es-en, then en-fr (add-lingua-spanish-translation-pivot D1)", () => {
    const route = routeOf(parseCatalogue(committed), "es-fr");
    expect(route.map((m) => [m.version, m.from, m.to])).toEqual([
      [ES_EN, "es", "en"],
      [EN_FR, "en", "fr"],
    ]);
    // Pinned as Mozilla's registry lists it: the decompressed model's sha256 is its uncompressedHash.
    expect(route[0].files.model.sha256).toBe("4aed7734152ae0045d1a69ae49c86cfda18f53c61f90e95e1d1de1c7c7c3b033");
    expect(totalSize(route[0])).toBe(26_241_052);
    expect(totalSize(route[0]) + totalSize(route[1])).toBe(51_993_524);
  });

  it("translates English and Spanish into each other directly: es-en is the es-en model alone, en-es the en-es model alone (matrix-models D2)", () => {
    const c = parseCatalogue(committed);
    expect(routeOf(c, "es-en").map((m) => [m.version, m.from, m.to])).toEqual([[ES_EN, "es", "en"]]);
    expect(routeOf(c, "en-es").map((m) => [m.version, m.from, m.to])).toEqual([[EN_ES, "en", "es"]]);
    // es-en's route alone: its own model, nothing of en-fr.
    expect(totalSize(routeOf(c, "es-en")[0]!)).toBe(26_241_052);
  });

  it("Pinned against Mozilla's publications: en-es 2.1's files, their digests, and 25 373 354 bytes to download (matrix-models D1)", () => {
    const [model, ...rest] = routeOf(parseCatalogue(committed), "en-es");
    expect(rest).toEqual([]);
    expect(Object.fromEntries(Object.entries(model!.files).map(([role, f]) => [role, f.sha256]))).toEqual(EN_ES_PINNED);
    expect(Object.fromEntries(Object.entries(model!.files).map(([role, f]) => [role, [f.size, f.unpacked]]))).toEqual({
      model: [22_698_792, 31_561_787],
      lex: [2_265_250, 4_198_436],
      vocab: [409_312, 816_054],
    });
    expect(totalSize(model!)).toBe(25_373_354);
    expect(unpackedSize(model!)).toBe(36_576_277);
    // Named as the other models' files, under the id and the decompressed sha256.
    expect(Object.values(model!.files).map((f) => f.path)).toEqual([
      `${EN_ES}/${EN_ES_PINNED.model}/model.bin.gz`,
      `${EN_ES}/${EN_ES_PINNED.lex}/lex.bin.gz`,
      `${EN_ES}/${EN_ES_PINNED.vocab}/vocab.spm.gz`,
    ]);
    // Where Mozilla's registry publishes them, with its mirror release and its licence.
    const raw = committed.models[EN_ES];
    expect(raw.licence).toBe("MPL-2.0");
    expect(raw.mirror).toBe("https://github.com/NEETROF/cymbra/releases/download/lingua-model-en-es-base-memory-2.1/");
    expect(Object.values(raw.files).map((f) => (f as { source: { path: string } }).source.path)).toEqual([
      "models/en-es/retrain_hr_fix_names_CUAEXUHoQum_cFqh-ZAryw/exported/model.enes.intgemm.alphas.bin.gz",
      "models/en-es/retrain_hr_fix_names_CUAEXUHoQum_cFqh-ZAryw/exported/lex.50.50.enes.s2t.bin.gz",
      "models/en-es/retrain_hr_fix_names_CUAEXUHoQum_cFqh-ZAryw/exported/vocab.enes.spm.gz",
    ]);
  });

  it("shares a file between es-en and en-es: the same vocabulary once decompressed, from two gzip files (matrix-models D3)", () => {
    const c = parseCatalogue(committed);
    const [esEn] = routeOf(c, "es-en");
    const [enEs] = routeOf(c, "en-es");
    // The device stores a file by its decompressed sha256: one held for es-en is not fetched again for en-es.
    expect(enEs!.files.vocab.sha256).toBe(esEn!.files.vocab.sha256);
    expect(committed.models[EN_ES].files.vocab.source.sha256).not.toBe(
      committed.models[ES_EN].files.vocab.source.sha256,
    );
    // The cost sums the files of the models needed, the shared one in each (the setting's cost requirement).
    expect(modelsFor(c, ["es-en", "en-es"]).reduce((sum, m) => sum + totalSize(m), 0)).toBe(26_241_052 + 25_373_354);
  });

  it("Pinned against Mozilla's publications: fr-en 2.0's files, their digests, 26 234 715 bytes to download and 37 200 311 on the device (french-translation D1)", () => {
    const [model, ...rest] = routeOf(parseCatalogue(committed), "fr-en");
    expect(rest).toEqual([]);
    expect([model!.from, model!.to]).toEqual(["fr", "en"]);
    expect(Object.fromEntries(Object.entries(model!.files).map(([role, f]) => [role, f.sha256]))).toEqual(FR_EN_PINNED);
    expect(Object.fromEntries(Object.entries(model!.files).map(([role, f]) => [role, [f.size, f.unpacked]]))).toEqual({
      model: [23_175_075, 31_561_787],
      lex: [2_649_934, 4_824_120],
      vocab: [409_706, 814_404],
    });
    expect(totalSize(model!)).toBe(26_234_715);
    expect(unpackedSize(model!)).toBe(37_200_311);
    // Named as the other models' files, under the id and the decompressed sha256.
    expect(Object.values(model!.files).map((f) => f.path)).toEqual([
      `${FR_EN}/${FR_EN_PINNED.model}/model.bin.gz`,
      `${FR_EN}/${FR_EN_PINNED.lex}/lex.bin.gz`,
      `${FR_EN}/${FR_EN_PINNED.vocab}/vocab.spm.gz`,
    ]);
    // Where Mozilla's registry publishes them — the run of its one fr-en entry — with the sha256 of the
    // gzip file it serves, its mirror release and its licence.
    const raw = committed.models[FR_EN];
    expect(raw.licence).toBe("MPL-2.0");
    expect(raw.mirror).toBe("https://github.com/NEETROF/cymbra/releases/download/lingua-model-fr-en-base-memory-2.0/");
    expect(
      Object.values(raw.files).map((f) => {
        const { source } = f as { source: { path: string; sha256: string } };
        return [source.path, source.sha256];
      }),
    ).toEqual([
      [
        "models/fr-en/retrain_hr_EFgIftH_RrCyzl5gjemVNg/exported/model.fren.intgemm.alphas.bin.gz",
        "06b1eeedd3944260d00a393d12ec7192490c5b7c930b97212faf4d5f5c90a2ee",
      ],
      [
        "models/fr-en/retrain_hr_EFgIftH_RrCyzl5gjemVNg/exported/lex.50.50.fren.s2t.bin.gz",
        "395aa7767220e1bcfc085f2b2787ff98005d0075e55451f1e0832885e3d9642a",
      ],
      [
        "models/fr-en/retrain_hr_EFgIftH_RrCyzl5gjemVNg/exported/vocab.fren.spm.gz",
        "8d15b219ffd32327b4cabf0d94a05a4fecb8923a4f386badbcbe5e86ede453a7",
      ],
    ]);
  });

  it("translates French into English directly, and into Spanish through English: fr-en then en-es, 51 608 069 bytes (french-translation D2)", () => {
    const c = parseCatalogue(committed);
    expect(routeOf(c, "fr-en").map((m) => [m.version, m.from, m.to])).toEqual([[FR_EN, "fr", "en"]]);
    const route = routeOf(c, "fr-es");
    expect(route.map((m) => [m.version, m.from, m.to])).toEqual([
      [FR_EN, "fr", "en"],
      [EN_ES, "en", "es"],
    ]);
    // Mozilla publishes no French–Spanish model: the pivot downloads both, as es-fr does.
    expect(route.reduce((sum, m) => sum + totalSize(m), 0)).toBe(51_608_069);
  });

  it("A vocabulary fr-en shares with en-fr: the same bytes once decompressed, from two gzip files (french-translation D3)", () => {
    const c = parseCatalogue(committed);
    const [enFr] = routeOf(c, "en-fr");
    const [frEn] = routeOf(c, "fr-en");
    // The device stores a file by its decompressed sha256: one held for en-fr is kept, and not fetched
    // again, for fr-en.
    expect(frEn!.files.vocab.sha256).toBe(enFr!.files.vocab.sha256);
    expect(frEn!.files.vocab.unpacked).toBe(enFr!.files.vocab.unpacked);
    expect(committed.models[FR_EN].files.vocab.source.sha256).not.toBe(
      committed.models[EN_FR].files.vocab.source.sha256,
    );
    // Nothing else is shared: the model and the lexicon are fr-en's own.
    for (const role of ["model", "lex"] as const) expect(frEn!.files[role].sha256).not.toBe(enFr!.files[role].sha256);
  });

  it("Pinned against Mozilla's publications: it-en 2.0's and en-it 2.1's files, their digests, 26 242 301 and 26 509 835 bytes to download (italian-translation-models D1)", () => {
    const c = parseCatalogue(committed);
    const [itEn, ...afterItEn] = routeOf(c, "it-en");
    const [enIt, ...afterEnIt] = routeOf(c, "en-it");
    expect(afterItEn).toEqual([]);
    expect(afterEnIt).toEqual([]);
    expect([itEn!.version, itEn!.from, itEn!.to]).toEqual([IT_EN, "it", "en"]);
    expect([enIt!.version, enIt!.from, enIt!.to]).toEqual([EN_IT, "en", "it"]);
    expect(digests(itEn!)).toEqual(IT_EN_PINNED);
    expect(digests(enIt!)).toEqual(EN_IT_PINNED);
    expect(sizes(itEn!)).toEqual({
      model: [23_301_774, 31_561_787],
      lex: [2_532_929, 4_713_616],
      vocab: [407_598, 812_724],
    });
    expect(sizes(enIt!)).toEqual({
      model: [23_892_889, 31_561_787],
      lex: [2_209_348, 4_133_192],
      vocab: [407_598, 812_724],
    });
    expect(totalSize(itEn!)).toBe(26_242_301);
    expect(unpackedSize(itEn!)).toBe(37_088_127);
    expect(totalSize(enIt!)).toBe(26_509_835);
    expect(unpackedSize(enIt!)).toBe(36_507_703);
    // Named as the other models' files, under the id and the decompressed sha256.
    expect(Object.values(itEn!.files).map((f) => f.path)).toEqual([
      `${IT_EN}/${IT_EN_PINNED.model}/model.bin.gz`,
      `${IT_EN}/${IT_EN_PINNED.lex}/lex.bin.gz`,
      `${IT_EN}/${IT_EN_PINNED.vocab}/vocab.spm.gz`,
    ]);
    expect(Object.values(enIt!.files).map((f) => f.path)).toEqual([
      `${EN_IT}/${EN_IT_PINNED.model}/model.bin.gz`,
      `${EN_IT}/${EN_IT_PINNED.lex}/lex.bin.gz`,
      `${EN_IT}/${EN_IT_PINNED.vocab}/vocab.spm.gz`,
    ]);
    // Where Mozilla's registry publishes them — the run of its one entry for each — with the sha256 of
    // the gzip file it serves, their mirror releases and their licence.
    expect(committed.models[IT_EN].licence).toBe("MPL-2.0");
    expect(committed.models[EN_IT].licence).toBe("MPL-2.0");
    expect(committed.models[IT_EN].mirror).toBe(
      "https://github.com/NEETROF/cymbra/releases/download/lingua-model-it-en-base-memory-2.0/",
    );
    expect(committed.models[EN_IT].mirror).toBe(
      "https://github.com/NEETROF/cymbra/releases/download/lingua-model-en-it-base-memory-2.1/",
    );
    expect(sources(IT_EN)).toEqual([
      [
        "models/it-en/retrain_hr_d1nPDwVeSD6t-HdtWA4xNg/exported/model.iten.intgemm.alphas.bin.gz",
        "0bc7221c2730723ac4477e8aa1d1d5bd0b9c3450cb8d006a6f86fe5da9129c27",
      ],
      [
        "models/it-en/retrain_hr_d1nPDwVeSD6t-HdtWA4xNg/exported/lex.50.50.iten.s2t.bin.gz",
        "504877085b31edfa3d31a1918a9a42c20fcbf85ff018caf46c54641db5538e62",
      ],
      [
        "models/it-en/retrain_hr_d1nPDwVeSD6t-HdtWA4xNg/exported/vocab.iten.spm.gz",
        "df05fae498f519a828eb388cd08ec1b7205a8bcaf2405a5feb9f5e30c49a0977",
      ],
    ]);
    expect(sources(EN_IT)).toEqual([
      [
        "models/en-it/retrain_hr_fix_names_GPSr_u8PSmC9fOx1ZDCHoA/exported/model.enit.intgemm.alphas.bin.gz",
        "cfff470e9f29e27132723589a8bc985648f92773410e5f6fc43293d9e19f97f1",
      ],
      [
        "models/en-it/retrain_hr_fix_names_GPSr_u8PSmC9fOx1ZDCHoA/exported/lex.50.50.enit.s2t.bin.gz",
        "cd7ef06c0b872c15286a9796f105eb62693f14665490546ac70ce1c6c8f16d47",
      ],
      [
        "models/en-it/retrain_hr_fix_names_GPSr_u8PSmC9fOx1ZDCHoA/exported/vocab.enit.spm.gz",
        "8f99a6f493bf7e1a03fbba19928d7dc7bf9a1b7938238467f3e80a58c0d50485",
      ],
    ]);
  });

  it("translates Italian to and from English directly, and to and from French and Spanish through English (italian-translation-models D2)", () => {
    const c = parseCatalogue(committed);
    const route = (pair: string) => routeOf(c, pair).map((m) => [m.version, m.from, m.to]);
    const download = (pair: string) => routeOf(c, pair).reduce((sum, m) => sum + totalSize(m), 0);
    expect(route("it-en")).toEqual([[IT_EN, "it", "en"]]);
    expect(route("en-it")).toEqual([[EN_IT, "en", "it"]]);
    // Mozilla publishes no Italian–French nor Italian–Spanish model: each goes through English, two models.
    expect(route("it-fr")).toEqual([
      [IT_EN, "it", "en"],
      [EN_FR, "en", "fr"],
    ]);
    expect(route("it-es")).toEqual([
      [IT_EN, "it", "en"],
      [EN_ES, "en", "es"],
    ]);
    expect(route("fr-it")).toEqual([
      [FR_EN, "fr", "en"],
      [EN_IT, "en", "it"],
    ]);
    expect(route("es-it")).toEqual([
      [ES_EN, "es", "en"],
      [EN_IT, "en", "it"],
    ]);
    expect(download("it-fr")).toBe(51_994_773);
    expect(download("it-es")).toBe(51_615_655);
    expect(download("fr-it")).toBe(52_744_550);
    expect(download("es-it")).toBe(52_750_887);
  });

  it("A vocabulary the two Italian models share: the same 812 724 bytes once decompressed, from two gzip files (italian-translation-models D3)", () => {
    const c = parseCatalogue(committed);
    const [itEn] = routeOf(c, "it-en");
    const [enIt] = routeOf(c, "en-it");
    // The device stores a file by its decompressed sha256: one held for it-en is kept, and not fetched
    // again, for en-it, and the other way round.
    expect(enIt!.files.vocab.sha256).toBe(itEn!.files.vocab.sha256);
    expect(enIt!.files.vocab.unpacked).toBe(812_724);
    expect(itEn!.files.vocab.unpacked).toBe(812_724);
    expect(committed.models[EN_IT].files.vocab.source.sha256).not.toBe(
      committed.models[IT_EN].files.vocab.source.sha256,
    );
    // Nothing else is shared: each model and lexicon is its own, and no other model's vocabulary is theirs.
    for (const role of ["model", "lex"] as const) expect(enIt!.files[role].sha256).not.toBe(itEn!.files[role].sha256);
    const others = [EN_FR, ES_EN, EN_ES, FR_EN].map((id) => c.models[id]!.files.vocab.sha256);
    expect(others).not.toContain(itEn!.files.vocab.sha256);
    // The cost sums the files of the models needed, the shared one in each (the setting's cost requirement).
    expect(modelsFor(c, ["it-en", "en-it"]).reduce((sum, m) => sum + totalSize(m), 0)).toBe(26_242_301 + 26_509_835);
  });

  it("Every native language's pairs, Italian's included: the routes keyed by each need three models together — en-fr, es-en and it-en for French; es-en, fr-en and it-en for English; en-es, fr-en and it-en for Spanish; en-it, fr-en and es-en for Italian — each route two at most (italian-translation-models D2)", () => {
    const c = parseCatalogue(committed);
    const routesInto = (native: string) => Object.keys(c.routes).filter((pair) => nativeOf(pair) === native);
    expect(routesInto("fr")).toEqual(["en-fr", "es-fr", "it-fr"]);
    expect(routesInto("en")).toEqual(["es-en", "fr-en", "it-en"]);
    expect(routesInto("es")).toEqual(["en-es", "fr-es", "it-es"]);
    expect(routesInto("it")).toEqual(["en-it", "fr-it", "es-it"]);
    const needs = (native: string) => modelsFor(c, routesInto(native));
    expect(needs("fr").map((m) => m.version)).toEqual([EN_FR, ES_EN, IT_EN]);
    expect(needs("en").map((m) => m.version)).toEqual([ES_EN, FR_EN, IT_EN]);
    expect(needs("es").map((m) => m.version)).toEqual([EN_ES, FR_EN, IT_EN]);
    expect(needs("it").map((m) => m.version)).toEqual([EN_IT, FR_EN, ES_EN]);
    // Three each once Italian's pairs are routed; the engine still loads two at a time, a route's.
    for (const native of ["fr", "en", "es", "it"]) {
      for (const pair of routesInto(native)) expect(routeOf(c, pair).length, pair).toBeLessThanOrEqual(2);
    }
    // The download sums, from the gzip sizes measured for each model (task 1.1 for it-en and en-it).
    const download = (native: string) => needs(native).reduce((sum, m) => sum + totalSize(m), 0);
    expect(download("fr")).toBe(25_752_472 + 26_241_052 + 26_242_301);
    expect(download("fr")).toBe(78_235_825);
    expect(download("en")).toBe(26_241_052 + 26_234_715 + 26_242_301);
    expect(download("en")).toBe(78_718_068);
    expect(download("es")).toBe(25_373_354 + 26_234_715 + 26_242_301);
    expect(download("es")).toBe(77_850_370);
    expect(download("it")).toBe(26_509_835 + 26_234_715 + 26_241_052);
    expect(download("it")).toBe(78_985_602);
    // Without Italian's routes, each native language's pairs still need the two models of before (french-translation D3).
    const before = (native: string) =>
      modelsFor(
        c,
        routesInto(native).filter((pair) => studiedOf(pair) !== "it"),
      ).map((m) => m.version);
    expect(before("fr")).toEqual([EN_FR, ES_EN]);
    expect(before("en")).toEqual([ES_EN, FR_EN]);
    expect(before("es")).toEqual([EN_ES, FR_EN]);
  });

  it("has no route for a pair nothing translates yet — nor for a studied language asked alone", () => {
    expect(routeOf(parseCatalogue(committed), "de-fr")).toEqual([]);
    expect(routeOf(parseCatalogue(committed), "en")).toEqual([]);
  });

  it("A route of a pair not shipped: no reader's pairs need es-en or en-es, and the es-en model is held only as es-fr's first model", () => {
    const c = parseCatalogue(committed);
    // The scenario's list: en-fr and es-fr, the shipped pairs before change 34 (enable-lingua-english-speakers).
    const shipped = ["en-fr", "es-fr"];
    // A reader of French: the needs, the downloads and the routes loaded are those of before.
    expect(readerPairs(["en", "es"], "fr", shipped)).toEqual(["en-fr", "es-fr"]);
    expect(modelsFor(c, readerPairs(["en", "es"], "fr", shipped)).map((m) => m.version)).toEqual([EN_FR, ES_EN]);
    expect(modelsFor(c, readerPairs(["en"], "fr", shipped)).map((m) => m.version)).toEqual([EN_FR]);
    expect(modelsFor(c, readerPairs(["es"], "fr", shipped)).map((m) => m.version)).toEqual([ES_EN, EN_FR]);
    // A reader of another native language has no shipped pair: nothing is needed for them.
    expect(readerPairs(["en"], "es", shipped)).toEqual([]);
    expect(readerPairs(["es"], "en", shipped)).toEqual([]);
    expect(modelsFor(c, readerPairs(["en"], "es", shipped))).toEqual([]);
    expect(modelsFor(c, readerPairs(["es"], "en", shipped))).toEqual([]);
    // Whatever a reader studies, en-es is never needed, and es-en only through es-fr.
    for (const native of ["fr", "en", "es"]) {
      for (const languages of [["en"], ["es"], ["en", "es"], ["es", "en"]]) {
        const pairs = readerPairs(languages, native, shipped);
        expect(pairs.every((pair) => pair === "en-fr" || pair === "es-fr")).toBe(true);
        const needed = modelsFor(c, pairs).map((m) => m.version);
        expect(needed).not.toContain(EN_ES);
        if (needed.includes(ES_EN)) expect(pairs).toContain("es-fr");
      }
    }
  });

  it("A route of a pair studying French: no reader's pairs need fr-en or fr-es, and no reader fetches the fr-en model (french-translation D2)", () => {
    const c = parseCatalogue(committed);
    // Routed, on a list where no shipped pair studies French: change 35's, before change 52 listed fr-en and
    // fr-es (enable-lingua-french D5, below).
    const shipped = ["en-fr", "es-fr", "es-en", "en-es"];
    expect(Object.keys(c.routes)).toEqual(expect.arrayContaining(["fr-en", "fr-es"]));
    expect(shipped.some((pair) => studiedOf(pair) === "fr")).toBe(false);
    for (const native of ["fr", "en", "es"]) {
      for (const languages of [["fr"], ["en", "fr"], ["es", "fr"], ["fr", "en", "es"]]) {
        const pairs = readerPairs(languages, native, shipped);
        expect(pairs).not.toContain("fr-en");
        expect(pairs).not.toContain("fr-es");
        expect(modelsFor(c, pairs).map((m) => m.version)).not.toContain(FR_EN);
      }
    }
    // A reader of French downloads what they did before French was routed.
    expect(modelsFor(c, readerPairs(["en", "es", "fr"], "fr", shipped)).map((m) => m.version)).toEqual([EN_FR, ES_EN]);
    expect(readerPairs(["fr"], "en", shipped)).toEqual([]);
    expect(readerPairs(["fr"], "es", shipped)).toEqual([]);
  });

  it("A route of an Italian pair: it-en, en-it, it-fr, it-es, fr-it and es-it are routed, and over today's shipped pairs no reader's pairs need them and no reader fetches the it-en or the en-it model (italian-translation-models D2)", () => {
    const c = parseCatalogue(committed);
    const italian = ["it-en", "en-it", "it-fr", "it-es", "fr-it", "es-it"];
    expect(Object.keys(c.routes)).toEqual(expect.arrayContaining(italian));
    // Today's list: no shipped pair studies Italian or is glossed in it. Change 20 (enable-lingua-italian)
    // ships it-fr, and freezes the list this test reads then, as the French test did.
    expect(SHIPPED_PAIRS.some((pair) => studiedOf(pair) === "it" || nativeOf(pair) === "it")).toBe(false);
    const languages = [["it"], ["en", "it"], ["es", "it"], ["fr", "it"], ["it", "en", "es", "fr"], ["en", "es", "fr"]];
    for (const native of ["fr", "en", "es", "it"]) {
      for (const studied of languages) {
        const pairs = readerPairs(studied, native);
        const where = `${native}: ${studied.join(",")}`;
        for (const pair of italian) expect(pairs, where).not.toContain(pair);
        const needed = modelsFor(c, pairs).map((m) => m.version);
        expect(needed, where).not.toContain(IT_EN);
        expect(needed, where).not.toContain(EN_IT);
      }
    }
    // An Italian-native reader has no shipped pair: nothing is offered, downloaded, kept or loaded.
    expect(readerPairs(["en", "es", "fr"], "it")).toEqual([]);
    // Every reader downloads what they did before Italian was routed.
    expect(modelsFor(c, readerPairs(["en", "es", "it"], "fr")).map((m) => m.version)).toEqual([EN_FR, ES_EN]);
    expect(modelsFor(c, readerPairs(["es", "fr", "it"], "en")).map((m) => m.version)).toEqual([ES_EN, FR_EN]);
    expect(modelsFor(c, readerPairs(["en", "fr", "it"], "es")).map((m) => m.version)).toEqual([EN_ES, FR_EN]);
  });

  it("An English-native reader of Spanish, es-en shipping (change 34): the es-en route alone; a reader of French as before", () => {
    const c = parseCatalogue(committed);
    // The scenario's list: change 34's, es-en beside the French-native pairs.
    const shipped = ["en-fr", "es-fr", "es-en"];
    // A reader of French: the needs, the downloads and the routes loaded are still those of before (D2).
    expect(readerPairs(["en", "es"], "fr", shipped)).toEqual(["en-fr", "es-fr"]);
    expect(modelsFor(c, readerPairs(["en", "es"], "fr", shipped)).map((m) => m.version)).toEqual([EN_FR, ES_EN]);
    expect(modelsFor(c, readerPairs(["en"], "fr", shipped)).map((m) => m.version)).toEqual([EN_FR]);
    expect(modelsFor(c, readerPairs(["es"], "fr", shipped)).map((m) => m.version)).toEqual([ES_EN, EN_FR]);
    // A reader of English studies Spanish through es-en: its model alone, nothing of en-fr.
    expect(readerPairs(["es"], "en", shipped)).toEqual(["es-en"]);
    expect(modelsFor(c, readerPairs(["es"], "en", shipped)).map((m) => m.version)).toEqual([ES_EN]);
    // A reader of Spanish has no shipped pair in that list (en-es ships with change 35).
    expect(readerPairs(["en"], "es", shipped)).toEqual([]);
    // Whatever a reader studies, en-es is never needed.
    for (const native of ["fr", "en", "es"]) {
      for (const languages of [["en"], ["es"], ["en", "es"], ["es", "en"]]) {
        expect(modelsFor(c, readerPairs(languages, native, shipped)).map((m) => m.version)).not.toContain(EN_ES);
      }
    }
  });

  it("A Spanish-native reader of English, en-es shipping (change 35): the en-es route alone; readers of French and English as before", () => {
    const c = parseCatalogue(committed);
    // The scenario's list: change 35's, en-es beside change 34's, before change 52 listed French.
    const shipped = ["en-fr", "es-fr", "es-en", "en-es"];
    // A reader of French: the needs, the downloads and the routes loaded are still those of before (D2).
    expect(readerPairs(["en", "es"], "fr", shipped)).toEqual(["en-fr", "es-fr"]);
    expect(modelsFor(c, readerPairs(["en", "es"], "fr", shipped)).map((m) => m.version)).toEqual([EN_FR, ES_EN]);
    expect(modelsFor(c, readerPairs(["en"], "fr", shipped)).map((m) => m.version)).toEqual([EN_FR]);
    expect(modelsFor(c, readerPairs(["es"], "fr", shipped)).map((m) => m.version)).toEqual([ES_EN, EN_FR]);
    // A reader of English: es-en's model alone, as change 34 shipped it.
    expect(readerPairs(["es"], "en", shipped)).toEqual(["es-en"]);
    expect(modelsFor(c, readerPairs(["es"], "en", shipped)).map((m) => m.version)).toEqual([ES_EN]);
    // A reader of Spanish studies English through en-es: its model alone, nothing of en-fr or es-en.
    expect(readerPairs(["en"], "es", shipped)).toEqual(["en-es"]);
    expect(modelsFor(c, readerPairs(["en"], "es", shipped)).map((m) => m.version)).toEqual([EN_ES]);
    // en-es is needed by a reader of Spanish who studies English, and by no one else.
    for (const native of ["fr", "en", "es"]) {
      for (const languages of [["en"], ["es"], ["en", "es"], ["es", "en"]]) {
        const needed = modelsFor(c, readerPairs(languages, native, shipped)).map((m) => m.version);
        expect(needed.includes(EN_ES), `${native}: ${languages.join(",")}`).toBe(
          native === "es" && languages.includes("en"),
        );
      }
    }
  });

  it("French studied by readers of English and Spanish (enable-lingua-french D5): fr-en's route, and fr-es's through English; a reader of French as before", () => {
    const c = parseCatalogue(committed);
    expect(SHIPPED_PAIRS).toEqual(["en-fr", "es-fr", "es-en", "en-es", "fr-en", "fr-es"]);
    const download = (pairs: string[]) => modelsFor(c, pairs).reduce((sum, m) => sum + totalSize(m), 0);
    // A reader of French: French is never their pair, and fr-en never their download.
    expect(readerPairs(["en", "es", "fr"], "fr")).toEqual(["en-fr", "es-fr"]);
    expect(modelsFor(c, readerPairs(["en", "es", "fr"], "fr")).map((m) => m.version)).toEqual([EN_FR, ES_EN]);
    expect(readerPairs(["fr"], "fr")).toEqual([]);
    // An English-native reader of French: fr-en's model alone; of Spanish and French, es-en's and fr-en's.
    expect(readerPairs(["fr"], "en")).toEqual(["fr-en"]);
    expect(modelsFor(c, readerPairs(["fr"], "en")).map((m) => m.version)).toEqual([FR_EN]);
    expect(download(readerPairs(["fr"], "en"))).toBe(26_234_715);
    expect(readerPairs(["es", "fr"], "en")).toEqual(["es-en", "fr-en"]);
    expect(modelsFor(c, readerPairs(["fr", "es"], "en")).map((m) => m.version)).toEqual([FR_EN, ES_EN]);
    expect(download(readerPairs(["es", "fr"], "en"))).toBe(52_475_767);
    // A Spanish-native reader of French: fr-en then en-es; of English and French, the same two models.
    expect(readerPairs(["fr"], "es")).toEqual(["fr-es"]);
    expect(modelsFor(c, readerPairs(["fr"], "es")).map((m) => m.version)).toEqual([FR_EN, EN_ES]);
    expect(download(readerPairs(["fr"], "es"))).toBe(51_608_069);
    expect(modelsFor(c, readerPairs(["en", "fr"], "es")).map((m) => m.version)).toEqual([EN_ES, FR_EN]);
    expect(download(readerPairs(["fr", "en"], "es"))).toBe(51_608_069);
    // en-es is needed by a reader of Spanish who studies English or French, and by no one else; fr-en by a
    // reader of English or of Spanish who studies French, and by no French-native reader.
    for (const native of ["fr", "en", "es"]) {
      for (const languages of [["en"], ["es"], ["fr"], ["en", "es"], ["es", "fr"], ["fr", "en"], ["fr", "en", "es"]]) {
        const needed = modelsFor(c, readerPairs(languages, native)).map((m) => m.version);
        const where = `${native}: ${languages.join(",")}`;
        expect(needed.includes(EN_ES), where).toBe(
          native === "es" && (languages.includes("en") || languages.includes("fr")),
        );
        expect(needed.includes(FR_EN), where).toBe(native !== "fr" && languages.includes("fr"));
      }
    }
  });
});

describe("parseCatalogue", () => {
  const valid = (): ModelCatalogue => parseCatalogue(committed);
  /** The committed catalogue with one model more, and `routes` as given. */
  const withRoutes = (routes: Record<string, string[]>) => {
    const c = valid();
    const esEn = { ...c.models[EN_FR]!, from: "es", to: "en" };
    return { ...c, models: { ...c.models, "es-en/base-memory/2.0": esEn }, routes };
  };

  it("refuses a catalogue without a base, models or routes", () => {
    expect(() => parseCatalogue(null)).toThrow(/required/);
    expect(() => parseCatalogue({ ...valid(), base: 2 })).toThrow(/required/);
    expect(() => parseCatalogue({ ...valid(), models: undefined })).toThrow(/required/);
    expect(() => parseCatalogue({ ...valid(), routes: undefined })).toThrow(/required/);
  });

  it("refuses a base that is not an http(s) address", () => {
    expect(() => parseCatalogue({ ...valid(), base: "file:///tmp/" })).toThrow(/base/);
  });

  it("refuses a file without its pins", () => {
    const c = valid();
    const model = c.models[EN_FR]!;
    const files = (over: object) => ({ ...c, models: { [EN_FR]: { ...model, files: { ...model.files, ...over } } } });
    expect(() => parseCatalogue(files({ lex: { ...model.files.lex, sha256: "abc" } }))).toThrow(/lex/);
    expect(() => parseCatalogue(files({ lex: { ...model.files.lex, unpacked: undefined } }))).toThrow(/unpacked/);
    expect(() => parseCatalogue(files({ vocab: undefined }))).toThrow(/vocab/);
  });

  it("refuses a model without its languages", () => {
    const c = valid();
    const model = { ...c.models[EN_FR]!, to: undefined };
    expect(() => parseCatalogue({ ...c, models: { [EN_FR]: model } })).toThrow(/languages/);
  });

  it("refuses a route that names a model the catalogue does not list", () => {
    expect(() => parseCatalogue(withRoutes({ "en-fr": ["en-fr/tiny/1.0"] }))).toThrow(/not in the catalogue/);
  });

  it("A route that does not start from its studied language: en-es starting with the es→en model is refused", () => {
    expect(() => parseCatalogue(withRoutes({ "en-es": [ES_EN] }))).toThrow(/translates from es, with en/);
    expect(() => parseCatalogue(withRoutes({ "es-fr": [EN_FR] }))).toThrow(/translates from en, with es/);
  });

  it("refuses a route that breaks the chain", () => {
    expect(() => parseCatalogue(withRoutes({ "es-fr": [ES_EN, ES_EN] }))).toThrow(/translates from es, with en/);
  });

  it("A route that does not reach French: es-fr ending in English is refused, and nothing is fetched", () => {
    expect(() => parseCatalogue(withRoutes({ "es-fr": [ES_EN] }))).toThrow(/ends in en, not fr/);
    expect(() => parseCatalogue(withRoutes({ "es-fr": [] }))).toThrow(/names no model/);
  });

  it("A route that does not reach its native language: en-es ending in French is refused", () => {
    // The parser no longer knows French: the native language is read from the key (routes-by-pair D1).
    expect(() => parseCatalogue(withRoutes({ "en-es": [EN_FR] }))).toThrow(/ends in fr, not es/);
  });

  it("A route whose key is no pair: `en`, `en-` and `-fr` are refused", () => {
    expect(() => parseCatalogue(withRoutes({ en: [EN_FR] }))).toThrow(/en is no pair/);
    expect(() => parseCatalogue(withRoutes({ "en-": [EN_FR] }))).toThrow(/en- is no pair/);
    expect(() => parseCatalogue(withRoutes({ "-fr": [EN_FR] }))).toThrow(/-fr is no pair/);
  });

  it("splits a key on its first `-`, as a pack's name is: `en-fr-x` is the pair en / fr-x, which no route ends in", () => {
    expect(() => parseCatalogue(withRoutes({ "en-fr-x": [EN_FR] }))).toThrow(/ends in fr, not fr-x/);
    // The same reading as analyzer/pairs.ts's: a model into `fr-x` would make the route whole.
    expect(nativeOf("en-fr-x")).toBe("fr-x");
    expect(studiedOf("en-fr-x")).toBe("en");
    const base = withRoutes({ "en-fr-x": ["en-frx/base-memory/2.0"] });
    const models: ModelCatalogue["models"] = base.models;
    const raw = {
      ...base,
      models: { ...models, "en-frx/base-memory/2.0": { ...models[EN_FR]!, from: "en", to: "fr-x" } },
    };
    expect(routeOf(parseCatalogue(raw), "en-fr-x").map((m) => m.to)).toEqual(["fr-x"]);
  });

  it("accepts a route through English, in order", () => {
    const c = parseCatalogue(withRoutes({ "en-fr": [EN_FR], "es-fr": [ES_EN, EN_FR] }));
    expect(routeOf(c, "es-fr").map((m) => m.version)).toEqual([ES_EN, EN_FR]);
  });

  it("accepts a pair glossed in another native language, with its own route", () => {
    const c = parseCatalogue(withRoutes({ "en-fr": [EN_FR], "es-en": [ES_EN] }));
    expect(routeOf(c, "es-en").map((m) => m.version)).toEqual([ES_EN]);
  });

  it("keeps only what the runtime needs", () => {
    const c = parseCatalogue({ ...committed, extra: true });
    expect(Object.keys(c).sort()).toEqual(["base", "models", "routes"]);
    expect(Object.keys(c.models[EN_FR]!).sort()).toEqual(["files", "from", "to"]);
    expect(Object.keys(c.models[EN_FR]!.files.model).sort()).toEqual(["path", "sha256", "size", "unpacked"]);
  });
});

describe("loading the package's catalogue", () => {
  it("reads the package's own copy — a file of the extension", async () => {
    const fetchFn = vi.fn(async () => new Response(JSON.stringify(committed)));
    const c = await loadBundledCatalogue(fetchFn as unknown as typeof fetch);
    expect(fetchFn).toHaveBeenCalledWith(MANIFEST_PATH);
    expect(Object.keys(c.models)).toEqual([EN_FR, ES_EN, EN_ES, FR_EN, IT_EN, EN_IT]);
  });

  it("fails on a missing file rather than guess", async () => {
    const fetchFn = vi.fn(async () => new Response("", { status: 404 }));
    await expect(loadBundledCatalogue(fetchFn as unknown as typeof fetch)).rejects.toThrow(/404/);
  });
});

describe("the models a device needs (generalise-lingua-translation-model-state D2)", () => {
  /** The committed catalogue with es-en and a Spanish route through English. */
  const pivot = () =>
    parseCatalogue({
      ...committed,
      models: { ...committed.models, "es-en/base-memory/2.0": { ...committed.models[EN_FR], from: "es", to: "en" } },
      routes: { ...committed.routes, "es-fr": ["es-en/base-memory/2.0", EN_FR] },
    });

  it("is the union of the pairs' routes, each model once, in order (routes-by-pair D5)", () => {
    expect(modelsFor(pivot(), ["en-fr", "es-fr"]).map((m) => m.version)).toEqual([EN_FR, "es-en/base-memory/2.0"]);
    expect(modelsFor(pivot(), ["es-fr", "en-fr"]).map((m) => m.version)).toEqual(["es-en/base-memory/2.0", EN_FR]);
  });

  it("needs nothing for a pair without a route", () => {
    expect(modelsFor(parseCatalogue(committed), ["de-fr"])).toEqual([]);
    expect(modelsFor(parseCatalogue(committed), ["en-fr", "de-fr"]).map((m) => m.version)).toEqual([EN_FR]);
  });

  it("needs the one model of es-en's and en-es's routes, for the pairs that will ship them (changes 34, 35)", () => {
    expect(modelsFor(parseCatalogue(committed), ["es-en"]).map((m) => m.version)).toEqual([ES_EN]);
    expect(modelsFor(parseCatalogue(committed), ["en-es"]).map((m) => m.version)).toEqual([EN_ES]);
  });

  it("needs fr-en's model for fr-en, and fr-en's and en-es's for fr-es, each once — for the pairs change 52 ships", () => {
    expect(modelsFor(parseCatalogue(committed), ["fr-en"]).map((m) => m.version)).toEqual([FR_EN]);
    expect(modelsFor(parseCatalogue(committed), ["fr-es"]).map((m) => m.version)).toEqual([FR_EN, EN_ES]);
    expect(modelsFor(parseCatalogue(committed), ["en-es", "fr-es"]).map((m) => m.version)).toEqual([EN_ES, FR_EN]);
  });

  it("needs it-en's model for it-en and as the first of it-fr and it-es, en-it's for en-it and as the second of fr-it and es-it, each once — for the pairs the Italian programme ships", () => {
    const c = parseCatalogue(committed);
    expect(modelsFor(c, ["it-en"]).map((m) => m.version)).toEqual([IT_EN]);
    expect(modelsFor(c, ["it-fr", "it-es"]).map((m) => m.version)).toEqual([IT_EN, EN_FR, EN_ES]);
    expect(modelsFor(c, ["en-it", "fr-it", "es-it"]).map((m) => m.version)).toEqual([EN_IT, FR_EN, ES_EN]);
  });

  it("needs both models of the committed es-fr route, en-fr once", () => {
    expect(modelsFor(parseCatalogue(committed), ["en-fr", "es-fr"]).map((m) => m.version)).toEqual([EN_FR, ES_EN]);
    expect(modelsFor(parseCatalogue(committed), ["es-fr"]).map((m) => m.version)).toEqual([ES_EN, EN_FR]);
  });

  it("finds models by id, leaving out an id the catalogue does not list", () => {
    expect(modelsById(pivot(), ["es-en/base-memory/2.0", "de-en/tiny/1.0"]).map((m) => [m.version, m.from])).toEqual([
      ["es-en/base-memory/2.0", "es"],
    ]);
  });
});
