// Vitest suites run in Node, but the app's tsconfig deliberately carries no node
// types (browser code must not reach for them). The percussion parity suite reads
// the real wasm binary and the real MusicXML fixtures from disk, so declare the one
// built-in it needs — no more (adding @types/node would type-bless node imports
// across the whole browser codebase).
declare module "node:fs" {
  export function readFileSync(path: string): Uint8Array;
  export function readFileSync(path: string, encoding: "utf-8"): string;
}
