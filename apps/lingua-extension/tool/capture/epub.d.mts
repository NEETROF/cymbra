/** Declared for the tests: `tool/` is plain ESM, outside the TypeScript project. */
export interface FixtureBook {
  title: string;
  author: string;
  language: string;
  chapter: string;
  paragraphs: string[];
}
export declare function parseBook(text: string): FixtureBook;
export declare function xmlEscape(text: string): string;
export declare function epubEntries(book: FixtureBook, id: string): Record<string, string>;
export declare function assembleEpub(book: FixtureBook, id: string): Promise<Uint8Array>;
