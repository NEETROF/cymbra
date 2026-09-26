// The model the engine may download (add-lingua-translation-delivery D3), as the package states
// it: model-manifest.json, bundled by build.mjs. Each file's address, its size as served and the
// sha256 of its DECOMPRESSED bytes. Because the package carries it, the reviewed package decides
// what is accepted; the host only serves bytes, and cannot substitute a model.

export const MODEL_ROLES = ["model", "lex", "vocab"] as const;
export type ModelRole = (typeof MODEL_ROLES)[number];

export interface ModelFile {
  /** Relative to `base`; content-addressed, so a path never changes meaning. */
  path: string;
  /** Bytes as served (gzip), for progress. */
  size: number;
  /** sha256 of the decompressed bytes: what is checked, and the key it is stored under. */
  sha256: string;
}

export interface ModelManifest {
  /** Which model this is; the stored copy records it, so a new version is a new download. */
  version: string;
  base: string;
  files: Record<ModelRole, ModelFile>;
}

/** Where the package keeps it, relative to the extension's root. */
export const MANIFEST_PATH = "model-manifest.json";

const HEX64 = /^[0-9a-f]{64}$/;

/** The manifest, or an error naming what is wrong with it: a broken one must fetch nothing. */
export function parseManifest(raw: unknown): ModelManifest {
  const m = raw as Partial<ModelManifest> | null;
  if (!m || typeof m.version !== "string" || typeof m.base !== "string" || !m.files) {
    throw new Error("model-manifest.json: version, base and files are required");
  }
  if (!/^https?:\/\//.test(m.base)) throw new Error("model-manifest.json: base must be an http(s) address");
  const files = {} as Record<ModelRole, ModelFile>;
  for (const role of MODEL_ROLES) {
    const f = m.files[role] as Partial<ModelFile> | undefined;
    if (!f || typeof f.path !== "string" || typeof f.size !== "number" || !HEX64.test(f.sha256 ?? "")) {
      throw new Error(`model-manifest.json: ${role} needs a path, a size and a sha256`);
    }
    files[role] = { path: f.path, size: f.size, sha256: f.sha256! };
  }
  return { version: m.version, base: m.base, files };
}

export function fileUrl(manifest: ModelManifest, file: ModelFile): string {
  return new URL(file.path, manifest.base).href;
}

/** What the reader downloads, all files together. */
export function totalSize(manifest: ModelManifest): number {
  return MODEL_ROLES.reduce((sum, role) => sum + manifest.files[role].size, 0);
}

/** The package's own copy — a file of the extension, never fetched from anywhere else. */
export async function loadBundledManifest(fetchFn: typeof fetch = fetch): Promise<ModelManifest> {
  const response = await fetchFn(MANIFEST_PATH);
  if (!response.ok) throw new Error(`${MANIFEST_PATH}: ${response.status}`);
  return parseManifest(await response.json());
}
