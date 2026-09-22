## Context

The pack answers words and fixed expressions. It cannot answer the two things readers actually
select — a whole sentence, and a fragment they want understood inside that sentence — because
neither is known in advance. Bergamot, the engine Firefox translates pages with, can, entirely
on the device.

Every number below was measured, not estimated: a CI build of the engine, and a throwaway MV3
extension driven over CDP with the real `en→fr` model. Where a figure came from reading rather
than running, it says so. The measurements replace the estimates the phrase-translation roadmap
carried, and should not be reopened without new ones.

The relevant existing shape: `LinguaPort` is the seam the reading code already talks to, and
`create-port.ts` picks its implementation per target — in the content script on Chromium, in the
event page on Firefox and Safari. `sentenceForRange` finds the sentence a selection sits in by
position; it computed the selection's offsets inside it and discarded them, so this change keeps
them (`sentenceAndSelection`, which `sentenceForRange` now delegates to).

## Goals / Non-Goals

**Goals:**

- Produce the engine ourselves, reproducibly, from a pinned upstream commit.
- Answer a selection with its whole sentence translated, the selection marked in its French
  place.
- Never occupy a thread that paints, and never make another part of the extension wait.
- Keep the reading code ignorant of where the engine runs, as it is for the analyser.

**Non-Goals:**

- Putting the engine in any reader's hands. No setting, no download, no shipped artefact.
- Touching store copy or the privacy promises. Nothing here fetches anything.
- Safari and Android. Neither is measured; Safari is expected to take Apple's translation
  instead, and that decision is not made here.
- Storing a machine translation. It is computed for display and discarded; `gloss` keeps
  dictionary data only.

## Decisions

### Build the engine from source, at a pinned commit

`mozilla/translations` is checked out at `a6310e24669df32a9098faadccbb1206448b51cb` (Bergamot
v0.6.0) with submodules, and `inference/scripts/build-wasm.py` is run. Measured on
`ubuntu-latest`: **7 min 48 s**, no patching, no `-j 1` fallback. It yields
`bergamot-translator.wasm` (4 963 402 B, 1 082 904 B brotli) and `bergamot-translator.js`
(92 034 B, 18 647 B brotli).

*Why not a prebuilt artefact:* Mozilla publishes no standalone build; what Firefox ships is
inside Firefox. A pinned source build is the only reproducible supply we control.

*Why pinned rather than a branch:* a moving engine invalidates every measurement taken on it.

*Two traps the recipe hides:* the build script refuses to run outside Docker unless
`ALLOW_RUN_ON_HOST=1` is set — a CI runner is already a clean container — and it warns that it
breaks on macOS AArch64, which is the only machine available locally. The build belongs in CI,
not on a developer's Mac.

### The engine runs in a classic Worker, owned by an offscreen document on Chromium

The constraint is that nothing may freeze, and that is stricter than "the page stays smooth".

Measured, in a real MV3 service worker with the real model: a synchronous translation there does
**not** touch the page — the page's worst frame gap is 18 ms both idle and during translation,
against 202 ms in a control where the page itself blocks for 200 ms. A service worker is its own
thread. But it *does* stop answering its own RPCs for the duration, and on Firefox and Safari the
analyser reaches the reader through exactly that channel.

So the engine goes in a Worker of its own. On Chromium the service worker cannot construct one —
measured, `typeof Worker` is `undefined` there — so an offscreen document created with
`reasons: ["WORKERS"]` owns it. That document costs **18 ms** to create and does no work itself.
On Firefox the event page constructs the Worker directly and no offscreen document exists.

Measured with the engine in that Worker: the service worker answers in **1 ms worst, 0 ms
median** over 25 pings taken during a translation, and the sentence comes back in 40 ms.

*Why a classic Worker rather than a module one:* the generated glue assumes sloppy mode.
`exportAsmFunctions` does `var global_object = this`, which is `undefined` in an ES module and
takes the init down with "Cannot set property". A classic worker loads it with `importScripts`,
where `this` is the global object, and **the pristine upstream artefact runs unpatched** —
verified. Loading it as a module would mean post-processing Mozilla's output on every build.

*The trap this hides:* that failure leaves the init promise unsettled forever. It looks like a
slow engine, not a crash. An init that does not settle within a bound must be treated as failed.

*Rejected — the engine in the content script*, the way the analyser runs on Chromium. There it
is on the page's main thread and the translation becomes exactly the 202 ms freeze the control
measured. `create-port.ts` is the obvious model to copy and the wrong one; the seam must make
that impossible rather than warn against it.

*Rejected — the engine in the service worker directly.* It works and the page stays smooth, but
it blocks the extension's own RPCs for the length of a translation. Bounded (a selection is
capped at 120 characters, and a 114-character sentence costs 178–198 ms), but not zero.

### The fragment is marked with a tag, not resolved through alignments

The reader wants the fragment translated *in context*. Translating it alone loses the context —
that is the whole complaint.

Bergamot computes a soft alignment matrix, but the wasm bindings expose only sentence-level byte
ranges; `Response::alignments` exists in C++ and is not reachable from JS. Exposing it means
patching the bindings, which means maintaining a fork.

Unnecessary: with `html: true` in the response options, the engine **repositions markup onto the
corresponding target span**. Wrapping the selection in a tag and translating the sentence gives
the fragment's place in the French for free. Measured:

