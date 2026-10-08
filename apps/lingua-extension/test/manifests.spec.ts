import { join } from "node:path";
import { describe, expect, it } from "vitest";
import { readLocales } from "../tool/locales.mjs";
import {
  buildManifest,
  COMMAND_MESSAGES,
  defaultLocale,
  DESCRIPTION_MESSAGE,
  LITERAL_NATIVE,
  LOCALISED_MESSAGES,
  localised,
} from "../tool/manifests.mjs";
import { shippedNatives, shippedPairs } from "../tool/packs.mjs";

const app = join(__dirname, "..");
const TARGETS = ["chromium", "firefox", "safari"] as const;
/** The list before change 34: every pair French-native (M22). */
const TODAY = ["en-fr", "es-fr"];
/** That list beside the first English-glossed pair: packs.json in change 34. */
const WITH_ES_EN = [...TODAY, "es-en"];
/** That list beside the first Spanish-glossed pair alone (change 35 before 34). */
const WITH_EN_ES = [...TODAY, "en-es"];
/** Change 34's list beside the first Spanish-glossed pair: packs.json since change 35. */
const WITH_BOTH = [...WITH_ES_EN, "en-es"];

type Manifest = Record<string, unknown> & {
  description: string;
  commands: Record<string, { description: string }>;
  default_locale?: string;
  name: string;
  action: { default_title: string };
};

const base = (await import("../manifest.json")) as unknown as Manifest;
const messages = readLocales(app);

/** The manifest step as build.mjs runs it, with the build's own inputs replaced by fixed ones. */
function step(target: (typeof TARGETS)[number], pairs: readonly string[], own = messages) {
  return buildManifest({
    base,
    target,
    pairs,
    messages: own,
    version: "1.7.0",
    grpcWebUrl: "https://api.cymbra.app",
    extKey: "KEY",
    offscreen: target === "chromium",
  }) as { manifest: Manifest; locales: string[] };
}

/** A manifest's text, as the build writes it. */
const text = (manifest: unknown) => `${JSON.stringify(manifest, null, 2)}\n`;

describe("the committed messages (localise-lingua-manifest D1)", () => {
  it("exist for French, English and Spanish, each holding every message a localised manifest references", () => {
    expect(Object.keys(messages)).toEqual(["en", "es", "fr"]);
    for (const [language, own] of Object.entries(messages)) {
      for (const key of LOCALISED_MESSAGES) {
        expect(typeof own[key]?.message, `${language}: ${key}`).toBe("string");
        expect(own[key]!.message.length, `${language}: ${key}`).toBeGreaterThan(0);
        expect(typeof own[key]?.description, `${language}: ${key} needs a note for translators`).toBe("string");
      }
    }
  });

  it("name every command of the manifest, and the French is the manifest's literal", () => {
    expect(Object.keys(base.commands).sort()).toEqual(Object.keys(COMMAND_MESSAGES).sort());
    const french = messages[LITERAL_NATIVE]!;
    expect(french[DESCRIPTION_MESSAGE]!.message).toBe(base.description);
    for (const [id, command] of Object.entries(base.commands)) {
      expect(french[COMMAND_MESSAGES[id]!]!.message).toBe(command.description);
    }
  });

  it("each name what their own readers study, not a translation of the French", () => {
    // English speakers study Spanish (es-en), Spanish speakers English (en-es).
    expect(messages.en![DESCRIPTION_MESSAGE]!.message).toMatch(/Spanish/);
    expect(messages.en![DESCRIPTION_MESSAGE]!.message).not.toMatch(/English/);
    expect(messages.es![DESCRIPTION_MESSAGE]!.message).toMatch(/inglés/);
    expect(messages.es![DESCRIPTION_MESSAGE]!.message).not.toMatch(/español/);
  });
});

describe("the shipped natives decide the manifest's languages (D2)", () => {
  it("call for a localised manifest only when a native but French ships", () => {
    expect(localised(["fr"])).toBe(false);
    expect(localised(["fr", "en"])).toBe(true);
    expect(localised(["es"])).toBe(true);
    // packs.json since change 35 (enable-lingua-spanish-speakers): es-en and en-es ship beside French.
    expect(localised(shippedNatives(shippedPairs()))).toBe(true);
  });

  it("default to English when an English-glossed pair ships (M13), to French otherwise", () => {
    expect(defaultLocale(["fr", "en"])).toBe("en");
    expect(defaultLocale(["en"])).toBe("en");
    expect(defaultLocale(["fr", "es"])).toBe("fr");
    expect(defaultLocale(["fr"])).toBe("fr");
  });
});

