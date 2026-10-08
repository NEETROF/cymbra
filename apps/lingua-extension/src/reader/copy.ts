import { reader as enReader } from "../i18n/en/reader.ts";
import { reader as esReader } from "../i18n/es/reader.ts";
import { reader as frReader } from "../i18n/fr/reader.ts";
import { DEFAULT_INTERFACE_LANGUAGE, type InterfaceLanguage } from "../i18n/index.ts";
import type { ImportFailure } from "./library.ts";

// Every sentence the reader page shows, from the catalogue's `reader` module for the interface
// language (localise-lingua-reading-surfaces D1), in the shape the page reads it: the module's
// entries, with the four import failures gathered under `importFailed` by their reason. A failure
// is one plain sentence saying what happened — never a raw error.

/** The catalogue's `reader` module, in one language. */
export type ReaderModule = typeof frReader;

const MODULES: Record<InterfaceLanguage, ReaderModule> = { fr: frReader, en: enReader, es: esReader };

/** The module for the interface language (reader.html's static text reads it). */
export function readerModule(language: InterfaceLanguage): ReaderModule {
  return MODULES[language];
}

/** The reader page's copy: the module, its import failures keyed by reason. */
export type ReaderCopy = Omit<
  ReaderModule,
  "importProtected" | "importNotEpub" | "importUnreadable" | "importStorage"
> & {
  importFailed: Record<ImportFailure, string>;
};

/** The reader page's copy in the interface language; French when not given. */
export function readerCopy(language: InterfaceLanguage = DEFAULT_INTERFACE_LANGUAGE): ReaderCopy {
  const { importProtected, importNotEpub, importUnreadable, importStorage, ...rest } = readerModule(language);
  return {
    ...rest,
    importFailed: {
      protected: importProtected,
      notEpub: importNotEpub,
      unreadable: importUnreadable,
      storage: importStorage,
    },
  };
}

/** The French copy: what a page built without a language shows, byte for byte what it showed before. */
export const COPY: ReaderCopy = readerCopy(DEFAULT_INTERFACE_LANGUAGE);