| selection, marked in its sentence | comes back | |
|---|---|---|
| `<b>the effects of inflation</b>` | `<b>les effets de l'inflation</b>` | 136 ms |
| `<b>put up with</b>` | `<b>supporter</b>` | 18 ms |
| `<b>seldom</b>` | `<b>rarement</b>` | 11 ms |
| `<b>gave up</b>` | `<b>a abandonné</b>` | 24 ms |

The last is the case that justifies the whole design: alone, the fragment yields an infinitive;
in its sentence it is conjugated and agreed with its subject. The second shows three English
words collapsing to one French word with the tag still correct.

*Consequence:* the page's text becomes HTML input, so it must be escaped before the tag is
placed. An unescaped `<` in a page breaks the call.

### The model is side-loaded, and nothing is downloaded

`en→fr` `base-memory`: 23 045 432 + 2 297 334 + 409 706 = **25 752 472 B** compressed as served,
36 749 127 B on disk. `metadata.json` gives sha256
`6322e296d4fecfe395a8d5723da4ec37ecbe6d7613bb1dfcf4b28e2a47498b68`, BLEU 49.6, COMET 0.8697.
The `tiny` variant is rejected: 16.9 MB saves about 9 MB and its mistakes would be memorised.

In this change a developer places those files by hand in a local build. Nothing fetches them, so
**No network requests** keeps its meaning and store copy does not move. The download, its
hosting, its pinning and the setting that gates it are the next change.

### `INITIAL_MEMORY` is left at Mozilla's value

Measured at 64, 128 and 223 MiB: the working set settles at **195.4 MiB either way**. Below it
the heap grows into that figure during the first translation, buying a copy and saving nothing.
Mozilla's 234 291 200 is a pre-allocation, not a requirement, and is kept for that reason.

### The card shows it; a slow engine never costs the pack's answer

The proposal named no display, which left the development build showing nothing. The expression
card now asks the pack and the engine together; neither can take the other down. The engine's
wait is bounded at 2.5 s, **below** the card's own 3 s timeout, so a cold or slow engine ends in
exactly the card the reader had before — and the engine keeps warming for the next selection.

With a translation the card shows « Dans votre phrase — traduction automatique », the sentence
with the selection in the answer colour, built from text nodes only (the sentence came from the
page). It shows no word-by-word rows beside it — the existing requirement already says those
appear only when the card has no better answer — and no "the pack has no translation" note
above one. An expression's dictionary gloss stays. A single word keeps its dictionary card: only
a selection of several words is translated. No gesture carries the translation, so no path can
store it.

### Measured in the built extension

The figures above came from a throwaway harness. The same measurements were retaken on the
extension this change builds (`LINGUA_TRANSLATION_ENGINE`, Chrome 153 headless, macOS ARM), with
the real model, through the real path — content script or extension page, service worker,
offscreen document, worker:

| | worst | median |
|---|---|---|
| Analyser RPC (`lingua-rpc`), idle | 0.4 ms | 0.2 ms |
| Analyser RPC during 20 back-to-back translations | 0.7 ms | 0.4 ms |
| Reader's page frame gap, idle | 18 ms | |
| Reader's page frame gap during 20 translations | 18 ms | |
| Control: the page itself blocks 200 ms | 214 ms | |

A first translation, cold (offscreen document, model load, wasm init), took 229 ms; warm ones
24–47 ms. On a real page through the real content script, `gave up` came back
`Elle [a abandonné] après la troisième tentative…` and `put up with` came back
`Il a dû [supporter] le bruit…`. The page's own `<b>` and `<i>` came back as text. Built with the
engine and no model, the same selections gave exactly the card they gave before the engine
existed, with nothing reported to the reader.

## Risks / Trade-offs

**The engine outside Firefox never gets the optimised GEMM** → `WebAssembly.mozIntGemm` is a
privileged Firefox API; everywhere else `import-gemm-module.js` falls back. Every figure above
was taken *on* that fallback and is fast enough, but no performance claim Mozilla publishes
applies to us. Measure on our own artefact, always.

**An offscreen document is a new surface on Chromium** → it needs the `offscreen` permission and
store review will ask what it is for. Accepted deliberately, and it is why the document owns the
Worker and nothing else: its justification is exactly "runs the engine off every thread that
paints".

**A C++/emscripten build in a repo whose CI builds Rust, Dart and TypeScript** → it is pinned,
runs in under eight minutes, and produces an artefact rather than entering any other build. If
upstream ever stops building, the pin keeps the last good commit working.

**Latency was measured on one machine** → an Apple Silicon Mac, in headless Chrome. The numbers
bound the order of magnitude, not the slowest device a reader owns. Android is unmeasured on
purpose and out of scope.

**The engine and model together hold ~195 MiB while loaded** → it is a Worker that can be torn
down. When to release it is a question for the change that lets readers turn it on, since only
there does an idle engine cost anyone anything.

## Migration Plan

Nothing to migrate: no stored data changes shape, no reader-visible behaviour moves, and no
artefact ships. Backing the change out is deleting a workflow and an unused seam.

## Open Questions

- **When is the Worker torn down?** Deliberately unanswered here — it only matters once readers
  can enable the engine.
- **Does Firefox's event page survive a translation?** The event page is torn down after 30 s of
  inactivity; a translation is far shorter, but the Worker's lifetime against that teardown is
  not measured.
- **Safari.** Whether it takes this engine or Apple's translation is decided in its own change,
  after `add-lingua-apple` is archived.
