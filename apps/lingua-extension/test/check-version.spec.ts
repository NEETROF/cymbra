import { describe, expect, it } from "vitest";
import { manifestProblem, versionProblem } from "../tool/check_version.mjs";

describe("the release version guard", () => {
  it("passes a version the stores accept", () => {
    expect(versionProblem("1.4.0")).toBeNull();
  });

  it("refuses a pre-release, which a store rejects at upload", () => {
    // The failure this exists to move earlier: the tag is already pushed and the build
    // already done by the time a store says no.
    expect(versionProblem("1.4.0-rc.1")).toContain("three plain integers");
  });

  it("refuses a part above what a manifest version may hold", () => {
    expect(versionProblem("1.70000.0")).toContain("65535");
  });

  it("refuses a leading zero", () => {
    expect(versionProblem("1.04.0")).toContain("leading zero");
  });

  it("refuses a version put back into the source manifest", () => {
    // build.mjs stamps package.json's version onto every variant. A copy here is a second
    // source that drifts — and it is what release-please used to rewrite, reformatting the
    // whole file and failing the Prettier gate.
    expect(manifestProblem({ name: "Cymbra Lingua", version: "0.1.0" })).toContain("second source");
  });

  it("accepts a manifest that defers to package.json", () => {
    expect(manifestProblem({ name: "Cymbra Lingua" })).toBeNull();
  });

  it("holds the files the repository actually ships", async () => {
    // The guard is only worth having if it runs against the real files; this is the same
    // pair `yarn check:version` reads in CI.
    const pkg = (await import("../package.json")) as unknown as { version: string };
    const manifest = (await import("../manifest.json")) as unknown as Record<string, unknown>;

    expect(versionProblem(pkg.version)).toBeNull();
    expect(manifestProblem(manifest)).toBeNull();
  });
});
