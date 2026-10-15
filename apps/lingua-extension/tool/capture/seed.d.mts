/** Declared for the tests: `tool/` is plain ESM, outside the TypeScript project. */
export declare function recordedDay(backup: unknown): number;
export declare function shiftBackup<T>(backup: T, days: number): T;
export declare function seedForRun<T>(backup: T, nowMs: number): T;
