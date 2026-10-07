// The translation models' catalogue as the build and the tools read it
// (generalise-lingua-translation-catalogue D5): model-manifest.json, the models the package may
// download and, per pair, the route of models that translates its studied language into its native
// one (generalise-lingua-translation-routes-by-pair D1). One module,
// so the build, the variant check, the host's assembly, its check and its mirror releases never read
// the file each in their own way.
import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const appRoot = join(dirname(fileURLToPath(import.meta.url)), "..");

/** The committed catalogue. */
export const CATALOGUE_PATH = join(appRoot, "model-manifest.json");

/** A model's three files, in the order the runtime reads them. */
export const ROLES = ["model", "lex", "vocab"];

/** The committed catalogue, as written. */
export function readCatalogue(path = CATALOGUE_PATH) {
  return JSON.parse(readFileSync(path, "utf8"));
}

/** Every model with its id, in the catalogue's order. */
export function modelsOf(catalogue) {
  return Object.entries(catalogue.models).map(([id, model]) => ({ id, ...model }));
}

/**
 * The catalogue a package carries: what the runtime needs, fetched from `base`. Mozilla's sources
 * and the mirror releases are deployment details, left out.
 */
export function bundledCatalogue(catalogue, base = catalogue.base) {
  const models = Object.fromEntries(
    modelsOf(catalogue).map(({ id, from, to, licence, files }) => [
      id,
      {
        from,
        to,
        licence,
        files: Object.fromEntries(
          ROLES.map((role) => {
            const { path, size, unpacked, sha256 } = files[role];
            return [role, { path, size, unpacked, sha256 }];
          }),
        ),
      },
    ]),
  );
  return { base, models, routes: catalogue.routes };
}

/** The tag of a model's mirror release: the last segment of its address. */
export function mirrorTag(model) {
  return new URL(model.mirror).pathname.split("/").filter(Boolean).at(-1);
}
