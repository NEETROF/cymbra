import { describe, expect, it } from "vitest";
import { bundledCatalogue, mirrorTag, modelsOf, readCatalogue, ROLES } from "../tool/model-catalogue.mjs";
import { planMirrors } from "../tool/mirror_models.mjs";
import { parseCatalogue, routeOf } from "@/translate/host/model-manifest.ts";

// The catalogue as the build and the model host's tools read it (generalise-lingua-translation-catalogue
// D5): one module, so the bundled copy, the variant check, the host's assembly and check, and the
// mirror releases agree on what the models are.

const EN_FR = "en-fr/base-memory/2.0";

/** The committed catalogue with a second model and a Spanish route through English. */
function withSpanish() {
  const c = readCatalogue();
  const enFr = c.models[EN_FR]!;
  return {
    ...c,
    models: {
      ...c.models,
      "es-en/base-memory/2.0": {
        ...enFr,
        from: "es",
        to: "en",
        mirror: "https://github.com/NEETROF/cymbra/releases/download/lingua-model-es-en-base-memory-2.0/",
      },
    },
    routes: { ...c.routes, es: ["es-en/base-memory/2.0", EN_FR] },
  };
}

describe("the catalogue as the tools read it", () => {
  it("lists every model with its id", () => {
    expect(modelsOf(withSpanish()).map((m) => [m.id, m.from, m.to])).toEqual([
      [EN_FR, "en", "fr"],
      ["es-en/base-memory/2.0", "es", "en"],
    ]);
  });

  it("bundles what the runtime needs, every model and route, and no deployment detail", () => {
    const bundled = bundledCatalogue(withSpanish());
    expect(Object.keys(bundled).sort()).toEqual(["base", "models", "routes"]);
    expect(bundled.routes).toEqual({ en: [EN_FR], es: ["es-en/base-memory/2.0", EN_FR] });
    for (const model of Object.values(bundled.models)) {
      expect(Object.keys(model).sort()).toEqual(["files", "from", "licence", "to"]);
      expect(Object.keys(model.files)).toEqual([...ROLES]);
      for (const file of Object.values(model.files)) {
        expect(Object.keys(file).sort()).toEqual(["path", "sha256", "size", "unpacked"]);
      }
    }
    // What the package carries is what the runtime accepts.
    expect(routeOf(parseCatalogue(bundled), "es").map((m) => m.version)).toEqual(["es-en/base-memory/2.0", EN_FR]);
  });

  it("points a development build at another host", () => {
    expect(bundledCatalogue(readCatalogue(), "http://127.0.0.1:8765/").base).toBe("http://127.0.0.1:8765/");
    expect(bundledCatalogue(readCatalogue()).base).toBe("https://models.cymbra.app/");
  });

  it("names a model's mirror release by the last segment of its address", () => {
    expect(mirrorTag(readCatalogue().models[EN_FR] as { mirror: string })).toBe("lingua-model-en-fr-base-memory-2.0");
  });
});

describe("the mirror releases", () => {
  it("keeps a release that exists and creates the missing one, with its model's files and notice", () => {
    const plan = planMirrors(withSpanish(), "/site", (tag) => tag === "lingua-model-en-fr-base-memory-2.0");
    expect(plan.map((step) => [step.kind, step.tag])).toEqual([
      ["kept", "lingua-model-en-fr-base-memory-2.0"],
      ["create", "lingua-model-es-en-base-memory-2.0"],
    ]);
    const created = plan[1]!;
    expect(created.kind === "create" && created.files).toEqual([
      ...ROLES.map((role) => `/site/${withSpanish().models["es-en/base-memory/2.0"]!.files[role]!.path}`),
      "/site/es-en/base-memory/2.0/NOTICE.txt",
    ]);
    expect(created.kind === "create" && created.notes).toContain("`es-en/base-memory/2.0`");
  });

  it("creates nothing for a model without a mirror", () => {
    const c = withSpanish();
    delete (c.models["es-en/base-memory/2.0"] as { mirror?: string }).mirror;
    expect(planMirrors(c, "/site", () => false).map((step) => step.model)).toEqual([EN_FR]);
  });
});
