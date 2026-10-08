import { describe, expect, it } from "vitest";
import {
  heldModels,
  loaded,
  type LoadedRoute,
  MODEL_BOUND,
  modelsOf,
  NONE,
  orphans,
  type Residency,
  toLoad,
  used,
} from "@/translate/host/model-residency.ts";

// The catalogue's ids, and the routes of the matrix's natives (harden-lingua-translation-engine D3).
const EN_FR = "en-fr/base-memory/2.0";
const ES_EN = "es-en/base-memory/2.0";
const EN_ES = "en-es/base-memory/2.1";
const FR_EN = "fr-en/base-memory/2.0";
const ROUTE: Record<string, LoadedRoute> = {
  "en-fr": { pair: "en-fr", models: [EN_FR] },
  "es-fr": { pair: "es-fr", models: [ES_EN, EN_FR] },
  "es-en": { pair: "es-en", models: [ES_EN] },
  "en-es": { pair: "en-es", models: [EN_ES] },
  "fr-en": { pair: "fr-en", models: [FR_EN] },
  "fr-es": { pair: "fr-es", models: [FR_EN, EN_ES] },
};

/** Load `route` as the worker does: decide, then record — the models it was told to delete are gone. */
function load(residency: Residency, route: LoadedRoute) {
  const decision = toLoad(residency, route);
  return { ...decision, residency: loaded(decision.residency, route) };
}

const pairs = (residency: Residency) => residency.map((r) => r.pair);

