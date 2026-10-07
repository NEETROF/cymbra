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
//
// The invariant the decision keeps is that what the worker holds is the union of its routes. A
// route dropped with a deleted model leaves its other model with no route; that model is deleted
// too, unless the new route needs it — kept, it would be held and counted by nothing, and a third
// would fit beside it. The count is taken from the models the worker says it holds, not from the
// routes, so a model held outside every route is found and deleted whatever left it there.

/** The most models the worker holds: what one pair's route needs. */
export const MODEL_BOUND = 2;

/** A route loaded in the worker: its pair, and the catalogue ids of its models in order. */
export interface LoadedRoute {
  pair: string;
  models: readonly string[];
}

/** The loaded routes, least recently used first. */
export type Residency = readonly LoadedRoute[];

/**
 * What loading a route costs: the models to delete — those in no route first, then the least
 * recently used the route does not need, then those its dropped routes leave with none — and the
 * routes dropped with them.
 */
export interface LoadDecision {
  evict: string[];
  drop: string[];
  /** The residency once they are gone — the route itself is not in it until `loaded`. */
  residency: Residency;
}

/** What the worker is told before it builds: the models it holds, and the bound. */
export interface LoadOptions {
  /** The model ids the worker actually holds; the routes' own when not given. */
  held?: Iterable<string>;
  bound?: number;
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

/** Every model a route goes through, least recently used first: a model's last use is its most recent route's, the pivot's second model after its first. */
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

/** Among `held`, the models no route of `residency` goes through: held for nothing, whatever left them there. */
export function orphans(held: Iterable<string>, residency: Residency): string[] {
  const inRoutes = new Set(residency.flatMap((r) => r.models));
  return [...held].filter((id) => !inRoutes.has(id));
}

/**
 * What loading `route` costs, before anything is built: first every model held that no route
 * goes through, whatever the bound; then, among the models the route does not need, the least
 * recently used first, until the models held and the route's make `bound` at most; then the
 * models the dropped routes leave with no route. A route whose models are all held costs nothing;
 * a route of two models evicts everything else. The routes that went through an evicted model are
 * dropped with it, and so is a route over a model the worker does not hold.
 */
export function toLoad(residency: Residency, route: LoadedRoute, opts: LoadOptions = {}): LoadDecision {
  const bound = opts.bound ?? MODEL_BOUND;
  const needed = new Set(route.models);
  const held = new Set(opts.held ?? heldModels(residency));
  const evict = orphans(held, residency).filter((id) => !needed.has(id));
  let count = new Set([...held, ...needed]).size - evict.length;
  for (const id of heldModels(residency)) {
    if (count <= bound) break;
    if (needed.has(id) || !held.has(id)) continue;
    evict.push(id);
    count--;
  }
  const gone = new Set(evict);
  const dropped = residency.filter((r) => r.models.some((id) => gone.has(id) || !held.has(id)));
  const remaining = residency.filter((r) => !dropped.includes(r));
  for (const id of orphans(held, remaining)) {
    if (gone.has(id) || needed.has(id)) continue;
    evict.push(id);
    gone.add(id);
  }
  return { evict, drop: dropped.map((r) => r.pair), residency: remaining };
}
