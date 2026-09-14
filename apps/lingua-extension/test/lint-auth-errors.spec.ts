import { readdirSync, readFileSync, statSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { dirname, join } from "node:path";
import { describe, expect, it } from "vitest";

// Product rule (add-lingua-account-parity, design D7): a raw gRPC/Connect error string never
// reaches the reader. The surfaces map a category to copy (account/copy.ts), so no surface
// may read an error's `.message`, and the background must reply with categories only.

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
const SURFACES = ["src/popup", "src/account", "src/onboarding", "src/stats"];

function tsFiles(dir: string): string[] {
  const out: string[] = [];
  for (const name of readdirSync(dir)) {
    const p = join(dir, name);
    if (statSync(p).isDirectory()) out.push(...tsFiles(p));
    else if (p.endsWith(".ts")) out.push(p);
  }
  return out;
}

describe("no raw error message on account surfaces", () => {
  for (const file of SURFACES.flatMap((d) => tsFiles(join(root, d)))) {
    const rel = file.slice(root.length + 1);
    it(`no .message: ${rel}`, () => {
      expect(readFileSync(file, "utf8"), `${rel} must map a category to copy, not show .message`).not.toMatch(
        /\.message\b/,
      );
    });
  }

  it("the background replies to account messages with categories", () => {
    const text = readFileSync(join(root, "src/background.ts"), "utf8");
    expect(text).not.toMatch(/errorMessage\(/);
    expect(text).toContain("handleAccountMessage");
  });
});