describe("The engine holds at most two models (harden-lingua-translation-engine D3)", () => {
  it("the bound is two: what one pair's route needs, and what a reader of French with English and Spanish holds", () => {
    expect(MODEL_BOUND).toBe(2);
  });

  it("A reader of French with English and Spanish: en-fr then es-fr, en-fr shared, nothing deleted", () => {
    const english = load(NONE, ROUTE["en-fr"]!);
    expect(english).toMatchObject({ evict: [], drop: [] });
    const spanish = load(english.residency, ROUTE["es-fr"]!);
    expect(spanish).toMatchObject({ evict: [], drop: [] });
    expect(heldModels(spanish.residency)).toEqual([ES_EN, EN_FR]); // two, en-fr once
    expect(pairs(spanish.residency)).toEqual(["en-fr", "es-fr"]);
  });

  it("A third model: the least recently used model the route does not need is deleted first, its route dropped, two held", () => {
    let r = load(NONE, ROUTE["en-fr"]!).residency;
    r = load(r, ROUTE["es-en"]!).residency;
    const decision = toLoad(r, ROUTE["fr-en"]!);
    expect(decision.evict).toEqual([EN_FR]); // loaded first, used last the longest ago
    expect(decision.drop).toEqual(["en-fr"]);
    expect(pairs(decision.residency)).toEqual(["es-en"]);
    const after = loaded(decision.residency, ROUTE["fr-en"]!);
    expect(heldModels(after)).toEqual([ES_EN, FR_EN]);
    expect(heldModels(after)).toHaveLength(MODEL_BOUND);
  });

  it("a translation is a use: the route translated through last is kept over the one loaded last", () => {
    let r = load(NONE, ROUTE["en-fr"]!).residency;
    r = load(r, ROUTE["es-en"]!).residency;
    r = used(r, "en-fr"); // the reader went back to the English page
    const decision = toLoad(r, ROUTE["fr-en"]!);
    expect(decision.evict).toEqual([ES_EN]);
    expect(decision.drop).toEqual(["es-en"]);
  });

  it("a load is a use: a route loaded again, already held, becomes the most recent and costs nothing", () => {
    let r = load(NONE, ROUTE["en-fr"]!).residency;
    r = load(r, ROUTE["es-en"]!).residency;
    const again = load(r, ROUTE["en-fr"]!);
    expect(again).toMatchObject({ evict: [], drop: [] });
    expect(pairs(again.residency)).toEqual(["es-en", "en-fr"]);
    expect(toLoad(again.residency, ROUTE["fr-en"]!).evict).toEqual([ES_EN]);
  });

  it("Two loads at once, as the worker serialises them: the second decides on what the first left, never over a deleted model", () => {
    // Neither loaded; the worker runs them one at a time (D3).
    const first = load(NONE, ROUTE["en-fr"]!);
    expect(first.evict).toEqual([]);
    const second = load(first.residency, ROUTE["es-en"]!);
    expect(second.evict).toEqual([]);
    expect(heldModels(second.residency)).toHaveLength(2);

    // Two more, each at the bound: the second sees the first's eviction and does not count the deleted model.
    const third = load(second.residency, ROUTE["fr-en"]!);
    expect(third.evict).toEqual([EN_FR]);
    const fourth = load(third.residency, ROUTE["es-fr"]!); // es-en held, en-fr to be rebuilt
    expect(fourth.evict).toEqual([FR_EN]); // the one model held that es-fr does not need
    expect(fourth.drop).toEqual(["fr-en"]);
    expect(heldModels(fourth.residency)).toHaveLength(2);
    expect(modelsOf(fourth.residency, "es-fr")).toEqual([ES_EN, EN_FR]);
    for (const step of [first, second, third, fourth]) expect(heldModels(step.residency).length).toBeLessThanOrEqual(2);
  });

  it("Back to a deleted model's language: the route is loaded again, and the model it needs is not among the evicted", () => {
    let r = load(NONE, ROUTE["en-fr"]!).residency;
    r = load(r, ROUTE["es-en"]!).residency;
    r = load(r, ROUTE["fr-en"]!).residency; // en-fr deleted
    expect(modelsOf(r, "en-fr")).toEqual([]);
    const back = toLoad(r, ROUTE["en-fr"]!);
    expect(back.evict).toEqual([ES_EN]);
    expect(back.drop).toEqual(["es-en"]);
    expect(pairs(loaded(back.residency, ROUTE["en-fr"]!))).toEqual(["fr-en", "en-fr"]);
  });

  it("a route of two models evicts everything else, and drops every route that went through it", () => {
    let r = load(NONE, ROUTE["fr-en"]!).residency;
    r = load(r, ROUTE["en-es"]!).residency;
    r = load(r, ROUTE["fr-es"]!).residency; // shares both: nothing deleted
    expect(heldModels(r)).toHaveLength(2);
    const decision = toLoad(r, ROUTE["es-fr"]!);
    expect(decision.evict).toEqual([FR_EN, EN_ES]);
    expect(decision.drop).toEqual(["fr-en", "en-es", "fr-es"]);
    expect(decision.residency).toEqual([]);
  });

  it("a pivot's second model is used after its first: es-fr's es-en goes before its en-fr", () => {
    let r = load(NONE, ROUTE["en-fr"]!).residency;
    r = load(r, ROUTE["es-fr"]!).residency;
    // Both models were last used by es-fr; en-fr, its second, is the more recent — and the en-fr route still needs it.
    expect(heldModels(r)).toEqual([ES_EN, EN_FR]);
    const decision = toLoad(r, ROUTE["fr-en"]!);
    expect(decision.evict).toEqual([ES_EN]);
    expect(decision.drop).toEqual(["es-fr"]); // en-fr's route stands
    expect(pairs(decision.residency)).toEqual(["en-fr"]);
  });

  it("a deleted model drops every route that went through it, a one-model route included", () => {
    let r = load(NONE, ROUTE["es-fr"]!).residency;
    r = load(r, ROUTE["en-fr"]!).residency; // en-fr held already: nothing deleted
    r = used(r, "es-fr");
    const decision = toLoad(r, ROUTE["es-en"]!); // es-en is held: at the bound, nothing to delete
    expect(decision.evict).toEqual([]);
    const third = toLoad(loaded(decision.residency, ROUTE["es-en"]!), ROUTE["en-es"]!);
    expect(third.evict).toEqual([EN_FR]);
    expect(third.drop.sort()).toEqual(["en-fr", "es-fr"]);
  });

  it("a use of a pair not loaded, and a load of nothing, change nothing", () => {
    expect(used(NONE, "en-fr")).toEqual([]);
    const r = load(NONE, ROUTE["en-fr"]!).residency;
    expect(used(r, "es-fr")).toEqual(r);
    expect(toLoad(NONE, ROUTE["es-fr"]!)).toEqual({ evict: [], drop: [], residency: [] });
  });

  it("is pure: a decision leaves the residency it was given as it was", () => {
    const r = load(load(NONE, ROUTE["en-fr"]!).residency, ROUTE["es-en"]!).residency;
    const before = JSON.stringify(r);
    toLoad(r, ROUTE["fr-en"]!);
    used(r, "en-fr");
    loaded(r, ROUTE["fr-en"]!);
    expect(JSON.stringify(r)).toBe(before);
  });
});

