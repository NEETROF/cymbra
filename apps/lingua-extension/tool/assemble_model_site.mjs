// Assemble the static site that serves the translation model (add-lingua-translation-delivery D3)
// into <out>: every file model-manifest.json names, at its content-addressed path, exactly as
// Mozilla publishes it — and nothing else.
//
// Each file is kept only if BOTH its digests hold: the sha256 of the gzip file as published
// (`source.sha256`), and the sha256 of its decompressed bytes (`sha256`) — the one the extension
// checks. So the host can only ever be filled with the model the package pins, whichever of these
// places the bytes came from, tried in order:
//   1. Mozilla's model registry (`sourceBase` + `source.path`) — the origin. Mozilla has moved
//      these files once already (the Git LFS copies answer 410), and the path names a training run.
//   2. Our own copy (`mirror`), a GitHub Release that lingua-model-deploy fills once.
//   3. The deployed host itself (`base`): a redeploy never needs anything but what is served.
//
// The site also carries:
// - `_headers`: `Access-Control-Allow-Origin: *` (the extension fetches under CORS; a host
//   permission would disable the installed extension until the reader accepts it) and
//   `Cache-Control: immutable` (a path never changes meaning).
// - `404.html`: without it a static Pages project answers every unknown path with 200 and its
//   index — the trap the cymbra.app site already falls into.
// - A NOTICE beside the files: the MPL-2.0 licence and where the source is.
//
// Usage: node tool/assemble_model_site.mjs <out-dir>   (used by .github/workflows/lingua-model-deploy.yml)
import { createHash } from "node:crypto";
import { mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { basename, dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { gunzipSync } from "node:zlib";

const appRoot = join(dirname(fileURLToPath(import.meta.url)), "..");
const out = process.argv[2];
if (!out) {
  console.error("usage: node tool/assemble_model_site.mjs <out-dir>");
  process.exit(2);
}

const manifest = JSON.parse(readFileSync(join(appRoot, "model-manifest.json"), "utf8"));
const sha256 = (bytes) => createHash("sha256").update(bytes).digest("hex");

/** Where a file may be taken from, in order of preference. */
function sourcesOf(file) {
  return [
    { from: "Mozilla's registry", url: new URL(file.source.path, manifest.sourceBase).href },
    { from: "our mirror", url: new URL(basename(file.path), manifest.mirror).href },
    { from: "the deployed host", url: new URL(file.path, manifest.base).href },
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
  return { gz };
}

rmSync(out, { recursive: true, force: true });
mkdirSync(out, { recursive: true });

for (const [role, file] of Object.entries(manifest.files)) {
  const tried = [];
  let gz = null;
  for (const { from, url } of sourcesOf(file)) {
    const got = await take(url, file);
    if (got.gz) {
      gz = got.gz;
      console.log(`${role}: from ${from} — ${gz.byteLength} bytes, both digests match`);
      break;
    }
    tried.push(`${from} (${url}): ${got.why}`);
  }
  if (!gz) throw new Error(`${role}: no source serves the pinned file —\n  ${tried.join("\n  ")}`);
  if (gz.byteLength !== file.size) throw new Error(`${role}: ${gz.byteLength} bytes, the manifest says ${file.size}`);
  const target = join(out, file.path);
  mkdirSync(dirname(target), { recursive: true });
  writeFileSync(target, gz);
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
// MPL-2.0 §3: whoever receives the files is told the licence and where their source is.
writeFileSync(
  join(out, manifest.version, "NOTICE.txt"),
  [
    `The files under ${manifest.version}/ are Mozilla's Firefox Translations ${manifest.from}→${manifest.to}`,
    "translation model, redistributed unmodified by Cymbra for the Cymbra Lingua extension.",
    "",
    `Licence: Mozilla Public License 2.0 (${manifest.licence}) — https://mozilla.org/MPL/2.0/`,
    "Source: https://github.com/mozilla/firefox-translations-models and Mozilla's model registry,",
    `${manifest.sourceBase}`,
    "",
  ].join("\n"),
);
console.log(`Model site assembled in ${out} (${manifest.version}, licence ${manifest.licence}).`);
