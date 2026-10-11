import type { NativeLanguage } from "../analyzer/types.ts";

// The interface's language (add-lingua-interface-language D3): the reader's native language,
// under a key of its own in chrome.storage.local, so that every surface — the popup before it has
// an engine, the account page that never has one — reads it with its other preferences, before
// it renders its copy. The store's owner writes it from the backup's profile: at its start when
// the key is absent, and after each write of the backup (state/store.ts), so the key follows the
// profile and cannot go stale behind it. A device that predates the key reads French (M22); a
// later override (M2's reservation) would be a different key, with no migration.

/**
 * The languages the interface speaks: the native languages the engine glosses in, one set by
 * decision — the interface language is the native language (M2), so a language the engine can
 * gloss in is one the interface speaks, and no other. The day an override parts them (M2's
 * reservation), this is where they part.
 */
export type InterfaceLanguage = NativeLanguage;

export const INTERFACE_LANGUAGES: readonly InterfaceLanguage[] = ["fr", "en", "es"];

/** What a device that never wrote the key reads: every installed reader stays in French. */
export const DEFAULT_INTERFACE_LANGUAGE: InterfaceLanguage = "fr";

/** The key, in chrome.storage.local, holding `fr`, `en` or `es`. */
export const INTERFACE_LANGUAGE_KEY = "cymbra-lingua-interface-language";

export function isInterfaceLanguage(value: unknown): value is InterfaceLanguage {
  return (INTERFACE_LANGUAGES as readonly unknown[]).includes(value);
}

/** What the key is read through: chrome.storage.local, or a surface's area over it. */
export interface InterfaceLanguageArea {
  get(keys: string): Promise<Record<string, unknown>>;
}

/**
 * How long a surface waits for the key before it shows French: a storage read that never settles
 * (a wedged extension context) must not keep a page hidden until its 1.5 s reveal, unfilled.
 */
export const INTERFACE_LANGUAGE_READ_TIMEOUT_MS = 500;

/**
 * The interface language the device holds; `fr` when the key is absent or unknown — and when the
 * read itself fails, or has not answered within `INTERFACE_LANGUAGE_READ_TIMEOUT_MS`. It never
 * rejects nor hangs: every surface reads it before anything else, and a page hidden until its copy is
 * filled (localise-lingua-reading-surfaces D2) or a reading session not yet built would otherwise
 * stay so; French is what such a device showed before the key.
 */
export async function interfaceLanguage(area: InterfaceLanguageArea): Promise<InterfaceLanguage> {
  let got: Record<string, unknown> | undefined;
  let timer: ReturnType<typeof setTimeout> | undefined;
  const timeout = new Promise<"timeout">((resolve) => {
    timer = setTimeout(() => resolve("timeout"), INTERFACE_LANGUAGE_READ_TIMEOUT_MS);
  });
  try {
    const answer = await Promise.race([area.get(INTERFACE_LANGUAGE_KEY), timeout]);
    if (answer === "timeout") {
      console.warn(
        `[Cymbra Lingua] the interface language was not read within ${INTERFACE_LANGUAGE_READ_TIMEOUT_MS} ms, showing the default (fr)`,
      );
      return DEFAULT_INTERFACE_LANGUAGE;
    }
    got = answer;
  } catch (e) {
    console.warn("[Cymbra Lingua] could not read the interface language, showing the default (fr):", e);
    return DEFAULT_INTERFACE_LANGUAGE;
  } finally {
    clearTimeout(timer);
  }
  const value = got?.[INTERFACE_LANGUAGE_KEY];
  return isInterfaceLanguage(value) ? value : DEFAULT_INTERFACE_LANGUAGE;
}

/** Say which language the page speaks, for the browser's spell-check, hyphenation and voices. */
export function setDocumentLanguage(document: Document, language: InterfaceLanguage): void {
  document.documentElement.lang = language;
}
