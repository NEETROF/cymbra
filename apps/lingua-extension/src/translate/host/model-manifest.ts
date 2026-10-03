// The translation models the engine may download (add-lingua-translation-delivery D3), as the
// package states them: model-manifest.json, bundled by build.mjs. Since
// generalise-lingua-translation-catalogue it is a catalogue: the models, each file with its address,
// its size as served, its size once decompressed and the sha256 of its DECOMPRESSED bytes; and for
// each studied language the route of models that translates it into French. Because the package
// carries it, the reviewed package decides what is accepted; the host only serves bytes, and cannot
// substitute a model.

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
  /** For each studied language, the ids of the models that translate it into French, in order. */
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

/** The language every route ends in: the reader's. */
export const TARGET_LANGUAGE = "fr";

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
 * The catalogue, or an error naming what is wrong with it: a broken one must fetch nothing
 * (generalise-lingua-translation-catalogue D2). Each route names known models, starts from its
 * language, chains each model's target to the next one's source, and ends in French.
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
  for (const [language, route] of Object.entries(c.routes as Record<string, unknown>)) {
    if (!Array.isArray(route) || route.length === 0) {
      throw new Error(`model-manifest.json: the ${language} route names no model`);
    }
    let at = language;
    for (const id of route) {
      const model = models[id as string];
      if (!model)
        throw new Error(`model-manifest.json: the ${language} route names ${String(id)}, not in the catalogue`);
      if (model.from !== at) {
        throw new Error(
          `model-manifest.json: the ${language} route reaches ${id}, which translates from ${model.from}, with ${at}`,
        );
      }
      at = model.to;
    }
    if (at !== TARGET_LANGUAGE) throw new Error(`model-manifest.json: the ${language} route ends in ${at}, not French`);
    routes[language] = route as string[];
  }
  return { base: c.base, models, routes };
}

/** The models that translate `language` into French, in order; none when the catalogue has no route. */
export function routeOf(catalogue: ModelCatalogue, language: string): ModelManifest[] {
  return (catalogue.routes[language] ?? []).map((id) => {
    const { from, to, files } = catalogue.models[id]!;
    return { version: id, base: catalogue.base, from, to, files };
  });
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
 * The model the setting downloads and the engine loads: the route of `language`, which holds a
 * single model until generalise-lingua-translation-model-state (catalogue D3).
 */
export async function loadTranslationModel(language: string, fetchFn: typeof fetch = fetch): Promise<ModelManifest> {
  const route = routeOf(await loadBundledCatalogue(fetchFn), language);
  if (route.length !== 1) throw new Error(`${MANIFEST_PATH}: the ${language} route must be a single model`);
  return route[0]!;
}
