/** Declared for the tests (the extension's and the site's): `tool/` is plain ESM, outside the TypeScript project. */
type Interface = "fr" | "en" | "es";
type Studied = "en" | "es" | "fr";
interface PageWords {
  card: string[];
  second: string[];
  known: string[];
  seedKnown: string[];
  seedDeck: string[];
  names: string[];
}
export declare const INTERFACE_LANGUAGES: readonly Interface[];
export declare const LOCALE: Record<Interface, string>;
export declare const TIME_ZONE: string;
export declare const STUDIED: Record<Interface, readonly [Studied, Studied]>;
export declare const LEVEL: string;
export declare const READER_LEVELS: Partial<Record<Interface, string>>;
export declare const WORDS: Record<Studied, PageWords>;
export declare const OVERRIDES: Partial<Record<Interface, Partial<Record<Studied, Partial<PageWords>>>>>;
export declare function wordsFor(interfaceLanguage: Interface, studied: Studied): PageWords;
export declare const BOOK: Record<Studied, Studied>;
export declare const SEED_REVIEW: readonly ("recalled" | "missed")[];
export declare const STORE_SHOTS: readonly string[];
export declare const SITE_SHOTS: readonly string[];
export declare const GLOSS_LIMIT: number;
export declare function glossProblems(gloss: string | null | undefined): string[];
