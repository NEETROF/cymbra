import { readdirSync, readFileSync, statSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { dirname, join } from "node:path";
import { describe, expect, it } from "vitest";

// Identity rule (design D5): no colour literal lives outside the single token sheet.
// The guard scans every stylesheet under src/ and asserts only tokens.css carries a
// hex colour; all other CSS must reference the --cymbra-lingua-* custom properties.

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
const SRC = join(root, "src");
const TOKEN_SHEET = "styles/tokens.css";
const HEX = /#[0-9a-fA-F]{3,8}\b/;

function cssFiles(dir: string): string[] {
  const out: string[] = [];
  for (const name of readdirSync(dir)) {
    const p = join(dir, name);
    if (statSync(p).isDirectory()) {
      if (name !== "pkg") out.push(...cssFiles(p));
    } else if (p.endsWith(".css")) {
      out.push(p);
    }
  }
  return out;
}

describe("no hex colour outside the token sheet", () => {
  const files = cssFiles(SRC);

  it("finds the token sheet and at least one other stylesheet", () => {
    const rels = files.map((f) => f.slice(SRC.length + 1));
    expect(rels).toContain(TOKEN_SHEET);
    expect(files.length).toBeGreaterThan(1);
  });

  for (const file of files) {
    const rel = file.slice(SRC.length + 1);
    if (rel === TOKEN_SHEET) continue;
    it(`token-only: ${rel}`, () => {
      const lines = readFileSync(file, "utf8").split("\n");
      const offenders = lines.map((line, i) => ({ line, n: i + 1 })).filter(({ line }) => HEX.test(line));
      expect(offenders, `${rel} has hex colours: ${offenders.map((o) => o.n).join(", ")}`).toEqual([]);
    });
  }
});
