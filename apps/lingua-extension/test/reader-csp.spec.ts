import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { dirname, join } from "node:path";
import { describe, expect, it } from "vitest";

// The book reader renders a book's own XHTML in frames of the EXTENSION's origin (foliate-js
// loads each section from a blob: URL the reader page creates — add-lingua-reader D3). A script
// in a book would run with the extension's privileges, and some books carry scripts: Pro Git's
// chapters do. What keeps them inert is the extension pages' Content Security Policy, which the
// blob: frames inherit: scripts from the extension's own files, nothing inline, nothing from a
// blob: or data: URL, nothing from the network. Loosening it would arm every book imported.

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
const manifest = JSON.parse(readFileSync(join(root, "manifest.json"), "utf8"));

function directive(policy: string, name: string): string[] {
  const found = policy
    .split(";")
    .map((d) => d.trim().split(/\s+/))
    .find(([n]) => n === name);
  return found ? found.slice(1) : [];
}

describe("the extension pages' policy keeps a book's scripts inert", () => {
  const policy: string = manifest.content_security_policy?.extension_pages ?? "";

  it("runs scripts from the extension's own files only", () => {
    expect(directive(policy, "script-src")).toEqual(["'self'", "'wasm-unsafe-eval'"]);
  });

  it("never allows inline code, eval, or a script from a blob, data or remote URL", () => {
    for (const unsafe of ["'unsafe-inline'", "'unsafe-eval'", "blob:", "data:", "http:", "https:", "*"]) {
      expect(policy).not.toContain(unsafe);
    }
  });
});
