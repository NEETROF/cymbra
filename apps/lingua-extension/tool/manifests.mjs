// The manifest step of the build (localise-lingua-manifest D2), pure: the base manifest, a target,
// the shipped pairs and the committed messages in; the variant's manifest and the `_locales`
// folders to copy beside it out. build.mjs reads the files and writes the result;
// test/manifests.spec.ts runs this step with a pair list that holds es-en.
//
// What the browser shows ABOUT the extension — its description in the extensions page, which both
// stores take as the listing's summary, and its commands' descriptions in the shortcuts page — comes
// from the manifest and follows the BROWSER's language through `_locales`; it cannot follow the
// reader's choice, which the interface follows (src/i18n). A language is shipped only when a pair
// glossed in it ships: an English description shown before an English-glossed pair ships would offer
// a product that reader cannot use yet. While every shipped pair is French-native, the built
// manifests are byte for byte what they were — literal French, no `_locales`, no `default_locale`.
// Pure helpers only: shippedNatives is always given `pairs` here, so this step never reads packs.json.
import { packFile, shippedNatives } from "./packs.mjs";

/** The message the description is read from once the manifest is localised (D1). */
export const DESCRIPTION_MESSAGE = "extensionDescription";

/** The message each command's description is read from, by the command's id (D1). */
export const COMMAND_MESSAGES = Object.freeze({
  "lingua-capture-selection": "commandCaptureSelection",
  "lingua-toggle-drawer": "commandToggleDrawer",
  "lingua-side-panel": "commandSidePanel",
});

/** Every message a localised manifest references: each shipped language's messages.json holds them all. */
export const LOCALISED_MESSAGES = Object.freeze([DESCRIPTION_MESSAGE, ...Object.values(COMMAND_MESSAGES)]);

/** The language the source manifest is written in: the native of every pair shipped before the matrix (M22). */
export const LITERAL_NATIVE = "fr";

/** Whether the shipped natives call for a localised manifest: any native but French (D2). */
export function localised(natives) {
  return natives.some((native) => native !== LITERAL_NATIVE);
}

/**
 * The locale a localised manifest falls back to for any other browser language: English when an
 * English-glossed pair ships (M13), French otherwise.
 */
export function defaultLocale(natives) {
  return natives.includes("en") ? "en" : LITERAL_NATIVE;
}

/** `https://host/*` match pattern for the backend origin (host_permissions). */
export function hostPattern(url) {
  const u = new URL(url);
  return `${u.protocol}//${u.host}/*`;
}

/** Transform the (Chromium) base manifest into the Firefox variant. */
export function firefoxManifest(base) {
  const m = structuredClone(base);
  delete m.minimum_chrome_version;
  // Firefox MV3 background is an event page (scripts), not a service worker.
  m.background = { scripts: ["background.js"] };
  // No lateral panel on Firefox: the reader stays in the page via the injected drawer, so
  // there is no side_panel AND no sidebar_action. A sidebar_action would make Firefox
  // auto-open its native sidebar on install/reload (unwanted) and duplicate the drawer.
  delete m.side_panel;
  // Firefox requires an add-on id; it has no "sidePanel" permission.
  //
  // data_collection_permissions mirrors the privacy policy's Annex B (cymbra.app/confidentialite):
  // everything here is OPTIONAL because signing in is optional (offline/anonymous use sends
  // nothing at all — see REVIEWERS.md, "Where the add-on reaches the network"). Once signed in
  // and syncing: authenticationInfo (the Cymbra account/session), personallyIdentifyingInfo (its
  // email), websiteContent (the deck's stored source sentence — no URL, but still page text), and
  // technicalAndInteraction (daily stats, the random install id; Mozilla requires this one to be
  // optional regardless). No location/health/financial/communications/search/bookmarks data, and
  // no browsingActivity — the page address never leaves the device (Annex B). Firefox only shows
  // this consent UI from version 140; strict_min_version stays 128.0 for now (AMO currently only
  // warns, not blocks, on a missing key), so this key alone isn't yet a full compliance story for
  // Firefox 128–139 — see https://mzl.la/firefox-builtin-data-consent, "older Firefox versions".
  m.browser_specific_settings = {
    gecko: {
      id: "lingua@cymbra.app",
      strict_min_version: "128.0",
      data_collection_permissions: {
        // AMO refuses the key without `required`; "none" says nothing is collected unasked.
        required: ["none"],
        optional: ["authenticationInfo", "personallyIdentifyingInfo", "websiteContent", "technicalAndInteraction"],
      },
    },
    // AMO marks a version compatible with Firefox for Android only when the manifest says so:
    // without this key every upload was desktop-only and AMO greyed out "Add to Firefox" on
    // Android, although the same package runs there (yarn dogfood:firefox-android).
    gecko_android: {
      strict_min_version: "128.0",
    },
  };
  m.permissions = (m.permissions ?? []).filter((p) => p !== "sidePanel");
  return m;
}

