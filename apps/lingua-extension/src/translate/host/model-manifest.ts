// The translation models the engine may download (add-lingua-translation-delivery D3), as the
// package states them: model-manifest.json, bundled by build.mjs. Since
// generalise-lingua-translation-catalogue it is a catalogue: the models, each file with its address,
// its size as served, its size once decompressed and the sha256 of its DECOMPRESSED bytes; and for
// each pair the route of models that translates its studied language into its native language
// (generalise-lingua-translation-routes-by-pair D1). Because the package carries it, the reviewed
// package decides what is accepted; the host only serves bytes, and cannot substitute a model.

export const MODEL_ROLES = ["model", "lex", "vocab"] as const;
export type ModelRole = (typeof MODEL_ROLES)[number];

export interface ModelFile {
  /** Relative to `base`; content-addressed, so a path never changes meaning. */
  path: string;
  /** Bytes as served (gzip), for progress. */
  size: number;
  /** Bytes once decompressed: what the file takes on the device. */
  unpacked: number;
  /** sha256 of the decompressed bytes: what is checked, and the key it is stored under. */
  sha256: string;
}

/** A model of the catalogue: the languages it translates between, and its files. */
export interface CatalogueModel {
  from: string;
  to: string;
  files: Record<ModelRole, ModelFile>;
}

export interface ModelCatalogue {
  base: string;
  /** By id: the version a device records once the model is stored (`en-fr/base-memory/2.0`). */
  models: Record<string, CatalogueModel>;
  /** For each pair (`en-fr`), the ids of the models that translate its studied language into its native one, in order. */
  routes: Record<string, string[]>;
}

/** One model as the runtime downloads, stores and loads it: its id is the version stored. */
export interface ModelManifest {
  version: string;
  base: string;
  from: string;
  to: string;
  files: Record<ModelRole, ModelFile>;
}

/** Where the package keeps it, relative to the extension's root. */
export const MANIFEST_PATH = "model-manifest.json";

const HEX64 = /^[0-9a-f]{64}$/;

const isSize = (n: unknown): n is number => typeof n === "number" && Number.isInteger(n) && n >= 0;

function parseFiles(id: string, raw: unknown): Record<ModelRole, ModelFile> {
  const given = (raw ?? {}) as Record<string, Partial<ModelFile> | undefined>;
  const files = {} as Record<ModelRole, ModelFile>;
  for (const role of MODEL_ROLES) {
    const f = given[role];
    if (!f || typeof f.path !== "string" || !isSize(f.size) || !isSize(f.unpacked) || !HEX64.test(f.sha256 ?? "")) {
      throw new Error(`model-manifest.json: ${id} ${role} needs a path, a size, an unpacked size and a sha256`);
    }
    files[role] = { path: f.path, size: f.size, unpacked: f.unpacked, sha256: f.sha256! };
  }
  return files;
}

/**
 * A route's key as its two languages — split on its first `-`, as analyzer/pairs.ts splits a pack's
 * name — or null when it is no pair: no `-`, or an empty side.
 */
function splitPair(key: string): { studied: string; native: string } | null {
  const dash = key.indexOf("-");
  if (dash < 1 || dash === key.length - 1) return null;
  return { studied: key.slice(0, dash), native: key.slice(dash + 1) };
}

/**
 * The catalogue, or an error naming what is wrong with it: a broken one must fetch nothing
 * (generalise-lingua-translation-catalogue D2). Each route is keyed `<studied>-<native>`, names
 * known models, starts from the studied language, chains each model's target to the next one's
 * source, and ends in the native language (generalise-lingua-translation-routes-by-pair D1).
 */
export function parseCatalogue(raw: unknown): ModelCatalogue {
  const c = raw as Partial<ModelCatalogue> | null;
  if (!c || typeof c.base !== "string" || !c.models || typeof c.models !== "object" || !c.routes) {
    throw new Error("model-manifest.json: base, models and routes are required");
  }
  if (!/^https?:\/\//.test(c.base)) throw new Error("model-manifest.json: base must be an http(s) address");
  const models: Record<string, CatalogueModel> = {};
  for (const [id, m] of Object.entries(c.models as Record<string, Partial<CatalogueModel>>)) {
    if (typeof m?.from !== "string" || typeof m.to !== "string") {
      throw new Error(`model-manifest.json: ${id} needs the languages it translates between`);
    }
    models[id] = { from: m.from, to: m.to, files: parseFiles(id, m.files) };
  }
  const routes: Record<string, string[]> = {};
  for (const [pair, route] of Object.entries(c.routes as Record<string, unknown>)) {
    const languages = splitPair(pair);
    if (!languages) throw new Error(`model-manifest.json: the route key ${pair} is no pair (<studied>-<native>)`);
    if (!Array.isArray(route) || route.length === 0) {
      throw new Error(`model-manifest.json: the ${pair} route names no model`);
    }
    let at = languages.studied;
    for (const id of route) {
      const model = models[id as string];
      if (!model) throw new Error(`model-manifest.json: the ${pair} route names ${String(id)}, not in the catalogue`);
      if (model.from !== at) {
        throw new Error(
          `model-manifest.json: the ${pair} route reaches ${id}, which translates from ${model.from}, with ${at}`,
        );
      }
      at = model.to;
    }
    if (at !== languages.native) {
      throw new Error(`model-manifest.json: the ${pair} route ends in ${at}, not ${languages.native}`);
    }
    routes[pair] = route as string[];
  }
  return { base: c.base, models, routes };
}

/** The models of `pair`'s route, in order; none when the catalogue lists no route for it. */
export function routeOf(catalogue: ModelCatalogue, pair: string): ModelManifest[] {
  return modelsById(catalogue, catalogue.routes[pair] ?? []);
}

export function fileUrl(manifest: ModelManifest, file: ModelFile): string {
  return new URL(file.path, manifest.base).href;
}

/** What the reader downloads, all files together. */
export function totalSize(manifest: ModelManifest): number {
  return MODEL_ROLES.reduce((sum, role) => sum + manifest.files[role].size, 0);
}

/** What the model takes on the device once its files are decompressed. */
export function unpackedSize(manifest: ModelManifest): number {
  return MODEL_ROLES.reduce((sum, role) => sum + manifest.files[role].unpacked, 0);
}

/** The package's own copy — a file of the extension, never fetched from anywhere else. */
export async function loadBundledCatalogue(fetchFn: typeof fetch = fetch): Promise<ModelCatalogue> {
  const response = await fetchFn(MANIFEST_PATH);
  if (!response.ok) throw new Error(`${MANIFEST_PATH}: ${response.status}`);
  return parseCatalogue(await response.json());
}

/**
 * The models a device needs for `pairs`: the union of their routes, in their order, each once
 * (generalise-lingua-translation-model-state D2, routes-by-pair D5). A pair without a route needs
 * nothing.
 */
export function modelsFor(catalogue: ModelCatalogue, pairs: readonly string[]): ModelManifest[] {
  const needed = new Map<string, ModelManifest>();
  for (const pair of pairs) {
    for (const model of routeOf(catalogue, pair)) if (!needed.has(model.version)) needed.set(model.version, model);
  }
  return [...needed.values()];
}

/** The catalogue's models named by `ids`, in that order; an id it does not list is left out. */
export function modelsById(catalogue: ModelCatalogue, ids: readonly string[]): ModelManifest[] {
  return ids.flatMap((id) => {
    const model = catalogue.models[id];
    return model ? [{ version: id, base: catalogue.base, from: model.from, to: model.to, files: model.files }] : [];
  });
}
