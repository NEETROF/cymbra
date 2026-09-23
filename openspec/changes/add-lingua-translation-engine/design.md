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
| `<b>seldom</b>` (in "They seldom ship on Friday.") | `<b>rarement</b>` | 11 ms |
| `<b>gave up</b>` | `<b>a abandonné</b>` | 24 ms |

The last is the case that justifies the whole design: alone, the fragment yields an infinitive;
in its sentence it is conjugated and agreed with its subject. The second shows three English
words collapsing to one French word with the tag still correct.

*Consequence:* the page's text becomes HTML input, so it must be escaped before the tag is
placed. An unescaped `<` in a page breaks the call.

### The tag's position is checked against the selection translated alone

*Added during device testing.* The tag is placed by the engine's alignment, and the alignment
can be wrong. The `seldom` row above holds in its short sentence; in a longer one, "They
<b>seldom</b> ship on Friday, even when the customer asks nicely." came back "Ils
<b>expédient</b> rarement…" — a right translation with the mark on the verb. And one tag can only
mark one run of words, where the reader's may land apart: "They seldom" is "Ils … rarement", with
"expédient", which is *ship*, between.

So the relay makes a second request, the selection alone, and `translate/reconcile.ts` checks
the tag's marks against it. The lone translation has lost the context's grammar — that is why it
is never shown — but it says which target words belong to the selection:

1. Found exactly once in the sentence, clear of the tag's marks: the tag landed on a neighbour,
   and the lone translation's place replaces it (`seldom` → "rarement").
2. Otherwise the marks are checked word by word, within the tagged words and the full neighbouring
   words the lone translation also contains. A tagged word stays when the lone translation
   contains it, when it is short (under four letters: an article, an auxiliary, a pronoun takes
   its form from the sentence, so the fragment alone is no evidence against it), or when the lone
   translation is not wholly found (it may have used a synonym). An untagged word joins on the
   lone translation's evidence, or — if short — only as the glue between two kept words. A stray
   run of short words the lone translation lacks is dropped. Words compare case-folded, with an
   elided clitic removed (`s'attendait` = `attendait`), and as the same word when they share a
   prefix of at least five letters covering 70 % of the shorter (`expédient` / `expédiés`).
3. Sharing no word with the marks, the lone translation says nothing about them, and they stand.

Marks may therefore be several spans: "They seldom" → "[Ils] expédient [rarement]". The check
never invents a mark where the engine placed none, and never removes every mark. It is a check,
not a dependency: if the lone request fails or times out, the tag's marks stand as they were.

*Rejected — one contiguous span, grown to cover the lone translation.* It turned "They seldom"
into "[Ils expédient rarement]", marking *ship*, which the reader did not select.

*Rejected — keeping every word inside the region when the lone translation is not wholly found.*
It turned "[Nous] rendons [souvent]" (*We often*), which was right, into one mark over
"rendons". An untagged word is never admitted on the ground that the evidence fell short.

*Cost:* one more short request, to the same worker, right behind the sentence. Measured in the
Safari spike, which batched the two into one engine call: a median 28 ms (max 129 ms) on the iOS
simulator.

**Measured on 100 sentences** (en→fr, the real engine in Safari on the iOS simulator, each with
one selection: phrasal verbs, adverbs, adjective–noun inversions, noun phrases, idioms,
discontinuous selections, auxiliaries and negation, single words, clauses), replayed through this
module and judged by hand:

| | tag alone | tag, checked |
|---|---|---|
| Marks right, of 87 the model translated correctly | 77 | 83 |
| Marks made wrong by the check | — | 0 |
| Split marks joined into one over their short glue words | — | 5 |

