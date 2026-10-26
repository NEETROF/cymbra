# Cymbra Lingua — the Italian wording review (change 44)

Change 44 of the [Italian programme](italian-programme.md) (`review-lingua-italian-wording`, I2): the
dedicated reviewer agent (`.claude/agents/lingua-italian-reviewer.md`) read every Italian surface
together, once, before the build the owner dogfoods and photographs. Run on 2026-10-27 over `main` at
`e65cadceb` (changes 33, 34, 39, 40, 41, 43, 45 and 46 merged), in six passes, one per group of
surfaces, then a seventh over the corrections. Each finding below is applied in this pull request or
answered with its source; a finding that would change meaning, or that touches Music or Cymbra ID,
goes to the owner. The vocabulary the pass settled is the [glossary](italian-glossary.md).

## What was read

| Surface | Files | Strings read |
|---|---|---|
| The interface (pass 1) | the 23 text modules of `apps/lingua-extension/src/i18n/it/`, each against its French; the README's « Italian » section; `language-labels.ts`'s Italian parts | 743 texts, as the parity test's `texts()` yields them (every plural form) |
| The word card (pass 2) | `src/i18n/it/grammar.ts` and `card.ts`; the shared `grammar-description.ts` | 180 grammar lines of `test/baseline/word-card-grammar-it.txt`; the en-it card golden, `word-card-en-it.txt` (178 lines) |
| The manifest (pass 3) | `apps/lingua-extension/_locales/it/messages.json` | 4 messages, each against its cap |
| The Safari app (pass 3) | `copy.js`'s `it` table; `SignInCopy`'s `it` table (`LinguaSignIn`) | 15 and 11 entries |
| Cymbra ID's e-mails (pass 3, read only: design, Non-Goals) | `backend/platform/src/email_template/mod.rs`, the `SupportedLocale::It` arms | the verification and reset copy, the footer |
| The site (pass 4) | `apps/site/src/pages/it/` (home, Music, privacy, terms, support, account deletion, 404, Lingua's leaving page); `src/lib/i18n.ts`'s `it` dictionary (the islands); `music-media.ts`, `stores.ts`; the layouts and components | 8 pages (5,861 words, markup included); 107 dictionary entries |
| The Lingua page and its captures (pass 5) | `lingua-text.ts`'s `LINGUA_TEXT.it`, `GRAMMAR.it` and `HOME_IT`; the 9 Italian captures, each looked at | 76 entries; every Italian string visible in a capture listed against the catalogue |
| The listings (pass 6) | `apps/lingua-extension/STORE-LISTING.md` (_Italian for Italian speakers_, the Spanish texts' Italian line, the dashboards); `apps/lingua-apple/STORE-LISTING.md` (_Italian for Italian speakers_) | 15 fields, every capped one recounted |

## Findings and what was done

A **must-fix** is wrong Italian, a broken rule (I6, the agent's rules 1–8) or a meaning that differs
from the French; a **suggestion** is a more natural wording of a correct sentence.

### The interface (pass 1: 9 must-fix, 10 suggestions)

| Where | Finding | Level | Done |
|---|---|---|---|
| `account-errors.ts` `storageFull`, `sync.ts` `storageFull`, `selection.ts` `expressionKind` | « dell'estensione », « d'origine »: write U+2019 where the French does (rule 6) | must-fix | **Answered**: change 34's D4 sets one apostrophe in the extension's Italian, U+0027, held by `test/i18n.spec.ts`; change 34's review moved these very strings to it. The glossary page records that D4 takes precedence over rule 6. |
| `account-errors.ts` `linkUnauthenticated` | « Riprova; se il problema persiste » — the French « — » becomes a colon or a full stop | must-fix | Applied: « Riprova. Se il problema persiste, accedi di nuovo. » |
| `onboarding.ts` `levelNote` | the French « — et tu pourras » becomes a full stop; « lo » too far from its noun | must-fix | Applied: « …finché non hai scelto. Potrai cambiare livello in qualsiasi momento nelle impostazioni. » |
| `onboarding.ts` `stepPin` | « Fissa » is a browser's menu word with no cited source, not marked unverified | must-fix | Applied: marked unverified in the module's header and the README; the owner's checklist (E12) |
| `review.ts` `keepOther` | « Tieni un'altra traduzione… » says "another"; the French « Garder une traduction… » does not, and the field opens prefilled with that translation | must-fix | Applied: « Tieni una traduzione… » |
| README « Italian », terms | « pannello laterale » for the browser's side panel | must-fix | Applied: « riquadro laterale » (change 39), with « pannello di ripasso sulla pagina » and « acquisisci la selezione » added, and the glossary named |
| README « Italian », unverified list | « Fissa » missing | must-fix | Applied |
| `app.ts` `syncFailed` | « partiranno » copies « partiront » | suggestion | Applied: « verranno inviati più tardi » |
| `card.ts` and `stats.ts` `relearn`, `stats.ts` `markedNote` | « Rimetti da imparare » is clumsy | suggestion | Applied: « Segna come da imparare »; « Segnane una come «da imparare» per vederla di nuovo evidenziata. » |
| `colours.ts` `paperBackground`, `paperChosenText`, `paperText` | « Pagina carta » can be heard as "card page" | suggestion | Applied: « Pagina Carta: … », the theme's name capitalised |
| `languages.ts` `borrowedTypicalNote` | « riprende le dimensioni dei livelli » calques « tailles » and echoes « ripreso » | suggestion | Applied: « di cui lo spagnolo adotta il numero di parole per livello » |
| `onboarding.ts` `stepPin` | « barra del browser » → Chrome's and Firefox's « barra degli strumenti » | suggestion | Applied |
| `onboarding.ts` `accountLead` | « i loro stati » after « mazzo » reads as the deck's | suggestion | Applied: « le tue parole, il loro stato e il tuo mazzo » (and the site's account card with it) |
| `settings.ts` `showPercentPill`, `pillNote` | « badge » may collide with the icon's count | suggestion | **Not applied**: « badge » names the pill in the interface, on the site and in the App Store text alike, and no string names the icon's count; the glossary forbids calling that count « badge » |
| `studied-languages.ts` `studiedNote` | « che contiene » reads with « che » as the subject | suggestion | Applied: « che vi compare » |
| README, terms | add the drawer's and the capture's words | suggestion | Applied |
| Across: « Stats » / « Statistiche » (handed from change 33) | keep « Stats » where the French writes it (the HUD and the drawer, one row on a phone) | recommendation | Kept; the owner's question 1 |
| Across: « nome utente » (handed from change 33) | keep it, an exception to precedence 2 (Music says « Handle ») | recommendation | Kept; recorded in the glossary (`username`) |
| Across: « Da rivedere » / « da ripassare » | deliberate, record both | note | Recorded (`review`, its exception) |

### The word card (pass 2: 0 must-fix, 3 suggestions)

| Where | Finding | Level | Done |
|---|---|---|---|
| `grammar.ts` `formOf` / `mayAlsoBe` (handed from change 34) | « plurale e maschile e femminile plurale di grande » | suggestion | Applied, Italian only: a comma before a name that joins two itself — « plurale, maschile e femminile plurale di grande ». English and Spanish keep their own renderers' wording and goldens |
| `grammar.ts` `persons` (handed from change 34) | the tense named twice for « sono » | suggestion | Applied, Italian only, through an optional `ReadingWords.bothNumbers` the French, English and Spanish renderers leave unset: « prima persona singolare e terza persona plurale dell'indicativo presente di essere »; after « può anche essere » each half takes « la » |
| `card.ts` `seenForm`, `listenSeenFormLabel` | « forma vista » calques « forme vue » | suggestion | **The owner's** (question 2): the hero, close-up, share image, video and store captures show it, so it needs the Italian re-capture |
| Unverified (handed from change 34): « presente semplice », « forma in -ing » vs schoolbook English names | recommendation: keep change 34's D3 | — | Kept; the listing no longer claims « come si impara a scuola » (pass 6); change 55's Italian readers settle it |
| Snapshots | the grammar snapshot re-blessed on its two lines (L368, L466); the en-it card snapshot does not move | — | Done |

### The manifest, the Safari app, the e-mails (pass 3: 1 must-fix, 4 suggestions)

| Where | Finding | Level | Done |
|---|---|---|---|
| Design D1 | « cattura la selezione » named as change 39's word; what shipped is « acquisisci la selezione » | must-fix | Applied: D1 corrected; the glossary's `capture` accepts « acquisisci », refuses « cattura » |
| `SignInCopy` `it.appLede` | « lo stesso account di Safari » reads as Safari's own account | suggestion | Applied: « Usa lo stesso account che usi in Safari. » (`swift test` green) |
| E-mail `code_note`, `outro`, `subject` (the first handed from change 43) | « codice di reimpostazione », « la reimpostazione », « la password di Cymbra » | suggestions | **Not edited here** (design, Non-Goals: Cymbra ID untouched); the owner's question 5 |
| Glossary | a settings tab is named by its path, never « scheda » | suggestion | Recorded (`settings`) |
| `_locales/it` | 105 / 112; commands 65, 49, 36 characters, each holding Réglages' words | — | No finding |

### The site (pass 4: 2 must-fix, 7 suggestions)

| Where | Finding | Level | Done |
|---|---|---|---|
| `music.astro` card | « Collegato ai tuoi strumenti » reads as said of the reader | must-fix | Applied: « Si collega ai tuoi strumenti » |
| `privacy.md` §7 | « (Impostazioni → Elimina account) » is not where Music puts it — nor is the French « (Réglages → Supprimer mon compte) » | must-fix | **The owner's** (question 3): the French prevails (I13), so the French changes first |
| `index.astro` « Funziona offline » | « andirivieni » | suggestion | Applied: « nessuno scambio con il server » |
| `music.astro` title | « corsi di musica collegati » reads as "related courses" | suggestion | Applied: « pratica e corsi di musica con lo strumento collegato » |
| `privacy.md` Allegato B | « quando viene disattivata » can attach to the copy | suggestion | Applied: « quando l'impostazione viene disattivata » |
| `termini.md` §3 | no colon between « a non » and its infinitive | suggestion | Applied; no legal meaning moves |
| `404.astro` | the page inherits the French default description | suggestion | Applied: an Italian `description` |
| `i18n.ts` `deleteAppNote` | « Account → Elimina account »: Music's item sits in the account menu | suggestion | Applied: « menu dell'account → Elimina account », as `/it/assistenza/` says |
| « Riscontro » / « feedback » | pick one | suggestion | **Kept « riscontro »**: the Accademia della Crusca's Italian for « feedback »; Music's store copy is Music's (question 4) |
| « Termini di utilizzo » / Music's « Termini di servizio » (handed) | keep the site's: the French « Conditions d'utilisation » prevails, Music's follows its English arb | recommendation | Kept; glossary `terms-of-use`; Music's string is question 4 |
| « Modalità attesa » / « Modalità Attesa » / « Wait Mode » (handed) | the site writes Music's app label, a capital only on the first word | recommendation | Kept; glossary `wait-mode`; Music's strings are question 4 |

### The Lingua page and its captures (pass 5: 5 must-fix, 9 suggestions)

| Where | Finding | Level | Done |
|---|---|---|---|
| `lingua-text.ts` `percentage` card, `HOME_IT.read` | « percentuale onesta » reads "a fair cut"; `_locales/it` chose « reale » | must-fix (×2) | Applied, with the three tests that quote it; glossary `true-percentage` |
| `lingua-text.ts` `deck` card | « pannello laterale » | must-fix | Applied: « riquadro laterale » |
| README terms; design D1 | the same « pannello laterale »; « cattura » | must-fix (×2) | Applied (pass 1, pass 3) |
| `deck` card | « che metti da parte » → the product's verb | suggestion | Applied: « che acquisisci » |
| `highlight` card | « sul posto », « senza rompere nulla » | suggestion | Applied: « Evidenziazione nella pagina », « senza alterare il sito » |
| `click` card | a second colon | suggestion | Applied: the three buttons in parentheses |
| `level` card | « badge fatto in casa » beside the pill's « badge » | suggestion | Applied: « distintivo fatto in casa » |
| `account` card | « i tuoi stati » | suggestion | Applied, with onboarding's order: « le tue parole, il loro stato e il tuo mazzo » |
| `genderCard` | « In italiano, … » reads as the interface | suggestion | Applied: « Per le parole in {inStudied}, … », its test updated |
| captions, `glossesOrigin` | « definizione » for a gloss | suggestion | Applied: « traduzione », « I significati provengono dal Wikizionario » |
| « Stats » | « Statistiche » would fit the drawer's row | suggestion | **The owner's** (question 1), against pass 1's recommendation; it needs `review.webp` re-captured |
| Captures | every visible Italian string is the catalogue's current one; no applied correction touches one | — | No re-capture (D4) |
| Unverified, meaning | the page never says the English card names the tense; review « nel riquadro laterale » only (Safari and Firefox for Android have none); the 🧭 card for a one-pair reader | — | **The owner's** (question 6), every language alike |

### The listings (pass 6: 6 must-fix, 11 suggestions)

| Where | Finding | Level | Done |
|---|---|---|---|
| ext IT §2 | « come si impara a scuola » is backed by no claim; Italian schoolbooks keep « past simple » | must-fix | Applied: the clause goes (the owner's question E11.4 answered by the safe side) |
| ext permission justifications | « +134 » is +153 | must-fix | Applied |
| apple What's New | « ogni parola inglese mostra il suo significato » over-claims (92.1 / 83.1 / 70.6 %) | must-fix | Applied: « le parole inglesi mostrano il loro significato » |
| site × 2, design D1 | as passes 3 and 5 | must-fix | Applied |
| ext §4 | « i tuoi file … I tuoi libri » | suggestion | Applied |
| ext §6, apple « TRADUZIONE ESTESA » | « all'interno », « dal motore », « scaricato da Cymbra » | suggestion | Applied: « nel contesto », « con il motore », « una sola volta dai server di Cymbra » |
| ext §7, apple | « I nostri dati » | suggestion | Applied: « Le cifre sono pubblicate » |
| ext §9 | « che vi hanno accesso » | suggestion | Applied: « per gli account abilitati » |
| ext permissions note | `scripting` left out of the unchanged list | suggestion | Applied |
| ext claims table | the e-ink claim names no file | suggestion | Applied: `presetEinkMono`, `presetEinkColour` |
| apple What's New | « per chi parla italiano » echoes « parla italiano »; the voice is preferred, not exclusive | suggestion | Applied: « per gli italofoni che imparano l'inglese », « usa di preferenza una voce americana o britannica » |
| apple keywords | « inglese » repeats the subtitle | suggestion | **Kept**: the owner's option A for French pays twice on purpose |
| Unverified | Apple's paths; « riquadro laterale » also pasted on AMO, where Firefox has none (every language) | — | The owner's checklist; question 6 |

The counts after the corrections, by each file's own rules: Chrome Web Store and AMO Italian 3,116 /
16,000 (3,104 without it-es, 2,596 withheld); App Store it-IT description 3,196 / 4,000 (3,184,
2,751), What's New 423 / 4,000 (346 without the voice, 303 without translation); the other fields
unchanged.

## For the owner

1. **« Stats » or « Statistiche »** in the HUD and the drawer. Pass 1 keeps « Stats » (the French and
   Spanish clip there; the drawer's row fills at 125 % on a 360 px phone); passes 5 and 6 prefer
   « Statistiche ». Recommendation: keep « Stats »; check the drawer on a phone at large text sizes.
   Changing it re-captures `review.webp` and the store's review shot.
2. **« forma vista » → « forma nel testo »** on the card. Recommendation: in change 55, with the one
   Italian re-capture the dogfood's corrections need.
3. **The privacy policy's in-app deletion path**, wrong in the French too. Recommendation: correct the
   French (« menu du compte → Supprimer le compte »), then English, Spanish and Italian, in a site
   change.
4. **Music's own Italian**: « Termini di servizio », « Modalità Attesa », « Wait Mode » (also in the
   French and Spanish arbs), « Handle », « il tuo SoundFont », « spartito » / « partitura »,
   « feedback ». Recommendation: one Music change.
5. **Cymbra ID's Italian reset e-mail** (`code_note`, `outro`, `subject`). Recommendation: a small
   backend change (I10), or change 55.
6. **Meaning on the Lingua page and the listings, every language**: the English card's tense
   unmentioned; review placed in the side panel only; the 🧭 card for a one-pair reader.
   Recommendation: change 55, or a site change, in every language at once.
7. **The reviewer agent's rule 6** (« the apostrophe as the French source writes it ») contradicts
   change 34's D4 and produced three must-fixes answered above. Recommendation: say U+0027 for the
   extension's Italian in `.claude/agents/lingua-italian-reviewer.md` (left to the owner: it is the
   agent's configuration).

## Verification pass

A seventh pass read every corrected string, the glossary and its page. Its findings and what was done
with them are below.
