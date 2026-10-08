import { join } from "node:path";
import { describe, expect, it } from "vitest";
import { localeProblems, manifestProblems, versionProblem } from "../tool/check_version.mjs";
import { readLocales } from "../tool/locales.mjs";

const app = join(__dirname, "..");
const manifest = (await import("../manifest.json")) as unknown as Record<string, unknown>;
const locales = readLocales(app);

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
    const problems = manifestProblems({ description: "ok", version: "0.1.0" });

    expect(problems.join(" ")).toContain("second source");
  });

  it("refuses a description Apple would reject, which only the upload used to reveal", () => {
    // 113 characters. Apple's limit is 112 and it is checked when the signed archive is
    // uploaded — lingua-apple-v1.1.0 died there, after a full build. Chrome allows 132, so
    // calibrating on Chrome is exactly how you get an archive Apple refuses.
    const problems = manifestProblems({ description: "x".repeat(113) });

    expect(problems).toHaveLength(1);
    expect(problems[0]).toContain("113 characters");
    expect(problems[0]).toContain("112");
  });

  it("accepts a description at the limit", () => {
    expect(manifestProblems({ description: "x".repeat(112) })).toEqual([]);
  });

  it("refuses a manifest with no description at all", () => {
    expect(manifestProblems({}).join(" ")).toContain("no description");
  });

  it("accepts a manifest that defers to package.json", () => {
    expect(manifestProblems({ description: "Lisez l'anglais sur le web." })).toEqual([]);
  });

  it("holds the files the repository actually ships", async () => {
    // The guard is only worth having if it runs against the real files; this is the same
    // set `yarn check:version` reads in CI.
    const pkg = (await import("../package.json")) as unknown as { version: string };

    expect(versionProblem(pkg.version)).toBeNull();
    expect(manifestProblems(manifest)).toEqual([]);
    expect(Object.keys(locales)).toEqual(["en", "es", "fr"]);
    expect(localeProblems(manifest, locales)).toEqual([]);
  });
});

describe("the committed `_locales` guard (localise-lingua-manifest D3)", () => {
  it("refuses a Spanish description Apple would reject, naming the language", () => {
    // The scenario *A description too long for Apple*: 113 characters in es alone.
    const problems = localeProblems(manifest, {
      ...locales,
      es: { ...locales.es, extensionDescription: { message: "x".repeat(113) } },
    });

    expect(problems).toHaveLength(1);
    expect(problems[0]).toContain("the es description");
    expect(problems[0]).toContain("_locales/es/messages.json");
    expect(problems[0]).toContain("113 characters");
    expect(problems[0]).toContain("112");
  });

  it("accepts every language's description at the limit", () => {
    const atLimit = Object.fromEntries(
      Object.entries(locales).map(([language, messages]) => [
        language,
        { ...messages, extensionDescription: { message: "x".repeat(112) } },
      ]),
    );
    expect(localeProblems({ ...manifest, description: "x".repeat(112) }, atLimit)).toEqual([]);
  });

  it("refuses a French message that differs from the manifest's literal, naming the key", () => {
    // The French has two homes: the literal every package carries today, and the message a
    // localised package reads. A drift between them would change the text at the first localised build.
    const problems = localeProblems(manifest, {
      ...locales,
      fr: { ...locales.fr, commandSidePanel: { message: "Ouvrir le panneau latéral" } },
    });

    expect(problems).toHaveLength(1);
    expect(problems[0]).toContain('"commandSidePanel"');
    expect(problems[0]).toContain('"Ouvrir le panneau latéral"');
    expect(problems[0]).toContain("held equal");
    expect(localeProblems({ ...manifest, description: "Autre" }, locales).join(" ")).toContain(
      '"extensionDescription"',
    );
  });

  it("refuses a language with no description, and a missing French folder", () => {
    const { extensionDescription: _dropped, ...noDescription } = locales.en!;
    void _dropped;
    expect(localeProblems(manifest, { ...locales, en: noDescription }).join(" ")).toContain(
      '_locales/en/messages.json has no "extensionDescription" message',
    );
    const { fr: _french, ...withoutFrench } = locales;
    void _french;
    expect(localeProblems(manifest, withoutFrench).join(" ")).toContain("_locales/fr/messages.json is missing");
  });
});
