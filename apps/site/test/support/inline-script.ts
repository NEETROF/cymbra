import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import ts from "typescript";

// A component's inline `<script>`, run in the test's jsdom as the browser runs Astro's bundle of it
// (change: add-site-lingua-showcase): its TypeScript transpiled, each `import { … } from "…"` line
// answered by the test's own `modules` (an import the test did not give fails), then executed once
// per call, so that each test starts the script on the document and the browser it set up.

/** The source of the first `<script>` of `component` (a path under `apps/site/`). */
export function scriptOf(component: string): string {
  const source = readFileSync(resolve(__dirname, "../..", component), "utf8");
  const body = source.match(/<script>([\s\S]*?)<\/script>/)?.[1];
  if (body === undefined) throw new Error(`${component} has no <script>`);
  return body;
}

/** Run `component`'s inline script now, its imports taken from `modules` (specifier → exports). */
export function runScript(component: string, modules: Record<string, Record<string, unknown>> = {}): void {
  const imports: { names: string[]; from: string }[] = [];
  const body = scriptOf(component).replace(/^\s*import\s*\{([^}]*)\}\s*from\s*['"]([^'"]+)['"];?\s*$/gm, (_line, names: string, from: string) => {
    imports.push({ names: names.split(",").map((n) => n.trim()).filter(Boolean), from });
    return "";
  });
  const js = ts.transpileModule(body, { compilerOptions: { target: ts.ScriptTarget.ES2022, module: ts.ModuleKind.None } }).outputText;
  const parameters: string[] = [];
  const values: unknown[] = [];
  for (const { names, from } of imports) {
    const module = modules[from];
    if (!module) throw new Error(`${component} imports ${from}: give it to runScript`);
    for (const name of names) {
      if (!(name in module)) throw new Error(`${from} has no ${name}`);
      parameters.push(name);
      values.push(module[name]);
    }
  }
  new Function(...parameters, js)(...values);
}