describe("the manifest step while every shipped pair is French-native", () => {
  it.each(TARGETS)("%s: writes today's literal French manifest, no `_locales`, no `default_locale`", (target) => {
    const { manifest, locales } = step(target, TODAY);
    expect(locales).toEqual([]);
    expect(manifest.description).toBe(base.description);
    expect(manifest.commands).toEqual(base.commands);
    expect("default_locale" in manifest).toBe(false);
    expect(text(manifest)).not.toContain("__MSG_");
    expect(text(manifest)).not.toContain("_locales");
  });

  it.each(TARGETS)("%s: keeps the variant's own shape", (target) => {
    const { manifest } = step(target, TODAY);
    expect(manifest.version).toBe("1.7.0");
    expect(manifest.name).toBe("Cymbra Lingua");
    expect(manifest.host_permissions).toEqual(["https://api.cymbra.app/*"]);
    expect((manifest.web_accessible_resources as { resources: string[] }[])[0]!.resources).toEqual([
      "wasm/lingua_wasm.js",
      "wasm/lingua_wasm_bg.wasm",
      "assets/packs/en-fr.lingua",
      "assets/packs/es-fr.lingua",
    ]);
    const permissions = manifest.permissions as string[];
    if (target === "chromium") {
      expect(manifest.key).toBe("KEY");
      expect(manifest.background).toEqual({ service_worker: "background.js", type: "module" });
      expect(manifest.content_scripts).toBeUndefined();
      expect(permissions).toContain("offscreen");
      expect(permissions).toContain("sidePanel");
    } else {
      expect(manifest.key).toBeUndefined();
      expect(manifest.content_scripts).toEqual([
        { matches: ["<all_urls>"], js: ["content.js"], run_at: "document_idle" },
      ]);
      expect(permissions).not.toContain("offscreen");
      expect(permissions).not.toContain("sidePanel");
      expect(manifest.side_panel).toBeUndefined();
    }
    if (target === "firefox") {
      expect(manifest.background).toEqual({ scripts: ["background.js"] });
      expect((manifest.browser_specific_settings as { gecko: { id: string } }).gecko.id).toBe("lingua@cymbra.app");
    }
    if (target === "safari") {
      expect(manifest.background).toEqual({ scripts: ["background.js"], persistent: false });
      expect(manifest.browser_specific_settings).toBeUndefined();
      expect(permissions).toContain("nativeMessaging");
      expect(permissions).not.toContain("identity");
    }
  });

  it("forces the static reader on Chromium only when asked (LINGUA_ALL_URLS=1)", () => {
    const forced = buildManifest({
      base,
      target: "chromium",
      pairs: TODAY,
      messages,
      version: "1.7.0",
      grpcWebUrl: "http://localhost:50051",
      extKey: "",
      offscreen: true,
      allUrls: true,
    }).manifest as Manifest;
    expect(forced.content_scripts).toEqual([{ matches: ["<all_urls>"], js: ["content.js"], run_at: "document_idle" }]);
    expect(forced.key).toBeUndefined();
    expect(forced.host_permissions).toEqual(["http://localhost:50051/*"]);
  });

  it("refuses a target it does not build", () => {
    expect(() => step("edge" as never, TODAY)).toThrow('Unknown build target "edge"');
  });
});

