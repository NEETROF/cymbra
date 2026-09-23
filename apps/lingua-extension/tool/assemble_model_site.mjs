// Assemble the static site that serves the translation model (add-lingua-translation-delivery D3)
// into <out>: every file model-manifest.json names, at its content-addressed path, exactly as
// Mozilla publishes it — and nothing else.
//
// Each file is taken from Mozilla's model registry (`sourceBase` + `source.path`) and kept only if
// BOTH digests hold: the sha256 of the gzip file as published (`source.sha256`), and the sha256 of
// its decompressed bytes (`sha256`) — the one the extension checks. So the host can only ever be
// filled with the model the package pins.
//
// The site also carries:
// - `_headers`: `Access-Control-Allow-Origin: *` (the extension fetches under CORS; a host
//   permission would disable the installed extension until the reader accepts it) and
//   `Cache-Control: immutable` (a path never changes meaning).
// - `404.html`: without it a static Pages project answers every unknown path with 200 and its
//   index — the trap the cymbra.app site already falls into.
//
// Usage: node tool/assemble_model_site.mjs <out-dir>   (used by .github/workflows/lingua-model-deploy.yml)
import { createHash } from "node:crypto";
import { mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
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

rmSync(out, { recursive: true, force: true });
mkdirSync(out, { recursive: true });

for (const [role, file] of Object.entries(manifest.files)) {
  const url = new URL(file.source.path, manifest.sourceBase).href;
  const response = await fetch(url);
  if (!response.ok) throw new Error(`${role}: ${url} answered ${response.status}`);
  const gz = new Uint8Array(await response.arrayBuffer());
  if (sha256(gz) !== file.source.sha256) {
    throw new Error(`${role}: the file Mozilla publishes is not the pinned one (sha256 ${sha256(gz)})`);
  }
  if (gz.byteLength !== file.size) throw new Error(`${role}: ${gz.byteLength} bytes, the manifest says ${file.size}`);
  const raw = gunzipSync(gz);
  if (sha256(raw) !== file.sha256) {
    throw new Error(`${role}: decompressed, it is not the pinned model (sha256 ${sha256(raw)})`);
  }
  const target = join(out, file.path);
  mkdirSync(dirname(target), { recursive: true });
  writeFileSync(target, gz);
  console.log(`${role}: ${file.path} — ${gz.byteLength} bytes, both digests match`);
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
