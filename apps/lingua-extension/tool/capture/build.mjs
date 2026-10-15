// The capture build (add-site-lingua-showcase D1): the Chromium variant with the development switch
// that declares the reader on every page (`LINGUA_ALL_URLS=1`, tool/manifests.mjs), written into
// `dist-capture/` — never `dist-chromium/`, which a dogfood browser may have loaded. Before the tool
// loads it, a plain Chromium build made with the same settings into a temporary folder must hold the
// same files byte for byte, but for a manifest that differs by that content script alone.
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { mkdtempSync, readdirSync, readFileSync, rmSync, statSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, relative } from "node:path";
import { isDeepStrictEqual } from "node:util";

/** The content script the switch adds, as tool/manifests.mjs writes it. */
export const STATIC_READER = { matches: ["<all_urls>"], js: ["content.js"], run_at: "document_idle" };

/** Every file under `dir`, as paths relative to it, sorted. */
export function filesUnder(dir) {
  const out = [];
  const visit = (at) => {
    for (const name of readdirSync(at)) {
      const path = join(at, name);
      if (statSync(path).isDirectory()) visit(path);
      else out.push(relative(dir, path));
    }
  };
  visit(dir);
  return out.sort();
}

/**
 * What separates the capture build from a plain one, as sentences; empty when the only difference is
 * the static reader in the manifest.
 * @param {string} captureDir
 * @param {string} plainDir
 * @returns {string[]}
 */
export function buildDifferences(captureDir, plainDir) {
  const problems = [];
  const capture = filesUnder(captureDir);
  const plain = filesUnder(plainDir);
  for (const file of capture) if (!plain.includes(file)) problems.push(`${file} is only in the capture build`);
  for (const file of plain) if (!capture.includes(file)) problems.push(`${file} is missing from the capture build`);
  for (const file of capture.filter((f) => plain.includes(f) && f !== "manifest.json")) {
    if (!readFileSync(join(captureDir, file)).equals(readFileSync(join(plainDir, file)))) {
      problems.push(`${file} differs`);
    }
  }
  if (capture.includes("manifest.json") && plain.includes("manifest.json")) {
    problems.push(
      ...manifestDifferences(readJson(join(captureDir, "manifest.json")), readJson(join(plainDir, "manifest.json"))),
    );
  }
  return problems;
}

/** What separates two manifests but the static reader the capture's declares. */
export function manifestDifferences(capture, plain) {
  const problems = [];
  const { content_scripts: scripts, ...rest } = capture;
  if (!isDeepStrictEqual(scripts, [STATIC_READER])) {
    problems.push(`the capture manifest's content_scripts are ${JSON.stringify(scripts)}, not the static reader alone`);
  }
  if ("content_scripts" in plain) problems.push("the plain Chromium manifest declares content scripts");
  if (!isDeepStrictEqual(rest, plain)) {
    const keys = [...new Set([...Object.keys(rest), ...Object.keys(plain)])].filter(
      (key) => !isDeepStrictEqual(rest[key], plain[key]),
    );
    problems.push(`the manifests also differ in ${keys.join(", ")}`);
  }
  return problems;
}

function readJson(path) {
  return JSON.parse(readFileSync(path, "utf8"));
}

/** Run build.mjs for Chromium into `distDir`, with the static reader or without. */
function buildChromium(appDir, distDir, allUrls) {
  const env = { ...process.env, LINGUA_DIST_DIR: distDir };
  if (allUrls) env.LINGUA_ALL_URLS = "1";
  else delete env.LINGUA_ALL_URLS;
  const run = spawnSync(process.execPath, ["build.mjs", "chromium"], { cwd: appDir, env, encoding: "utf8" });
  if (run.status !== 0) throw new Error(`the Chromium build into ${distDir} failed:\n${run.stdout}\n${run.stderr}`);
}

/**
 * The id Chromium gives an unpacked extension whose manifest holds `key` (base64 of the public key's
 * DER): the first 32 hex digits of its SHA-256, each written as a letter from `a` to `p`.
 */
export function extensionId(key) {
  const digest = createHash("sha256").update(Buffer.from(key, "base64")).digest("hex").slice(0, 32);
  return [...digest].map((d) => String.fromCharCode(97 + parseInt(d, 16))).join("");
}

/**
 * The capture build's extension id, from its manifest's `key`. Without one Chromium draws an id per
 * profile and the tool cannot open the extension's pages: a store build's `LINGUA_EXT_KEY=""` left
 * in the shell does that, and the tool says so.
 * @param {{ key?: string }} manifest
 */
export function captureExtensionId(manifest) {
  if (!manifest.key) {
    throw new Error(
      "the capture build's manifest has no key — LINGUA_EXT_KEY is set to an empty value in this shell, as for a " +
        "store build: unset it (`env -u LINGUA_EXT_KEY yarn capture`) so that the development key pins the id",
    );
  }
  return extensionId(manifest.key);
}

/**
 * Build `dist-capture/`, check it against a plain build made with the same settings, delete the
 * plain one; refuse to go on, naming every difference, when they differ by more than the reader.
 * @returns {string} the capture build's folder
 */
export function buildForCapture(appDir) {
  const captureDir = join(appDir, "dist-capture");
  buildChromium(appDir, captureDir, true);
  // Named dist-…, as build.mjs's LINGUA_DIST_DIR requires (tool/dist-dir.mjs).
  const plainDir = mkdtempSync(join(tmpdir(), "dist-plain-"));
  try {
    buildChromium(appDir, plainDir, false);
    const problems = buildDifferences(captureDir, plainDir);
    if (problems.length > 0) {
      throw new Error(
        `the capture build is not the Chromium package plus the static reader:\n- ${problems.join("\n- ")}`,
      );
    }
  } finally {
    rmSync(plainDir, { recursive: true, force: true });
  }
  return captureDir;
}
