// Assemble the static site that serves the translation models (add-lingua-translation-delivery D3)
// into <out>: every file of every model model-manifest.json lists
// (generalise-lingua-translation-catalogue D5), at its content-addressed path, exactly as Mozilla
// publishes it — and nothing else.
//
// Each file is kept only if ALL its pins hold: the sha256 of the gzip file as published
// (`source.sha256`), the sha256 of its decompressed bytes (`sha256`) — the one the extension
// checks — and its decompressed size (`unpacked`), which the setting states. So the host can only
// ever be filled with the models the package pins, whichever of these places the bytes came from,
// tried in order:
//   1. Mozilla's model registry (`sourceBase` + `source.path`) — the origin. Mozilla has moved
//      these files once already (the Git LFS copies answer 410), and the path names a training run.
//   2. Our own copy (the model's `mirror`), a GitHub Release that lingua-model-deploy fills once.
//   3. The deployed host itself (`base`): a redeploy never needs anything but what is served.
//
// The site also carries:
// - `_headers`: `Access-Control-Allow-Origin: *` (the extension fetches under CORS; a host
//   permission would disable the installed extension until the reader accepts it) and
//   `Cache-Control: immutable` (a path never changes meaning).
// - `404.html`: without it a static Pages project answers every unknown path with 200 and its
//   index — the trap the cymbra.app site already falls into.
// - A NOTICE beside each model's files: the MPL-2.0 licence and where the source is.
//
// Usage: node tool/assemble_model_site.mjs <out-dir>   (used by .github/workflows/lingua-model-deploy.yml)
import { createHash } from "node:crypto";
import { mkdirSync, rmSync, writeFileSync } from "node:fs";
import { basename, dirname, join } from "node:path";
import { gunzipSync } from "node:zlib";
import { modelsOf, readCatalogue, ROLES } from "./model-catalogue.mjs";

const out = process.argv[2];
if (!out) {
  console.error("usage: node tool/assemble_model_site.mjs <out-dir>");
  process.exit(2);
}

const catalogue = readCatalogue();
const sha256 = (bytes) => createHash("sha256").update(bytes).digest("hex");

/** Where a model's file may be taken from, in order of preference. */
function sourcesOf(model, file) {
  return [
    { from: "Mozilla's registry", url: new URL(file.source.path, catalogue.sourceBase).href },
    { from: "our mirror", url: new URL(basename(file.path), model.mirror).href },
    { from: "the deployed host", url: new URL(file.path, catalogue.base).href },
  ];
}

/** The file's gzip bytes if `url` serves the pinned ones, or why not. */
async function take(url, file) {
  let response;
  try {
    response = await fetch(url);
  } catch (e) {
    return { why: String(e.cause?.code ?? e.message) };
  }
  if (!response.ok) return { why: `HTTP ${response.status}` };
  const gz = new Uint8Array(await response.arrayBuffer());
  if (sha256(gz) !== file.source.sha256) return { why: `not the published file (sha256 ${sha256(gz)})` };
  let raw;
  try {
    raw = gunzipSync(gz);
  } catch {
    return { why: "not gzip" };
  }
  if (sha256(raw) !== file.sha256) return { why: `decompressed, not the pinned model (sha256 ${sha256(raw)})` };
  if (raw.byteLength !== file.unpacked) {
    return { why: `decompressed to ${raw.byteLength} bytes, the catalogue says ${file.unpacked}` };
  }
  return { gz };
}

rmSync(out, { recursive: true, force: true });
mkdirSync(out, { recursive: true });

for (const model of modelsOf(catalogue)) {
  for (const role of ROLES) {
    const file = model.files[role];
    const tried = [];
    let gz = null;
    for (const { from, url } of sourcesOf(model, file)) {
      const got = await take(url, file);
      if (got.gz) {
        gz = got.gz;
        console.log(`${model.id} ${role}: from ${from} — ${gz.byteLength} bytes, every pin matches`);
        break;
      }
      tried.push(`${from} (${url}): ${got.why}`);
    }
    if (!gz) throw new Error(`${model.id} ${role}: no source serves the pinned file —\n  ${tried.join("\n  ")}`);
    if (gz.byteLength !== file.size) {
      throw new Error(`${model.id} ${role}: ${gz.byteLength} bytes, the catalogue says ${file.size}`);
    }
    const target = join(out, file.path);
    mkdirSync(dirname(target), { recursive: true });
    writeFileSync(target, gz);
  }
  // MPL-2.0 §3: whoever receives the files is told the licence and where their source is.
  mkdirSync(join(out, model.id), { recursive: true });
  writeFileSync(
    join(out, model.id, "NOTICE.txt"),
    [
      `The files under ${model.id}/ are Mozilla's Firefox Translations ${model.from}→${model.to}`,
      "translation model, redistributed unmodified by Cymbra for the Cymbra Lingua extension.",
      "",
      `Licence: Mozilla Public License 2.0 (${model.licence}) — https://mozilla.org/MPL/2.0/`,
      "Source: https://github.com/mozilla/firefox-translations-models and Mozilla's model registry,",
      `${catalogue.sourceBase}`,
      "",
    ].join("\n"),
  );
}

writeFileSync(
  join(out, "_headers"),
  [
    "/*",
    "  Access-Control-Allow-Origin: *",
    "  Cache-Control: public, max-age=31536000, immutable",
    "  X-Content-Type-Options: nosniff",
    "",
  ].join("\n"),
);
writeFileSync(join(out, "404.html"), "<!doctype html><title>404</title><p>Not found.</p>\n");
const listed = modelsOf(catalogue).map((m) => `${m.id} (${m.licence})`);
console.log(`Model site assembled in ${out}: ${listed.join(", ")}.`);
