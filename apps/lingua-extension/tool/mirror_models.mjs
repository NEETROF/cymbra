// Our own copy of each translation model, for good (generalise-lingua-translation-catalogue D5): a
// GitHub Release per model of model-manifest.json, holding Mozilla's exact files and the model's
// notice, created once and never replaced. It is the second source of every later assembly of the
// model host (assemble_model_site.mjs). The lingua-model-deploy workflow runs it after assembling the
// site; a release that exists is left as it is.
//
// Usage: node tool/mirror_models.mjs <assembled-dir> [--dry-run]
//   --dry-run says what it would create and creates nothing: run it locally, with `gh` signed in,
//   to see what a dispatch would do.
import { spawnSync } from "node:child_process";
import { appendFileSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { fileURLToPath } from "node:url";
import { mirrorTag, modelsOf, readCatalogue, ROLES } from "./model-catalogue.mjs";

/** The release notes of a model's mirror. */
export function mirrorNotes(model) {
  return [
    `Mozilla's Firefox Translations \`${model.id}\` model, unmodified (Mozilla Public License 2.0),`,
    "kept by Cymbra as a second source for `models.cymbra.app`. Every consumer checks each file",
    "against the sha256 in `apps/lingua-extension/model-manifest.json`.",
    "",
  ].join("\n");
}

/**
 * What to do for each model that names a mirror: keep a release that exists, or create it with the
 * model's three files and its notice, as `dir` holds them once assembled.
 */
export function planMirrors(catalogue, dir, exists) {
  return modelsOf(catalogue)
    .filter((model) => model.mirror)
    .map((model) => {
      const tag = mirrorTag(model);
      if (exists(tag)) return { kind: "kept", tag, model: model.id };
      const files = [...ROLES.map((role) => join(dir, model.files[role].path)), join(dir, model.id, "NOTICE.txt")];
      return { kind: "create", tag, model: model.id, files, notes: mirrorNotes(model) };
    });
}

function gh(args, options = {}) {
  return spawnSync("gh", args, { encoding: "utf8", ...options });
}

function summary(line) {
  console.log(line);
  if (process.env.GITHUB_STEP_SUMMARY) appendFileSync(process.env.GITHUB_STEP_SUMMARY, `${line}\n`);
}

function main() {
  const args = process.argv.slice(2);
  const dryRun = args.includes("--dry-run");
  const dir = args.find((a) => !a.startsWith("--"));
  if (!dir) {
    console.error("usage: node tool/mirror_models.mjs <assembled-dir> [--dry-run]");
    process.exit(2);
  }
  const exists = (tag) => gh(["release", "view", tag], { stdio: "ignore" }).status === 0;
  for (const step of planMirrors(readCatalogue(), dir, exists)) {
    if (step.kind === "kept") {
      summary(`${step.tag} already holds ${step.model}.`);
      continue;
    }
    if (dryRun) {
      summary(`${step.tag} would be created for ${step.model}, with ${step.files.length} files.`);
      continue;
    }
    const notes = join(tmpdir(), `${step.tag}-notes.md`);
    writeFileSync(notes, step.notes);
    const target = process.env.GITHUB_SHA ? ["--target", process.env.GITHUB_SHA] : [];
    const created = gh(
      [
        "release",
        "create",
        step.tag,
        ...step.files,
        ...target,
        "--title",
        `Lingua translation model ${step.model}`,
        "--notes-file",
        notes,
        "--latest=false",
      ],
      { stdio: "inherit" },
    );
    if (created.status !== 0) throw new Error(`${step.tag}: gh release create failed`);
    summary(`Published ${step.tag}.`);
  }
}

if (process.argv[1] && fileURLToPath(import.meta.url) === process.argv[1]) main();
