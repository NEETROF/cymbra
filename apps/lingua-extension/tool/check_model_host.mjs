// What a reader's extension will meet when it downloads the model (add-lingua-translation-delivery
// D3), checked from outside: each file model-manifest.json names, at its address, served under
// CORS and as immutable, decompressing to the pinned bytes; and an unknown path that is not a 200.
//
// Run after deploying the host (lingua-model-deploy), and before submitting a package to a store
// (lingua-extension-release): a package whose model cannot be downloaded offers a setting that
// fails for every reader who ticks it.
//
// Usage: node tool/check_model_host.mjs [<base-url>]   (default: the manifest's `base`)
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { gunzipSync } from "node:zlib";

const appRoot = join(dirname(fileURLToPath(import.meta.url)), "..");
const manifest = JSON.parse(readFileSync(join(appRoot, "model-manifest.json"), "utf8"));
const base = process.argv[2] ?? manifest.base;
const sha256 = (bytes) => createHash("sha256").update(bytes).digest("hex");
// Any extension origin: the host must answer every one of them, so the header is `*`.
const ORIGIN = "chrome-extension://abcdefghijklmnopabcdefghijklmnop";

const problems = [];
for (const [role, file] of Object.entries(manifest.files)) {
  const url = new URL(file.path, base).href;
  let response;
  try {
    response = await fetch(url, { headers: { Origin: ORIGIN } });
  } catch (e) {
    problems.push(`${role}: ${url} cannot be reached (${e.cause?.code ?? e.message})`);
    continue;
  }
  if (!response.ok) {
    problems.push(`${role}: ${url} answered ${response.status}`);
    continue;
  }
  if (response.headers.get("access-control-allow-origin") !== "*") {
    problems.push(`${role}: served without Access-Control-Allow-Origin: * — the extension's fetch would be refused`);
  }
  if (!/immutable/.test(response.headers.get("cache-control") ?? "")) {
    problems.push(`${role}: served without Cache-Control: immutable`);
  }
  const body = new Uint8Array(await response.arrayBuffer());
  let raw;
  try {
    raw = body[0] === 0x1f && body[1] === 0x8b ? gunzipSync(body) : body;
  } catch {
    problems.push(`${role}: not a readable gzip file`);
    continue;
  }
  if (sha256(raw) !== file.sha256) {
    problems.push(`${role}: does not decompress to the pinned model (sha256 ${sha256(raw)})`);
  } else {
    console.log(`${role}: ok`);
  }
}

const unknown = new URL("this-path-does-not-exist", base).href;
try {
  const status = (await fetch(unknown)).status;
  if (status === 200)
    problems.push(`an unknown path answers 200 — the extension would refuse it, but the host must not`);
} catch {
  // Unreachable was already reported for the files themselves.
}

if (problems.length > 0) {
  console.error(`${base} does not serve the model as the extension expects:\n- ${problems.join("\n- ")}`);
  process.exit(1);
}
console.log(`${base} serves ${manifest.version} as the extension expects.`);