The 13 left out are the model's own mistranslations, which no mark can make right: nine of ten
idioms rendered word for word ("mordre la balle", "au-dessus de la lune"), and four sentences
("savé", "récupérer" for *back up*, "ont été menées" for *fell through*, a garbled "should
have"). The four still wrong after the check: `go up` marked on an article, `look forward to` on
"se" alone, `hardly ever` still holding "regardons", and `used to` with no mark (the imperfect
"jouais" absorbs it — the clean failure).

The thresholds were set on French. A second target language should be measured the same way
before it relies on them.

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

### The card answers from the pack at once, and upgrades

*Revised during device testing.* The card used to ask the pack and the engine together and wait
for both, with the engine bounded at 2 500 ms — below the card's own 3 000 ms — so a slow engine
could never cost the reader the pack's answer. On a Galaxy Tab S6 Lite a cold engine takes
**4 783 ms**, so that bound could never be met: the reader waited 2.5 s and then got the *lesser*
answer. The worst of both.

They are no longer raced. The pack answers first and is shown immediately, with a quiet line
saying a translation is still coming; when the translation lands it replaces the word-by-word rows,
and when it is known that none is coming the line simply goes away. `TRANSLATION_WAIT_MS` stops
being a wait the reader sits through and becomes when the card stops expecting one — now 15 s,
above the engine's own start bound rather than below the card's.

Showing the completed card bumps the generation, so the upgrade is measured against *that* card: a
card the reader has closed, replaced or acted on is never written over.

*Rejected — waiting longer.* Raising both bounds past 4.8 s would have meant five seconds of
"Traduction en cours…" with nothing else on the card, on the device least able to afford it, and it
would have delayed the pack's answer for everyone.

*Rejected — showing the rows silently and swapping them.* The rows would have read as the answer,
which is what the original decision refused. The line saying a translation is coming is what makes
the upgrade honest rather than a surprise.

On Chromium none of this shows: a cold translation there is 270 ms.

### The reader keeps the engine's host busy while it reads

The device pass above showed the cost of not doing it: the event page is torn down when idle,
the Worker and the 36.7 MB model go with it, and the next selection pays a cold start the card
will not wait for. A minute between selections is ordinary reading, so on that hardware nearly
every selection fell back to the pack's word-by-word.

**An open port does not hold that page.** That was tried first, as the mechanism every platform
note describes, and measured on the device: with two reader ports connected throughout and nobody
touching the tablet, the background restarted every 8–24 s. Worse, reconnecting after each
teardown woke it again, so the reader was restarting the background in a loop and keeping nothing.

What holds it is **being answered**. The reader pings the background every 5 s — well inside the
shortest gap measured — and the background replies. Re-measured the same way: **zero restarts in
110 s at rest**, against a restart every 8–24 s before.

*The pings start with the page's first translation, never before.* Until then the engine has never
been loaded and there is nothing to keep, so a reader who opens a page and selects nothing never
wakes the background and never pays the ~195 MiB. `keepWarm` wraps the translator port for exactly
that: the first `translate()` starts the pings.

*Rejected — warming the engine when the page opens.* It would make even the first selection warm,
at the price of holding ~195 MiB on every page a reader visits, whether or not they translate
anything. On the tablet that showed the problem, that is the wrong trade.

*Rejected — raising `TRANSLATION_WAIT_MS`.* The card waits for the pack and the engine together,
so a longer bound delays the answer the reader would otherwise already have. It is also not where
the problem was: reloading a 36.7 MB model between two selections is.

A ping that fails means the extension context is gone — the page is orphaned, nothing else in it
works either — so the reader stops rather than ping a dead background forever.

### Measured on the device, after the fix

Same tablet, real selections, timed in the background from the request arriving to the answer
leaving:

| | |
|---|---|
| First translation, background just started | 4 783 ms |
| A second, after the background was killed again | 3 885 ms |
| The next, engine loaded | 1 119 ms |
| **70 s later, nothing touched in between** | **377 ms** |

The last row is the fix: before it, a selection after that pause reloaded the model; now it finds
it. No restart happened during that pause.

The first row is what remains: a cold start on this hardware costs ~4.8 s, well past the card's
2 500 ms, so the first selection after the background has died still falls back to word-by-word —
once, rather than on nearly every selection. One restart was also seen *despite* the pings, right
after a model load, which on a 4 GB tablet is most likely the system reclaiming memory rather than
the idle timeout.

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
- **Does Firefox's event page survive a translation?** *Answered by a device pass, and the answer
  is no.* On a Galaxy Tab S6 Lite (SM-P610, Firefox), three selections in order: the first fell
  back to word-by-word and a retry of the same one translated; a second, made straight after,
  translated; a third, after a minute's pause, fell back again. The teardown takes the Worker and
  the loaded model with it, and rebuilding them on that hardware costs more than the card's
  `TRANSLATION_WAIT_MS`. Nothing keeps the page alive — by design, there is no heartbeat here.
  This is the change's first Android data point. *Fixed here, in this change* — see below;
  the measurement stands as the reason. Chromium is unaffected in the same way: its worker lives
  in an offscreen document, which Chrome may still close, but that was not what this pass
  measured.
- **Safari.** Whether it takes this engine or Apple's translation is decided in its own change,
  after `add-lingua-apple` is archived.