/** Transform the base manifest into the Safari variant (macOS + iOS), hosted by apps/lingua-apple. */
export function safariManifest(base) {
  // Safari takes the Firefox path (event-page engine, static reader, in-page drawer) — the
  // add-lingua-apple spike ran it unchanged on macOS, the iOS simulator and an iPhone.
  const m = firefoxManifest(base);
  // Safari needs no add-on id (the host app's bundle identifies it).
  delete m.browser_specific_settings;
  // iOS refuses a persistent background page; the event page is suspended and woken on demand.
  m.background = { ...m.background, persistent: false };
  // Safari does not support the identity API; Apple and Google come from the host app instead,
  // collected through the extension's native handler (add-lingua-connected-clients D6).
  m.permissions = [...m.permissions.filter((p) => p !== "identity"), "nativeMessaging"];
  return m;
}

const manifestFor = { chromium: structuredClone, firefox: firefoxManifest, safari: safariManifest };

/**
 * The manifest with its description and its commands' descriptions read from `_locales` (D2): each
 * becomes a `__MSG_<message>__` reference, `default_locale` follows the description, and nothing else
 * moves — the brand name and the action's title stay the literal « Cymbra Lingua ». Every shipped
 * native must hold every referenced message, or the browser would show the reference itself.
 */
function localiseManifest(manifest, natives, messages) {
  for (const native of natives) {
    const own = messages[native];
    if (!own) {
      throw new Error(
        `a pair glossed in "${native}" ships, and _locales/${native}/messages.json does not exist: every shipped native needs its messages (localise-lingua-manifest D1).`,
      );
    }
    for (const key of LOCALISED_MESSAGES) {
      if (typeof own[key]?.message !== "string" || own[key].message.length === 0) {
        throw new Error(
          `_locales/${native}/messages.json has no "${key}" message, which the localised manifest references.`,
        );
      }
    }
  }
  const out = {};
  for (const [key, value] of Object.entries(manifest)) {
    if (key === "description") {
      out.description = `__MSG_${DESCRIPTION_MESSAGE}__`;
      out.default_locale = defaultLocale(natives);
    } else if (key === "commands") {
      out.commands = Object.fromEntries(
        Object.entries(value).map(([id, command]) => {
          const message = COMMAND_MESSAGES[id];
          if (!message)
            throw new Error(`manifest.json's command "${id}" has no message in tool/manifests.mjs's COMMAND_MESSAGES.`);
          return [id, { ...command, description: `__MSG_${message}__` }];
        }),
      );
    } else {
      out[key] = value;
    }
  }
  return out;
}

