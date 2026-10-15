/** Declared for the tests: `tool/` is plain ESM, outside the TypeScript project. */
export declare const STATIC_READER: { matches: string[]; js: string[]; run_at: string };
export declare function filesUnder(dir: string): string[];
export declare function buildDifferences(captureDir: string, plainDir: string): string[];
export declare function manifestDifferences(capture: Record<string, unknown>, plain: Record<string, unknown>): string[];
export declare function buildForCapture(appDir: string): string;
export declare function extensionId(key: string): string;
export declare function captureExtensionId(manifest: { key?: string }): string;
