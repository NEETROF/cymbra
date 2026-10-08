import { describe, expect, it } from "vitest";
import { bundledCatalogue, mirrorTag, modelsOf, readCatalogue, ROLES } from "../tool/model-catalogue.mjs";
import { planMirrors } from "../tool/mirror_models.mjs";
import { parseCatalogue, routeOf } from "@/translate/host/model-manifest.ts";

// The catalogue as the build and the model host's tools read it (generalise-lingua-translation-catalogue
// D5): one module, so the bundled copy, the variant check, the host's assembly and check, and the
// mirror releases agree on what the models are.

const EN_FR = "en-fr/base-memory/2.0";
const ES_EN = "es-en/base-memory/2.0";
const EN_ES = "en-es/base-memory/2.1";

describe("the catalogue as the tools read it", () => {
  it("lists every model with its id, in the order they were pinned", () => {
    expect(modelsOf(readCatalogue()).map((m) => [m.id, m.from, m.to])).toEqual([
      [EN_FR, "en", "fr"],
      [ES_EN, "es", "en"],
      [EN_ES, "en", "es"],
    ]);
  });

  it("bundles what the runtime needs, every model and route, and no deployment detail", () => {
    const bundled = bundledCatalogue(readCatalogue());
    expect(Object.keys(bundled).sort()).toEqual(["base", "models", "routes"]);
    // The routes go through as JSON, keyed by pair (routes-by-pair D1): check_variants compares them so.
    expect(bundled.routes).toEqual({
      "en-fr": [EN_FR],
      "es-fr": [ES_EN, EN_FR],
      "es-en": [ES_EN],
      "en-es": [EN_ES],
    });
    for (const model of Object.values(bundled.models)) {
      expect(Object.keys(model).sort()).toEqual(["files", "from", "licence", "to"]);
      expect(Object.keys(model.files)).toEqual([...ROLES]);
      for (const file of Object.values(model.files)) {
        expect(Object.keys(file).sort()).toEqual(["path", "sha256", "size", "unpacked"]);
      }
    }
    // What the package carries is what the runtime accepts.
    const parsed = parseCatalogue(bundled);
    expect(routeOf(parsed, "es-fr").map((m) => m.version)).toEqual([ES_EN, EN_FR]);
    expect(routeOf(parsed, "en-es").map((m) => m.version)).toEqual([EN_ES]);
  });

  it("points a development build at another host", () => {
    expect(bundledCatalogue(readCatalogue(), "http://127.0.0.1:8765/").base).toBe("http://127.0.0.1:8765/");
    expect(bundledCatalogue(readCatalogue()).base).toBe("https://models.cymbra.app/");
  });

  it("names a model's mirror release by the last segment of its address", () => {
    expect(mirrorTag(readCatalogue().models[EN_FR] as { mirror: string })).toBe("lingua-model-en-fr-base-memory-2.0");
    expect(mirrorTag(readCatalogue().models[EN_ES] as { mirror: string })).toBe("lingua-model-en-es-base-memory-2.1");
  });
});

describe("the mirror releases", () => {
  /** The releases lingua-model-deploy created for the two models pinned before en-es. */
  const before = (tag: string) =>
    tag === "lingua-model-en-fr-base-memory-2.0" || tag === "lingua-model-es-en-base-memory-2.0";

  it("keeps the releases that exist and creates en-es 2.1's, with its model's files and notice (matrix-models D1)", () => {
    const plan = planMirrors(readCatalogue(), "/site", before);
    expect(plan.map((step) => [step.kind, step.tag])).toEqual([
      ["kept", "lingua-model-en-fr-base-memory-2.0"],
      ["kept", "lingua-model-es-en-base-memory-2.0"],
      ["create", "lingua-model-en-es-base-memory-2.1"],
    ]);
    const created = plan[2]!;
    expect(created.kind === "create" && created.files).toEqual([
      ...ROLES.map((role) => `/site/${readCatalogue().models[EN_ES]!.files[role]!.path}`),
      "/site/en-es/base-memory/2.1/NOTICE.txt",
    ]);
    expect(created.kind === "create" && created.notes).toContain("`en-es/base-memory/2.1`");
  });

  it("creates nothing for a model without a mirror", () => {
    const c = readCatalogue();
    delete (c.models[EN_ES] as { mirror?: string }).mirror;
    expect(planMirrors(c, "/site", () => false).map((step) => step.model)).toEqual([EN_FR, ES_EN]);
  });
});
