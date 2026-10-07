/** Declared for the tests: `tool/` is plain ESM, outside the TypeScript project. */
export declare function shippedPairs(appDir?: string): string[];
export declare function packFile(pair: string): string;
export declare function studiedOf(pair: string): string;
export declare function nativeOf(pair: string): string;
export declare const ANALYZER_CONSTANTS: Readonly<Record<string, string>>;
export declare function coreAnalyzerVersion(language: string, modRs: string): string | null;
export declare function packMeta(bytes: Uint8Array): {
  studied: string | null;
  native: string | null;
  analyzerVersion: string | null;
};
export declare function assertPacksMatchEngine(options?: {
  appDir?: string;
  pairs?: readonly string[];
  modRs?: string;
}): void;
