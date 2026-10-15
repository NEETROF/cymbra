// Refresh the Lingua page's byte pin from the build (apps/site/README.md): run `yarn build` with
// `PUBLIC_DISCORD_URL` unset, then `node test/support/refresh-lingua-fixtures.mjs`. Writes the
// `<main>` of `/lingua/` and `/en/lingua/`, their media addresses left aside (`media.mjs`), to
// `test/fixtures/lingua/main.{fr,en}.html`. Review the diff: it is the page's change.
import { readFileSync, writeFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { pinnedMain } from "./media.mjs";

const site = resolve(dirname(fileURLToPath(import.meta.url)), "../..");
for (const [lang, page] of [
  ["fr", "dist/lingua/index.html"],
  ["en", "dist/en/lingua/index.html"],
]) {
  const main = pinnedMain(readFileSync(resolve(site, page), "utf8"));
  if (main === undefined) throw new Error(`${page} has no <main>`);
  writeFileSync(resolve(site, `test/fixtures/lingua/main.${lang}.html`), main);
  console.log(`test/fixtures/lingua/main.${lang}.html ← ${page}`);
}
