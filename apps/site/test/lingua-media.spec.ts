import { mkdirSync, mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { afterAll, describe, expect, it } from "vitest";
import { demoVideo } from "../src/lib/lingua-media";

// A built Lingua page's demo video must exist (change: add-site-italian-locale, D6): the video is
// only a path under `public/`, so without this check `/it/lingua/`, built as soon as en-it ships,
// would go out with a dead `<source>` and pass every other check.

describe("the Lingua page's demo video", () => {
  const root = mkdtempSync(join(tmpdir(), "lingua-media-"));
  afterAll(() => rmSync(root, { recursive: true, force: true }));

  it("is the path the site serves when its file exists", () => {
    mkdirSync(join(root, "public/media/lingua/en"), { recursive: true });
    writeFileSync(join(root, "public/media/lingua/en/demo.mp4"), "");
    expect(demoVideo("en", root)).toBe("/media/lingua/en/demo.mp4");
  });

  it("fails the build, naming the file, when it is missing", () => {
    expect(() => demoVideo("it", root)).toThrow("public/media/lingua/it/demo.mp4 is missing");
  });

  it("finds today's videos in the site's own public/", () => {
    for (const lang of ["fr", "en", "es"] as const) expect(demoVideo(lang)).toBe(`/media/lingua/${lang}/demo.mp4`);
  });
});
