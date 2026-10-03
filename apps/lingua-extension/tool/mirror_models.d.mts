/** Declared for the tests: `tool/` is plain ESM, outside the TypeScript project. */
import type { Catalogue } from "./model-catalogue.mjs";

export type MirrorStep =
  | { kind: "kept"; tag: string; model: string }
  | { kind: "create"; tag: string; model: string; files: string[]; notes: string };
export declare function mirrorNotes(model: { id: string }): string;
export declare function planMirrors(catalogue: Catalogue, dir: string, exists: (tag: string) => boolean): MirrorStep[];
