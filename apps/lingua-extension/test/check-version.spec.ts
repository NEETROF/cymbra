import { describe, expect, it } from "vitest";
import { versionProblems } from "../tool/check_version.mjs";

describe("the release version guard", () => {
  it("passes a version the stores accept, written in both files", () => {
    expect(versionProblems("1.4.0", "1.4.0")).toEqual([]);
  });

  it("catches the two files drifting apart", () => {
    // What release-please writing only one of them would look like — or a hand edit.
    const problems = versionProblems("1.4.0", "1.3.0");

    expect(problems).toHaveLength(1);
    expect(problems[0]).toContain("package.json");
    expect(problems[0]).toContain("manifest.json");
    expect(problems[0]).toContain("edited by hand");
  });

  it("refuses a pre-release, which a store rejects at upload", () => {
    // The failure this exists to move earlier: the tag is already pushed and the build
    // already done by the time a store says no.
    const problems = versionProblems("1.4.0-rc.1", "1.4.0-rc.1");

    expect(problems).toHaveLength(1); // once, not once per file
    expect(problems[0]).toContain("three plain integers");
  });

  it("refuses a part above what a manifest version may hold", () => {
    expect(versionProblems("1.70000.0", "1.70000.0")[0]).toContain("65535");
  });

  it("refuses a leading zero", () => {
    expect(versionProblems("1.04.0", "1.04.0")[0]).toContain("leading zero");
  });

  it("reports both a drift and a bad shape at once", () => {
    const problems = versionProblems("1.4.0", "1.4.0-rc.1");

    expect(problems).toHaveLength(2);
  });

  it("holds the versions the repository actually ships", async () => {
    // The guard is only worth having if it runs against the real files; this is the same
    // pair `yarn check:version` reads in CI.
    const pkg = (await import("../package.json")) as unknown as { version: string };
    const manifest = (await import("../manifest.json")) as unknown as { version: string };

    expect(versionProblems(pkg.version, manifest.version)).toEqual([]);
  });
});
