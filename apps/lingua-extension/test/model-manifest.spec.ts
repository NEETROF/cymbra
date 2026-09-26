import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { describe, expect, it, vi } from "vitest";
import {
  fileUrl,
  loadBundledManifest,
  MANIFEST_PATH,
  type ModelManifest,
  parseManifest,
  totalSize,
} from "@/translate/host/model-manifest.ts";

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
const committed = JSON.parse(readFileSync(join(root, "model-manifest.json"), "utf8"));

// The sha256 of each file's DECOMPRESSED bytes, as TRANSLATION.md pinned them when the engine was
// measured (add-lingua-translation-engine). The download is checked against these, nothing else.
const PINNED = {
  model: "6322e296d4fecfe395a8d5723da4ec37ecbe6d7613bb1dfcf4b28e2a47498b68",
  lex: "2585ed98d3af0bc949865aedeb390493d591f56870814376e73e4144c41ed059",
  vocab: "783abf3abe075afdf8d85d233994bef2c3a064e935ab1bed946820aff6ac002a",
};

describe("the committed model manifest", () => {
  it("pins the en→fr base-memory model the engine was measured on", () => {
    const m = parseManifest(committed);
    expect(m.version).toBe("en-fr/base-memory/2.0");
    expect(Object.fromEntries(Object.entries(m.files).map(([role, f]) => [role, f.sha256]))).toEqual(PINNED);
  });

  it("downloads 25 752 472 bytes — Mozilla's own gzip files", () => {
    expect(totalSize(parseManifest(committed))).toBe(25_752_472);
  });

  it("serves each file under a content-addressed path, from Cymbra over https", () => {
    const m = parseManifest(committed);
    expect(m.base).toMatch(/^https:\/\/[a-z.]+cymbra\.app\/$/);
    for (const file of Object.values(m.files)) {
      expect(file.path).toContain(`/${file.sha256}/`);
      expect(file.path).toMatch(/\.gz$/);
      expect(fileUrl(m, file)).toBe(`${m.base}${file.path}`);
    }
  });
});

describe("parseManifest", () => {
  const valid = (): ModelManifest => parseManifest(committed);

  it("refuses a manifest without a version, a base or files", () => {
    expect(() => parseManifest(null)).toThrow(/required/);
    expect(() => parseManifest({ ...valid(), version: 2 })).toThrow(/required/);
    expect(() => parseManifest({ ...valid(), files: undefined })).toThrow(/required/);
  });

  it("refuses a base that is not an http(s) address", () => {
    expect(() => parseManifest({ ...valid(), base: "file:///tmp/" })).toThrow(/base/);
  });

  it("refuses a file without its pin", () => {
    const m = valid();
    expect(() => parseManifest({ ...m, files: { ...m.files, lex: { ...m.files.lex, sha256: "abc" } } })).toThrow(/lex/);
    const noVocab: Partial<ModelManifest["files"]> = { ...m.files };
    delete noVocab.vocab;
    expect(() => parseManifest({ ...m, files: noVocab })).toThrow(/vocab/);
  });

  it("keeps only what the runtime needs", () => {
    const m = parseManifest({ ...committed, extra: true });
    expect(Object.keys(m).sort()).toEqual(["base", "files", "version"]);
    expect(Object.keys(m.files.model).sort()).toEqual(["path", "sha256", "size"]);
  });
});

describe("loadBundledManifest", () => {
  it("reads the package's own copy — a file of the extension", async () => {
    const fetchFn = vi.fn(async () => new Response(JSON.stringify(committed)));
    const m = await loadBundledManifest(fetchFn as unknown as typeof fetch);
    expect(fetchFn).toHaveBeenCalledWith(MANIFEST_PATH);
    expect(m.version).toBe("en-fr/base-memory/2.0");
  });

  it("fails on a missing file rather than guess", async () => {
    const fetchFn = vi.fn(async () => new Response("", { status: 404 }));
    await expect(loadBundledManifest(fetchFn as unknown as typeof fetch)).rejects.toThrow(/404/);
  });
});
