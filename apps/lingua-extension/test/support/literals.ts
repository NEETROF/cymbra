import { readdirSync, statSync } from "node:fs";
import { join, relative, sep } from "node:path";
import ts from "typescript";

// What the copy lints read: the literals of a source or a page, and the files of a tree — shared
// by test/lint-copy.spec.ts (French outside the catalogue) and test/lint-language-labels.spec.ts
// (a language's name outside its module), which once each held a copy. Not a spec: it declares no
// test, and vitest does not collect it (test/**/*.spec.ts).

/** A literal a lint reads, by line. */
export interface Hit {
  line: number;
  text: string;
}

/** A file's path as a baseline writes it: relative to `treeRoot`, with `/` whatever the platform's separator. */
export function baselinePath(treeRoot: string, path: string): string {
  return relative(treeRoot, path).split(sep).join("/");
}

/**
 * The string and template literals of a TypeScript source, by line — read from the syntax tree, so
 * a comment or a regular expression is not a hit. A template's chunks are joined by NUL, where its
 * expressions were.
 */
export function tsLiterals(path: string, text: string): Hit[] {
  const source = ts.createSourceFile(path, text, ts.ScriptTarget.ES2022, true);
  const hits: Hit[] = [];
  const line = (node: ts.Node): number => source.getLineAndCharacterOfPosition(node.getStart(source)).line + 1;
  const visit = (node: ts.Node): void => {
    if (ts.isStringLiteral(node) || ts.isNoSubstitutionTemplateLiteral(node)) {
      hits.push({ line: line(node), text: node.text });
    } else if (ts.isTemplateExpression(node)) {
      const chunks = [node.head.text, ...node.templateSpans.map((span) => span.literal.text)];
      hits.push({ line: line(node), text: chunks.join("\u0000") });
    }
    ts.forEachChild(node, visit);
  };
  visit(source);
  return hits;
}

/** The text nodes and attribute values of an HTML page, by line; comments do not count. */
export function htmlLiterals(text: string): Hit[] {
  const stripped = text.replace(/<!--[\s\S]*?-->/g, (c) => c.replace(/[^\n]/g, " "));
  const hits: Hit[] = [];
  const line = (index: number): number => stripped.slice(0, index).split("\n").length;
  for (const m of stripped.matchAll(/>([^<>]+)</g)) {
    const t = m[1].replace(/\s+/g, " ").trim();
    if (t) hits.push({ line: line(m.index), text: t });
  }
  for (const m of stripped.matchAll(/\s[a-zA-Z-]+="([^"]*)"/g)) {
    if (m[1]) hits.push({ line: line(m.index), text: m[1] });
  }
  return hits;
}

/**
 * Every `.ts` source (declarations aside) and `.html` page under `src`, in name order, the
 * directories named in `skip` left out wherever they sit — the generated code (`pkg`, `gen`), and
 * whatever a lint reads elsewhere.
 */
export function sources(src: string, skip: readonly string[]): string[] {
  const walk = (dir: string): string[] =>
    [...readdirSync(dir)].sort().flatMap((name) => {
      const path = join(dir, name);
      if (statSync(path).isDirectory()) return skip.includes(name) ? [] : walk(path);
      return /\.(ts|html)$/.test(name) && !name.endsWith(".d.ts") ? [path] : [];
    });
  return walk(src);
}