describe("The worker's bookkeeping follows the decision: what it holds is the union of its routes", () => {
  /**
   * engine-worker.ts's steps, over a set of model ids standing for the engine's models: decide on
   * what is held, delete, build, record — and, when a build fails, delete what no route goes through.
   */
  function worker() {
    const models = new Set<string>();
    let residency: Residency = NONE;
    const union = () => [...new Set(residency.flatMap((r) => r.models))].sort();
    return {
      models,
      residency: () => residency,
      /** What the worker holds is its routes' union, and never more than the bound. */
      invariant() {
        expect([...models].sort()).toEqual(union());
        expect(models.size).toBeLessThanOrEqual(MODEL_BOUND);
      },
      load(route: LoadedRoute) {
        if (modelsOf(residency, route.pair).length > 0) {
          residency = used(residency, route.pair);
          return { evict: [], drop: [] };
        }
        const decision = toLoad(residency, route, { held: models });
        for (const id of decision.evict) models.delete(id);
        residency = decision.residency;
        for (const id of route.models) models.add(id);
        residency = loaded(residency, route);
        return decision;
      },
      /** The decision applied, `built` built, then the build failed: as the worker, delete what no route goes through. */
      failToLoad(route: LoadedRoute, built: string[]) {
        const decision = toLoad(residency, route, { held: models });
        for (const id of decision.evict) models.delete(id);
        residency = decision.residency;
        for (const id of built) models.add(id);
        for (const id of orphans(models, residency)) models.delete(id);
        return decision;
      },
    };
  }

  it("es-fr, then fr-en, then es-en: the model es-fr leaves behind is deleted with its route, and the worker holds two", () => {
    const w = worker();
    w.load(ROUTE["es-fr"]!);
    expect([...w.models]).toEqual([ES_EN, EN_FR]);
    const french = w.load(ROUTE["fr-en"]!);
    // es-en goes by the bound and drops es-fr; en-fr, es-fr's other model, is left with no route and goes with it.
    expect(french.evict).toEqual([ES_EN, EN_FR]);
    expect(french.drop).toEqual(["es-fr"]);
    expect([...w.models]).toEqual([FR_EN]);
    w.invariant();
    const spanish = w.load(ROUTE["es-en"]!);
    expect(spanish.evict).toEqual([]);
    expect([...w.models]).toEqual([FR_EN, ES_EN]); // two — not en-fr, fr-en and es-en
    w.invariant();
  });

  it("es-fr, then en-es, then es-en: the same, through the other native's route", () => {
    const w = worker();
    w.load(ROUTE["es-fr"]!);
    const first = w.load(ROUTE["en-es"]!);
    expect(first.evict).toEqual([ES_EN, EN_FR]);
    expect(first.drop).toEqual(["es-fr"]);
    expect([...w.models]).toEqual([EN_ES]);
    w.load(ROUTE["es-en"]!);
    expect([...w.models]).toEqual([EN_ES, ES_EN]);
    w.invariant();
  });

  it("every walk of four loads over the matrix's six routes keeps the worker at two models, the union of its routes", () => {
    const routes = Object.values(ROUTE);
    for (const a of routes) {
      for (const b of routes) {
        for (const c of routes) {
          for (const d of routes) {
            const w = worker();
            for (const route of [a, b, c, d]) {
              w.load(route);
              w.invariant();
            }
          }
        }
      }
    }
  });

  it("a model held outside every route is deleted first, whatever the bound, and before a route is dropped for it", () => {
    const r = load(NONE, ROUTE["en-fr"]!).residency;
    const decision = toLoad(r, ROUTE["fr-en"]!, { held: [EN_FR, ES_EN] }); // es-en held by no route
    expect(decision.evict).toEqual([ES_EN]);
    expect(decision.drop).toEqual([]); // en-fr fits beside fr-en once the orphan is gone
    expect(pairs(decision.residency)).toEqual(["en-fr"]);
  });

  it("a route over a model the worker does not hold is dropped: a route over a deleted model is no route", () => {
    const r = load(load(NONE, ROUTE["en-fr"]!).residency, ROUTE["es-en"]!).residency;
    const decision = toLoad(r, ROUTE["fr-en"]!, { held: [ES_EN] });
    expect(decision.drop).toEqual(["en-fr"]);
    expect(decision.evict).toEqual([]);
    expect(pairs(decision.residency)).toEqual(["es-en"]);
  });

  it("orphans: the models held that no route goes through", () => {
    const r = load(NONE, ROUTE["es-fr"]!).residency;
    expect(orphans([ES_EN, EN_FR, FR_EN], r)).toEqual([FR_EN]);
    expect(orphans([ES_EN], NONE)).toEqual([ES_EN]);
    expect(orphans([], r)).toEqual([]);
  });

  it("a build that fails after the decision leaves the worker holding its routes' models and no more", () => {
    const w = worker();
    // The first model of es-fr built, the second not: the first is deleted with the failure.
    expect(w.failToLoad(ROUTE["es-fr"]!, [ES_EN]).evict).toEqual([]);
    expect([...w.models]).toEqual([]);
    w.invariant();
    // es-fr held; a route over es-en and a model that cannot be built: en-fr goes by the bound and
    // es-fr with it, es-en stays for the route — which fails. Kept, es-en would be held for nothing.
    w.load(ROUTE["es-fr"]!);
    const decision = w.failToLoad({ pair: "es-it", models: [ES_EN, "en-it/base-memory/2.0"] }, []);
    expect(decision.evict).toEqual([EN_FR]);
    expect(decision.drop).toEqual(["es-fr"]);
    expect(w.residency()).toEqual([]);
    expect([...w.models]).toEqual([]);
    w.invariant();
  });
});
