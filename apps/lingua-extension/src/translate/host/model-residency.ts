// What the engine's worker holds, and what loading one more route costs it
// (harden-lingua-translation-engine D3). Pure: the worker keeps a value of this shape beside its
// models and asks it before it builds anything; nothing here touches the engine, so the decision
// is tested apart from it (test/model-residency.spec.ts).
//
// The bound is two models. A route chains two at most (LONG_ROUTE), so two is what one pair
// needs, and it is what a reader of French with English and Spanish holds: en-fr, and es-en with
// en-fr shared. Measured, one model is 195.4 MiB in the worker and es-fr's two are 321.8 MiB;
// the models of a language the reader left, never deleted, made 463 MiB. Above the bound, the
// least recently used model the new route does not need goes first — a load and a translation
// both count as uses, and in a pivot the second model is used after the first — and every route
// that went through it is dropped with it: a route over a deleted model is no route.

/** The most models the worker holds: what one pair's route needs. */
export const MODEL_BOUND = 2;

/** A route loaded in the worker: its pair, and the catalogue ids of its models in order. */
export interface LoadedRoute {
  pair: string;
  models: readonly string[];
}

/** The loaded routes, least recently used first. */
export type Residency = readonly LoadedRoute[];

/** What loading a route costs: the models to delete, least recently used first, and the routes dropped with them. */
export interface LoadDecision {
  evict: string[];
  drop: string[];
  /** The residency once they are gone — the route itself is not in it until `loaded`. */
  residency: Residency;
}

/** Nothing loaded. */
export const NONE: Residency = [];

/** `pair`'s route used now — loaded, or translated through: it becomes the most recently used. A pair not loaded changes nothing. */
export function used(residency: Residency, pair: string): Residency {
  const route = residency.find((r) => r.pair === pair);
  if (!route) return residency;
  return [...residency.filter((r) => r !== route), route];
}

/** `route` built: recorded as the most recently used. A route already there is used, not doubled. */
export function loaded(residency: Residency, route: LoadedRoute): Residency {
  return [...residency.filter((r) => r.pair !== route.pair), { pair: route.pair, models: [...route.models] }];
}

/** The models of `pair`'s route, in the order a translation goes through them; empty for a pair not loaded. */
export function modelsOf(residency: Residency, pair: string): readonly string[] {
  return residency.find((r) => r.pair === pair)?.models ?? [];
}

/** Every model held, least recently used first: a model's last use is its most recent route's, the pivot's second model after its first. */
export function heldModels(residency: Residency): string[] {
  const recent: string[] = [];
  for (let i = residency.length - 1; i >= 0; i--) {
    const models = residency[i]!.models;
    for (let j = models.length - 1; j >= 0; j--) {
      const id = models[j]!;
      if (!recent.includes(id)) recent.push(id);
    }
  }
  return recent.reverse();
}

/**
 * What loading `route` costs, before anything is built: among the models it does not need, the
 * least recently used first, until the models held and the route's make `bound` at most. A route
 * whose models are all held costs nothing; a route of two models evicts everything else. The
 * routes that went through an evicted model are dropped with it.
 */
export function toLoad(residency: Residency, route: LoadedRoute, bound: number = MODEL_BOUND): LoadDecision {
  const needed = new Set(route.models);
  const held = heldModels(residency);
  const candidates = held.filter((id) => !needed.has(id));
  const evict: string[] = [];
  let count = new Set([...held, ...needed]).size;
  for (const id of candidates) {
    if (count <= bound) break;
    evict.push(id);
    count--;
  }
  const gone = new Set(evict);
  const dropped = residency.filter((r) => r.models.some((id) => gone.has(id)));
  return {
    evict,
    drop: dropped.map((r) => r.pair),
    residency: residency.filter((r) => !dropped.includes(r)),
  };
}
