/** Declared for the tests: `tool/` is plain ESM, outside the TypeScript project. */
export interface CatalogueFile {
  path: string;
  size: number;
  unpacked: number;
  sha256: string;
  source?: { path: string; sha256: string };
}
export interface CatalogueModel {
  from: string;
  to: string;
  licence: string;
  mirror?: string;
  files: Record<string, CatalogueFile>;
}
export interface Catalogue {
  base: string;
  sourceBase?: string;
  models: Record<string, CatalogueModel>;
  routes: Record<string, string[]>;
}
export declare const CATALOGUE_PATH: string;
export declare const ROLES: readonly string[];
export declare function readCatalogue(path?: string): Catalogue;
export declare function modelsOf(catalogue: Catalogue): (CatalogueModel & { id: string })[];
export declare function bundledCatalogue(catalogue: Catalogue, base?: string): Catalogue;
export declare function mirrorTag(model: { mirror: string }): string;
