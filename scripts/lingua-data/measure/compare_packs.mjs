// Copyright 2026 NEETROF
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file
// except in compliance with the License. You may obtain a copy of the License at
// http://www.apache.org/licenses/LICENSE-2.0
//
// What two packs change for a reader, measured with the extension's own engine (the wasm glue,
// under Node) over the same corpus: token by token, the lemma each pack gives, whether that lemma
// is in the pack's lexicon, and whether a gloss is shown. Per sample (the corpus's `kind`) and in
// total. Used for switch-lingua-inflections-to-esdb; any change to the tables can be read the same way.
//
//   node compare_packs.mjs <lingua_wasm.js> <lingua_wasm_bg.wasm> <A.lingua> <B.lingua> \
//     <corpus text.json> <A freq.tsv> <B freq.tsv> [report.json]
import { readFileSync, writeFileSync } from "node:fs";
import { pathToFileURL } from "node:url";

const [glue, wasm, packA, packB, corpusPath, freqA, freqB, out = "report.json"] = process.argv.slice(2);
const { initSync, LinguaEngine } = await import(pathToFileURL(glue).href);
initSync({ module: readFileSync(wasm) });

const lemmas = (p) => new Set(readFileSync(p, "utf8").split("\n").filter(Boolean).map((l) => l.split("\t")[0]));
const lexicon = { A: lemmas(freqA), B: lemmas(freqB) };

// Blocks as the extension sees them: paragraphs, each remembering its document's sample.
const blocks = [];
for (const doc of JSON.parse(readFileSync(corpusPath, "utf8"))) {
  for (const p of doc.text.split(/\n+/)) if (p.trim()) blocks.push({ text: p, kind: doc.kind });
}

function analyse(pack) {
  const engine = new LinguaEngine(new Uint8Array(readFileSync(pack)));
  const tokens = [];
  for (let i = 0; i < blocks.length; i += 200) {
    const res = JSON.parse(engine.analyse(blocks.slice(i, i + 200).map((b) => b.text)));
    for (const t of res.tokens) tokens.push({ ...t, block: t.block + i });
  }
  return tokens;
}

const a = analyse(packA);
const b = analyse(packB);
if (a.length !== b.length) throw new Error(`token counts differ: ${a.length} vs ${b.length}`);

const blank = () => ({ tokens: 0, lemmaChanged: 0, resolvedA: 0, resolvedB: 0, glossA: 0, glossB: 0, gained: 0, lost: 0 });
const stats = { total: blank() };
const changes = new Map();
const gained = new Map();
const lost = new Map();
const bump = (m, k) => m.set(k, (m.get(k) ?? 0) + 1);
for (let i = 0; i < a.length; i++) {
  const x = a[i];
  const y = b[i];
  if (x.block !== y.block || x.start !== y.start) throw new Error(`token ${i} misaligned`);
  if (x.class === "ProperNounOutOfLexicon" && y.class === "ProperNounOutOfLexicon") continue;
  const kind = blocks[x.block].kind;
  stats[kind] ??= blank();
  const ra = lexicon.A.has(x.lemma);
  const rb = lexicon.B.has(y.lemma);
  const s = x.surface.toLowerCase();
  for (const st of [stats.total, stats[kind]]) {
    st.tokens++;
    if (ra) st.resolvedA++;
    if (rb) st.resolvedB++;
    if (x.gloss) st.glossA++;
    if (y.gloss) st.glossB++;
    if (x.lemma !== y.lemma) st.lemmaChanged++;
    if (!ra && rb) st.gained++;
    if (ra && !rb) st.lost++;
  }
  if (x.lemma !== y.lemma) bump(changes, `${s}: ${x.lemma}→${y.lemma}`);
  if (!ra && rb) bump(gained, `${s}→${y.lemma}`);
  if (ra && !rb) bump(lost, `${s}→${x.lemma}`);
}
const top = (m, n) => [...m].sort((p, q) => q[1] - p[1] || (p[0] < q[0] ? -1 : 1)).slice(0, n);
writeFileSync(
  out,
  JSON.stringify(
    { stats, topChanges: top(changes, 80), topGained: top(gained, 40), topLost: top(lost, 40) },
    null,
    1,
  ),
);
console.log("sample     tokens   glosses    resolved   lemma changes");
for (const [kind, st] of Object.entries(stats)) {
  const d = (n) => (n >= 0 ? `+${n}` : `${n}`);
  console.log(
    `${kind.padEnd(10)} ${String(st.tokens).padStart(7)}   ${d(st.glossB - st.glossA).padStart(6)}   ` +
      `${d(st.resolvedB - st.resolvedA).padStart(6)} (+${st.gained}/-${st.lost})   ${st.lemmaChanged} (${((100 * st.lemmaChanged) / st.tokens).toFixed(2)} %)`,
  );
}