describe("the manifest step once es-en ships (the scenario *An English-glossed pair ships*)", () => {
  it.each(TARGETS)("%s: reads the description and the commands from `_locales`, defaults to English", (target) => {
    const { manifest, locales } = step(target, WITH_ES_EN);
    expect(locales).toEqual(["fr", "en"]); // the shipped natives in listed order; Spanish's folder stays home
    expect(manifest.default_locale).toBe("en");
    expect(manifest.description).toBe(`__MSG_${DESCRIPTION_MESSAGE}__`);
    expect(manifest.commands).toEqual({
      "lingua-capture-selection": {
        suggested_key: { default: "Alt+L" },
        description: "__MSG_commandCaptureSelection__",
      },
      "lingua-toggle-drawer": { suggested_key: { default: "Alt+Shift+D" }, description: "__MSG_commandToggleDrawer__" },
      "lingua-side-panel": { suggested_key: { default: "Alt+Shift+S" }, description: "__MSG_commandSidePanel__" },
    });
    // The brand stays a literal in every language.
    expect(manifest.name).toBe("Cymbra Lingua");
    expect(manifest.action.default_title).toBe("Cymbra Lingua");
    // Every reference resolves in every shipped folder.
    const references = [...new Set([...text(manifest).matchAll(/__MSG_(\w+)__/g)].map((m) => m[1]!))];
    expect(references.sort()).toEqual([...LOCALISED_MESSAGES].sort());
    for (const language of locales) {
      for (const key of references) expect(messages[language]![key]?.message, `${language}: ${key}`).toBeTruthy();
    }
  });

  it.each(TARGETS)("%s: moves nothing else, and puts `default_locale` beside the description", (target) => {
    const literal = step(target, TODAY).manifest;
    const { manifest } = step(target, WITH_ES_EN);
    const keys = Object.keys(manifest);
    expect(keys[keys.indexOf("description") + 1]).toBe("default_locale");
    // The same manifest, read back with the literal strings and without `default_locale`.
    const restored = Object.fromEntries(
      Object.entries(manifest)
        .filter(([key]) => key !== "default_locale")
        .map(([key, value]) => {
          if (key === "description") return [key, base.description];
          if (key === "commands") return [key, base.commands];
          return [key, value];
        }),
    );
    // es-en's pack is exposed like the others: that is the only difference the pair list makes.
    const exposed = (restored.web_accessible_resources as { resources: string[] }[])[0]!;
    exposed.resources = exposed.resources.filter((r) => r !== "assets/packs/es-en.lingua");
    expect(text(restored)).toBe(text(literal));
  });

  it("defaults to French when the pair that joins is glossed in Spanish alone", () => {
    for (const target of TARGETS) {
      const { manifest, locales } = step(target, WITH_EN_ES);
      expect(locales).toEqual(["fr", "es"]);
      expect(manifest.default_locale).toBe("fr");
      expect(manifest.description).toBe(`__MSG_${DESCRIPTION_MESSAGE}__`);
    }
  });

  it("ships every native's folder once both join, English the default", () => {
    const { manifest, locales } = step("chromium", [...TODAY, "es-en", "en-es"]);
    expect(locales).toEqual(["fr", "en", "es"]);
    expect(manifest.default_locale).toBe("en");
  });

  it("runs on packs.json, which lists es-en since change 34 and en-es since change 35 (enable-lingua-spanish-speakers)", () => {
    expect(shippedPairs()).toEqual(WITH_BOTH);
    for (const target of TARGETS) {
      const { manifest, locales } = step(target, shippedPairs());
      expect(locales).toEqual(["fr", "en", "es"]); // the shipped natives in listed order
      expect(manifest.default_locale).toBe("en"); // M13: English, the first `_locales` change 34 shipped
      expect(manifest.description).toBe(`__MSG_${DESCRIPTION_MESSAGE}__`);
    }
  });

  it.each(TARGETS)("%s: change 35 adds en-es's pack and moves nothing else of change 34's manifest", (target) => {
    const before = step(target, WITH_ES_EN).manifest;
    const { manifest } = step(target, WITH_BOTH);
    const exposed = (manifest.web_accessible_resources as { resources: string[] }[])[0]!;
    expect(exposed.resources).toContain("assets/packs/en-es.lingua");
    exposed.resources = exposed.resources.filter((r) => r !== "assets/packs/en-es.lingua");
    expect(text(manifest)).toBe(text(before));
  });

  it("refuses a list whose default the package would not carry (no French-native pair, M22)", () => {
    // en-es alone: Spanish the only native, French the default, and no _locales/fr to read it from.
    for (const target of TARGETS) {
      expect(() => step(target, ["en-es"])).toThrow(
        'the shipped natives are es, and the manifest would default to "fr", whose _locales/fr no shipped pair brings',
      );
    }
    // es-en alone: English is its own default, so there is nothing to refuse.
    const { manifest, locales } = step("chromium", ["es-en"]);
    expect(locales).toEqual(["en"]);
    expect(manifest.default_locale).toBe("en");
  });

  it("refuses a shipped native with no messages, or with a message missing, naming the file", () => {
    const { fr, es } = messages;
    expect(() => step("firefox", WITH_ES_EN, { fr: fr!, es: es! })).toThrow(
      'a pair glossed in "en" ships, and _locales/en/messages.json does not exist',
    );
    const { commandSidePanel: _dropped, ...partial } = messages.en!;
    void _dropped;
    expect(() => step("firefox", WITH_ES_EN, { ...messages, en: partial })).toThrow(
      '_locales/en/messages.json has no "commandSidePanel" message',
    );
    expect(() =>
      step("firefox", WITH_ES_EN, { ...messages, en: { ...messages.en, commandSidePanel: { message: "" } } }),
    ).toThrow('_locales/en/messages.json has no "commandSidePanel" message');
  });

  it("refuses a command the messages do not name", () => {
    const stranger = {
      ...base,
      commands: { ...base.commands, "lingua-other": { suggested_key: { default: "Alt+O" }, description: "Autre" } },
    };
    expect(() =>
      buildManifest({
        base: stranger,
        target: "chromium",
        pairs: WITH_ES_EN,
        messages,
        version: "1.7.0",
        grpcWebUrl: "https://api.cymbra.app",
        extKey: "",
        offscreen: true,
      }),
    ).toThrow('manifest.json\'s command "lingua-other" has no message');
    // Literal French needs no message: the command is written as it is.
    const literal = buildManifest({
      base: stranger,
      target: "chromium",
      pairs: TODAY,
      messages,
      version: "1.7.0",
      grpcWebUrl: "https://api.cymbra.app",
      extKey: "",
      offscreen: true,
    }).manifest as Manifest;
    expect(literal.commands["lingua-other"]!.description).toBe("Autre");
  });
});

describe("the committed `_locales` reader", () => {
  it("reads nothing where the folder does not exist", () => {
    expect(readLocales(join(app, "icons"))).toEqual({});
  });
});