/**
 * The manifest of `target`, and the languages whose `_locales/<language>/` folder the package carries
 * beside it — the shipped natives in listed order once any of them is not French, none otherwise.
 * A language with committed messages but no shipped pair is never among them (D2).
 *
 * @param {object} options
 * @param {object} options.base manifest.json, parsed
 * @param {"chromium" | "firefox" | "safari"} options.target
 * @param {readonly string[]} options.pairs the shipped pairs (packs.json), the default language's first
 * @param {Record<string, Record<string, { message: string }>>} options.messages every committed `_locales/<language>/messages.json`, by language
 * @param {string} options.version package.json's, stamped onto every variant
 * @param {string} options.grpcWebUrl the backend origin, granted in host_permissions
 * @param {string} options.extKey the unpacked Chromium id pin; "" lets the Web Store assign the id
 * @param {boolean} options.offscreen whether the variant hosts the translation engine in an offscreen document
 * @param {boolean} [options.allUrls] force the static content script on Chromium (LINGUA_ALL_URLS=1)
 * @returns {{ manifest: object, locales: string[] }}
 */
export function buildManifest({
  base,
  target,
  pairs,
  messages,
  version,
  grpcWebUrl,
  extKey,
  offscreen,
  allUrls = false,
}) {
  const build = manifestFor[target];
  if (!build)
    throw new Error(`Unknown build target "${target}" — expected one of: ${Object.keys(manifestFor).join(", ")}.`);
  let manifest = build(base);
  manifest.version = version;
  // Each listed pack is fetched by the engine from whatever context hosts it: expose them all.
  manifest.web_accessible_resources = manifest.web_accessible_resources.map((entry, i) =>
    i === 0 ? { ...entry, resources: [...entry.resources, ...pairs.map(packFile)] } : entry,
  );
  // Chromium's service worker cannot construct a Worker, so an offscreen document owns the
  // engine's — and that needs the permission. The variant that carries the engine asks for it.
  if (offscreen) manifest.permissions = [...manifest.permissions, "offscreen"];
  // Grant the configured backend origin so the sync transport's gRPC-web fetch is
  // allowed (the server must also allow the extension origin via CYMBRA_ALLOWED_WEB_ORIGINS).
  manifest.host_permissions = [...new Set([...(manifest.host_permissions ?? []), hostPattern(grpcWebUrl)])];
  // Pin the unpacked Chromium id (stable chrome.identity redirect URL); Firefox uses its
  // gecko id and Safari its host app's bundle, so neither must carry `key`.
  if (target === "chromium" && extKey) manifest.key = extKey;
  // Reader injection strategy differs by browser:
  //  - Firefox (incl. Android): a STATIC content script on every page, ALWAYS. MV3 dynamic
  //    registration (scripting.registerContentScripts) does not reliably fire on GeckoView,
  //    and browser.contentScripts.register() dies with the non-persistent event page — so the
  //    browser-level static injection is the only thing that runs on every load AND reload.
  //    The global "Surlignage activé" toggle (default on) is the off switch; the reader is
  //    entirely local (no network), so always-on is an acceptable trade for reliability.
  //  - Chromium: activeTab-first by design — no static script; scripting.registerContentScripts
  //    (which works there) powers "Toujours surligner". LINGUA_ALL_URLS=1 forces the static
  //    script for dev testing of the always-on path on Chrome.
  //  - Safari (macOS + iOS) follows Firefox: a static content script, like the event-page engine.
  if (target !== "chromium" || allUrls) {
    manifest.content_scripts = [{ matches: ["<all_urls>"], js: ["content.js"], run_at: "document_idle" }];
  }
  // The manifest's languages (D2): the shipped natives', once a pair glossed in another language
  // than French ships — until then the literal French above, and no `_locales` at all.
  const natives = shippedNatives(pairs);
  if (!localised(natives)) return { manifest, locales: [] };
  // `default_locale` must name a packaged folder: the rule assumes a French-native pair always ships
  // (M22), so a list such as ["en-es"] alone — Spanish the only native, French the default — would
  // ship a default the package does not carry. Refused rather than built.
  const fallback = defaultLocale(natives);
  if (!natives.includes(fallback)) {
    throw new Error(
      `the shipped natives are ${natives.join(", ")}, and the manifest would default to "${fallback}", whose _locales/${fallback} no shipped pair brings: a French-native pair always ships (M22), and default_locale must name a packaged folder (localise-lingua-manifest D2).`,
    );
  }
  manifest = localiseManifest(manifest, natives, messages);
  return { manifest, locales: [...natives] };
}
