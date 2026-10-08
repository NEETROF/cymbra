/** Declared for the tests: `tool/` is plain ESM, outside the TypeScript project. */
/** One `_locales/<language>/messages.json`, in the browsers' format. */
export type Messages = Record<string, { message: string; description?: string }>;
/** Every committed language's messages, by language. */
export type Locales = Record<string, Messages>;
export declare const LOCALES_DIR: string;
export declare function messagesFile(language: string): string;
export declare function readLocales(dir: string): Locales;
